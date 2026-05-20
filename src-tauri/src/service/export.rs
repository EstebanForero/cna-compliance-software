use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

use crate::audience::{
    display_instrument_column_label, display_public_label, instrument_title_public,
    InstrumentAudience, InstrumentColumn,
};
use crate::domain::{
    ExportKind, ExportWorkbookRequest, ExportWorkbookResult, GuidelineAspect,
    InstrumentPublicOption, Question, QuestionDiffKind, QuestionStatus,
};
use crate::error::AppError;
use crate::service::baseline::diff_questions;
use crate::service::AutoEvaluationService;

impl AutoEvaluationService {
    pub async fn export_workbook(
        &self,
        request: ExportWorkbookRequest,
    ) -> Result<ExportWorkbookResult, AppError> {
        let questions = self.repository.list_questions().await?;
        let snapshots = self.repository.list_original_snapshots().await?;
        if snapshots.is_empty() {
            return Err(AppError::Validation(
                "mark an original baseline before exporting change-colored workbooks".into(),
            ));
        }
        let aspects = self.repository.list_guideline_aspects().await?;

        let diffed = diff_questions(&questions, &snapshots);
        match request.kind {
            ExportKind::Consolidated => {
                write_consolidated_workbook(&request.path, &diffed, &aspects)?
            }
            ExportKind::Instruments => write_instruments_workbook(
                &request.path,
                &diffed,
                &aspects,
                request.instrument_public.as_deref(),
            )?,
        }

        Ok(ExportWorkbookResult {
            path: request.path,
            kind: request.kind,
            exported_questions: questions.len(),
            added_questions: diffed
                .iter()
                .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Added))
                .count(),
            modified_questions: diffed
                .iter()
                .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Modified))
                .count(),
            removed_questions: diffed
                .iter()
                .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Removed))
                .count(),
        })
    }

    pub async fn list_instrument_public_options(
        &self,
    ) -> Result<Vec<InstrumentPublicOption>, AppError> {
        let questions = self.repository.list_questions().await?;
        let snapshots = self.repository.list_original_snapshots().await?;
        let diffed = if snapshots.is_empty() {
            questions
                .into_iter()
                .map(|question| (question, QuestionDiffKind::Unchanged))
                .collect::<Vec<_>>()
        } else {
            diff_questions(&questions, &snapshots)
        };

        Ok(group_instruments_by_public(&diffed)
            .into_iter()
            .map(|instrument| InstrumentPublicOption {
                label: display_public_label(&instrument.public),
                public: instrument.public,
                subpublics: instrument
                    .subpublics
                    .into_iter()
                    .map(|subpublic| subpublic.label)
                    .collect(),
                question_count: instrument.questions.len(),
            })
            .collect())
    }
}

fn write_consolidated_workbook(
    path: &str,
    questions: &[(Question, QuestionDiffKind)],
    aspects: &[GuidelineAspect],
) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let formats = ExportFormats::new();

    let headers = [
        "N° Factor",
        "Descripción Factor",
        "N° Característica",
        "Nombre característica",
        "Descripción Característica",
        "N° Aspecto",
        "Descripción Aspecto",
        "Tipo pregunta",
        "Estado pregunta",
        "Convención opción de respuesta",
        "N° pregunta",
        "Pregunta",
        "Público",
        "Tipo de público",
        "Observaciones",
    ];

    let worksheet = workbook.add_worksheet();
    worksheet.set_name("BASEvs3")?;
    set_consolidated_dimensions(worksheet)?;
    write_headers(worksheet, &headers, &formats.header)?;
    worksheet.set_row_height(0, 45)?;
    worksheet.set_freeze_panes(1, 0)?;

    let mut row = 1;
    for (question, diff) in questions {
        let format = match diff {
            QuestionDiffKind::Removed => Some(&formats.removed),
            QuestionDiffKind::Modified => Some(&formats.modified),
            QuestionDiffKind::Added => Some(&formats.added),
            QuestionDiffKind::Unchanged => None,
        };
        let audiences = if question.audiences.is_empty() {
            vec![String::new()]
        } else {
            question.audiences.clone()
        };
        let (factor_code, factor_name) = split_number_name(&question.factor);
        let (characteristic_code, characteristic_name) =
            split_number_name(&question.characteristic);
        let (aspect_code, aspect_description) = split_number_name(&question.aspect);

        for audience in audiences {
            let audience = InstrumentAudience::parse(&audience);
            let subpublic = audience.subpublic();
            let values = [
                factor_code.as_str(),
                factor_name.as_str(),
                characteristic_code.as_str(),
                characteristic_name.as_str(),
                characteristic_name.as_str(),
                aspect_code.as_str(),
                aspect_description.as_str(),
                question.scope.as_str(),
                status_excel_label(&question.status),
                question.convention_code.as_deref().unwrap_or(""),
                question.code.as_str(),
                question.text.as_str(),
                audience.public.as_str(),
                subpublic.as_str(),
                question.justification.as_deref().unwrap_or(""),
            ];

            worksheet.set_row_height(row, consolidated_row_height(question.text.as_str()))?;
            for (column, value) in values.iter().enumerate() {
                worksheet.write_string_with_format(
                    row,
                    column as u16,
                    *value,
                    format.unwrap_or(&formats.wrapped),
                )?;
            }
            row += 1;
        }
    }

    for aspect in aspects {
        write_lineament_row(worksheet, row, aspect, &formats.wrapped)?;
        row += 1;
    }

    let lineaments = workbook.add_worksheet();
    lineaments.set_name("Lineamientos")?;
    set_consolidated_dimensions(lineaments)?;
    write_headers(lineaments, &headers, &formats.header)?;
    for (index, aspect) in aspects.iter().enumerate() {
        write_lineament_row(lineaments, (index + 1) as u32, aspect, &formats.wrapped)?;
    }
    write_convention_sheet(&mut workbook, &formats.header)?;
    workbook.save(path)?;
    Ok(())
}

fn write_instruments_workbook(
    path: &str,
    questions: &[(Question, QuestionDiffKind)],
    _aspects: &[GuidelineAspect],
    instrument_public: Option<&str>,
) -> Result<(), AppError> {
    let output = InstrumentOutput::from_path(path);
    std::fs::create_dir_all(&output.directory)?;
    let instruments = group_instruments_by_public(questions)
        .into_iter()
        .filter(|instrument| {
            instrument_public
                .map(|public| instrument.public == public)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if instruments.is_empty() {
        return Err(AppError::Validation(
            "no questions were found for the selected public".into(),
        ));
    }
    for instrument in instruments {
        let file_path = output.file_path(&instrument.public);
        write_single_instrument_workbook(&file_path, &instrument)?;
    }
    Ok(())
}

fn write_single_instrument_workbook(
    path: &Path,
    instrument: &InstrumentWorkbook,
) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let formats = ExportFormats::new();

    let by_lineament = workbook.add_worksheet();
    by_lineament.set_name("Por lineamiento")?;
    write_instrument_lineament_sheet(
        by_lineament,
        &instrument.questions,
        &instrument.subpublics,
        &instrument.public,
        &formats,
    )?;

    let by_order = workbook.add_worksheet();
    by_order.set_name("Por orden")?;
    write_instrument_order_sheet(
        by_order,
        &instrument.questions,
        &instrument.subpublics,
        &instrument.public,
        &formats,
    )?;

    write_convention_sheet(&mut workbook, &formats.header)?;
    workbook.save(path)?;
    Ok(())
}

#[derive(Debug)]
struct InstrumentOutput {
    directory: PathBuf,
    prefix: String,
}

impl InstrumentOutput {
    fn from_path(path: &str) -> Self {
        let path = PathBuf::from(path);
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("xlsx"))
            .unwrap_or(false)
        {
            return Self {
                directory: path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .to_path_buf(),
                prefix: path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("instrumento")
                    .to_string(),
            };
        }

        Self {
            directory: path,
            prefix: "instrumento".into(),
        }
    }

    fn file_path(&self, public: &str) -> PathBuf {
        self.directory.join(format!(
            "{}-{}.xlsx",
            self.prefix,
            slugify_file_part(public)
        ))
    }
}

#[derive(Debug)]
struct InstrumentWorkbook {
    public: String,
    subpublics: Vec<InstrumentColumn>,
    questions: Vec<(Question, QuestionDiffKind)>,
}

fn group_instruments_by_public(
    questions: &[(Question, QuestionDiffKind)],
) -> Vec<InstrumentWorkbook> {
    let mut groups: BTreeMap<String, BTreeMap<String, Vec<(Question, QuestionDiffKind)>>> =
        BTreeMap::new();

    for (question, diff) in questions {
        let audiences = if question.audiences.is_empty() {
            vec![InstrumentAudience::fallback()]
        } else {
            question
                .audiences
                .iter()
                .map(|audience| InstrumentAudience::parse(audience))
                .collect::<Vec<_>>()
        };

        for audience in audiences {
            let public_entry = groups.entry(audience.public).or_default();
            public_entry
                .entry(audience.column)
                .or_default()
                .push((question.clone(), diff.clone()));
        }
    }

    groups
        .into_iter()
        .map(|(public, subpublic_map)| {
            let subpublics = subpublic_map
                .keys()
                .map(|key| InstrumentColumn {
                    key: key.clone(),
                    label: display_instrument_column_label(&public, key),
                })
                .collect::<Vec<_>>();
            let mut by_question: BTreeMap<String, (Question, QuestionDiffKind)> = BTreeMap::new();
            for questions in subpublic_map.values() {
                for (question, diff) in questions {
                    by_question
                        .entry(question.code.clone())
                        .or_insert_with(|| (question.clone(), diff.clone()));
                }
            }
            InstrumentWorkbook {
                public,
                subpublics,
                questions: by_question.into_values().collect(),
            }
        })
        .collect()
}

fn slugify_file_part(value: &str) -> String {
    let slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "general".into()
    } else {
        slug
    }
}

struct ExportFormats {
    header: Format,
    title: Format,
    wrapped: Format,
    removed: Format,
    modified: Format,
    added: Format,
}

impl ExportFormats {
    fn new() -> Self {
        Self {
            header: Format::new()
                .set_bold()
                .set_text_wrap()
                .set_align(FormatAlign::VerticalCenter)
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin)
                .set_background_color(Color::RGB(0xD9EAF7)),
            title: Format::new()
                .set_bold()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_border(FormatBorder::Thin)
                .set_background_color(Color::RGB(0xD9EAF7)),
            wrapped: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::VerticalCenter)
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin),
            removed: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::VerticalCenter)
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin)
                .set_font_color(Color::RGB(0x9C0006))
                .set_background_color(Color::RGB(0xFFC7CE)),
            modified: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::VerticalCenter)
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin)
                .set_font_color(Color::RGB(0x1F4E79))
                .set_background_color(Color::RGB(0xCFE2FF)),
            added: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::VerticalCenter)
                .set_align(FormatAlign::Center)
                .set_border(FormatBorder::Thin)
                .set_font_color(Color::RGB(0x006100))
                .set_background_color(Color::RGB(0xC6EFCE)),
        }
    }
}

fn write_instrument_lineament_sheet(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    questions: &[(Question, QuestionDiffKind)],
    audiences: &[InstrumentColumn],
    public: &str,
    formats: &ExportFormats,
) -> Result<(), AppError> {
    let title = format!(
        "INSTRUMENTO {} POR LINEAMIENTOS DEL CNA",
        instrument_title_public(public)
    );
    worksheet.merge_range(
        1,
        0,
        1,
        (7 + audiences.len()) as u16,
        &title,
        &formats.title,
    )?;
    worksheet.set_freeze_panes(4, 8)?;
    worksheet.set_row_height(1, 18.75)?;
    worksheet.set_row_height(2, 24.75)?;
    worksheet.set_row_height(3, 22.5)?;
    worksheet.set_column_width(0, 11.4)?;
    worksheet.set_column_width(1, 22.1)?;
    worksheet.set_column_width(2, 14.0)?;
    worksheet.set_column_width(3, 45.6)?;
    worksheet.set_column_width(4, 10.7)?;
    worksheet.set_column_width(5, 53.1)?;
    worksheet.set_column_width(6, 16.5)?;
    worksheet.set_column_width(7, 13.1)?;
    for audience_index in 0..audiences.len() {
        worksheet.set_column_width((8 + audience_index) as u16, 49.4)?;
    }
    let fixed_headers = [
        "FACTOR",
        "",
        "CARACTERISTICA",
        "",
        "ASPECTO",
        "",
        "# Pregunta",
        "Convencion opcion de respuesta",
    ];
    for (index, header) in fixed_headers.iter().enumerate() {
        worksheet.write_string_with_format(2, index as u16, *header, &formats.header)?;
    }
    for (index, audience) in audiences.iter().enumerate() {
        worksheet.write_string_with_format(
            2,
            (8 + index) as u16,
            &audience.label,
            &formats.header,
        )?;
    }
    let subheaders = ["#", "Descripcion", "#", "Descripcion", "#", "Descripcion"];
    for (index, header) in subheaders.iter().enumerate() {
        worksheet.write_string_with_format(3, index as u16, *header, &formats.header)?;
    }

    let mut sorted = questions.iter().collect::<Vec<_>>();
    sorted.sort_by(|(left, _), (right, _)| {
        left.factor
            .cmp(&right.factor)
            .then(left.characteristic.cmp(&right.characteristic))
            .then(left.aspect.cmp(&right.aspect))
            .then_with(|| compare_question_codes(&left.code, &right.code))
    });
    for (index, (question, diff)) in sorted.into_iter().enumerate() {
        let row = (index + 4) as u32;
        let (factor_code, factor_name) = split_number_name(&question.factor);
        let (characteristic_code, characteristic_name) =
            split_number_name(&question.characteristic);
        let (aspect_code, aspect_description) = split_number_name(&question.aspect);
        let values = [
            factor_code,
            factor_name,
            characteristic_code,
            characteristic_name,
            aspect_code,
            aspect_description,
            question.code.clone(),
            question.convention_code.clone().unwrap_or_default(),
        ];
        let mut row_texts = values.clone().to_vec();
        row_texts.push(question.text.clone());
        worksheet.set_row_height(row, instrument_row_height(&row_texts))?;
        write_values_with_diff(worksheet, row, 0, &values, diff, formats)?;
        for (audience_index, audience) in audiences.iter().enumerate() {
            if question
                .audiences
                .iter()
                .any(|item| InstrumentAudience::parse(item).column == audience.key)
            {
                write_value_with_diff(
                    worksheet,
                    row,
                    (8 + audience_index) as u16,
                    &question.text,
                    diff,
                    formats,
                )?;
            }
        }
    }
    Ok(())
}

fn write_instrument_order_sheet(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    questions: &[(Question, QuestionDiffKind)],
    audiences: &[InstrumentColumn],
    public: &str,
    formats: &ExportFormats,
) -> Result<(), AppError> {
    let title = format!("INSTRUMENTO {} POR ORDEN", instrument_title_public(public));
    worksheet.merge_range(
        1,
        0,
        1,
        (1 + audiences.len()) as u16,
        &title,
        &formats.title,
    )?;
    worksheet.set_freeze_panes(3, 2)?;
    worksheet.set_row_height(1, 18.75)?;
    worksheet.set_row_height(2, 47.25)?;
    worksheet.set_column_width(0, 12)?;
    worksheet.set_column_width(1, 13.1)?;
    for audience_index in 0..audiences.len() {
        worksheet.set_column_width((2 + audience_index) as u16, 49.4)?;
    }
    worksheet.write_string_with_format(2, 0, "# Pregunta", &formats.header)?;
    worksheet.write_string_with_format(2, 1, "Convencion opcion de respuesta", &formats.header)?;
    for (index, audience) in audiences.iter().enumerate() {
        worksheet.write_string_with_format(
            2,
            (2 + index) as u16,
            &audience.label,
            &formats.header,
        )?;
    }

    let mut sorted = questions.iter().collect::<Vec<_>>();
    sorted.sort_by(|(left, _), (right, _)| compare_question_codes(&left.code, &right.code));
    for (index, (question, diff)) in sorted.into_iter().enumerate() {
        let row = (index + 3) as u32;
        worksheet.set_row_height(row, instrument_row_height(&[question.text.clone()]))?;
        write_value_with_diff(worksheet, row, 0, &question.code, diff, formats)?;
        write_value_with_diff(
            worksheet,
            row,
            1,
            question.convention_code.as_deref().unwrap_or(""),
            diff,
            formats,
        )?;
        for (audience_index, audience) in audiences.iter().enumerate() {
            if question
                .audiences
                .iter()
                .any(|item| InstrumentAudience::parse(item).column == audience.key)
            {
                write_value_with_diff(
                    worksheet,
                    row,
                    (2 + audience_index) as u16,
                    &question.text,
                    diff,
                    formats,
                )?;
            }
        }
    }
    Ok(())
}

fn write_values_with_diff(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    start_column: u16,
    values: &[String],
    diff: &QuestionDiffKind,
    formats: &ExportFormats,
) -> Result<(), AppError> {
    for (index, value) in values.iter().enumerate() {
        write_value_with_diff(
            worksheet,
            row,
            start_column + index as u16,
            value,
            diff,
            formats,
        )?;
    }
    Ok(())
}

fn write_value_with_diff(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    column: u16,
    value: &str,
    diff: &QuestionDiffKind,
    formats: &ExportFormats,
) -> Result<(), AppError> {
    match diff {
        QuestionDiffKind::Removed => {
            worksheet.write_string_with_format(row, column, value, &formats.removed)?;
        }
        QuestionDiffKind::Modified => {
            worksheet.write_string_with_format(row, column, value, &formats.modified)?;
        }
        QuestionDiffKind::Added => {
            worksheet.write_string_with_format(row, column, value, &formats.added)?;
        }
        QuestionDiffKind::Unchanged => {
            worksheet.write_string_with_format(row, column, value, &formats.wrapped)?;
        }
    }
    Ok(())
}

fn set_consolidated_dimensions(worksheet: &mut rust_xlsxwriter::Worksheet) -> Result<(), AppError> {
    let widths = [
        11.4, 16.9, 20.7, 18.7, 50.4, 10.7, 59.9, 13.6, 15.3, 16.4, 22.4, 101.6, 20.7, 26.4, 27.1,
    ];
    for (column, width) in widths.iter().enumerate() {
        worksheet.set_column_width(column as u16, *width)?;
    }
    Ok(())
}

fn consolidated_row_height(question_text: &str) -> f64 {
    if question_text.chars().count() > 180 {
        90.0
    } else {
        75.0
    }
}

fn instrument_row_height(values: &[String]) -> f64 {
    let max_len = values
        .iter()
        .map(|value| value.chars().count())
        .max()
        .unwrap_or(0);
    let estimated = ((max_len as f64 / 48.0).ceil() * 15.0).max(45.0);
    estimated.min(420.0)
}

fn compare_question_codes(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = question_code_parts(left);
    let right_parts = question_code_parts(right);
    left_parts.cmp(&right_parts).then_with(|| left.cmp(right))
}

fn question_code_parts(value: &str) -> Vec<QuestionCodePart> {
    value
        .split(|character: char| !(character.is_ascii_alphanumeric()))
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u32>()
                .map(QuestionCodePart::Number)
                .unwrap_or_else(|_| QuestionCodePart::Text(part.to_ascii_lowercase()))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum QuestionCodePart {
    Number(u32),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::compare_question_codes;

    #[test]
    fn question_codes_sort_naturally_for_instruments() {
        let mut codes = vec!["10", "2", "7.1", "7", "12.1", "12"];
        codes.sort_by(|left, right| compare_question_codes(left, right));
        assert_eq!(codes, vec!["2", "7", "7.1", "10", "12", "12.1"]);
    }
}

fn write_convention_sheet(workbook: &mut Workbook, header_format: &Format) -> Result<(), AppError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Convención")?;
    sheet.set_column_width(0, 2.7)?;
    for column in 4..=11 {
        sheet.set_column_width(column, 11.4)?;
    }
    sheet.set_column_width(12, 15.3)?;
    let headers = [
        "Convencion",
        "Calificacion",
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
    ];
    write_headers_at(sheet, 3, &headers, header_format)?;
    let rows: [[&str; 13]; 6] = [
        [
            "",
            "1",
            "Total desacuerdo",
            "Nada",
            "Muy malo",
            "Nunca",
            "Nulo",
            "Nada exigentes",
            "No favorece para nada",
            "Nada probable",
            "Nada satisfecho",
            "En ninguna medida",
            "Si",
        ],
        [
            "",
            "2",
            "Desacuerdo",
            "Poco",
            "Malo",
            "Casi nunca",
            "Bajo",
            "Poco exigentes",
            "No favorece",
            "Poco probable",
            "Poco satisfecho",
            "En muy baja medida",
            "No",
        ],
        [
            "",
            "3",
            "Medianamente de acuerdo",
            "Regular",
            "Regular",
            "Algunas veces",
            "Medio",
            "Medianamente exigentes",
            "Es indiferente",
            "Medianamente probable",
            "Medianamente satisfecho",
            "En baja medida",
            "",
        ],
        [
            "",
            "4",
            "Acuerdo",
            "Bien",
            "Bueno",
            "Casi siempre",
            "Alto",
            "Exigentes",
            "Favorece",
            "Muy probable",
            "Satisfecho",
            "En alta medida",
            "",
        ],
        [
            "",
            "5",
            "Total acuerdo",
            "Muy Bien",
            "Excelente",
            "Siempre",
            "Muy Alto",
            "Muy exigentes",
            "Favorece totalmente",
            "Totalmente probable",
            "Muy satisfecho",
            "En muy alta medida",
            "",
        ],
        [
            "",
            "NS/NA",
            "No sabe / No aplica(*)",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ],
    ];
    for (row_index, values) in rows.iter().enumerate() {
        let row = (row_index + 4) as u32;
        for (column, value) in values.iter().enumerate() {
            sheet.write_string_with_format(row, column as u16, *value, header_format)?;
        }
    }
    Ok(())
}

fn write_headers(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    headers: &[&str],
    format: &Format,
) -> Result<(), AppError> {
    write_headers_at(worksheet, 0, headers, format)
}

fn write_headers_at(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    headers: &[&str],
    format: &Format,
) -> Result<(), AppError> {
    for (column, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(row, column as u16, *header, format)?;
    }
    Ok(())
}

fn write_lineament_row(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    aspect: &GuidelineAspect,
    format: &Format,
) -> Result<(), AppError> {
    let values = [
        aspect.factor_code.as_str(),
        aspect.factor_name.as_str(),
        aspect.characteristic_code.as_str(),
        aspect.characteristic_name.as_str(),
        aspect.characteristic_name.as_str(),
        aspect.aspect_code.as_str(),
        aspect.aspect_description.as_str(),
        aspect.scope.as_str(),
        "",
        "",
        "",
        "",
        "",
        "",
        "",
    ];
    for (column, value) in values.iter().enumerate() {
        worksheet.write_string_with_format(row, column as u16, *value, format)?;
    }
    Ok(())
}

fn split_number_name(value: &str) -> (String, String) {
    let Some((number, name)) = value.split_once(". ") else {
        return (String::new(), value.to_string());
    };
    if number.trim().is_empty() || name.trim().is_empty() {
        (String::new(), value.to_string())
    } else {
        (number.trim().to_string(), name.trim().to_string())
    }
}

fn status_excel_label(status: &QuestionStatus) -> &'static str {
    match status {
        QuestionStatus::Keep => "Mantener",
        QuestionStatus::Modify => "Modificar",
        QuestionStatus::Add => "Agregar",
        QuestionStatus::Delete => "Eliminar",
    }
}
