use chrono::Utc;
use libsql::{params, Connection};
use uuid::Uuid;

use crate::db::helpers::{count_related_questions, find_guideline_aspect_id, join_number_name};
use crate::domain::{
    CnaFactorCode, GuidelineAspect, NewGuidelineAspect, QuestionScope, QuestionStatus,
    UpdateGuidelineAspectResult,
};
use crate::error::AppError;

pub(super) async fn list_guideline_aspects(
    connection: &Connection,
) -> Result<Vec<GuidelineAspect>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, guideline_title, scope, factor_code, factor_name,
                    characteristic_code, characteristic_name, aspect_code,
                    aspect_description, requires_appreciation
             FROM guideline_aspects
             ORDER BY scope, factor_code, characteristic_code, aspect_code",
            (),
        )
        .await?;
    let mut aspects = Vec::new();

    while let Some(row) = rows.next().await? {
        aspects.push(GuidelineAspect {
            id: row.get(0)?,
            guideline_title: row.get(1)?,
            scope: QuestionScope::from_str(&row.get::<String>(2)?),
            factor_code: CnaFactorCode::from_str(&row.get::<String>(3)?).ok_or_else(|| {
                AppError::Validation("invalid CNA factor code in database".into())
            })?,
            factor_name: row.get(4)?,
            characteristic_code: row.get(5)?,
            characteristic_name: row.get(6)?,
            aspect_code: row.get(7)?,
            aspect_description: row.get(8)?,
            requires_appreciation: row.get::<i64>(9)? == 1,
        });
    }

    Ok(aspects)
}

pub(super) async fn insert_guideline_aspect(
    connection: &Connection,
    aspect: NewGuidelineAspect,
) -> Result<GuidelineAspect, AppError> {
    let persisted = GuidelineAspect {
        id: Uuid::new_v4().to_string(),
        guideline_title: aspect.guideline_title,
        scope: aspect.scope,
        factor_code: aspect.factor_code,
        factor_name: aspect.factor_name,
        characteristic_code: aspect.characteristic_code,
        characteristic_name: aspect.characteristic_name,
        aspect_code: aspect.aspect_code,
        aspect_description: aspect.aspect_description,
        requires_appreciation: aspect.requires_appreciation,
    };

    connection
        .execute(
            "INSERT INTO guideline_aspects
                (id, guideline_title, scope, factor_code, factor_name,
                 characteristic_code, characteristic_name, aspect_code,
                 aspect_description, requires_appreciation)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                persisted.id.as_str(),
                persisted.guideline_title.as_str(),
                persisted.scope.as_str(),
                persisted.factor_code.as_str(),
                persisted.factor_name.as_str(),
                persisted.characteristic_code.as_str(),
                persisted.characteristic_name.as_str(),
                persisted.aspect_code.as_str(),
                persisted.aspect_description.as_str(),
                if persisted.requires_appreciation {
                    1_i64
                } else {
                    0_i64
                },
            ],
        )
        .await?;

    Ok(persisted)
}

pub(super) async fn update_guideline_aspect(
    connection: &Connection,
    aspect_id: &str,
    aspect: NewGuidelineAspect,
) -> Result<UpdateGuidelineAspectResult, AppError> {
    let mut rows = connection
        .query(
            "SELECT factor_code, factor_name, characteristic_code,
                    characteristic_name, aspect_code, aspect_description
             FROM guideline_aspects
             WHERE id = ?1
             LIMIT 1",
            [aspect_id],
        )
        .await?;
    let Some(row) = rows.next().await? else {
        return Err(AppError::Validation(
            "guideline aspect was not found".into(),
        ));
    };

    let old_factor = join_number_name(row.get::<String>(0)?, row.get::<String>(1)?);
    let old_characteristic = join_number_name(row.get::<String>(2)?, row.get::<String>(3)?);
    let old_aspect = join_number_name(row.get::<String>(4)?, row.get::<String>(5)?);
    let new_factor = join_number_name(
        aspect.factor_code.as_str().to_string(),
        aspect.factor_name.clone(),
    );
    let new_characteristic = join_number_name(
        aspect.characteristic_code.clone(),
        aspect.characteristic_name.clone(),
    );
    let new_aspect = join_number_name(
        aspect.aspect_code.clone(),
        aspect.aspect_description.clone(),
    );
    let hierarchy_changed = old_factor != new_factor
        || old_characteristic != new_characteristic
        || old_aspect != new_aspect;
    let affected_questions = if hierarchy_changed {
        count_related_questions(connection, &old_factor, &old_characteristic, &old_aspect).await?
    } else {
        0
    };

    let persisted = GuidelineAspect {
        id: aspect_id.to_string(),
        guideline_title: aspect.guideline_title,
        scope: aspect.scope,
        factor_code: aspect.factor_code,
        factor_name: aspect.factor_name,
        characteristic_code: aspect.characteristic_code,
        characteristic_name: aspect.characteristic_name,
        aspect_code: aspect.aspect_code,
        aspect_description: aspect.aspect_description,
        requires_appreciation: aspect.requires_appreciation,
    };

    connection
        .execute(
            "UPDATE guideline_aspects
             SET guideline_title = ?2, scope = ?3, factor_code = ?4,
                 factor_name = ?5, characteristic_code = ?6,
                 characteristic_name = ?7, aspect_code = ?8,
                 aspect_description = ?9, requires_appreciation = ?10
             WHERE id = ?1",
            params![
                persisted.id.as_str(),
                persisted.guideline_title.as_str(),
                persisted.scope.as_str(),
                persisted.factor_code.as_str(),
                persisted.factor_name.as_str(),
                persisted.characteristic_code.as_str(),
                persisted.characteristic_name.as_str(),
                persisted.aspect_code.as_str(),
                persisted.aspect_description.as_str(),
                if persisted.requires_appreciation {
                    1_i64
                } else {
                    0_i64
                },
            ],
        )
        .await?;

    if hierarchy_changed {
        connection
            .execute(
                "UPDATE questions
                 SET status = ?1, factor = ?2, characteristic = ?3, aspect = ?4, updated_at = ?5
                 WHERE factor = ?6 AND characteristic = ?7 AND aspect = ?8",
                params![
                    QuestionStatus::Modify.as_str(),
                    new_factor.as_str(),
                    new_characteristic.as_str(),
                    new_aspect.as_str(),
                    Utc::now().to_rfc3339(),
                    old_factor.as_str(),
                    old_characteristic.as_str(),
                    old_aspect.as_str(),
                ],
            )
            .await?;
    }

    Ok(UpdateGuidelineAspectResult {
        aspect: persisted,
        affected_questions,
    })
}

pub(super) async fn upsert_guideline_aspects(
    connection: &Connection,
    aspects: Vec<NewGuidelineAspect>,
) -> Result<usize, AppError> {
    let mut imported = 0;

    for aspect in aspects {
        let existing_id = find_guideline_aspect_id(
            connection,
            aspect.scope.as_str(),
            aspect.factor_code.as_str(),
            &aspect.characteristic_code,
            &aspect.aspect_code,
        )
        .await?;

        if let Some(id) = existing_id {
            connection
                .execute(
                    "UPDATE guideline_aspects
                     SET guideline_title = ?2, factor_name = ?3, characteristic_name = ?4,
                         aspect_description = ?5, requires_appreciation = ?6
                     WHERE id = ?1",
                    params![
                        id.as_str(),
                        aspect.guideline_title.as_str(),
                        aspect.factor_name.as_str(),
                        aspect.characteristic_name.as_str(),
                        aspect.aspect_description.as_str(),
                        if aspect.requires_appreciation {
                            1_i64
                        } else {
                            0_i64
                        },
                    ],
                )
                .await?;
        } else {
            insert_guideline_aspect(connection, aspect).await?;
        }
        imported += 1;
    }

    Ok(imported)
}

pub(super) async fn delete_guideline_aspect_and_related_questions(
    connection: &Connection,
    aspect_id: &str,
) -> Result<usize, AppError> {
    let mut rows = connection
        .query(
            "SELECT factor_code, factor_name, characteristic_code,
                    characteristic_name, aspect_code, aspect_description
             FROM guideline_aspects
             WHERE id = ?1
             LIMIT 1",
            [aspect_id],
        )
        .await?;
    let Some(row) = rows.next().await? else {
        return Err(AppError::Validation(
            "guideline aspect was not found".into(),
        ));
    };

    let factor = join_number_name(row.get::<String>(0)?, row.get::<String>(1)?);
    let characteristic = join_number_name(row.get::<String>(2)?, row.get::<String>(3)?);
    let aspect = join_number_name(row.get::<String>(4)?, row.get::<String>(5)?);
    let affected = count_related_questions(connection, &factor, &characteristic, &aspect).await?;

    connection
        .execute(
            "DELETE FROM questions
             WHERE factor = ?1 AND characteristic = ?2 AND aspect = ?3",
            params![factor.as_str(), characteristic.as_str(), aspect.as_str()],
        )
        .await?;
    connection
        .execute("DELETE FROM guideline_aspects WHERE id = ?1", [aspect_id])
        .await?;

    Ok(affected)
}
