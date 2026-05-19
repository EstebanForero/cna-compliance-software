use std::collections::BTreeSet;
use std::sync::Arc;

use calamine::{open_workbook_auto, Reader};
use chrono::{NaiveDate, Utc};
use mockall::predicate::always;
use uuid::Uuid;

use super::*;
use crate::db::LibSqlAutoEvalRepository;
use crate::domain::{
    CycleStatus, ExportKind, ExportWorkbookRequest, GuidelineAspect, ImportWorkbookRequest,
    MarkOriginalBaselineRequest, NewProviderLink, NewQuestion, ProviderQuestionReview,
    ProviderQuestionReviewStatus, QuestionScope, ResetProviderQuestionReviewsRequest, SurveyCycle,
};
use crate::importer::parse_questions_workbook;
use crate::repository::{AutoEvalRepository, MockAutoEvalRepository};

fn cycle() -> SurveyCycle {
    SurveyCycle {
        id: "cycle-1".into(),
        name: "2026-2027".into(),
        starts_on: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        application_starts_on: NaiveDate::from_ymd_opt(2027, 1, 15).unwrap(),
        status: CycleStatus::Planning,
        notes: "Preparacion del ciclo CNA".into(),
    }
}

fn question(status: QuestionStatus, convention_code: Option<String>) -> Question {
    Question {
        id: "q-1".into(),
        code: "EST-001".into(),
        text: "La institucion comunica claramente los resultados de autoevaluacion.".into(),
        scope: QuestionScope::Institutional,
        format: "likert".into(),
        convention_code,
        status,
        factor: "Factor 1".into(),
        characteristic: "Caracteristica 1".into(),
        aspect: "Comunicacion de resultados".into(),
        audiences: vec!["Pregrado".into()],
        justification: None,
        updated_at: Utc::now(),
    }
}

fn new_question_from(question: &Question) -> NewQuestion {
    NewQuestion {
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
        justification: question.justification.clone(),
    }
}

#[tokio::test]
async fn dashboard_counts_pending_changes_and_blocking_validations() {
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_active_cycle()
        .returning(|| Ok(Some(cycle())));
    repository
        .expect_list_provider_links()
        .returning(|| Ok(vec![]));
    repository.expect_list_questions().returning(|| {
        Ok(vec![
            question(QuestionStatus::Keep, Some("A".into())),
            question(QuestionStatus::Modify, None),
        ])
    });

    let service = AutoEvaluationService::new(Arc::new(repository));
    let summary = service.dashboard(workspace()).await.unwrap();

    assert_eq!(summary.total_questions, 2);
    assert_eq!(summary.pending_changes, 1);
    assert_eq!(summary.blocking_validations, 1);
}

#[tokio::test]
async fn create_question_rejects_missing_audience_before_repository_call() {
    let mut repository = MockAutoEvalRepository::new();
    repository.expect_insert_question().never();

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .create_question(NewQuestion {
            code: "NEW-1".into(),
            text: "Pregunta nueva".into(),
            scope: QuestionScope::Program,
            format: "likert".into(),
            convention_code: Some("B".into()),
            status: QuestionStatus::Add,
            factor: "Factor 2".into(),
            characteristic: "Caracteristica 2".into(),
            aspect: "Aspecto".into(),
            audiences: vec![],
            justification: None,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn update_question_marks_kept_question_as_modify_when_content_changes() {
    let existing = question(QuestionStatus::Keep, Some("A".into()));
    let mut update = new_question_from(&existing);
    update.text = "Texto actualizado".into();
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_list_questions()
        .returning(move || Ok(vec![existing.clone()]));
    repository
        .expect_create_history_snapshot()
        .returning(|_, _| {
            Ok(crate::domain::HistorySnapshot {
                id: Uuid::new_v4().to_string(),
                summary: "snapshot".into(),
                editor_name: "Editor".into(),
                snapshot_kind: "auto".into(),
                created_at: Utc::now(),
            })
        });
    repository
        .expect_update_question()
        .withf(|id, candidate| id == "q-1" && candidate.status == QuestionStatus::Modify)
        .returning(|_, candidate| {
            let mut persisted =
                question(candidate.status.clone(), candidate.convention_code.clone());
            persisted.text = candidate.text.clone();
            Ok(persisted)
        });

    let service = AutoEvaluationService::new(Arc::new(repository));
    let updated = service
        .update_question(
            UpdateQuestionRequest {
                question_id: "q-1".into(),
                question: update,
            },
            "Editor",
        )
        .await
        .unwrap();

    assert_eq!(updated.status, QuestionStatus::Modify);
}

#[tokio::test]
async fn update_question_noops_when_content_did_not_change() {
    let existing = question(QuestionStatus::Keep, Some("A".into()));
    let update = new_question_from(&existing);
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_list_questions()
        .returning(move || Ok(vec![existing.clone()]));
    repository.expect_create_history_snapshot().never();
    repository.expect_update_question().never();

    let service = AutoEvaluationService::new(Arc::new(repository));
    let updated = service
        .update_question(
            UpdateQuestionRequest {
                question_id: "q-1".into(),
                question: update,
            },
            "Editor",
        )
        .await
        .unwrap();

    assert_eq!(updated.status, QuestionStatus::Keep);
}

#[tokio::test]
async fn provider_links_must_use_https() {
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_insert_provider_link()
        .with(always())
        .never();

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .record_provider_link(NewProviderLink {
            subaudience: "Pregrado".into(),
            url: "http://example.test".into(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn provider_review_items_are_split_by_instrument_audience() {
    let base_question = Question {
        audiences: vec!["Estudiantes".into(), "Profesores".into()],
        ..question(QuestionStatus::Keep, Some("A".into()))
    };
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_list_questions()
        .returning(move || Ok(vec![base_question.clone()]));
    repository
        .expect_list_provider_question_reviews()
        .returning(|| {
            Ok(vec![ProviderQuestionReview {
                id: "review-1".into(),
                question_id: "q-1".into(),
                instrument_audience: "Profesores".into(),
                status: ProviderQuestionReviewStatus::Missing,
                observation: "No esta en el instrumento".into(),
                evidence_path: None,
                updated_at: Utc::now(),
            }])
        });

    let service = AutoEvaluationService::new(Arc::new(repository));
    let items = service.list_provider_question_review_items().await.unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].instrument_audience, "Estudiantes");
    assert!(items[0].review.is_none());
    assert_eq!(items[1].instrument_audience, "Profesores");
    assert_eq!(
        items[1].review.as_ref().map(|review| &review.status),
        Some(&ProviderQuestionReviewStatus::Missing)
    );
}

#[tokio::test]
async fn reset_provider_reviews_requires_explicit_confirmation() {
    let mut repository = MockAutoEvalRepository::new();
    repository.expect_reset_provider_question_reviews().never();

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .reset_provider_question_reviews(ResetProviderQuestionReviewsRequest {
            confirmation_text: "reiniciar".into(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn reset_provider_reviews_returns_deleted_count() {
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_reset_provider_question_reviews()
        .returning(|| Ok(7));

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .reset_provider_question_reviews(ResetProviderQuestionReviewsRequest {
            confirmation_text: "REINICIAR REVISION".into(),
        })
        .await
        .unwrap();

    assert_eq!(result.deleted_reviews, 7);
}

#[tokio::test]
async fn imports_and_exports_consolidated_workbook_without_losing_structure() {
    let input_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../example-files/Consolidado de preguntas Enc de Aut Ins y Pr 2024 1.xlsx");
    if !input_path.exists() {
        return;
    }

    let repository = Arc::new(LibSqlAutoEvalRepository::open_in_memory().await.unwrap());
    let service = AutoEvaluationService::new(repository.clone());
    let import_result = service
        .import_workbook(ImportWorkbookRequest {
            path: input_path.to_string_lossy().into_owned(),
            cycle_name: Some("Roundtrip".into()),
        })
        .await
        .unwrap();

    let imported_questions = repository.list_questions().await.unwrap();
    let imported_aspects = repository.list_guideline_aspects().await.unwrap();
    assert_eq!(imported_questions.len(), import_result.imported_questions);
    assert_eq!(
        imported_aspects.len(),
        import_result.imported_guideline_aspects
    );

    service
        .mark_original_baseline(
            MarkOriginalBaselineRequest {
                source_document_id: Some(import_result.source_document_id),
                confirmation_text: "FIJAR ORIGINAL".into(),
                acknowledge_replacement: true,
                acknowledge_backup: true,
            },
            "Test Editor",
        )
        .await
        .unwrap();

    let export_path =
        std::env::temp_dir().join(format!("autoevaluacion-roundtrip-{}.xlsx", Uuid::new_v4()));
    service
        .export_workbook(ExportWorkbookRequest {
            path: export_path.to_string_lossy().into_owned(),
            kind: ExportKind::Consolidated,
        })
        .await
        .unwrap();

    let exported = parse_questions_workbook(&export_path).unwrap();
    assert_eq!(exported.questions.len(), imported_questions.len());
    assert_eq!(
        question_structure_set(&exported.questions),
        imported_questions
            .iter()
            .map(|question| {
                (
                    question.code.clone(),
                    question.text.clone(),
                    question.factor.clone(),
                    question.characteristic.clone(),
                    question.aspect.clone(),
                )
            })
            .collect()
    );
    assert_eq!(
        factor_characteristic_set(&exported.guideline_aspects),
        factor_characteristic_set_from_persisted(&imported_aspects)
    );
    assert!(!exported.guideline_aspects.is_empty());

    let instrument_path = std::env::temp_dir().join(format!(
        "autoevaluacion-instruments-{}.xlsx",
        Uuid::new_v4()
    ));
    service
        .export_workbook(ExportWorkbookRequest {
            path: instrument_path.to_string_lossy().into_owned(),
            kind: ExportKind::Instruments,
        })
        .await
        .unwrap();

    let mut workbook = open_workbook_auto(&instrument_path).unwrap();
    let sheet_names = workbook.sheet_names().to_vec();
    assert!(sheet_names.contains(&"Por lineamiento".to_string()));
    assert!(sheet_names.contains(&"Por orden".to_string()));
    assert!(sheet_names.contains(&"Convención".to_string()));
    let order = workbook.worksheet_range("Por orden").unwrap();
    assert!(order
        .get_value((0, 0))
        .map(|value| value.to_string().contains("INSTRUMENTO POR ORDEN"))
        .unwrap_or(false));
    assert_eq!(
        order.get_value((1, 0)).map(|value| value.to_string()),
        Some("# Pregunta".into())
    );
    assert!(order
        .rows()
        .nth(1)
        .unwrap()
        .iter()
        .skip(2)
        .any(|value| !value.to_string().trim().is_empty()));

    let _ = std::fs::remove_file(export_path);
    let _ = std::fs::remove_file(instrument_path);
}

fn workspace() -> WorkspaceStatus {
    WorkspaceStatus {
        database_path: "/tmp/autoevaluacion-cna.db".into(),
        configured_onedrive_path: None,
        microsoft_account: None,
        microsoft_auth_config: None,
        editor_profile: None,
        graph_sync_available: false,
        has_questions: true,
    }
}

fn question_structure_set(
    questions: &[NewQuestion],
) -> BTreeSet<(String, String, String, String, String)> {
    questions
        .iter()
        .map(|question| {
            (
                question.code.clone(),
                question.text.clone(),
                question.factor.clone(),
                question.characteristic.clone(),
                question.aspect.clone(),
            )
        })
        .collect()
}

fn factor_characteristic_set(
    aspects: &[NewGuidelineAspect],
) -> BTreeSet<(String, String, String, String, String)> {
    aspects
        .iter()
        .map(|aspect| {
            (
                aspect.scope.as_str().to_string(),
                aspect.factor_code.as_str().to_string(),
                aspect.factor_name.clone(),
                aspect.characteristic_code.clone(),
                aspect.characteristic_name.clone(),
            )
        })
        .collect()
}

fn factor_characteristic_set_from_persisted(
    aspects: &[GuidelineAspect],
) -> BTreeSet<(String, String, String, String, String)> {
    aspects
        .iter()
        .map(|aspect| {
            (
                aspect.scope.as_str().to_string(),
                aspect.factor_code.as_str().to_string(),
                aspect.factor_name.clone(),
                aspect.characteristic_code.clone(),
                aspect.characteristic_name.clone(),
            )
        })
        .collect()
}
