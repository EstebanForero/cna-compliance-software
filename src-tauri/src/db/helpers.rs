use chrono::NaiveDate;
use libsql::{params, Connection};

use crate::error::AppError;

pub(super) fn parse_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|error| AppError::InvalidDate(error.to_string()))
}

pub(super) fn empty_to_none(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(super) fn join_number_name(number: String, name: String) -> String {
    match (number.is_empty(), name.is_empty()) {
        (true, true) => String::new(),
        (true, false) => name,
        (false, true) => number,
        (false, false) => format!("{number}. {name}"),
    }
}

pub(super) async fn count_related_questions(
    connection: &Connection,
    factor: &str,
    characteristic: &str,
    aspect: &str,
) -> Result<usize, AppError> {
    let mut rows = connection
        .query(
            "SELECT COUNT(*) FROM questions
             WHERE factor = ?1 AND characteristic = ?2 AND aspect = ?3",
            params![factor, characteristic, aspect],
        )
        .await?;
    let Some(row) = rows.next().await? else {
        return Ok(0);
    };
    Ok(row.get::<i64>(0)? as usize)
}

pub(super) async fn find_question_id(
    connection: &Connection,
    code: &str,
) -> Result<Option<String>, AppError> {
    let mut rows = connection
        .query("SELECT id FROM questions WHERE code = ?1 LIMIT 1", [code])
        .await?;
    rows.next()
        .await?
        .map(|row| row.get::<String>(0))
        .transpose()
        .map_err(Into::into)
}

pub(super) async fn find_guideline_aspect_id(
    connection: &Connection,
    scope: &str,
    factor_code: &str,
    characteristic_code: &str,
    aspect_code: &str,
) -> Result<Option<String>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id FROM guideline_aspects
             WHERE scope = ?1
               AND factor_code = ?2
               AND characteristic_code = ?3
               AND aspect_code = ?4
             LIMIT 1",
            params![scope, factor_code, characteristic_code, aspect_code],
        )
        .await?;
    rows.next()
        .await?
        .map(|row| row.get::<String>(0))
        .transpose()
        .map_err(Into::into)
}
