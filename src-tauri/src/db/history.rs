use chrono::Utc;
use libsql::{params, Connection};
use uuid::Uuid;

use crate::db::rows::parse_history_snapshot_row;
use crate::domain::{GuidelineAspect, HistorySnapshot, OriginalQuestionSnapshot, Question};
use crate::error::AppError;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct PersistedSnapshot {
    pub(super) questions: Vec<Question>,
    pub(super) guideline_aspects: Vec<GuidelineAspect>,
    #[serde(default)]
    pub(super) original_snapshots: Vec<OriginalQuestionSnapshot>,
}

pub(super) async fn create_snapshot(
    connection: &Connection,
    summary: &str,
    editor_name: &str,
    snapshot_kind: &str,
    prune_auto_snapshots: bool,
    data: PersistedSnapshot,
) -> Result<HistorySnapshot, AppError> {
    let snapshot = HistorySnapshot {
        id: Uuid::new_v4().to_string(),
        summary: summary.into(),
        editor_name: editor_name.into(),
        snapshot_kind: snapshot_kind.into(),
        created_at: Utc::now(),
    };

    connection
        .execute(
            "INSERT INTO history_snapshots
                (id, summary, editor_name, snapshot_kind, created_at, snapshot_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snapshot.id.as_str(),
                snapshot.summary.as_str(),
                snapshot.editor_name.as_str(),
                snapshot.snapshot_kind.as_str(),
                snapshot.created_at.to_rfc3339(),
                serde_json::to_string(&data)?,
            ],
        )
        .await?;

    if prune_auto_snapshots {
        connection
            .execute(
                "DELETE FROM history_snapshots
                 WHERE snapshot_kind = 'auto'
                   AND id NOT IN (
                        SELECT id FROM history_snapshots
                        WHERE snapshot_kind = 'auto'
                        ORDER BY created_at DESC
                        LIMIT 30
                   )",
                (),
            )
            .await?;
    }

    Ok(snapshot)
}

pub(super) async fn list_snapshots(
    connection: &Connection,
) -> Result<Vec<HistorySnapshot>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, summary, editor_name, snapshot_kind, created_at
             FROM history_snapshots
             ORDER BY created_at DESC",
            (),
        )
        .await?;
    let mut snapshots = Vec::new();
    while let Some(row) = rows.next().await? {
        snapshots.push(parse_history_snapshot_row(&row)?);
    }
    Ok(snapshots)
}

pub(super) async fn load_snapshot(
    connection: &Connection,
    snapshot_id: &str,
) -> Result<PersistedSnapshot, AppError> {
    let mut rows = connection
        .query(
            "SELECT snapshot_json FROM history_snapshots WHERE id = ?1 LIMIT 1",
            [snapshot_id],
        )
        .await?;
    let Some(row) = rows.next().await? else {
        return Err(AppError::Validation(
            "history snapshot was not found".into(),
        ));
    };
    Ok(serde_json::from_str(&row.get::<String>(0)?)?)
}

pub(super) async fn delete_manual_snapshot(
    connection: &Connection,
    snapshot_id: &str,
) -> Result<(), AppError> {
    connection
        .execute(
            "DELETE FROM history_snapshots
             WHERE id = ?1 AND snapshot_kind = 'manual'",
            [snapshot_id],
        )
        .await?;
    Ok(())
}
