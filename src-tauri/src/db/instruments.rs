use chrono::Utc;
use libsql::{params, Connection};
use uuid::Uuid;

use crate::audience::{display_instrument_column_label, display_public_label, InstrumentAudience};
use crate::db::rows::parse_rfc3339_utc;
use crate::domain::{InstrumentDefinition, SaveInstrumentDefinitionRequest};
use crate::error::AppError;

pub(super) async fn list_definitions(
    connection: &Connection,
) -> Result<Vec<InstrumentDefinition>, AppError> {
    let mut rows = connection
        .query(
            "SELECT id, instrument_key, label, is_system, updated_at
             FROM instrument_definitions
             ORDER BY label",
            (),
        )
        .await?;
    let mut definitions = Vec::new();
    while let Some(row) = rows.next().await? {
        let id: String = row.get(0)?;
        let (public_keys, public_labels) = list_public_assignments(connection, &id).await?;
        definitions.push(InstrumentDefinition {
            id,
            key: row.get(1)?,
            label: row.get(2)?,
            is_system: row.get::<i64>(3)? != 0,
            updated_at: parse_rfc3339_utc(&row.get::<String>(4)?)?,
            public_keys,
            public_labels,
        });
    }
    Ok(definitions)
}

pub(super) async fn save_definition(
    connection: &Connection,
    request: SaveInstrumentDefinitionRequest,
    is_system: bool,
) -> Result<InstrumentDefinition, AppError> {
    let label = request.label.trim();
    if label.is_empty() {
        return Err(AppError::Validation("instrument label is required".into()));
    }
    let public_keys = normalized_public_keys(request.public_keys);
    if public_keys.is_empty() {
        return Err(AppError::Validation(
            "assign at least one public to the instrument".into(),
        ));
    }

    let id = request.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let key = if is_system {
        InstrumentAudience::parse(&public_keys[0]).public
    } else {
        slugify_key(label)
    };
    let updated_at = Utc::now();
    connection
        .execute(
            "INSERT OR REPLACE INTO instrument_definitions
                (id, instrument_key, label, is_system, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.as_str(),
                key.as_str(),
                label,
                if is_system { 1 } else { 0 },
                updated_at.to_rfc3339(),
            ],
        )
        .await?;
    connection
        .execute(
            "DELETE FROM instrument_public_assignments WHERE instrument_id = ?1",
            [id.as_str()],
        )
        .await?;
    for public_key in &public_keys {
        connection
            .execute(
                "INSERT INTO instrument_public_assignments
                    (instrument_id, public_key, public_label)
                 VALUES (?1, ?2, ?3)",
                params![
                    id.as_str(),
                    public_key.as_str(),
                    display_assignment_label(public_key),
                ],
            )
            .await
            .map_err(|error| {
                AppError::Validation(format!(
                    "public '{public_key}' is already assigned to another instrument: {error}"
                ))
            })?;
    }
    Ok(InstrumentDefinition {
        id,
        key,
        label: label.into(),
        public_labels: public_keys
            .iter()
            .map(|public| display_assignment_label(public))
            .collect(),
        public_keys,
        is_system,
        updated_at,
    })
}

fn display_assignment_label(public_key: &str) -> String {
    let parsed = InstrumentAudience::parse(public_key);
    if parsed.public == public_key {
        display_public_label(public_key)
    } else {
        display_instrument_column_label(&parsed.public, public_key)
    }
}

async fn list_public_assignments(
    connection: &Connection,
    instrument_id: &str,
) -> Result<(Vec<String>, Vec<String>), AppError> {
    let mut rows = connection
        .query(
            "SELECT public_key, public_label
             FROM instrument_public_assignments
             WHERE instrument_id = ?1
             ORDER BY public_label",
            [instrument_id],
        )
        .await?;
    let mut keys = Vec::new();
    let mut labels = Vec::new();
    while let Some(row) = rows.next().await? {
        keys.push(row.get(0)?);
        labels.push(row.get(1)?);
    }
    Ok((keys, labels))
}

fn normalized_public_keys(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn slugify_key(value: &str) -> String {
    let key = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if key.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        key
    }
}
