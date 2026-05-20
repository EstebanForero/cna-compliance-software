use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    normalize_aspect_code, ChangeLogEntry, DashboardSummary, DeleteGuidelineAspectRequest,
    DeleteGuidelineAspectResult, DeleteHistorySnapshotRequest, GuidelineAspect, HistorySnapshot,
    ImportWorkbookPreviewResult, ImportWorkbookRequest, ImportWorkbookResult, NewGuidelineAspect,
    NewQuestion, NewSourceDocument, Question, QuestionStatus, ResetDatabaseRequest,
    ResetDatabaseResult, RestoreHistorySnapshotRequest, StatusCount, UpdateGuidelineAspectRequest,
    UpdateGuidelineAspectResult, UpdateQuestionRequest, ValidationSeverity, WorkspaceStatus,
};
use crate::error::AppError;
use crate::importer::parse_questions_workbook;
use crate::repository::AutoEvalRepository;

mod baseline;
mod export;
mod provider;
#[cfg(test)]
mod tests;
mod validation;

pub struct AutoEvaluationService {
    pub(crate) repository: Arc<dyn AutoEvalRepository>,
}

impl AutoEvaluationService {
    pub fn new(repository: Arc<dyn AutoEvalRepository>) -> Self {
        Self { repository }
    }

    pub async fn dashboard(
        &self,
        mut workspace: WorkspaceStatus,
    ) -> Result<DashboardSummary, AppError> {
        let active_cycle = self.repository.active_cycle().await?;
        let questions = self.repository.list_questions().await?;
        let links = self.repository.list_provider_links().await?;
        let validations = validation::validate_questions(&questions);
        workspace.has_questions = !questions.is_empty();

        Ok(DashboardSummary {
            active_cycle,
            workspace,
            total_questions: questions.len(),
            pending_changes: questions
                .iter()
                .filter(|question| question.status != QuestionStatus::Keep)
                .count(),
            blocking_validations: validations
                .iter()
                .filter(|issue| issue.severity == ValidationSeverity::Blocking)
                .count(),
            provider_links_pending: links
                .iter()
                .filter(|link| link.validation_status != "validated")
                .count(),
            questions_by_status: self.count_by_status(&questions),
        })
    }

    pub async fn list_questions(&self) -> Result<Vec<Question>, AppError> {
        self.repository.list_questions().await
    }

    pub async fn list_guideline_aspects(&self) -> Result<Vec<GuidelineAspect>, AppError> {
        self.repository.list_guideline_aspects().await
    }

    pub async fn create_guideline_aspect(
        &self,
        mut aspect: NewGuidelineAspect,
    ) -> Result<GuidelineAspect, AppError> {
        normalize_guideline_aspect(&mut aspect)?;

        self.repository.insert_guideline_aspect(aspect).await
    }

    pub async fn update_guideline_aspect(
        &self,
        request: UpdateGuidelineAspectRequest,
        editor_name: &str,
    ) -> Result<UpdateGuidelineAspectResult, AppError> {
        let mut aspect = request.aspect;
        normalize_guideline_aspect(&mut aspect)?;
        self.repository
            .create_history_snapshot("Before updating guideline aspect", editor_name)
            .await?;
        self.repository
            .update_guideline_aspect(&request.aspect_id, aspect)
            .await
    }

    pub async fn create_question(&self, mut question: NewQuestion) -> Result<Question, AppError> {
        if question.text.trim().is_empty() {
            return Err(AppError::Validation("question text is required".into()));
        }
        if question.code.trim().is_empty() {
            let existing = self.repository.list_questions().await?;
            question.code = next_question_code(&existing);
        }

        if question.audiences.is_empty() {
            return Err(AppError::Validation(
                "at least one subaudience must be assigned".into(),
            ));
        }

        self.repository.insert_question(question).await
    }

    pub async fn update_question(
        &self,
        request: UpdateQuestionRequest,
        editor_name: &str,
    ) -> Result<Question, AppError> {
        let existing = self
            .repository
            .list_questions()
            .await?
            .into_iter()
            .find(|question| question.id == request.question_id)
            .ok_or_else(|| AppError::Validation("question not found".into()))?;
        let mut question = request.question;

        if question.text.trim().is_empty() {
            return Err(AppError::Validation("question text is required".into()));
        }
        if question.code.trim().is_empty() {
            question.code = existing.code.clone();
        }
        if question.audiences.is_empty() {
            return Err(AppError::Validation(
                "at least one subaudience must be assigned".into(),
            ));
        }

        if !question_content_changed(&existing, &question) {
            return Ok(existing);
        }

        question.status = next_question_status(&existing.status, &question.status);

        self.repository
            .create_history_snapshot("Before updating question", editor_name)
            .await?;
        self.repository
            .update_question(&request.question_id, question)
            .await
    }

    pub async fn delete_guideline_aspect(
        &self,
        request: DeleteGuidelineAspectRequest,
        editor_name: &str,
    ) -> Result<DeleteGuidelineAspectResult, AppError> {
        if request.confirmation_text.trim() != "ELIMINAR LINEAMIENTO" {
            return Err(AppError::Validation(
                "type ELIMINAR LINEAMIENTO to confirm deletion".into(),
            ));
        }
        if !request.acknowledge_related_questions {
            return Err(AppError::Validation(
                "acknowledge that related questions will be deleted".into(),
            ));
        }

        self.repository
            .create_history_snapshot("Before deleting guideline aspect", editor_name)
            .await?;
        let affected_questions = self
            .repository
            .delete_guideline_aspect_and_related_questions(&request.aspect_id)
            .await?;
        Ok(DeleteGuidelineAspectResult {
            deleted_aspect: true,
            affected_questions,
        })
    }

    pub async fn list_change_logs(&self) -> Result<Vec<ChangeLogEntry>, AppError> {
        self.repository.list_change_logs(100).await
    }

    pub async fn list_history_snapshots(&self) -> Result<Vec<HistorySnapshot>, AppError> {
        self.repository.list_history_snapshots().await
    }

    pub async fn save_manual_history_snapshot(
        &self,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError> {
        self.repository
            .create_manual_history_snapshot("Manual save", editor_name)
            .await
    }

    pub async fn delete_history_snapshot(
        &self,
        request: DeleteHistorySnapshotRequest,
    ) -> Result<(), AppError> {
        if request.confirmation_text.trim() != "ELIMINAR HISTORIAL" {
            return Err(AppError::Validation(
                "type ELIMINAR HISTORIAL to confirm deletion".into(),
            ));
        }
        self.repository
            .delete_history_snapshot(&request.snapshot_id)
            .await
    }

    pub async fn restore_history_snapshot(
        &self,
        request: RestoreHistorySnapshotRequest,
        editor_name: &str,
    ) -> Result<(), AppError> {
        if request.confirmation_text.trim() != "RESTAURAR HISTORIAL" {
            return Err(AppError::Validation(
                "type RESTAURAR HISTORIAL to confirm restore".into(),
            ));
        }
        self.repository
            .create_history_snapshot("Before restoring history snapshot", editor_name)
            .await?;
        self.repository
            .restore_history_snapshot(&request.snapshot_id)
            .await
    }

    pub async fn reset_database_data(
        &self,
        request: ResetDatabaseRequest,
    ) -> Result<ResetDatabaseResult, AppError> {
        if request.confirmation_text.trim() != "BORRAR DATOS" {
            return Err(AppError::Validation(
                "type BORRAR DATOS to confirm database cleanup".into(),
            ));
        }
        if !request.acknowledge_backup || !request.acknowledge_irreversible {
            return Err(AppError::Validation(
                "backup and irreversible action acknowledgements are required".into(),
            ));
        }

        self.repository
            .create_history_snapshot("Before database cleanup", "system")
            .await?;
        self.repository.reset_database_data().await?;

        Ok(ResetDatabaseResult {
            deleted: true,
            message:
                "Database content was deleted. Workspace settings and editor profile were kept."
                    .into(),
        })
    }

    pub async fn import_workbook(
        &self,
        request: ImportWorkbookRequest,
        editor_name: &str,
    ) -> Result<ImportWorkbookResult, AppError> {
        let path = std::path::PathBuf::from(&request.path);
        let parsed = parse_questions_workbook(&path)?;
        let should_fix_initial_original =
            self.repository.list_original_snapshots().await?.is_empty();
        let cycle_name = request
            .cycle_name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Ciclo importado".into());
        self.repository.ensure_cycle(&cycle_name).await?;
        let imported_guideline_aspects = self
            .repository
            .upsert_guideline_aspects(parsed.guideline_aspects)
            .await?;
        let imported = self.repository.upsert_questions(parsed.questions).await?;
        let source_document_id = Uuid::new_v4().to_string();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("consolidado.xlsx")
            .to_string();

        let source_document = NewSourceDocument {
            id: source_document_id.clone(),
            file_name: file_name.clone(),
            path: request.path,
            document_type: "questions_consolidated".into(),
            imported_rows: imported,
            skipped_rows: parsed.skipped_rows,
        };
        self.repository
            .save_source_document(source_document.clone())
            .await?;

        if should_fix_initial_original && imported > 0 {
            let questions = self.repository.list_questions().await?;
            let source_document = self
                .repository
                .get_source_document(&source_document_id)
                .await?
                .ok_or_else(|| {
                    AppError::Validation("imported source document was not saved".into())
                })?;
            self.mark_questions_as_original_baseline(
                questions,
                source_document,
                editor_name,
                "Fijacion inicial desde consolidado",
            )
            .await?;
        }

        Ok(ImportWorkbookResult {
            source_document_id,
            file_name,
            sheet_name: parsed.sheet_name,
            imported_questions: imported,
            imported_guideline_aspects,
            skipped_rows: parsed.skipped_rows,
            detected_columns: parsed.detected_columns,
        })
    }

    pub async fn preview_import_workbook(
        &self,
        request: ImportWorkbookRequest,
    ) -> Result<ImportWorkbookPreviewResult, AppError> {
        let parsed = parse_questions_workbook(std::path::Path::new(&request.path))?;
        let mut audiences = parsed
            .questions
            .iter()
            .flat_map(|question| question.audiences.clone())
            .collect::<Vec<_>>();
        audiences.sort();
        audiences.dedup();

        let mut warnings = Vec::new();
        if parsed.questions.is_empty() {
            warnings.push("No se detectaron preguntas importables.".into());
        }
        if parsed.guideline_aspects.is_empty() {
            warnings.push("No se detectaron lineamientos CNA.".into());
        }
        let questions_without_audience = parsed
            .questions
            .iter()
            .filter(|question| question.audiences.is_empty())
            .count();
        if questions_without_audience > 0 {
            warnings.push(format!(
                "{questions_without_audience} preguntas no tienen publico detectado."
            ));
        }

        Ok(ImportWorkbookPreviewResult {
            file_name: std::path::Path::new(&request.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("workbook.xlsx")
                .to_string(),
            sheet_name: parsed.sheet_name,
            detected_questions: parsed.questions.len(),
            detected_guideline_aspects: parsed.guideline_aspects.len(),
            skipped_rows: parsed.skipped_rows,
            detected_columns: parsed.detected_columns,
            detected_audiences: audiences,
            warnings,
        })
    }

    pub async fn record_change(
        &self,
        entity: &str,
        entity_id: &str,
        action: &str,
        editor_name: &str,
        summary: &str,
    ) -> Result<(), AppError> {
        self.repository
            .record_change(entity, entity_id, action, editor_name, summary)
            .await
    }

    fn count_by_status(&self, questions: &[Question]) -> Vec<StatusCount> {
        [
            QuestionStatus::Keep,
            QuestionStatus::Modify,
            QuestionStatus::Add,
            QuestionStatus::Delete,
        ]
        .into_iter()
        .map(|status| StatusCount {
            count: questions
                .iter()
                .filter(|question| question.status == status)
                .count(),
            status,
        })
        .collect()
    }
}

fn next_question_code(existing: &[Question]) -> String {
    let next = existing
        .iter()
        .filter_map(|question| question.code.strip_prefix("APP-"))
        .filter_map(|value| value.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    format!("APP-{next:04}")
}

fn question_content_changed(existing: &Question, question: &NewQuestion) -> bool {
    existing.code != question.code
        || existing.text != question.text
        || existing.scope != question.scope
        || existing.format != question.format
        || existing.convention_code != question.convention_code
        || existing.status != question.status
        || existing.factor != question.factor
        || existing.characteristic != question.characteristic
        || existing.aspect != question.aspect
        || existing.audiences != question.audiences
        || existing.justification != question.justification
}

fn next_question_status(existing: &QuestionStatus, requested: &QuestionStatus) -> QuestionStatus {
    match (existing, requested) {
        (_, QuestionStatus::Delete) => QuestionStatus::Delete,
        (QuestionStatus::Add, _) => QuestionStatus::Add,
        (QuestionStatus::Delete, _) => QuestionStatus::Delete,
        (QuestionStatus::Keep | QuestionStatus::Modify, QuestionStatus::Add) => QuestionStatus::Add,
        (QuestionStatus::Keep | QuestionStatus::Modify, _) => QuestionStatus::Modify,
    }
}

fn normalize_guideline_aspect(aspect: &mut NewGuidelineAspect) -> Result<(), AppError> {
    if aspect.guideline_title.trim().is_empty()
        || aspect.factor_code.as_str().trim().is_empty()
        || aspect.characteristic_code.trim().is_empty()
        || aspect.characteristic_name.trim().is_empty()
        || aspect.aspect_description.trim().is_empty()
    {
        return Err(AppError::Validation(
            "guideline title, factor, characteristic and aspect are required".into(),
        ));
    }

    if aspect.factor_name.trim().is_empty() {
        aspect.factor_name = aspect.factor_code.label();
    }

    aspect.guideline_title = aspect.guideline_title.trim().to_string();
    aspect.factor_name = aspect.factor_name.trim().to_string();
    aspect.characteristic_code = aspect.characteristic_code.trim().to_string();
    aspect.characteristic_name = aspect.characteristic_name.trim().to_string();
    aspect.aspect_description = aspect.aspect_description.trim().to_string();
    aspect.aspect_code = normalize_aspect_code(
        &aspect.factor_code,
        &aspect.characteristic_code,
        &aspect.aspect_code,
        &aspect.aspect_description,
    );

    Ok(())
}
