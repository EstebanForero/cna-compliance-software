use chrono::{DateTime, Utc};

use crate::db::helpers::empty_to_none;
use crate::domain::{
    ChangeLogEntry, HistorySnapshot, OriginalQuestionSnapshot, ProviderQuestionReview,
    ProviderQuestionReviewStatus, Question, QuestionScope, QuestionStatus, SourceDocument,
};
use crate::error::AppError;

pub(super) fn parse_question_row(row: &libsql::Row) -> Result<Question, AppError> {
    let convention_code = empty_to_none(row.get::<String>(5)?);
    let justification = empty_to_none(row.get::<String>(11)?);
    let updated_at = parse_rfc3339_utc(&row.get::<String>(12)?)?;

    Ok(Question {
        id: row.get(0)?,
        code: row.get(1)?,
        text: row.get(2)?,
        scope: QuestionScope::from_str(&row.get::<String>(3)?),
        format: row.get(4)?,
        convention_code,
        status: QuestionStatus::from_str(&row.get::<String>(6)?),
        factor: row.get(7)?,
        characteristic: row.get(8)?,
        aspect: row.get(9)?,
        audiences: serde_json::from_str(&row.get::<String>(10)?)?,
        justification,
        updated_at,
    })
}

pub(super) fn parse_source_document_row(row: &libsql::Row) -> Result<SourceDocument, AppError> {
    let imported_at = parse_rfc3339_utc(&row.get::<String>(4)?)?;

    Ok(SourceDocument {
        id: row.get(0)?,
        file_name: row.get(1)?,
        path: row.get(2)?,
        document_type: row.get(3)?,
        imported_at,
        imported_rows: row.get::<i64>(5)? as usize,
        skipped_rows: row.get::<i64>(6)? as usize,
    })
}

pub(super) fn parse_original_snapshot_row(
    row: &libsql::Row,
) -> Result<OriginalQuestionSnapshot, AppError> {
    let marked_at = parse_rfc3339_utc(&row.get::<String>(14)?)?;

    Ok(OriginalQuestionSnapshot {
        id: row.get(0)?,
        question_id: row.get(1)?,
        source_document_id: row.get(2)?,
        code: row.get(3)?,
        text: row.get(4)?,
        scope: QuestionScope::from_str(&row.get::<String>(5)?),
        format: row.get(6)?,
        convention_code: empty_to_none(row.get::<String>(7)?),
        status: QuestionStatus::from_str(&row.get::<String>(8)?),
        factor: row.get(9)?,
        characteristic: row.get(10)?,
        aspect: row.get(11)?,
        audiences: serde_json::from_str(&row.get::<String>(12)?)?,
        content_hash: row.get(13)?,
        marked_by: row.get(15)?,
        marked_at,
    })
}

pub(super) fn parse_change_log_row(row: &libsql::Row) -> Result<ChangeLogEntry, AppError> {
    let created_at = parse_rfc3339_utc(&row.get::<String>(6)?)?;

    Ok(ChangeLogEntry {
        id: row.get(0)?,
        entity: row.get(1)?,
        entity_id: row.get(2)?,
        action: row.get(3)?,
        editor_name: row.get(4)?,
        summary: row.get(5)?,
        created_at,
    })
}

pub(super) fn parse_history_snapshot_row(row: &libsql::Row) -> Result<HistorySnapshot, AppError> {
    let created_at = parse_rfc3339_utc(&row.get::<String>(4)?)?;

    Ok(HistorySnapshot {
        id: row.get(0)?,
        summary: row.get(1)?,
        editor_name: row.get(2)?,
        snapshot_kind: row.get(3)?,
        created_at,
    })
}

pub(super) fn parse_provider_question_review_row(
    row: &libsql::Row,
) -> Result<ProviderQuestionReview, AppError> {
    let updated_at = parse_rfc3339_utc(&row.get::<String>(6)?)?;
    Ok(ProviderQuestionReview {
        id: row.get(0)?,
        question_id: row.get(1)?,
        instrument_audience: row.get(2)?,
        status: ProviderQuestionReviewStatus::from_str(&row.get::<String>(3)?),
        observation: row.get(4)?,
        evidence_path: empty_to_none(row.get::<String>(5)?),
        updated_at,
    })
}

pub(super) fn parse_rfc3339_utc(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|error| AppError::InvalidDate(error.to_string()))
}
