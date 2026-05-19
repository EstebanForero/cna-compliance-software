use std::collections::HashMap;

use calamine::Data;

use crate::domain::{QuestionScope, QuestionStatus};

pub fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.trim().to_string(),
        Data::Float(value) => trim_number(*value),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::Error(value) => format!("{value:?}"),
    }
}

pub fn find_header(rows: &[Vec<String>]) -> Option<(usize, HashMap<String, usize>)> {
    for (index, row) in rows.iter().take(20).enumerate() {
        let mut columns = HashMap::new();
        for (column_index, value) in row.iter().enumerate() {
            let normalized = normalize_header(value);
            if !normalized.is_empty() {
                columns.insert(normalized, column_index);
            }
        }

        let mapped = alias_columns(columns);
        if mapped.contains_key("pregunta")
            && (mapped.contains_key("publico") || mapped.contains_key("tipo de publico"))
            && mapped.contains_key("n factor")
            && mapped.contains_key("n caracteristica")
            && mapped.contains_key("descripcion aspecto")
        {
            return Some((index, mapped));
        }
    }

    None
}

pub fn get(row: &[String], columns: &HashMap<String, usize>, key: &str) -> String {
    columns
        .get(key)
        .and_then(|index| row.get(*index))
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

pub fn parse_scope(value: &str) -> QuestionScope {
    let normalized = normalize_header(value);
    if normalized.contains("programa") || normalized == "program" {
        QuestionScope::Program
    } else {
        QuestionScope::Institutional
    }
}

pub fn parse_status(value: &str) -> QuestionStatus {
    match normalize_header(value).as_str() {
        "modificar" | "modify" => QuestionStatus::Modify,
        "agregar" | "add" => QuestionStatus::Add,
        "eliminar" | "delete" => QuestionStatus::Delete,
        _ => QuestionStatus::Keep,
    }
}

pub fn normalize_header(value: &str) -> String {
    value
        .replace(['\n', '\r'], " ")
        .replace('°', "")
        .replace('ú', "u")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('á', "a")
        .replace('é', "e")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn alias_columns(columns: HashMap<String, usize>) -> HashMap<String, usize> {
    let aliases: [(&str, &[&str]); 13] = [
        ("n factor", &["n factor", "factor"]),
        ("descripcion factor", &["descripcion factor"]),
        ("n caracteristica", &["n caracteristica"]),
        ("nombre caracteristica", &["nombre caracteristica"]),
        ("n aspecto", &["n aspecto"]),
        ("descripcion aspecto", &["descripcion aspecto"]),
        ("tipo pregunta", &["tipo pregunta"]),
        ("estado pregunta", &["estado pregunta"]),
        (
            "convencion opcion de respuesta",
            &["convencion opcion de respuesta", "convencion"],
        ),
        ("n pregunta", &["n pregunta"]),
        ("pregunta", &["pregunta"]),
        ("publico", &["publico"]),
        ("tipo de publico", &["tipo de publico"]),
    ];

    let mut mapped = HashMap::new();
    for (canonical, options) in aliases {
        for option in options {
            if let Some(index) = columns.get(*option) {
                mapped.insert(canonical.into(), *index);
                break;
            }
        }
    }
    mapped
}

fn trim_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}
