use uuid::Uuid;

use crate::domain::{Question, QuestionStatus, ValidationIssue, ValidationSeverity};
use crate::error::AppError;
use crate::service::AutoEvaluationService;

impl AutoEvaluationService {
    pub async fn run_validations(&self) -> Result<Vec<ValidationIssue>, AppError> {
        let questions = self.repository.list_questions().await?;
        let issues = validate_questions(&questions);
        self.repository.save_validation_run(&issues).await?;
        Ok(issues)
    }
}

pub(super) fn validate_questions(questions: &[Question]) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for question in questions {
        if question.audiences.is_empty() {
            issues.push(ValidationIssue {
                id: Uuid::new_v4().to_string(),
                severity: ValidationSeverity::Blocking,
                entity: "question".into(),
                entity_id: question.id.clone(),
                message: format!("{} has no assigned subaudiences", question.code),
            });
        }

        if question.convention_code.is_none() && question.format != "open" {
            issues.push(ValidationIssue {
                id: Uuid::new_v4().to_string(),
                severity: ValidationSeverity::Blocking,
                entity: "question".into(),
                entity_id: question.id.clone(),
                message: format!("{} has no response convention", question.code),
            });
        }

        if question.status == QuestionStatus::Delete && question.justification.is_none() {
            issues.push(ValidationIssue {
                id: Uuid::new_v4().to_string(),
                severity: ValidationSeverity::Warning,
                entity: "question".into(),
                entity_id: question.id.clone(),
                message: format!(
                    "{} is marked for deletion without justification",
                    question.code
                ),
            });
        }
    }

    issues
}
