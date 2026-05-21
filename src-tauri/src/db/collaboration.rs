use chrono::{DateTime, Utc};
use libsql::{params, Connection};

use crate::domain::{CollaborationLock, CollaborationPresence, CollaborationResourceType};
use crate::error::AppError;

pub(super) async fn list_locks(
    connection: &Connection,
) -> Result<Vec<CollaborationLock>, AppError> {
    prune_expired_locks(connection).await?;
    let mut rows = connection
        .query(
            "SELECT resource_type, resource_id, editor_name, locked_until, updated_at
             FROM collaboration_locks
             ORDER BY locked_until DESC",
            (),
        )
        .await?;
    let mut locks = Vec::new();
    while let Some(row) = rows.next().await? {
        locks.push(CollaborationLock {
            resource_type: CollaborationResourceType::from_str(&row.get::<String>(0)?),
            resource_id: row.get(1)?,
            editor_name: row.get(2)?,
            locked_until: parse_utc(&row.get::<String>(3)?)?,
            updated_at: parse_utc(&row.get::<String>(4)?)?,
        });
    }
    Ok(locks)
}

pub(super) async fn get_lock(
    connection: &Connection,
    resource_type: CollaborationResourceType,
    resource_id: &str,
) -> Result<Option<CollaborationLock>, AppError> {
    prune_expired_locks(connection).await?;
    find_lock(connection, resource_type.as_str(), resource_id).await
}

pub(super) async fn list_locks_for_resources(
    connection: &Connection,
    resource_type: CollaborationResourceType,
    resource_ids: &[String],
) -> Result<Vec<CollaborationLock>, AppError> {
    prune_expired_locks(connection).await?;
    let mut locks = Vec::new();
    for resource_id in resource_ids {
        if let Some(lock) = find_lock(connection, resource_type.as_str(), resource_id).await? {
            locks.push(lock);
        }
    }
    Ok(locks)
}

pub(super) async fn acquire_lock(
    connection: &Connection,
    resource_type: CollaborationResourceType,
    resource_id: &str,
    editor_name: &str,
    ttl_seconds: i64,
) -> Result<CollaborationLock, AppError> {
    let now = Utc::now();
    let locked_until = now + chrono::Duration::seconds(ttl_seconds.max(30));
    let resource_type_value = resource_type.as_str();

    prune_expired_locks(connection).await?;
    if let Some(existing) = find_lock(connection, resource_type_value, resource_id).await? {
        if existing.editor_name != editor_name && existing.locked_until > now {
            return Err(AppError::Validation(format!(
                "{} is editing this item until {}",
                existing.editor_name,
                existing.locked_until.format("%H:%M:%S")
            )));
        }
    }

    connection
        .execute(
            "INSERT OR REPLACE INTO collaboration_locks
                (resource_type, resource_id, editor_name, locked_until, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                resource_type_value,
                resource_id,
                editor_name,
                locked_until.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )
        .await?;

    Ok(CollaborationLock {
        resource_type,
        resource_id: resource_id.into(),
        editor_name: editor_name.into(),
        locked_until,
        updated_at: now,
    })
}

pub(super) async fn release_lock(
    connection: &Connection,
    resource_type: CollaborationResourceType,
    resource_id: &str,
    editor_name: &str,
) -> Result<(), AppError> {
    connection
        .execute(
            "DELETE FROM collaboration_locks
             WHERE resource_type = ?1 AND resource_id = ?2 AND editor_name = ?3",
            params![resource_type.as_str(), resource_id, editor_name],
        )
        .await?;
    Ok(())
}

pub(super) async fn heartbeat_presence(
    connection: &Connection,
    editor_name: &str,
) -> Result<CollaborationPresence, AppError> {
    let now = Utc::now();
    prune_stale_presence(connection).await?;
    connection
        .execute(
            "INSERT OR REPLACE INTO collaboration_presence
                (editor_name, last_seen_at)
             VALUES (?1, ?2)",
            params![editor_name, now.to_rfc3339()],
        )
        .await?;
    Ok(CollaborationPresence {
        editor_name: editor_name.into(),
        last_seen_at: now,
    })
}

pub(super) async fn list_presence(
    connection: &Connection,
) -> Result<Vec<CollaborationPresence>, AppError> {
    prune_stale_presence(connection).await?;
    let mut rows = connection
        .query(
            "SELECT editor_name, last_seen_at
             FROM collaboration_presence
             ORDER BY last_seen_at DESC",
            (),
        )
        .await?;
    let mut presence = Vec::new();
    while let Some(row) = rows.next().await? {
        presence.push(CollaborationPresence {
            editor_name: row.get(0)?,
            last_seen_at: parse_utc(&row.get::<String>(1)?)?,
        });
    }
    Ok(presence)
}

async fn prune_expired_locks(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute(
            "DELETE FROM collaboration_locks WHERE locked_until <= ?1",
            [Utc::now().to_rfc3339()],
        )
        .await?;
    Ok(())
}

async fn prune_stale_presence(connection: &Connection) -> Result<(), AppError> {
    let cutoff = Utc::now() - chrono::Duration::seconds(45);
    connection
        .execute(
            "DELETE FROM collaboration_presence WHERE last_seen_at <= ?1",
            [cutoff.to_rfc3339()],
        )
        .await?;
    Ok(())
}

async fn find_lock(
    connection: &Connection,
    resource_type: &str,
    resource_id: &str,
) -> Result<Option<CollaborationLock>, AppError> {
    let mut rows = connection
        .query(
            "SELECT resource_type, resource_id, editor_name, locked_until, updated_at
             FROM collaboration_locks
             WHERE resource_type = ?1 AND resource_id = ?2
             LIMIT 1",
            params![resource_type, resource_id],
        )
        .await?;
    rows.next()
        .await?
        .map(|row| {
            Ok(CollaborationLock {
                resource_type: CollaborationResourceType::from_str(&row.get::<String>(0)?),
                resource_id: row.get(1)?,
                editor_name: row.get(2)?,
                locked_until: parse_utc(&row.get::<String>(3)?)?,
                updated_at: parse_utc(&row.get::<String>(4)?)?,
            })
        })
        .transpose()
}

fn parse_utc(value: &str) -> Result<DateTime<Utc>, AppError> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|error| AppError::Validation(format!("invalid collaboration timestamp: {error}")))?
        .with_timezone(&Utc))
}
