use async_trait::async_trait;

use crate::domain::{
    ChangeLogEntry, GuidelineAspect, HistorySnapshot, NewGuidelineAspect, NewProviderLink,
    NewQuestion, NewSourceDocument, OriginalQuestionSnapshot, ProviderLink, ProviderQuestionReview,
    Question, SaveProviderQuestionReviewRequest, SourceDocument, SurveyCycle,
    UpdateGuidelineAspectResult, ValidationIssue,
};
use crate::error::AppError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AutoEvalRepository: Send + Sync {
    async fn active_cycle(&self) -> Result<Option<SurveyCycle>, AppError>;
    async fn ensure_cycle(&self, name: &str) -> Result<SurveyCycle, AppError>;
    async fn list_questions(&self) -> Result<Vec<Question>, AppError>;
    async fn list_guideline_aspects(&self) -> Result<Vec<GuidelineAspect>, AppError>;
    async fn insert_guideline_aspect(
        &self,
        aspect: NewGuidelineAspect,
    ) -> Result<GuidelineAspect, AppError>;
    async fn update_guideline_aspect(
        &self,
        aspect_id: &str,
        aspect: NewGuidelineAspect,
    ) -> Result<UpdateGuidelineAspectResult, AppError>;
    async fn upsert_guideline_aspects(
        &self,
        aspects: Vec<NewGuidelineAspect>,
    ) -> Result<usize, AppError>;
    async fn upsert_questions(&self, questions: Vec<NewQuestion>) -> Result<usize, AppError>;
    async fn save_source_document(&self, document: NewSourceDocument) -> Result<(), AppError>;
    async fn latest_source_document(&self) -> Result<Option<SourceDocument>, AppError>;
    async fn get_source_document(&self, id: &str) -> Result<Option<SourceDocument>, AppError>;
    async fn replace_original_snapshots(
        &self,
        source_document_id: &str,
        snapshots: Vec<OriginalQuestionSnapshot>,
    ) -> Result<(), AppError>;
    async fn list_original_snapshots(&self) -> Result<Vec<OriginalQuestionSnapshot>, AppError>;
    async fn record_change(
        &self,
        entity: &str,
        entity_id: &str,
        action: &str,
        editor_name: &str,
        summary: &str,
    ) -> Result<(), AppError>;
    async fn list_change_logs(&self, limit: usize) -> Result<Vec<ChangeLogEntry>, AppError>;
    async fn create_history_snapshot(
        &self,
        summary: &str,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError>;
    async fn create_manual_history_snapshot(
        &self,
        summary: &str,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError>;
    async fn create_baseline_history_snapshot(
        &self,
        summary: &str,
        editor_name: &str,
    ) -> Result<HistorySnapshot, AppError>;
    async fn list_history_snapshots(&self) -> Result<Vec<HistorySnapshot>, AppError>;
    async fn restore_history_snapshot(&self, snapshot_id: &str) -> Result<(), AppError>;
    async fn delete_history_snapshot(&self, snapshot_id: &str) -> Result<(), AppError>;
    async fn insert_question(&self, question: NewQuestion) -> Result<Question, AppError>;
    async fn update_question(
        &self,
        question_id: &str,
        question: NewQuestion,
    ) -> Result<Question, AppError>;
    async fn delete_guideline_aspect_and_related_questions(
        &self,
        aspect_id: &str,
    ) -> Result<usize, AppError>;
    async fn list_provider_links(&self) -> Result<Vec<ProviderLink>, AppError>;
    async fn insert_provider_link(&self, link: NewProviderLink) -> Result<ProviderLink, AppError>;
    async fn list_provider_question_reviews(&self)
        -> Result<Vec<ProviderQuestionReview>, AppError>;
    async fn save_provider_question_review(
        &self,
        review: SaveProviderQuestionReviewRequest,
    ) -> Result<ProviderQuestionReview, AppError>;
    async fn reset_provider_question_reviews(&self) -> Result<usize, AppError>;
    async fn save_validation_run(&self, issues: &[ValidationIssue]) -> Result<(), AppError>;
    async fn reset_database_data(&self) -> Result<(), AppError>;
}
