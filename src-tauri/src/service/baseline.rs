use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::{
    BaselineStatus, MarkOriginalBaselineRequest, OriginalQuestionSnapshot, Question,
    QuestionDiffKind, QuestionStatus, SourceDocument,
};
use crate::error::AppError;
use crate::service::AutoEvaluationService;

impl AutoEvaluationService {
    pub async fn baseline_status(&self) -> Result<BaselineStatus, AppError> {
        let current = self.repository.list_questions().await?;
        let snapshots = self.repository.list_original_snapshots().await?;
        let source_document = if let Some(snapshot) = snapshots.first() {
            self.repository
                .get_source_document(&snapshot.source_document_id)
                .await?
        } else {
            None
        };

        Ok(build_baseline_status(current, snapshots, source_document))
    }

    pub async fn mark_original_baseline(
        &self,
        request: MarkOriginalBaselineRequest,
        editor_name: &str,
    ) -> Result<BaselineStatus, AppError> {
        if request.confirmation_text.trim() != "FIJAR ORIGINAL" {
            return Err(AppError::Validation(
                "type FIJAR ORIGINAL to confirm the original baseline".into(),
            ));
        }
        if !request.acknowledge_replacement || !request.acknowledge_backup {
            return Err(AppError::Validation(
                "confirm replacement impact and backup acknowledgement before marking original"
                    .into(),
            ));
        }

        let questions = self.repository.list_questions().await?;
        if questions.is_empty() {
            return Err(AppError::Validation(
                "import questions before marking an original baseline".into(),
            ));
        }

        let source_document = match request.source_document_id {
            Some(id) => self.repository.get_source_document(&id).await?,
            None => self.repository.latest_source_document().await?,
        }
        .ok_or_else(|| AppError::Validation("no imported source document found".into()))?;

        self.mark_questions_as_original_baseline(
            questions,
            source_document,
            editor_name,
            "Fijacion de original",
        )
        .await
    }

    pub(super) async fn mark_questions_as_original_baseline(
        &self,
        questions: Vec<Question>,
        source_document: SourceDocument,
        editor_name: &str,
        history_summary: &str,
    ) -> Result<BaselineStatus, AppError> {
        let marked_at = Utc::now();
        let snapshots = questions
            .iter()
            .map(|question| OriginalQuestionSnapshot {
                id: Uuid::new_v4().to_string(),
                question_id: question.id.clone(),
                source_document_id: source_document.id.clone(),
                code: question.code.clone(),
                text: question.text.clone(),
                scope: question.scope.clone(),
                format: question.format.clone(),
                convention_code: question.convention_code.clone(),
                status: question.status.clone(),
                factor: question.factor.clone(),
                characteristic: question.characteristic.clone(),
                aspect: question.aspect.clone(),
                audiences: question.audiences.clone(),
                content_hash: question_hash(question),
                marked_by: editor_name.to_string(),
                marked_at,
            })
            .collect::<Vec<_>>();

        self.repository
            .replace_original_snapshots(&source_document.id, snapshots.clone())
            .await?;
        self.repository
            .create_baseline_history_snapshot(history_summary, editor_name)
            .await?;

        Ok(build_baseline_status(
            questions,
            snapshots,
            Some(source_document),
        ))
    }
}

pub(super) fn build_baseline_status(
    questions: Vec<Question>,
    snapshots: Vec<OriginalQuestionSnapshot>,
    source_document: Option<SourceDocument>,
) -> BaselineStatus {
    let diffed = diff_questions(&questions, &snapshots);
    BaselineStatus {
        has_original: !snapshots.is_empty(),
        source_document,
        original_questions: snapshots.len(),
        current_questions: questions.len(),
        unchanged_questions: diffed
            .iter()
            .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Unchanged))
            .count(),
        modified_questions: diffed
            .iter()
            .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Modified))
            .count(),
        added_questions: diffed
            .iter()
            .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Added))
            .count(),
        removed_questions: diffed
            .iter()
            .filter(|(_, kind)| matches!(kind, QuestionDiffKind::Removed))
            .count(),
    }
}

pub(super) fn diff_questions(
    questions: &[Question],
    snapshots: &[OriginalQuestionSnapshot],
) -> Vec<(Question, QuestionDiffKind)> {
    let original_by_code = snapshots
        .iter()
        .map(|snapshot| (snapshot.code.as_str(), snapshot))
        .collect::<std::collections::HashMap<_, _>>();

    questions
        .iter()
        .map(|question| {
            let kind = if question.status == QuestionStatus::Delete {
                QuestionDiffKind::Removed
            } else if let Some(snapshot) = original_by_code.get(question.code.as_str()) {
                if snapshot.content_hash == question_hash(question) {
                    QuestionDiffKind::Unchanged
                } else {
                    QuestionDiffKind::Modified
                }
            } else {
                QuestionDiffKind::Added
            };
            (question.clone(), kind)
        })
        .collect()
}

pub(super) fn question_hash(question: &Question) -> String {
    let mut hasher = Sha256::new();
    hasher.update(question.code.as_bytes());
    hasher.update(question.text.as_bytes());
    hasher.update(question.scope.as_str().as_bytes());
    hasher.update(question.format.as_bytes());
    hasher.update(
        question
            .convention_code
            .clone()
            .unwrap_or_default()
            .as_bytes(),
    );
    hasher.update(question.factor.as_bytes());
    hasher.update(question.characteristic.as_bytes());
    hasher.update(question.aspect.as_bytes());
    for audience in &question.audiences {
        hasher.update(audience.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
