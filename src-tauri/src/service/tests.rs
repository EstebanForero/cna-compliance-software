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
    MarkOriginalBaselineRequest, NewProviderLink, NewQuestion, NewSourceDocument,
    OriginalQuestionSnapshot, ProviderQuestionReview, ProviderQuestionReviewStatus,
    QuestionDiffKind, QuestionScope, ResetProviderQuestionReviewsRequest, SurveyCycle,
};
use crate::importer::parse_questions_workbook;
use crate::repository::{AutoEvalRepository, MockAutoEvalRepository};
use crate::service::baseline::{diff_questions, question_hash};

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

fn new_question_with_code_and_status(code: &str, status: QuestionStatus) -> NewQuestion {
    let mut question = new_question_from(&question(status, Some("A".into())));
    question.code = code.into();
    question
}

fn snapshot_from_question(question: &Question) -> OriginalQuestionSnapshot {
    OriginalQuestionSnapshot {
        id: format!("snapshot-{}", question.code),
        question_id: question.id.clone(),
        source_document_id: "source-1".into(),
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
        marked_by: "Test Editor".into(),
        marked_at: Utc::now(),
    }
}

#[test]
fn diff_questions_keeps_status_and_missing_originals_for_exports() {
    let mut original_removed = question(QuestionStatus::Keep, Some("A".into()));
    original_removed.id = "removed-id".into();
    original_removed.code = "REM-1".into();
    original_removed.audiences = vec!["Estudiantes Pregrado".into()];

    let mut explicit_modify = question(QuestionStatus::Modify, Some("B".into()));
    explicit_modify.id = "modify-id".into();
    explicit_modify.code = "MOD-1".into();
    explicit_modify.audiences = vec!["Estudiantes Pregrado".into()];
    let mut modify_snapshot = explicit_modify.clone();
    modify_snapshot.status = QuestionStatus::Keep;

    let diffed = diff_questions(
        &[explicit_modify.clone()],
        &[
            snapshot_from_question(&original_removed),
            snapshot_from_question(&modify_snapshot),
        ],
    );

    assert!(diffed.iter().any(|(question, kind)| {
        question.code == "MOD-1" && matches!(kind, QuestionDiffKind::Modified)
    }));
    assert!(diffed.iter().any(|(question, kind)| {
        question.code == "REM-1"
            && question.status == QuestionStatus::Delete
            && matches!(kind, QuestionDiffKind::Removed)
    }));
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
async fn update_question_allows_marking_kept_question_for_deletion() {
    let existing = question(QuestionStatus::Keep, Some("A".into()));
    let mut update = new_question_from(&existing);
    update.status = QuestionStatus::Delete;
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
        .withf(|id, candidate| id == "q-1" && candidate.status == QuestionStatus::Delete)
        .returning(|_, candidate| {
            Ok(question(
                candidate.status.clone(),
                candidate.convention_code.clone(),
            ))
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

    assert_eq!(updated.status, QuestionStatus::Delete);
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
    assert_eq!(items[0].instrument_label, "Estudiantes");
    assert!(items[0].review.is_none());
    assert_eq!(items[1].instrument_audience, "Profesores");
    assert_eq!(items[1].instrument_label, "Profesores");
    assert_eq!(
        items[1].review.as_ref().map(|review| &review.status),
        Some(&ProviderQuestionReviewStatus::Missing)
    );
}

#[tokio::test]
async fn provider_review_groups_subpublics_under_exported_instrument() {
    let public_question = Question {
        id: "q-public".into(),
        code: "PUB-1".into(),
        audiences: vec!["Estudiantes".into()],
        ..question(QuestionStatus::Keep, Some("A".into()))
    };
    let subpublic_question = Question {
        id: "q-subpublic".into(),
        code: "SUB-1".into(),
        audiences: vec![
            "Estudiantes Pregrado".into(),
            "Estudiantes Maestrías virtuales".into(),
        ],
        ..question(QuestionStatus::Keep, Some("A".into()))
    };
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_list_questions()
        .returning(move || Ok(vec![public_question.clone(), subpublic_question.clone()]));
    repository
        .expect_list_provider_question_reviews()
        .returning(|| Ok(vec![]));

    let service = AutoEvaluationService::new(Arc::new(repository));
    let items = service.list_provider_question_review_items().await.unwrap();
    let public_items = items
        .iter()
        .filter(|item| item.question.id == "q-public")
        .map(|item| {
            (
                item.instrument_audience.as_str(),
                item.instrument_label.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(public_items, vec![("Estudiantes", "Estudiantes")]);

    let subpublic_items = items
        .iter()
        .filter(|item| item.question.id == "q-subpublic")
        .map(|item| {
            (
                item.instrument_audience.as_str(),
                item.instrument_label.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(subpublic_items, vec![("Estudiantes", "Estudiantes")]);
}

#[tokio::test]
async fn reset_provider_reviews_requires_explicit_confirmation() {
    let mut repository = MockAutoEvalRepository::new();
    repository.expect_reset_provider_question_reviews().never();

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .reset_provider_question_reviews(ResetProviderQuestionReviewsRequest {
            confirmation_text: "reiniciar".into(),
            instrument_audience: Some("Estudiantes Pregrado".into()),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn reset_provider_reviews_returns_deleted_count() {
    let mut repository = MockAutoEvalRepository::new();
    repository
        .expect_reset_provider_question_reviews()
        .withf(|instrument| instrument.as_deref() == Some("Estudiantes"))
        .returning(|_| Ok(7));

    let service = AutoEvaluationService::new(Arc::new(repository));
    let result = service
        .reset_provider_question_reviews(ResetProviderQuestionReviewsRequest {
            confirmation_text: "REINICIAR REVISION".into(),
            instrument_audience: Some("0Estudiantes 00Pregrado".into()),
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
        .import_workbook(
            ImportWorkbookRequest {
                path: input_path.to_string_lossy().into_owned(),
                cycle_name: Some("Roundtrip".into()),
            },
            "Test Editor",
        )
        .await
        .unwrap();

    let imported_questions = repository.list_questions().await.unwrap();
    let imported_aspects = repository.list_guideline_aspects().await.unwrap();
    let initial_original = repository.list_original_snapshots().await.unwrap();
    let fixation_history = repository.list_history_snapshots().await.unwrap();
    assert_eq!(imported_questions.len(), import_result.imported_questions);
    assert_eq!(initial_original.len(), imported_questions.len());
    assert!(fixation_history.iter().any(|snapshot| {
        snapshot.snapshot_kind == "baseline"
            && snapshot.summary == "Fijacion inicial desde consolidado"
    }));
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
            instrument_public: None,
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

    let instrument_options = service.list_instrument_public_options().await.unwrap();
    let estudiantes = instrument_options
        .iter()
        .find(|option| option.public == "Estudiantes")
        .unwrap();
    assert_eq!(estudiantes.label, "Estudiantes");
    assert!(estudiantes
        .subpublics
        .iter()
        .any(|value| value == "Estudiantes Maestría Virtual"));
    let profesores = instrument_options
        .iter()
        .find(|option| option.public == "Profesores Planta")
        .unwrap();
    assert_eq!(profesores.label, "Profesores de planta");
    assert!(profesores
        .subpublics
        .iter()
        .any(|value| value == "Profesores Pregrado"));

    let all_instruments_path =
        std::env::temp_dir().join(format!("autoevaluacion-all-instruments-{}", Uuid::new_v4()));
    service
        .export_workbook(ExportWorkbookRequest {
            path: all_instruments_path.to_string_lossy().into_owned(),
            kind: ExportKind::Instruments,
            instrument_public: None,
        })
        .await
        .unwrap();
    assert!(all_instruments_path
        .join("instrumento-estudiantes.xlsx")
        .exists());
    assert!(all_instruments_path
        .join("instrumento-profesores-planta.xlsx")
        .exists());

    let instrument_path =
        std::env::temp_dir().join(format!("autoevaluacion-instruments-{}", Uuid::new_v4()));
    service
        .export_workbook(ExportWorkbookRequest {
            path: instrument_path.to_string_lossy().into_owned(),
            kind: ExportKind::Instruments,
            instrument_public: Some("Estudiantes".into()),
        })
        .await
        .unwrap();

    let estudiantes_path = instrument_path.join("instrumento-estudiantes.xlsx");
    let profesores_path = instrument_path.join("instrumento-profesores-planta.xlsx");
    assert!(estudiantes_path.exists());
    assert!(!profesores_path.exists());

    let mut workbook = open_workbook_auto(&estudiantes_path).unwrap();
    let sheet_names = workbook.sheet_names().to_vec();
    assert!(sheet_names.contains(&"Por lineamiento".to_string()));
    assert!(sheet_names.contains(&"Por orden".to_string()));
    assert!(sheet_names.contains(&"Convención".to_string()));
    let order = workbook.worksheet_range("Por orden").unwrap();
    assert!(order
        .get_value((1, 0))
        .map(|value| value
            .to_string()
            .contains("INSTRUMENTO ESTUDIANTES POR ORDEN"))
        .unwrap_or(false));
    assert_eq!(
        order.get_value((2, 0)).map(|value| value.to_string()),
        Some("# Pregunta".into())
    );
    let header_values = (0..16)
        .filter_map(|column| order.get_value((2, column)))
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    assert!(header_values
        .iter()
        .any(|value| value == "Estudiantes Pregrado"));
    assert!(!header_values
        .iter()
        .any(|value| value.starts_with("Profesores Planta")));

    let _ = std::fs::remove_file(export_path);
    let _ = std::fs::remove_dir_all(all_instruments_path);
    let _ = std::fs::remove_dir_all(instrument_path);
}

#[tokio::test]
async fn marking_original_baseline_preserves_current_question_statuses() {
    let repository = Arc::new(LibSqlAutoEvalRepository::open_in_memory().await.unwrap());
    let service = AutoEvaluationService::new(repository.clone());
    let source_document = NewSourceDocument {
        id: "source-1".into(),
        file_name: "consolidado.xlsx".into(),
        path: "consolidado.xlsx".into(),
        document_type: "questions_consolidated".into(),
        imported_rows: 3,
        skipped_rows: 0,
    };
    repository
        .save_source_document(source_document)
        .await
        .unwrap();
    repository
        .upsert_questions(vec![
            new_question_with_code_and_status("EST-001", QuestionStatus::Modify),
            new_question_with_code_and_status("EST-002", QuestionStatus::Add),
            new_question_with_code_and_status("EST-003", QuestionStatus::Delete),
        ])
        .await
        .unwrap();

    let before = repository.list_questions().await.unwrap();
    let source_document = repository
        .get_source_document("source-1")
        .await
        .unwrap()
        .unwrap();

    service
        .mark_questions_as_original_baseline(
            before,
            source_document,
            "Test Editor",
            "Fijacion de original",
        )
        .await
        .unwrap();

    let questions = repository.list_questions().await.unwrap();
    let statuses = questions
        .iter()
        .map(|question| (question.code.as_str(), &question.status))
        .collect::<Vec<_>>();

    assert_eq!(
        statuses,
        vec![
            ("EST-001", &QuestionStatus::Modify),
            ("EST-002", &QuestionStatus::Add),
            ("EST-003", &QuestionStatus::Delete),
        ]
    );
    assert_eq!(repository.list_original_snapshots().await.unwrap().len(), 3);
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
