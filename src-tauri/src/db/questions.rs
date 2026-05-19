use chrono::Utc;
use libsql::{params, Connection};
use uuid::Uuid;

use crate::db::helpers::find_question_id;
use crate::db::rows::parse_question_row;
use crate::domain::{NewQuestion, Question};
use crate::error::AppError;

pub(super) async fn list_questions(connection: &Connection) -> Result<Vec<Question>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, code, text, scope, format, convention_code, status, factor,
                    characteristic, aspect, audiences_json, justification, updated_at
             FROM questions ORDER BY code",
            (),
        )
        .await?;
    let mut questions = Vec::new();

    while let Some(row) = rows.next().await? {
        questions.push(parse_question_row(&row)?);
    }

    Ok(questions)
}

pub(super) async fn insert_question(
    connection: &Connection,
    question: NewQuestion,
) -> Result<Question, AppError> {
    let persisted = Question {
        id: Uuid::new_v4().to_string(),
        code: question.code,
        text: question.text,
        scope: question.scope,
        format: question.format,
        convention_code: question.convention_code,
        status: question.status,
        factor: question.factor,
        characteristic: question.characteristic,
        aspect: question.aspect,
        audiences: question.audiences,
        justification: question.justification,
        updated_at: Utc::now(),
    };

    connection
        .execute(
            "INSERT INTO questions
                (id, code, text, scope, format, convention_code, status, factor,
                 characteristic, aspect, audiences_json, justification, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                persisted.id.as_str(),
                persisted.code.as_str(),
                persisted.text.as_str(),
                persisted.scope.as_str(),
                persisted.format.as_str(),
                persisted.convention_code.clone().unwrap_or_default(),
                persisted.status.as_str(),
                persisted.factor.as_str(),
                persisted.characteristic.as_str(),
                persisted.aspect.as_str(),
                serde_json::to_string(&persisted.audiences)?,
                persisted.justification.clone().unwrap_or_default(),
                persisted.updated_at.to_rfc3339(),
            ],
        )
        .await?;

    Ok(persisted)
}

pub(super) async fn update_question(
    connection: &Connection,
    question_id: &str,
    question: NewQuestion,
) -> Result<Question, AppError> {
    let existing = list_questions(connection)
        .await?
        .into_iter()
        .find(|item| item.id == question_id)
        .ok_or_else(|| AppError::Validation("question not found".into()))?;
    let updated_at = Utc::now();
    let persisted = Question {
        id: existing.id,
        code: question.code,
        text: question.text,
        scope: question.scope,
        format: question.format,
        convention_code: question.convention_code,
        status: question.status,
        factor: question.factor,
        characteristic: question.characteristic,
        aspect: question.aspect,
        audiences: question.audiences,
        justification: question.justification,
        updated_at,
    };

    connection
        .execute(
            "UPDATE questions
             SET code = ?2, text = ?3, scope = ?4, format = ?5,
                 convention_code = ?6, status = ?7, factor = ?8,
                 characteristic = ?9, aspect = ?10, audiences_json = ?11,
                 justification = ?12, updated_at = ?13
             WHERE id = ?1",
            params![
                persisted.id.as_str(),
                persisted.code.as_str(),
                persisted.text.as_str(),
                persisted.scope.as_str(),
                persisted.format.as_str(),
                persisted.convention_code.clone().unwrap_or_default(),
                persisted.status.as_str(),
                persisted.factor.as_str(),
                persisted.characteristic.as_str(),
                persisted.aspect.as_str(),
                serde_json::to_string(&persisted.audiences)?,
                persisted.justification.clone().unwrap_or_default(),
                persisted.updated_at.to_rfc3339(),
            ],
        )
        .await?;

    Ok(persisted)
}

pub(super) async fn upsert_questions(
    connection: &Connection,
    questions: Vec<NewQuestion>,
) -> Result<usize, AppError> {
    let mut imported = 0;

    for question in questions {
        let existing_id = find_question_id(connection, &question.code).await?;
        if let Some(id) = existing_id {
            connection
                .execute(
                    "UPDATE questions
                     SET text = ?2, scope = ?3, format = ?4, convention_code = ?5,
                         status = ?6, factor = ?7, characteristic = ?8, aspect = ?9,
                         audiences_json = ?10, justification = ?11, updated_at = ?12
                     WHERE id = ?1",
                    params![
                        id.as_str(),
                        question.text.as_str(),
                        question.scope.as_str(),
                        question.format.as_str(),
                        question.convention_code.clone().unwrap_or_default(),
                        question.status.as_str(),
                        question.factor.as_str(),
                        question.characteristic.as_str(),
                        question.aspect.as_str(),
                        serde_json::to_string(&question.audiences)?,
                        question.justification.clone().unwrap_or_default(),
                        Utc::now().to_rfc3339(),
                    ],
                )
                .await?;
        } else {
            insert_question(connection, question).await?;
        }
        imported += 1;
    }

    Ok(imported)
}
