use libsql::Connection;

use crate::error::AppError;

pub(super) async fn migrate(connection: &Connection) -> Result<(), AppError> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS survey_cycles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                starts_on TEXT NOT NULL,
                application_starts_on TEXT NOT NULL,
                status TEXT NOT NULL,
                notes TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS questions (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                text TEXT NOT NULL,
                scope TEXT NOT NULL,
                format TEXT NOT NULL,
                convention_code TEXT NOT NULL,
                status TEXT NOT NULL,
                factor TEXT NOT NULL,
                characteristic TEXT NOT NULL,
                aspect TEXT NOT NULL,
                audiences_json TEXT NOT NULL,
                justification TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_links (
                id TEXT PRIMARY KEY,
                subaudience TEXT NOT NULL,
                url TEXT NOT NULL,
                validation_status TEXT NOT NULL,
                validated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_question_reviews (
                id TEXT PRIMARY KEY,
                question_id TEXT NOT NULL,
                instrument_audience TEXT NOT NULL,
                status TEXT NOT NULL,
                observation TEXT NOT NULL,
                evidence_path TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(question_id, instrument_audience)
            );

            CREATE TABLE IF NOT EXISTS validation_checks (
                id TEXT PRIMARY KEY,
                severity TEXT NOT NULL,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS source_documents (
                id TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                path TEXT NOT NULL,
                document_type TEXT NOT NULL,
                imported_at TEXT NOT NULL,
                imported_rows INTEGER NOT NULL,
                skipped_rows INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS change_logs (
                id TEXT PRIMARY KEY,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                editor_name TEXT NOT NULL,
                summary TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS guideline_aspects (
                id TEXT PRIMARY KEY,
                guideline_title TEXT NOT NULL,
                scope TEXT NOT NULL,
                factor_code TEXT NOT NULL,
                factor_name TEXT NOT NULL,
                characteristic_code TEXT NOT NULL,
                characteristic_name TEXT NOT NULL,
                aspect_code TEXT NOT NULL,
                aspect_description TEXT NOT NULL,
                requires_appreciation INTEGER NOT NULL
            );

            DELETE FROM questions
             WHERE id NOT IN (
                SELECT MIN(id)
                  FROM questions
                 GROUP BY code
             );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_questions_unique_code
                ON questions (code);

            CREATE TABLE IF NOT EXISTS question_original_snapshots (
                id TEXT PRIMARY KEY,
                question_id TEXT NOT NULL,
                source_document_id TEXT NOT NULL,
                code TEXT NOT NULL,
                text TEXT NOT NULL,
                scope TEXT NOT NULL,
                format TEXT NOT NULL,
                convention_code TEXT NOT NULL,
                status TEXT NOT NULL,
                factor TEXT NOT NULL,
                characteristic TEXT NOT NULL,
                aspect TEXT NOT NULL,
                audiences_json TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                marked_by TEXT NOT NULL,
                marked_at TEXT NOT NULL,
                is_current INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS instrument_exports (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                path TEXT NOT NULL,
                exported_at TEXT NOT NULL,
                exported_questions INTEGER NOT NULL,
                added_questions INTEGER NOT NULL,
                modified_questions INTEGER NOT NULL,
                removed_questions INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS instrument_definitions (
                id TEXT PRIMARY KEY,
                instrument_key TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                is_system INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS instrument_public_assignments (
                instrument_id TEXT NOT NULL,
                public_key TEXT NOT NULL UNIQUE,
                public_label TEXT NOT NULL,
                PRIMARY KEY(instrument_id, public_key),
                FOREIGN KEY(instrument_id) REFERENCES instrument_definitions(id)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS history_snapshots (
                id TEXT PRIMARY KEY,
                summary TEXT NOT NULL,
                editor_name TEXT NOT NULL,
                snapshot_kind TEXT NOT NULL DEFAULT 'auto',
                created_at TEXT NOT NULL,
                snapshot_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS collaboration_locks (
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                editor_name TEXT NOT NULL,
                locked_until TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY(resource_type, resource_id)
            );

            CREATE INDEX IF NOT EXISTS idx_collaboration_locks_expiry
                ON collaboration_locks (locked_until);

            CREATE TABLE IF NOT EXISTS collaboration_presence (
                editor_name TEXT PRIMARY KEY,
                last_seen_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_collaboration_presence_last_seen
                ON collaboration_presence (last_seen_at);

            DELETE FROM guideline_aspects
             WHERE id NOT IN (
                SELECT MIN(id)
                  FROM guideline_aspects
                 GROUP BY scope, factor_code, characteristic_code, aspect_code
             );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_guideline_aspects_unique_hierarchy
                ON guideline_aspects (scope, factor_code, characteristic_code, aspect_code);
            ",
        )
        .await?;

    migrate_provider_reviews_to_instruments(connection).await?;
    migrate_history_snapshots_kind(connection).await?;
    migrate_original_snapshots_current_marker(connection).await?;

    Ok(())
}

async fn migrate_original_snapshots_current_marker(
    connection: &Connection,
) -> Result<(), AppError> {
    let mut rows = connection
        .query("PRAGMA table_info(question_original_snapshots)", ())
        .await?;
    let mut has_is_current = false;
    while let Some(row) = rows.next().await? {
        let column_name: String = row.get(1)?;
        if column_name == "is_current" {
            has_is_current = true;
            break;
        }
    }

    if !has_is_current {
        connection
            .execute(
                "ALTER TABLE question_original_snapshots
                 ADD COLUMN is_current INTEGER NOT NULL DEFAULT 1",
                (),
            )
            .await?;
    }

    connection
        .execute_batch(
            "
            DROP INDEX IF EXISTS idx_original_snapshots_unique_code;

            CREATE UNIQUE INDEX IF NOT EXISTS idx_original_snapshots_current_unique_code
                ON question_original_snapshots (code)
                WHERE is_current = 1;
            ",
        )
        .await?;

    Ok(())
}

async fn migrate_history_snapshots_kind(connection: &Connection) -> Result<(), AppError> {
    let mut rows = connection
        .query("PRAGMA table_info(history_snapshots)", ())
        .await?;
    let mut has_snapshot_kind = false;
    while let Some(row) = rows.next().await? {
        let column_name: String = row.get(1)?;
        if column_name == "snapshot_kind" {
            has_snapshot_kind = true;
            break;
        }
    }

    if !has_snapshot_kind {
        connection
            .execute(
                "ALTER TABLE history_snapshots
                 ADD COLUMN snapshot_kind TEXT NOT NULL DEFAULT 'auto'",
                (),
            )
            .await?;
    }

    Ok(())
}

async fn migrate_provider_reviews_to_instruments(connection: &Connection) -> Result<(), AppError> {
    let mut rows = connection
        .query("PRAGMA table_info(provider_question_reviews)", ())
        .await?;
    let mut has_instrument_audience = false;
    while let Some(row) = rows.next().await? {
        let column_name: String = row.get(1)?;
        if column_name == "instrument_audience" {
            has_instrument_audience = true;
            break;
        }
    }

    if has_instrument_audience {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            ALTER TABLE provider_question_reviews RENAME TO provider_question_reviews_legacy;

            CREATE TABLE provider_question_reviews (
                id TEXT PRIMARY KEY,
                question_id TEXT NOT NULL,
                instrument_audience TEXT NOT NULL,
                status TEXT NOT NULL,
                observation TEXT NOT NULL,
                evidence_path TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(question_id, instrument_audience)
            );

            INSERT INTO provider_question_reviews
                (id, question_id, instrument_audience, status, observation, evidence_path, updated_at)
            SELECT id, question_id, '', status, observation, evidence_path, updated_at
              FROM provider_question_reviews_legacy;

            DROP TABLE provider_question_reviews_legacy;
            ",
        )
        .await?;

    Ok(())
}
