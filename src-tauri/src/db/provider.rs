use chrono::Utc;
use libsql::{params, Connection};
use uuid::Uuid;

use crate::db::helpers::empty_to_none;
use crate::db::rows::{parse_provider_question_review_row, parse_rfc3339_utc};
use crate::domain::{
    NewProviderLink, ProviderLink, ProviderQuestionReview, SaveProviderQuestionReviewRequest,
};
use crate::error::AppError;

pub(super) async fn list_provider_links(
    connection: &Connection,
) -> Result<Vec<ProviderLink>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, subaudience, url, validation_status, validated_at
             FROM provider_links ORDER BY subaudience",
            (),
        )
        .await?;
    let mut links = Vec::new();

    while let Some(row) = rows.next().await? {
        let validated_at = empty_to_none(row.get::<String>(4)?)
            .map(|value| parse_rfc3339_utc(&value))
            .transpose()?;

        links.push(ProviderLink {
            id: row.get(0)?,
            subaudience: row.get(1)?,
            url: row.get(2)?,
            validation_status: row.get(3)?,
            validated_at,
        });
    }

    Ok(links)
}

pub(super) async fn insert_provider_link(
    connection: &Connection,
    link: NewProviderLink,
) -> Result<ProviderLink, AppError> {
    let persisted = ProviderLink {
        id: Uuid::new_v4().to_string(),
        subaudience: link.subaudience,
        url: link.url,
        validation_status: "pending".into(),
        validated_at: None,
    };

    connection
        .execute(
            "INSERT INTO provider_links
                (id, subaudience, url, validation_status, validated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                persisted.id.as_str(),
                persisted.subaudience.as_str(),
                persisted.url.as_str(),
                persisted.validation_status.as_str(),
                ""
            ],
        )
        .await?;

    Ok(persisted)
}

pub(super) async fn list_provider_question_reviews(
    connection: &Connection,
) -> Result<Vec<ProviderQuestionReview>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, question_id, instrument_audience, status, observation, evidence_path, updated_at
             FROM provider_question_reviews
             ORDER BY instrument_audience, updated_at DESC",
            (),
        )
        .await?;
    let mut reviews = Vec::new();
    while let Some(row) = rows.next().await? {
        reviews.push(parse_provider_question_review_row(&row)?);
    }
    Ok(reviews)
}

pub(super) async fn save_provider_question_review(
    connection: &Connection,
    review: SaveProviderQuestionReviewRequest,
) -> Result<ProviderQuestionReview, AppError> {
    let existing_id = {
        let mut rows = connection
            .query(
                "SELECT id FROM provider_question_reviews
                 WHERE question_id = ?1 AND instrument_audience = ?2
                 LIMIT 1",
                params![
                    review.question_id.as_str(),
                    review.instrument_audience.as_str()
                ],
            )
            .await?;
        rows.next()
            .await?
            .map(|row| row.get::<String>(0))
            .transpose()?
    };
    let id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let updated_at = Utc::now();
    connection
        .execute(
            "INSERT OR REPLACE INTO provider_question_reviews
                (id, question_id, instrument_audience, status, observation, evidence_path, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id.as_str(),
                review.question_id.as_str(),
                review.instrument_audience.as_str(),
                review.status.as_str(),
                review.observation.as_str(),
                review.evidence_path.clone().unwrap_or_default(),
                updated_at.to_rfc3339(),
            ],
        )
        .await?;
    Ok(ProviderQuestionReview {
        id,
        question_id: review.question_id,
        instrument_audience: review.instrument_audience,
        status: review.status,
        observation: review.observation,
        evidence_path: review.evidence_path,
        updated_at,
    })
}

pub(super) async fn reset_provider_question_reviews(
    connection: &Connection,
) -> Result<usize, AppError> {
    let count = {
        let mut rows = connection
            .query("SELECT COUNT(*) FROM provider_question_reviews", ())
            .await?;
        rows.next()
            .await?
            .map(|row| row.get::<i64>(0))
            .transpose()?
            .unwrap_or(0)
    };

    connection
        .execute("DELETE FROM provider_question_reviews", ())
        .await?;

    Ok(count.max(0) as usize)
}
