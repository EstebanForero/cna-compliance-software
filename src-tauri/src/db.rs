use async_trait::async_trait;
use chrono::Utc;
use libsql::{params, Builder, Connection};
use uuid::Uuid;

use crate::domain::{
    ChangeLogEntry, CycleStatus, GuidelineAspect, HistorySnapshot, NewGuidelineAspect,
    NewProviderLink, NewQuestion, NewSourceDocument, OriginalQuestionSnapshot, ProviderLink,
    ProviderQuestionReview, Question, SaveProviderQuestionReviewRequest, SourceDocument,
    SurveyCycle, UpdateGuidelineAspectResult, ValidationIssue, ValidationSeverity,
};
use crate::error::AppError;
use crate::repository::AutoEvalRepository;

mod helpers;
mod history;
mod lineaments;
mod provider;
mod questions;
mod rows;
mod schema;
use helpers::parse_date;
use rows::{parse_change_log_row, parse_original_snapshot_row, parse_source_document_row};

pub struct LibSqlAutoEvalRepository {
    connection: Connection,
}

impl LibSqlAutoEvalRepository {
    pub async fn open(path: impl AsRef<std::path::Path>) -> Result<Self, AppError> {
        let database = Builder::new_local(path).build().await?;
        Self::from_database(database).await
    }

    #[cfg(test)]
    pub async fn open_in_memory() -> Result<Self, AppError> {
        let database = Builder::new_local(":memory:").build().await?;
        Self::from_database(database).await
    }

    async fn from_database(database: libsql::Database) -> Result<Self, AppError> {
        let connection = database.connect()?;
        let repository = Self { connection };
        repository.migrate().await?;
        Ok(repository)
    }

    async fn migrate(&self) -> Result<(), AppError> {
        schema::migrate(&self.connection).await
    }

    async fn create_history_snapshot_with_kind(
        &self,
        summary: &str,
        editor_name: &str,
        snapshot_kind: &str,
        prune_auto_snapshots: bool,
    ) -> Result<HistorySnapshot, AppError> {
        let data = history::PersistedSnapshot {
            questions: self.list_questions().await?,
            guideline_aspects: self.list_guideline_aspects().await?,
        };
        history::create_snapshot(
            &self.connection,
            summary,
            editor_name,
            snapshot_kind,
            prune_auto_snapshots,
            data,
        )
        .await
    }
}

#[async_trait]
impl AutoEvalRepository for LibSqlAutoEvalRepository {
    async fn active_cycle(&self) -> Result<Option<SurveyCycle>, AppError> {
        let mut rows = self
            .connection
            .query(
                "SELECT id, name, starts_on, application_starts_on, status, notes
                 FROM survey_cycles WHERE is_active = 1 LIMIT 1",
                (),
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(SurveyCycle {
            id: row.get(0)?,
            name: row.get(1)?,
            starts_on: parse_date(&row.get::<String>(2)?)?,
            application_starts_on: parse_date(&row.get::<String>(3)?)?,
            status: CycleStatus::from_str(&row.get::<String>(4)?),
            notes: row.get(5)?,
        }))
    }

    async fn ensure_cycle(&self, name: &str) -> Result<SurveyCycle, AppError> {
        if let Some(cycle) = self.active_cycle().await? {
            return Ok(cycle);
        }

        let today = Utc::now().date_naive();
        let application = today
            .checked_add_months(chrono::Months::new(6))
            .unwrap_or(today);
        let cycle = SurveyCycle {
            id: Uuid::new_v4().to_string(),
            name: name.trim().to_string(),
            starts_on: today,
            application_starts_on: application,
            status: CycleStatus::Planning,
            notes: "Creado automaticamente al importar el primer consolidado.".into(),
        };

        self.connection
            .execute(
                "INSERT INTO survey_cycles VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                params![
                    cycle.id.as_str(),
                    cycle.name.as_str(),
                    cycle.starts_on.to_string(),
                    cycle.application_starts_on.to_string(),
                    cycle.status.as_str(),
                    cycle.notes.as_str(),
                ],
            )
            .await?;

        Ok(cycle)
    }

    async fn list_questions(&self) -> Result<Vec<Question>, AppError> {
        questions::list_questions(&self.connection).await
    }

    async fn list_guideline_aspects(&self) -> Result<Vec<GuidelineAspect>, AppError> {
        lineaments::list_guideline_aspects(&self.connection).await
    }

    async fn insert_guideline_aspect(
        &self,
        aspect: NewGuidelineAspect,
    ) -> Result<GuidelineAspect, AppError> {
        lineaments::insert_guideline_aspect(&self.connection, aspect).await
    }

    async fn update_guideline_aspect(
        &self,
        aspect_id: &str,
        aspect: NewGuidelineAspect,
    ) -> Result<UpdateGuidelineAspectResult, AppError> {
        lineaments::update_guideline_aspect(&self.connection, aspect_id, aspect).await
    }

    async fn upsert_guideline_aspects(
        &self,
        aspects: Vec<NewGuidelineAspect>,
    ) -> Result<usize, AppError> {
        lineaments::upsert_guideline_aspects(&self.connection, aspects).await
    }

    async fn insert_question(&self, question: NewQuestion) -> Result<Question, AppError> {
        questions::insert_question(&self.connection, question).await
    }

    async fn update_question(
        &self,
        question_id: &str,
        question: NewQuestion,
    ) -> Result<Question, AppError> {
        questions::update_question(&self.connection, question_id, question).await
    }

    async fn delete_guideline_aspect_and_related_questions(
        &self,
        aspect_id: &str,
    ) -> Result<usize, AppError> {
        lineaments::delete_guideline_aspect_and_related_questions(&self.connection, aspect_id).await
    }

    async fn upsert_questions(&self, questions: Vec<NewQuestion>) -> Result<usize, AppError> {
        questions::upsert_questions(&self.connection, questions).await
    }

    async fn save_source_document(&self, document: NewSourceDocument) -> Result<(), AppError> {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO source_documents
                    (id, file_name, path, document_type, imported_at, imported_rows, skipped_rows)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    document.id.as_str(),
                    document.file_name.as_str(),
                    document.path.as_str(),
                    document.document_type.as_str(),
                    Utc::now().to_rfc3339(),
                    document.imported_rows as i64,
                    document.skipped_rows as i64,
                ],
            )
            .await?;
        Ok(())
    }

    async fn latest_source_document(&self) -> Result<Option<SourceDocument>, AppError> {
        let mut rows = self
            .connection
            .query(
                "SELECT id, file_name, path, document_type, imported_at, imported_rows, skipped_rows
                 FROM source_documents
                 WHERE document_type = 'questions_consolidated'
                 ORDER BY imported_at DESC
                 LIMIT 1",
                (),
            )
            .await?;

        rows.next()
            .await?
            .map(|row| parse_source_document_row(&row))
            .transpose()
    }

    async fn get_source_document(&self, id: &str) -> Result<Option<SourceDocument>, AppError> {
        let mut rows = self
            .connection
            .query(
                "SELECT id, file_name, path, document_type, imported_at, imported_rows, skipped_rows
                 FROM source_documents
                 WHERE id = ?1
                 LIMIT 1",
                [id],
            )
            .await?;

        rows.next()
            .await?
            .map(|row| parse_source_document_row(&row))
            .transpose()
    }

    async fn replace_original_snapshots(
        &self,
        source_document_id: &str,
        snapshots: Vec<OriginalQuestionSnapshot>,
    ) -> Result<(), AppError> {
        self.connection
            .execute("DELETE FROM question_original_snapshots", ())
            .await?;

        for snapshot in snapshots {
            self.connection
                .execute(
                    "INSERT INTO question_original_snapshots
                        (id, question_id, source_document_id, code, text, scope, format,
                         convention_code, status, factor, characteristic, aspect, audiences_json,
                         content_hash, marked_by, marked_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![
                        snapshot.id.as_str(),
                        snapshot.question_id.as_str(),
                        source_document_id,
                        snapshot.code.as_str(),
                        snapshot.text.as_str(),
                        snapshot.scope.as_str(),
                        snapshot.format.as_str(),
                        snapshot.convention_code.clone().unwrap_or_default(),
                        snapshot.status.as_str(),
                        snapshot.factor.as_str(),
                        snapshot.characteristic.as_str(),
                        snapshot.aspect.as_str(),
                        serde_json::to_string(&snapshot.audiences)?,
                        snapshot.content_hash.as_str(),
                        snapshot.marked_by.as_str(),
                        snapshot.marked_at.to_rfc3339(),
                    ],
                )
                .await?;
        }

        Ok(())
    }

    async fn list_original_snapshots(&self) -> Result<Vec<OriginalQuestionSnapshot>, AppError> {
        let mut rows = self
            .connection
            .query(
                "SELECT id, question_id, source_document_id, code, text, scope, format,
                        convention_code, status, factor, characteristic, aspect, audiences_json,
                        content_hash, marked_at, marked_by
                 FROM question_original_snapshots
                 ORDER BY code",
                (),
            )
            .await?;
        let mut snapshots = Vec::new();

        while let Some(row) = rows.next().await? {
            snapshots.push(parse_original_snapshot_row(&row)?);
        }

        Ok(snapshots)
    }

    async fn record_change(
        &self,
        entity: &str,
        entity_id: &str,
        action: &str,
        editor_name: &str,
        summary: &str,
    ) -> Result<(), AppError> {
        self.connection
            .execute(
                "INSERT INTO change_logs
                    (id, entity, entity_id, action, editor_name, summary, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    Uuid::new_v4().to_string(),
                    entity,
                    entity_id,
                    action,
                    editor_name,
                    summary,
                    Utc::now().to_rfc3339(),
                ],
            )
            .await?;
        Ok(())
    }

    async fn list_change_logs(&self, limit: usize) -> Result<Vec<ChangeLogEntry>, AppError> {
        let limit = limit.clamp(1, 100) as i64;
        let mut rows = self
            .connection
            .query(
                "SELECT id, entity, entity_id, action, editor_name, summary, created_at
                 FROM change_logs
                 ORDER BY created_at DESC
                 LIMIT ?1",
                [limit],
            )
            .await?;
        let mut entries = Vec::new();

        while let Some(row) = rows.next().await? {
            entries.push(parse_change_log_row(&row)?);
        }

        Ok(entries)
    }

    async fn create_history_snapshot(
        &self,
        summary: &str,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError> {
        self.create_history_snapshot_with_kind(summary, editor_name, "auto", true)
            .await
    }

    async fn create_manual_history_snapshot(
        &self,
        summary: &str,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError> {
        self.create_history_snapshot_with_kind(summary, editor_name, "manual", false)
            .await
    }

    async fn list_history_snapshots(&self) -> Result<Vec<HistorySnapshot>, AppError> {
        history::list_snapshots(&self.connection).await
    }

    async fn restore_history_snapshot(&self, snapshot_id: &str) -> Result<(), AppError> {
        let data = history::load_snapshot(&self.connection, snapshot_id).await?;

        self.connection.execute("DELETE FROM questions", ()).await?;
        self.connection
            .execute("DELETE FROM guideline_aspects", ())
            .await?;

        self.upsert_guideline_aspects(
            data.guideline_aspects
                .into_iter()
                .map(|aspect| NewGuidelineAspect {
                    guideline_title: aspect.guideline_title,
                    scope: aspect.scope,
                    factor_code: aspect.factor_code,
                    factor_name: aspect.factor_name,
                    characteristic_code: aspect.characteristic_code,
                    characteristic_name: aspect.characteristic_name,
                    aspect_code: aspect.aspect_code,
                    aspect_description: aspect.aspect_description,
                    requires_appreciation: aspect.requires_appreciation,
                })
                .collect(),
        )
        .await?;
        self.upsert_questions(
            data.questions
                .into_iter()
                .map(|question| NewQuestion {
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
                })
                .collect(),
        )
        .await?;

        Ok(())
    }

    async fn delete_history_snapshot(&self, snapshot_id: &str) -> Result<(), AppError> {
        history::delete_manual_snapshot(&self.connection, snapshot_id).await
    }

    async fn list_provider_links(&self) -> Result<Vec<ProviderLink>, AppError> {
        provider::list_provider_links(&self.connection).await
    }

    async fn insert_provider_link(&self, link: NewProviderLink) -> Result<ProviderLink, AppError> {
        provider::insert_provider_link(&self.connection, link).await
    }

    async fn list_provider_question_reviews(
        &self,
    ) -> Result<Vec<ProviderQuestionReview>, AppError> {
        provider::list_provider_question_reviews(&self.connection).await
    }

    async fn save_provider_question_review(
        &self,
        review: SaveProviderQuestionReviewRequest,
    ) -> Result<ProviderQuestionReview, AppError> {
        provider::save_provider_question_review(&self.connection, review).await
    }

    async fn reset_provider_question_reviews(&self) -> Result<usize, AppError> {
        provider::reset_provider_question_reviews(&self.connection).await
    }

    async fn save_validation_run(&self, issues: &[ValidationIssue]) -> Result<(), AppError> {
        self.connection
            .execute("DELETE FROM validation_checks", ())
            .await?;

        for issue in issues {
            let severity = match issue.severity {
                ValidationSeverity::Blocking => "blocking",
                ValidationSeverity::Warning => "warning",
            };
            self.connection
                .execute(
                    "INSERT INTO validation_checks
                        (id, severity, entity, entity_id, message, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        issue.id.as_str(),
                        severity,
                        issue.entity.as_str(),
                        issue.entity_id.as_str(),
                        issue.message.as_str(),
                        Utc::now().to_rfc3339()
                    ],
                )
                .await?;
        }

        Ok(())
    }

    async fn reset_database_data(&self) -> Result<(), AppError> {
        self.connection
            .execute_batch(
                "
                DELETE FROM validation_checks;
                DELETE FROM provider_links;
                DELETE FROM instrument_exports;
                DELETE FROM question_original_snapshots;
                DELETE FROM source_documents;
                DELETE FROM questions;
                DELETE FROM guideline_aspects;
                DELETE FROM survey_cycles;
                ",
            )
            .await?;

        Ok(())
    }
}
