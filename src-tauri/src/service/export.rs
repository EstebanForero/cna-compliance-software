use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

use crate::domain::{
    ExportKind, ExportWorkbookRequest, ExportWorkbookResult, GuidelineAspect, Question,
    QuestionDiffKind, QuestionStatus,
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
            ExportKind::Instruments => {
                write_instruments_workbook(&request.path, &diffed, &aspects)?
            }
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
        "N° Aspecto",
        "Descripción Aspecto",
        "Tipo pregunta",
        "Estado pregunta",
        "Convención opción de respuesta",
        "N° pregunta",
        "Pregunta",
        "Público",
        "Tipo de público",
    ];

    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Consolidado")?;
    write_headers(worksheet, &headers, &formats.header)?;

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
            let values = [
                factor_code.as_str(),
                factor_name.as_str(),
                characteristic_code.as_str(),
                characteristic_name.as_str(),
                aspect_code.as_str(),
                aspect_description.as_str(),
                question.scope.as_str(),
                status_excel_label(&question.status),
                question.convention_code.as_deref().unwrap_or(""),
                question.code.as_str(),
                question.text.as_str(),
                audience.as_str(),
                audience.as_str(),
            ];

            for (column, value) in values.iter().enumerate() {
                if let Some(cell_format) = format {
                    worksheet.write_string_with_format(row, column as u16, *value, cell_format)?;
                } else {
                    worksheet.write_string(row, column as u16, *value)?;
                }
            }
            row += 1;
        }
    }

    for aspect in aspects {
        write_lineament_row(worksheet, row, aspect)?;
        row += 1;
    }

    let lineaments = workbook.add_worksheet();
    lineaments.set_name("Lineamientos")?;
    write_headers(lineaments, &headers, &formats.header)?;
    for (index, aspect) in aspects.iter().enumerate() {
        write_lineament_row(lineaments, (index + 1) as u32, aspect)?;
    }
    write_convention_sheet(&mut workbook, &formats.header)?;
    workbook.save(path)?;
    Ok(())
}

fn write_instruments_workbook(
    path: &str,
    questions: &[(Question, QuestionDiffKind)],
    _aspects: &[GuidelineAspect],
) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let formats = ExportFormats::new();
    let mut audiences = questions
        .iter()
        .flat_map(|(question, _)| question.audiences.clone())
        .collect::<Vec<_>>();
    audiences.sort();
    audiences.dedup();

    let by_lineament = workbook.add_worksheet();
    by_lineament.set_name("Por lineamiento")?;
    write_instrument_lineament_sheet(by_lineament, questions, &audiences, &formats)?;

    let by_order = workbook.add_worksheet();
    by_order.set_name("Por orden")?;
    write_instrument_order_sheet(by_order, questions, &audiences, &formats)?;

    write_convention_sheet(&mut workbook, &formats.header)?;
    workbook.save(path)?;
    Ok(())
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
                .set_background_color(Color::RGB(0xE9EEF8)),
            title: Format::new()
                .set_bold()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_background_color(Color::RGB(0xD9EAF7)),
            wrapped: Format::new().set_text_wrap().set_align(FormatAlign::Top),
            removed: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::Top)
                .set_background_color(Color::RGB(0xFFC7CE)),
            modified: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::Top)
                .set_background_color(Color::RGB(0xCFE2FF)),
            added: Format::new()
                .set_text_wrap()
                .set_align(FormatAlign::Top)
                .set_background_color(Color::RGB(0xC6EFCE)),
        }
    }
}

fn write_instrument_lineament_sheet(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    questions: &[(Question, QuestionDiffKind)],
    audiences: &[String],
    formats: &ExportFormats,
) -> Result<(), AppError> {
    worksheet.merge_range(
        0,
        0,
        0,
        (7 + audiences.len()) as u16,
        "INSTRUMENTO POR LINEAMIENTOS DEL CNA - TODOS LOS PUBLICOS",
        &formats.title,
    )?;
    worksheet.set_freeze_panes(3, 8)?;
    worksheet.set_row_height(0, 24)?;
    worksheet.set_row_height(1, 34)?;
    worksheet.set_row_height(2, 24)?;
    worksheet.set_column_width(0, 8)?;
    worksheet.set_column_width(1, 26)?;
    worksheet.set_column_width(2, 10)?;
    worksheet.set_column_width(3, 34)?;
    worksheet.set_column_width(4, 10)?;
    worksheet.set_column_width(5, 44)?;
    worksheet.set_column_width(6, 12)?;
    worksheet.set_column_width(7, 16)?;
    for audience_index in 0..audiences.len() {
        worksheet.set_column_width((8 + audience_index) as u16, 44)?;
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
        worksheet.write_string_with_format(1, index as u16, *header, &formats.header)?;
    }
    for (index, audience) in audiences.iter().enumerate() {
        worksheet.write_string_with_format(1, (8 + index) as u16, audience, &formats.header)?;
    }
    let subheaders = ["#", "Descripcion", "#", "Descripcion", "#", "Descripcion"];
    for (index, header) in subheaders.iter().enumerate() {
        worksheet.write_string_with_format(2, index as u16, *header, &formats.header)?;
    }

    let mut sorted = questions.iter().collect::<Vec<_>>();
    sorted.sort_by(|(left, _), (right, _)| {
        left.factor
            .cmp(&right.factor)
            .then(left.characteristic.cmp(&right.characteristic))
            .then(left.aspect.cmp(&right.aspect))
            .then(left.code.cmp(&right.code))
    });
    for (index, (question, diff)) in sorted.into_iter().enumerate() {
        let row = (index + 3) as u32;
        worksheet.set_row_height(row, 88)?;
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
        write_values_with_diff(worksheet, row, 0, &values, diff, formats)?;
        for (audience_index, audience) in audiences.iter().enumerate() {
            if question.audiences.iter().any(|item| item == audience) {
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
    audiences: &[String],
    formats: &ExportFormats,
) -> Result<(), AppError> {
    worksheet.merge_range(
        0,
        0,
        0,
        (1 + audiences.len()) as u16,
        "INSTRUMENTO POR ORDEN - TODOS LOS PUBLICOS",
        &formats.title,
    )?;
    worksheet.set_freeze_panes(2, 2)?;
    worksheet.set_row_height(0, 24)?;
    worksheet.set_row_height(1, 34)?;
    worksheet.set_column_width(0, 12)?;
    worksheet.set_column_width(1, 18)?;
    for audience_index in 0..audiences.len() {
        worksheet.set_column_width((2 + audience_index) as u16, 44)?;
    }
    worksheet.write_string_with_format(1, 0, "# Pregunta", &formats.header)?;
    worksheet.write_string_with_format(1, 1, "Convencion opcion de respuesta", &formats.header)?;
    for (index, audience) in audiences.iter().enumerate() {
        worksheet.write_string_with_format(1, (2 + index) as u16, audience, &formats.header)?;
    }

    let mut sorted = questions.iter().collect::<Vec<_>>();
    sorted.sort_by(|(left, _), (right, _)| left.code.cmp(&right.code));
    for (index, (question, diff)) in sorted.into_iter().enumerate() {
        let row = (index + 2) as u32;
        worksheet.set_row_height(row, 88)?;
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
            if question.audiences.iter().any(|item| item == audience) {
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

fn write_convention_sheet(workbook: &mut Workbook, header_format: &Format) -> Result<(), AppError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("Convención")?;
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
    write_headers(sheet, &headers, header_format)?;
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
        let row = (row_index + 1) as u32;
        for (column, value) in values.iter().enumerate() {
            sheet.write_string(row, column as u16, *value)?;
        }
    }
    Ok(())
}

fn write_headers(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    headers: &[&str],
    format: &Format,
) -> Result<(), AppError> {
    for (column, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, column as u16, *header, format)?;
    }
    Ok(())
}

fn write_lineament_row(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    aspect: &GuidelineAspect,
) -> Result<(), AppError> {
    let values = [
        aspect.factor_code.as_str(),
        aspect.factor_name.as_str(),
        aspect.characteristic_code.as_str(),
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
    ];
    for (column, value) in values.iter().enumerate() {
        worksheet.write_string(row, column as u16, *value)?;
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
