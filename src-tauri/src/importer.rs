use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use calamine::{open_workbook_auto, Reader};

use crate::audience::audience_from_excel;
#[cfg(test)]
use crate::audience::normalize_audience_label;
use crate::domain::{
    normalize_aspect_code, CnaFactorCode, NewGuidelineAspect, NewQuestion, QuestionScope,
    QuestionStatus,
};
use crate::error::AppError;

mod columns;
mod marks;

use columns::{cell_to_string, find_header, get, normalize_header, parse_scope, parse_status};
#[cfg(test)]
use marks::xlsx_red_cells_by_sheet;
use marks::{xlsx_marked_cells_by_sheet, SheetMarks};

#[derive(Debug)]
pub struct ParsedWorkbook {
    pub sheet_name: String,
    pub questions: Vec<NewQuestion>,
    pub guideline_aspects: Vec<NewGuidelineAspect>,
    pub skipped_rows: usize,
    pub detected_columns: Vec<String>,
}

pub fn parse_questions_workbook(path: &Path) -> Result<ParsedWorkbook, AppError> {
    let mut workbook = open_workbook_auto(path)?;
    let mut merged = WorkbookAccumulator::default();
    let marked_cells_by_sheet = xlsx_marked_cells_by_sheet(path).unwrap_or_default();

    for sheet in workbook.sheet_names().to_owned() {
        let range = workbook.worksheet_range(&sheet)?;
        let rows: Vec<Vec<String>> = range
            .rows()
            .map(|row| row.iter().map(cell_to_string).collect())
            .collect();
        let Some((header_index, columns)) = find_header(&rows) else {
            continue;
        };

        let marked_cells = marked_cells_by_sheet
            .get(&sheet)
            .cloned()
            .unwrap_or_default();
        let parsed = parse_sheet(
            &sheet,
            &rows[(header_index + 1)..],
            columns,
            &marked_cells,
            header_index,
        )?;
        merged.absorb(parsed);
    }

    merged.finish().ok_or_else(|| {
        AppError::Validation(
            "No sheet with question consolidated headers was found in the workbook".into(),
        )
    })
}

fn parse_sheet(
    sheet_name: &str,
    rows: &[Vec<String>],
    columns: HashMap<String, usize>,
    marked_cells: &SheetMarks,
    header_index: usize,
) -> Result<ParsedWorkbook, AppError> {
    let mut grouped: HashMap<String, NewQuestion> = HashMap::new();
    let mut aspects: HashMap<AspectKey, NewGuidelineAspect> = HashMap::new();
    let mut skipped_rows = 0;
    let mut last_factor_code = String::new();
    let mut last_factor_name = String::new();
    let mut last_characteristic_code = String::new();
    let mut last_characteristic_name = String::new();
    let mut last_aspect_code = String::new();
    let mut last_aspect_description = String::new();

    for (offset, row) in rows.iter().enumerate() {
        let excel_row = header_index + offset + 2;
        let status = parse_status(&get(row, &columns, "estado pregunta"));
        if matches!(status, QuestionStatus::Delete)
            || row_has_red_import_signal(marked_cells, &columns, excel_row)
        {
            skipped_rows += 1;
            continue;
        }
        let status = status_from_marked_row(marked_cells, &columns, excel_row).unwrap_or(status);

        let scope = parse_scope(&get(row, &columns, "tipo pregunta"));
        let factor_code = forward_fill(get(row, &columns, "n factor"), &mut last_factor_code);
        let factor_name = forward_fill(
            get(row, &columns, "descripcion factor"),
            &mut last_factor_name,
        );
        let characteristic_code = forward_fill(
            get(row, &columns, "n caracteristica"),
            &mut last_characteristic_code,
        );
        let characteristic_name = forward_fill(
            get(row, &columns, "nombre caracteristica"),
            &mut last_characteristic_name,
        );
        let aspect_code = forward_fill(get(row, &columns, "n aspecto"), &mut last_aspect_code);
        let aspect_description = forward_fill(
            get(row, &columns, "descripcion aspecto"),
            &mut last_aspect_description,
        );

        let typed_factor_code = if factor_code.is_empty() {
            None
        } else {
            Some(parse_factor_code(&factor_code, sheet_name, offset + 2)?)
        };
        let normalized_aspect_code = typed_factor_code.as_ref().map(|typed_factor_code| {
            normalize_aspect_code(
                typed_factor_code,
                &characteristic_code,
                &aspect_code,
                &aspect_description,
            )
        });

        if !aspect_description.is_empty()
            && !factor_code.is_empty()
            && (!factor_name.is_empty() || !characteristic_name.is_empty())
        {
            let typed_factor_code = typed_factor_code
                .clone()
                .expect("factor code checked above");
            let aspect_code = normalized_aspect_code
                .clone()
                .expect("factor code checked above");
            let aspect_key = AspectKey::new(
                scope.clone(),
                typed_factor_code.clone(),
                &characteristic_code,
                &aspect_code,
            );
            merge_aspect(
                &mut aspects,
                aspect_key,
                NewGuidelineAspect {
                    guideline_title: "Lineamiento CNA importado".into(),
                    scope: scope.clone(),
                    factor_code: typed_factor_code.clone(),
                    factor_name: factor_name.clone(),
                    characteristic_code: characteristic_code.clone(),
                    characteristic_name: characteristic_name.clone(),
                    aspect_code: aspect_code.clone(),
                    aspect_description: aspect_description.clone(),
                    requires_appreciation: true,
                },
            );
        }

        let text = get(row, &columns, "pregunta");
        if text.is_empty() {
            skipped_rows += 1;
            continue;
        }
        if factor_code.is_empty()
            || characteristic_code.is_empty()
            || characteristic_name.is_empty()
            || aspect_description.is_empty()
        {
            skipped_rows += 1;
            continue;
        }
        let aspect_code = normalized_aspect_code.unwrap_or(aspect_code);

        let code = get(row, &columns, "n pregunta");
        let code = if code.is_empty() {
            stable_code(&text)
        } else {
            code
        };
        let audience = audience_from_excel(
            get(row, &columns, "publico"),
            get(row, &columns, "tipo de publico"),
        );

        let key = code.clone();
        let convention_code = none_if_empty(get(row, &columns, "convencion opcion de respuesta"));
        let format = question_format_from_convention(convention_code.as_deref(), &text);

        let entry = grouped.entry(key).or_insert_with(|| NewQuestion {
            code,
            text,
            scope,
            format,
            convention_code,
            status,
            factor: join_number_name(factor_code, factor_name),
            characteristic: join_number_name(characteristic_code, characteristic_name),
            aspect: join_number_name(aspect_code, aspect_description),
            audiences: Vec::new(),
            justification: none_if_empty(get(row, &columns, "observaciones")),
        });

        if !audience.is_empty() && !entry.audiences.iter().any(|value| value == &audience) {
            entry.audiences.push(audience);
        }
    }

    let mut questions: Vec<NewQuestion> = grouped.into_values().collect();
    questions.sort_by(|left, right| left.code.cmp(&right.code));
    let mut guideline_aspects: Vec<NewGuidelineAspect> = aspects.into_values().collect();
    guideline_aspects.sort_by(|left, right| {
        left.factor_code
            .cmp(&right.factor_code)
            .then(left.characteristic_code.cmp(&right.characteristic_code))
            .then(left.aspect_code.cmp(&right.aspect_code))
    });

    Ok(ParsedWorkbook {
        sheet_name: sheet_name.into(),
        questions,
        guideline_aspects,
        skipped_rows,
        detected_columns: columns.keys().cloned().collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AspectKey {
    scope: QuestionScope,
    factor_code: CnaFactorCode,
    characteristic_code: String,
    aspect_code: String,
}

impl AspectKey {
    fn new(
        scope: QuestionScope,
        factor_code: CnaFactorCode,
        characteristic_code: &str,
        aspect_code: &str,
    ) -> Self {
        Self {
            scope,
            factor_code,
            characteristic_code: normalize_key(characteristic_code),
            aspect_code: normalize_key(aspect_code),
        }
    }
}

#[derive(Default)]
struct WorkbookAccumulator {
    sheet_names: Vec<String>,
    questions: HashMap<String, NewQuestion>,
    guideline_aspects: HashMap<AspectKey, NewGuidelineAspect>,
    skipped_rows: usize,
    detected_columns: BTreeSet<String>,
}

impl WorkbookAccumulator {
    fn absorb(&mut self, parsed: ParsedWorkbook) {
        self.sheet_names.push(parsed.sheet_name);
        self.skipped_rows += parsed.skipped_rows;
        self.detected_columns.extend(parsed.detected_columns);

        for aspect in parsed.guideline_aspects {
            let key = AspectKey::new(
                aspect.scope.clone(),
                aspect.factor_code.clone(),
                &aspect.characteristic_code,
                &aspect.aspect_code,
            );
            merge_aspect(&mut self.guideline_aspects, key, aspect);
        }

        for question in parsed.questions {
            merge_question(&mut self.questions, question);
        }
    }

    fn finish(self) -> Option<ParsedWorkbook> {
        if self.sheet_names.is_empty() {
            return None;
        }

        let mut questions: Vec<NewQuestion> = self.questions.into_values().collect();
        questions.sort_by(|left, right| {
            natural_code_parts(&left.code).cmp(&natural_code_parts(&right.code))
        });

        let mut guideline_aspects: Vec<NewGuidelineAspect> =
            self.guideline_aspects.into_values().collect();
        guideline_aspects.sort_by(|left, right| {
            left.factor_code
                .cmp(&right.factor_code)
                .then(
                    natural_code_parts(&left.characteristic_code)
                        .cmp(&natural_code_parts(&right.characteristic_code)),
                )
                .then(
                    natural_code_parts(&left.aspect_code)
                        .cmp(&natural_code_parts(&right.aspect_code)),
                )
        });

        Some(ParsedWorkbook {
            sheet_name: self.sheet_names.join(", "),
            questions,
            guideline_aspects,
            skipped_rows: self.skipped_rows,
            detected_columns: self.detected_columns.into_iter().collect(),
        })
    }
}

fn merge_aspect(
    aspects: &mut HashMap<AspectKey, NewGuidelineAspect>,
    key: AspectKey,
    incoming: NewGuidelineAspect,
) {
    aspects
        .entry(key)
        .and_modify(|current| {
            if current.factor_name.trim().is_empty() {
                current.factor_name = incoming.factor_name.clone();
            }
            if current.characteristic_name.trim().is_empty() {
                current.characteristic_name = incoming.characteristic_name.clone();
            }
            if incoming.aspect_description.len() > current.aspect_description.len() {
                current.aspect_description = incoming.aspect_description.clone();
            }
            current.requires_appreciation |= incoming.requires_appreciation;
        })
        .or_insert(incoming);
}

fn merge_question(questions: &mut HashMap<String, NewQuestion>, incoming: NewQuestion) {
    questions
        .entry(incoming.code.clone())
        .and_modify(|current| {
            if incoming.text.len() > current.text.len() {
                current.text = incoming.text.clone();
            }
            if current.convention_code.is_none() {
                current.convention_code = incoming.convention_code.clone();
            }
            if current.factor.trim().is_empty() {
                current.factor = incoming.factor.clone();
            }
            if current.characteristic.trim().is_empty() {
                current.characteristic = incoming.characteristic.clone();
            }
            if current.aspect.trim().is_empty() {
                current.aspect = incoming.aspect.clone();
            }
            for audience in &incoming.audiences {
                if !current.audiences.iter().any(|value| value == audience) {
                    current.audiences.push(audience.clone());
                }
            }
            if matches!(current.status, QuestionStatus::Keep)
                && !matches!(incoming.status, QuestionStatus::Keep)
            {
                current.status = incoming.status.clone();
            }
            if current
                .justification
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                current.justification = incoming.justification.clone();
            }
        })
        .or_insert(incoming);
}

fn forward_fill(value: String, last_seen: &mut String) -> String {
    if value.trim().is_empty() {
        last_seen.clone()
    } else {
        *last_seen = value.clone();
        value
    }
}

fn parse_factor_code(
    value: &str,
    sheet_name: &str,
    row_number: usize,
) -> Result<CnaFactorCode, AppError> {
    CnaFactorCode::from_str(value).ok_or_else(|| {
        AppError::Validation(format!(
            "Invalid CNA factor code '{value}' in sheet '{sheet_name}', row {row_number}. Expected a non-empty factor code."
        ))
    })
}

fn normalize_key(value: &str) -> String {
    normalize_header(value)
        .replace('.', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn natural_code_parts(value: &str) -> Vec<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

fn join_number_name(number: String, name: String) -> String {
    match (number.is_empty(), name.is_empty()) {
        (true, true) => String::new(),
        (true, false) => name,
        (false, true) => number,
        (false, false) => format!("{number}. {name}"),
    }
}

fn none_if_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn question_format_from_convention(convention: Option<&str>, text: &str) -> String {
    let convention = convention.unwrap_or_default();
    let normalized_convention = normalize_header(convention);
    let normalized_text = normalize_header(text);

    if normalized_convention.contains("abierta") {
        return "open".into();
    }

    let convention_parts = normalized_convention.split_whitespace().collect::<Vec<_>>();
    let has_combined_convention_codes = convention_parts.len() > 1
        && convention_parts.iter().all(|part| {
            part.chars()
                .all(|character| character.is_ascii_alphabetic())
        });

    if convention
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        > 1
        || has_combined_convention_codes
        || normalized_text.contains("seleccione")
        || normalized_text.contains("marque")
        || normalized_text.contains("cuales")
    {
        return "multipleChoice".into();
    }

    if normalized_convention == "k" || normalized_convention == "si no" {
        return "singleChoice".into();
    }

    "likert".into()
}

fn stable_code(text: &str) -> String {
    let words = normalize_header(text)
        .split_whitespace()
        .take(5)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("-");
    format!("IMP-{words}")
}

fn row_has_red_import_signal(
    marked_cells: &SheetMarks,
    columns: &HashMap<String, usize>,
    excel_row: usize,
) -> bool {
    [
        "n factor",
        "descripcion factor",
        "n caracteristica",
        "nombre caracteristica",
        "n aspecto",
        "descripcion aspecto",
        "estado pregunta",
        "n pregunta",
        "pregunta",
    ]
    .iter()
    .filter_map(|key| columns.get(*key))
    .any(|column| marked_cells.red.contains(&(excel_row, *column)))
}

fn status_from_marked_row(
    marked_cells: &SheetMarks,
    columns: &HashMap<String, usize>,
    excel_row: usize,
) -> Option<QuestionStatus> {
    let question_columns = ["estado pregunta", "n pregunta", "pregunta"];
    if question_columns
        .iter()
        .filter_map(|key| columns.get(*key))
        .any(|column| marked_cells.green.contains(&(excel_row, *column)))
    {
        return Some(QuestionStatus::Add);
    }
    if question_columns
        .iter()
        .filter_map(|key| columns.get(*key))
        .any(|column| marked_cells.blue.contains(&(excel_row, *column)))
    {
        return Some(QuestionStatus::Modify);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_values_from_spanish_excel_labels() {
        assert_eq!(parse_status("Modificar"), QuestionStatus::Modify);
        assert_eq!(parse_status("Agregar"), QuestionStatus::Add);
        assert_eq!(parse_status("Eliminar"), QuestionStatus::Delete);
        assert_eq!(parse_status("Mantener"), QuestionStatus::Keep);
    }

    #[test]
    fn categorizes_question_format_from_convention() {
        assert_eq!(
            question_format_from_convention(Some("Abierta"), "Explique su respuesta"),
            "open"
        );
        assert_eq!(
            question_format_from_convention(Some("K"), "¿Cuenta con apoyo institucional?"),
            "singleChoice"
        );
        assert_eq!(
            question_format_from_convention(Some("B\nE"), "Seleccione los recursos usados"),
            "multipleChoice"
        );
        assert_eq!(
            question_format_from_convention(Some("A"), "Califique su nivel de acuerdo"),
            "likert"
        );
    }

    #[test]
    fn normalizes_audience_labels_from_versioned_sheets() {
        assert_eq!(normalize_audience_label("10Pregrado"), "Pregrado");
        assert_eq!(
            normalize_audience_label("1Profesores_Planta"),
            "Profesores Planta"
        );
        assert_eq!(
            normalize_audience_label("Unidad académica"),
            "Unidad académica"
        );
        assert_eq!(
            audience_from_excel("0Estudiantes".into(), "00Pregrado".into()),
            "Estudiantes Pregrado"
        );
    }

    #[test]
    fn imports_example_consolidated_workbook() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../example-files/Consolidado de preguntas Enc de Aut Ins y Pr 2024 1.xlsx");
        if !path.exists() {
            return;
        }

        let workbook = parse_questions_workbook(&path).unwrap();

        assert!(workbook.questions.len() > 100);
        assert!(workbook.guideline_aspects.len() > 20);
        assert!(workbook
            .detected_columns
            .iter()
            .any(|column| column == "pregunta"));
        assert!(workbook
            .questions
            .iter()
            .any(|question| !question.audiences.is_empty()));
        assert!(workbook.questions.iter().any(|question| question
            .audiences
            .iter()
            .any(|audience| audience == "Estudiantes Pregrado")));
        assert!(workbook
            .questions
            .iter()
            .all(|question| question.status != QuestionStatus::Delete));
        assert!(workbook.guideline_aspects.iter().all(|aspect| !aspect
            .characteristic_code
            .trim()
            .is_empty()
            && !aspect.characteristic_name.trim().is_empty()));
        let missing_question_characteristics = workbook
            .questions
            .iter()
            .filter(|question| question.characteristic.trim().is_empty())
            .map(|question| {
                format!(
                    "{} | factor='{}' | aspect='{}' | text='{}'",
                    question.code, question.factor, question.aspect, question.text
                )
            })
            .collect::<Vec<_>>();
        assert!(
            missing_question_characteristics.is_empty(),
            "questions without characteristic: {missing_question_characteristics:?}"
        );
        assert!(
            workbook
                .guideline_aspects
                .iter()
                .map(|aspect| (
                    aspect.factor_code.as_str().to_string(),
                    normalize_key(&aspect.characteristic_code),
                    normalize_key(&aspect.characteristic_name),
                ))
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 20
        );
        assert!(
            workbook
                .questions
                .iter()
                .map(|question| normalize_key(&question.characteristic))
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 20
        );
        assert!(workbook
            .questions
            .iter()
            .any(|question| question.format == "open"));
        assert!(workbook
            .questions
            .iter()
            .any(|question| question.format == "likert"));

        let mut unique_questions = std::collections::HashSet::new();
        for question in &workbook.questions {
            assert!(unique_questions.insert(question.code.clone()));
        }

        let mut unique_aspects = std::collections::HashSet::new();
        for aspect in &workbook.guideline_aspects {
            assert!(unique_aspects.insert((
                aspect.scope.as_str().to_string(),
                aspect.factor_code.as_str().to_string(),
                normalize_key(&aspect.characteristic_code),
                normalize_key(&aspect.aspect_code),
            )));
        }
    }

    #[test]
    fn links_long_appreciation_lineament_to_questions() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../example-files/Consolidado de preguntas Enc de Aut Ins y Pr 2024 1.xlsx");
        if !path.exists() {
            return;
        }

        let workbook = parse_questions_workbook(&path).unwrap();
        let aspect = workbook
            .guideline_aspects
            .iter()
            .find(|aspect| {
                aspect.aspect_description.contains(
                    "Apreciación por parte de la comunidad académica sobre las orientaciones",
                )
            })
            .expect("long appreciation lineament should be imported");
        let related = workbook
            .questions
            .iter()
            .filter(|question| {
                split_joined_code(&question.factor) == aspect.factor_code.as_str()
                    && split_joined_code(&question.characteristic) == aspect.characteristic_code
                    && split_joined_code(&question.aspect) == aspect.aspect_code
            })
            .count();

        assert!(
            related > 0,
            "long appreciation lineament should have related questions"
        );
    }

    #[test]
    fn detects_red_cells_in_example_consolidated_workbook() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../example-files/Consolidado de preguntas Enc de Aut Ins y Pr 2024 1.xlsx");
        if !path.exists() {
            return;
        }

        let red_cells = xlsx_red_cells_by_sheet(&path).unwrap();
        assert!(red_cells
            .iter()
            .any(|(sheet, cells)| sheet.trim() == "BASE" && !cells.is_empty()));
        assert!(red_cells
            .get("BASEvs2")
            .is_some_and(|cells| !cells.is_empty()));
        assert!(red_cells
            .get("BASEvs3")
            .is_some_and(|cells| !cells.is_empty()));
    }

    #[tokio::test]
    async fn imports_example_consolidated_workbook_into_libsql_database() {
        use crate::db::LibSqlAutoEvalRepository;
        use crate::repository::AutoEvalRepository;

        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../example-files/Consolidado de preguntas Enc de Aut Ins y Pr 2024 1.xlsx");
        if !path.exists() {
            return;
        }
        let db_path =
            std::env::temp_dir().join(format!("autoevaluacion-import-{}.db", uuid::Uuid::new_v4()));
        let repository = LibSqlAutoEvalRepository::open(&db_path).await.unwrap();
        let workbook = parse_questions_workbook(&path).unwrap();
        let expected = workbook.questions.len();
        let expected_aspects = workbook.guideline_aspects.len();

        repository.ensure_cycle("Test import").await.unwrap();
        let imported_aspects = repository
            .upsert_guideline_aspects(workbook.guideline_aspects)
            .await
            .unwrap();
        let imported = repository
            .upsert_questions(workbook.questions)
            .await
            .unwrap();
        let questions = repository.list_questions().await.unwrap();
        let aspects = repository.list_guideline_aspects().await.unwrap();

        assert_eq!(imported, expected);
        assert_eq!(imported_aspects, expected_aspects);
        assert_eq!(questions.len(), expected);
        assert_eq!(aspects.len(), expected_aspects);
        assert!(questions
            .iter()
            .any(|question| question.convention_code.is_some()));
        assert!(aspects
            .iter()
            .all(|aspect| !aspect.characteristic_code.trim().is_empty()
                && !aspect.characteristic_name.trim().is_empty()));
        let missing_question_characteristics = questions
            .iter()
            .filter(|question| question.characteristic.trim().is_empty())
            .map(|question| question.code.clone())
            .collect::<Vec<_>>();
        assert!(
            missing_question_characteristics.is_empty(),
            "questions without characteristic after db import: {missing_question_characteristics:?}"
        );
        assert!(
            aspects
                .iter()
                .map(|aspect| (
                    aspect.factor_code.as_str().to_string(),
                    normalize_key(&aspect.characteristic_code),
                    normalize_key(&aspect.characteristic_name),
                ))
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 20
        );

        let _ = std::fs::remove_file(db_path);
    }

    fn split_joined_code(value: &str) -> String {
        value
            .split_once(". ")
            .map(|(code, _)| code.trim().to_string())
            .unwrap_or_else(|| value.trim().to_string())
    }
}
