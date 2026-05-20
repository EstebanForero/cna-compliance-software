mod auth;
mod db;
mod domain;
mod error;
mod importer;
mod repository;
mod service;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use base64::Engine;
use db::LibSqlAutoEvalRepository;
use domain::{
    BaselineStatus, ChangeLogEntry, ConfigureWorkspaceRequest, DashboardSummary,
    DatabasePackageResult, DeleteGuidelineAspectRequest, DeleteGuidelineAspectResult,
    DeleteHistorySnapshotRequest, EditorProfile, ExportDatabasePackageRequest,
    ExportProviderReviewDocxRequest, ExportWorkbookRequest, ExportWorkbookResult, GuidelineAspect,
    HistorySnapshot, ImportWorkbookPreviewResult, ImportWorkbookRequest, ImportWorkbookResult,
    MarkOriginalBaselineRequest, MicrosoftAccount, MicrosoftAuthConfig, MicrosoftLoginRequest,
    MicrosoftLoginResult, NewGuidelineAspect, NewProviderLink, NewQuestion,
    OpenDatabasePackageRequest, OpenDatabaseRequest, ProviderLink, ProviderQuestionReview,
    ProviderQuestionReviewItem, Question, ResetDatabaseRequest, ResetDatabaseResult,
    ResetProviderQuestionReviewsRequest, ResetProviderQuestionReviewsResult,
    RestoreHistorySnapshotRequest, SaveEditorProfileRequest, SaveEvidenceAttachmentRequest,
    SaveEvidenceAttachmentResult, SaveProviderQuestionReviewRequest, SyncResult,
    UpdateGuidelineAspectRequest, UpdateGuidelineAspectResult, UpdateQuestionRequest,
    ValidationIssue, WorkspaceStatus,
};
use error::{AppError, CommandError};
use service::AutoEvaluationService;
use tauri::{Manager, State};

const DATABASE_PACKAGE_EXTENSION: &str = "acna";
const DATABASE_PACKAGE_DESCRIPTION: &str = "Autoevaluacion CNA database package";

struct RuntimeWorkspace {
    service: Arc<AutoEvaluationService>,
    database_path: PathBuf,
    onedrive_path: Option<PathBuf>,
    microsoft_account: Option<MicrosoftAccount>,
    microsoft_auth_config: Option<MicrosoftAuthConfig>,
    microsoft_access_token: Option<String>,
    editor_profile: Option<EditorProfile>,
}

struct AppState {
    workspace: RwLock<RuntimeWorkspace>,
    config_file: PathBuf,
}

#[tauri::command]
async fn get_workspace_status(state: State<'_, AppState>) -> Result<WorkspaceStatus, CommandError> {
    let (service, mut status) = state.snapshot()?;
    status.has_questions = !service.list_questions().await?.is_empty();
    Ok(status)
}

#[tauri::command]
async fn configure_onedrive_workspace(
    state: State<'_, AppState>,
    request: ConfigureWorkspaceRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let folder = PathBuf::from(request.folder_path.trim());
    std::fs::create_dir_all(&folder).map_err(AppError::from)?;
    let database_path = folder.join("autoevaluacion-cna.db");
    state.reopen(database_path, Some(folder), None, None)?;
    get_workspace_status(state).await
}

#[tauri::command]
async fn open_existing_database(
    state: State<'_, AppState>,
    request: OpenDatabaseRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let database_path = PathBuf::from(request.database_path.trim());
    state.reopen(database_path, None, None, None)?;
    get_workspace_status(state).await
}

#[tauri::command]
async fn export_database_package(
    state: State<'_, AppState>,
    request: ExportDatabasePackageRequest,
) -> Result<DatabasePackageResult, CommandError> {
    let target_path = database_package_path(&request.path)?;
    let source_path = {
        let workspace = state
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.database_path.clone()
    };

    if source_path == target_path {
        return Err(
            AppError::Validation("choose a different file for the database export".into()).into(),
        );
    }
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    let database = libsql::Builder::new_local(&source_path)
        .build()
        .await
        .map_err(AppError::from)?;
    let connection = database.connect().map_err(AppError::from)?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(FULL);")
        .await
        .map_err(AppError::from)?;
    std::fs::copy(&source_path, &target_path).map_err(AppError::from)?;

    Ok(DatabasePackageResult {
        path: target_path.to_string_lossy().to_string(),
        message: format!("{DATABASE_PACKAGE_DESCRIPTION} exported with current history."),
    })
}

#[tauri::command]
async fn open_database_package(
    state: State<'_, AppState>,
    request: OpenDatabasePackageRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let database_path = existing_database_package_path(&request.path)?;
    if !database_path.exists() {
        return Err(AppError::Validation("database package file was not found".into()).into());
    }
    state.reopen(database_path, None, None, None)?;
    get_workspace_status(state).await
}

#[tauri::command]
async fn login_with_microsoft(
    state: State<'_, AppState>,
    request: MicrosoftLoginRequest,
) -> Result<MicrosoftLoginResult, CommandError> {
    let (result, access_token) = auth::login_with_microsoft(request.clone()).await?;
    {
        let mut workspace = state
            .workspace
            .write()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.microsoft_account = Some(result.account.clone());
        workspace.microsoft_auth_config = Some(MicrosoftAuthConfig {
            client_id: request.client_id,
            tenant_id: result.tenant_id.clone(),
        });
        workspace.microsoft_access_token = Some(access_token);
        state.save_config(&workspace)?;
    }
    Ok(result)
}

#[tauri::command]
async fn save_editor_profile(
    state: State<'_, AppState>,
    request: SaveEditorProfileRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let full_name = request.full_name.trim();
    if full_name.len() < 3 {
        return Err(AppError::Validation("editor full name is required".into()).into());
    }
    {
        let mut workspace = state
            .workspace
            .write()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.editor_profile = Some(EditorProfile {
            full_name: full_name.into(),
        });
        state.save_config(&workspace)?;
    }
    get_workspace_status(state).await
}

#[tauri::command]
async fn sync_database_to_microsoft_graph(
    state: State<'_, AppState>,
) -> Result<SyncResult, CommandError> {
    let (database_path, token) = {
        let workspace = state
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        (
            workspace.database_path.clone(),
            workspace.microsoft_access_token.clone().ok_or_else(|| {
                AppError::Validation("sign in with Microsoft before Graph sync".into())
            })?,
        )
    };
    auth::upload_database_to_app_folder(&token, &database_path).await?;
    Ok(SyncResult {
        method: "microsoftGraphAppFolder".into(),
        message: "Database uploaded to Microsoft Graph app folder.".into(),
        database_path: database_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn sync_database_from_microsoft_graph(
    state: State<'_, AppState>,
) -> Result<SyncResult, CommandError> {
    let (database_path, token) = {
        let workspace = state
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        (
            workspace.database_path.clone(),
            workspace.microsoft_access_token.clone().ok_or_else(|| {
                AppError::Validation("sign in with Microsoft before Graph sync".into())
            })?,
        )
    };
    auth::download_database_from_app_folder(&token, &database_path).await?;
    state.reopen(database_path.clone(), None, None, None)?;
    Ok(SyncResult {
        method: "microsoftGraphAppFolder".into(),
        message: "Database downloaded from Microsoft Graph app folder.".into(),
        database_path: database_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardSummary, CommandError> {
    let (service, workspace) = state.snapshot()?;
    service.dashboard(workspace).await.map_err(Into::into)
}

#[tauri::command]
async fn list_questions(state: State<'_, AppState>) -> Result<Vec<Question>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_questions().await.map_err(Into::into)
}

#[tauri::command]
async fn create_question(
    state: State<'_, AppState>,
    question: NewQuestion,
) -> Result<Question, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let created = service
        .create_question(question)
        .await
        .map_err(CommandError::from)?;
    service
        .record_change(
            "question",
            &created.id,
            "create_question",
            &editor,
            "Question created from the desktop app",
        )
        .await?;
    Ok(created)
}

#[tauri::command]
async fn preview_import_workbook(
    state: State<'_, AppState>,
    request: ImportWorkbookRequest,
) -> Result<ImportWorkbookPreviewResult, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .preview_import_workbook(request)
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn update_question(
    state: State<'_, AppState>,
    request: UpdateQuestionRequest,
) -> Result<Question, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let updated = service
        .update_question(request, &editor)
        .await
        .map_err(CommandError::from)?;
    service
        .record_change(
            "question",
            &updated.id,
            "update_question",
            &editor,
            "Question updated from the desktop app",
        )
        .await?;
    Ok(updated)
}

#[tauri::command]
async fn list_guideline_aspects(
    state: State<'_, AppState>,
) -> Result<Vec<GuidelineAspect>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_guideline_aspects().await.map_err(Into::into)
}

#[tauri::command]
async fn create_guideline_aspect(
    state: State<'_, AppState>,
    aspect: NewGuidelineAspect,
) -> Result<GuidelineAspect, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let created = service.create_guideline_aspect(aspect).await?;
    service
        .record_change(
            "guideline_aspect",
            &created.id,
            "create_guideline_aspect",
            &editor,
            "CNA guideline aspect created from the desktop app",
        )
        .await?;
    Ok(created)
}

#[tauri::command]
async fn update_guideline_aspect(
    state: State<'_, AppState>,
    request: UpdateGuidelineAspectRequest,
) -> Result<UpdateGuidelineAspectResult, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let result = service.update_guideline_aspect(request, &editor).await?;
    service
        .record_change(
            "guideline_aspect",
            &result.aspect.id,
            "update_guideline_aspect",
            &editor,
            &format!(
                "Updated guideline aspect and marked {} related questions as modified",
                result.affected_questions
            ),
        )
        .await?;
    Ok(result)
}

#[tauri::command]
async fn delete_guideline_aspect(
    state: State<'_, AppState>,
    request: DeleteGuidelineAspectRequest,
) -> Result<DeleteGuidelineAspectResult, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let result = service.delete_guideline_aspect(request, &editor).await?;
    service
        .record_change(
            "guideline_aspect",
            "deleted",
            "delete_guideline_aspect",
            &editor,
            &format!(
                "Deleted guideline aspect and {} related questions",
                result.affected_questions
            ),
        )
        .await?;
    Ok(result)
}

#[tauri::command]
async fn import_workbook(
    state: State<'_, AppState>,
    request: ImportWorkbookRequest,
) -> Result<ImportWorkbookResult, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let result = service.import_workbook(request, &editor).await?;
    service
        .record_change(
            "source_document",
            &result.source_document_id,
            "import_workbook",
            &editor,
            &format!(
                "Imported {} questions from {}",
                result.imported_questions, result.file_name
            ),
        )
        .await?;
    Ok(result)
}

#[tauri::command]
async fn run_validations(state: State<'_, AppState>) -> Result<Vec<ValidationIssue>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.run_validations().await.map_err(Into::into)
}

#[tauri::command]
async fn get_baseline_status(state: State<'_, AppState>) -> Result<BaselineStatus, CommandError> {
    let (service, _) = state.snapshot()?;
    service.baseline_status().await.map_err(Into::into)
}

#[tauri::command]
async fn mark_original_baseline(
    state: State<'_, AppState>,
    request: MarkOriginalBaselineRequest,
) -> Result<BaselineStatus, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let status = service.mark_original_baseline(request, &editor).await?;
    service
        .record_change(
            "question_original_snapshot",
            "baseline",
            "mark_original_baseline",
            &editor,
            "Original question baseline marked with reinforced confirmation",
        )
        .await?;
    Ok(status)
}

#[tauri::command]
async fn export_workbook(
    state: State<'_, AppState>,
    request: ExportWorkbookRequest,
) -> Result<ExportWorkbookResult, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let result = service.export_workbook(request).await?;
    service
        .record_change(
            "instrument_export",
            &result.path,
            "export_workbook",
            &editor,
            &format!(
                "Exported {} questions with {} added, {} modified and {} removed",
                result.exported_questions,
                result.added_questions,
                result.modified_questions,
                result.removed_questions
            ),
        )
        .await?;
    Ok(result)
}

#[tauri::command]
async fn reset_database_data(
    state: State<'_, AppState>,
    request: ResetDatabaseRequest,
) -> Result<ResetDatabaseResult, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .reset_database_data(request)
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn list_change_logs(state: State<'_, AppState>) -> Result<Vec<ChangeLogEntry>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_change_logs().await.map_err(Into::into)
}

#[tauri::command]
async fn list_history_snapshots(
    state: State<'_, AppState>,
) -> Result<Vec<HistorySnapshot>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_history_snapshots().await.map_err(Into::into)
}

#[tauri::command]
async fn save_manual_history_snapshot(
    state: State<'_, AppState>,
) -> Result<HistorySnapshot, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let snapshot = service.save_manual_history_snapshot(&editor).await?;
    service
        .record_change(
            "history_snapshot",
            &snapshot.id,
            "save_manual_history_snapshot",
            &editor,
            "Manual recoverable database snapshot saved",
        )
        .await?;
    Ok(snapshot)
}

#[tauri::command]
async fn delete_history_snapshot(
    state: State<'_, AppState>,
    request: DeleteHistorySnapshotRequest,
) -> Result<(), CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let snapshot_id = request.snapshot_id.clone();
    service.delete_history_snapshot(request).await?;
    service
        .record_change(
            "history_snapshot",
            &snapshot_id,
            "delete_history_snapshot",
            &editor,
            "Manual history snapshot deleted",
        )
        .await?;
    Ok(())
}

#[tauri::command]
async fn restore_history_snapshot(
    state: State<'_, AppState>,
    request: RestoreHistorySnapshotRequest,
) -> Result<(), CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    service.restore_history_snapshot(request, &editor).await?;
    service
        .record_change(
            "history_snapshot",
            "restore",
            "restore_history_snapshot",
            &editor,
            "Restored database state from a persistent history snapshot",
        )
        .await?;
    Ok(())
}

#[tauri::command]
async fn list_provider_links(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderLink>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_provider_links().await.map_err(Into::into)
}

#[tauri::command]
async fn record_provider_link(
    state: State<'_, AppState>,
    link: NewProviderLink,
) -> Result<ProviderLink, CommandError> {
    let (service, _) = state.snapshot()?;
    service.record_provider_link(link).await.map_err(Into::into)
}

#[tauri::command]
async fn list_provider_question_review_items(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderQuestionReviewItem>, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .list_provider_question_review_items()
        .await
        .map_err(Into::into)
}

#[tauri::command]
async fn save_provider_question_review(
    state: State<'_, AppState>,
    review: SaveProviderQuestionReviewRequest,
) -> Result<ProviderQuestionReview, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let saved = service.save_provider_question_review(review).await?;
    service
        .record_change(
            "provider_question_review",
            &format!("{}::{}", saved.instrument_audience, saved.question_id),
            "save_provider_question_review",
            &editor,
            "Provider question review updated",
        )
        .await?;
    Ok(saved)
}

#[tauri::command]
async fn reset_provider_question_reviews(
    state: State<'_, AppState>,
    request: ResetProviderQuestionReviewsRequest,
) -> Result<ResetProviderQuestionReviewsResult, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let result = service.reset_provider_question_reviews(request).await?;
    service
        .record_change(
            "provider_question_review",
            "all",
            "reset_provider_question_reviews",
            &editor,
            "Provider question review checklist reset",
        )
        .await?;
    Ok(result)
}

#[tauri::command]
async fn save_evidence_attachment(
    state: State<'_, AppState>,
    request: SaveEvidenceAttachmentRequest,
) -> Result<SaveEvidenceAttachmentResult, CommandError> {
    let database_path = {
        let workspace = state
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.database_path.clone()
    };
    let folder = database_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("evidencias-proveedor");
    std::fs::create_dir_all(&folder).map_err(AppError::from)?;

    let (mime, encoded) = request
        .data_url
        .split_once(',')
        .ok_or_else(|| AppError::Validation("invalid pasted image data".into()))?;
    if !mime.starts_with("data:image/") || !mime.contains(";base64") {
        return Err(AppError::Validation("only pasted image evidence is supported".into()).into());
    }
    let extension = mime
        .strip_prefix("data:image/")
        .and_then(|value| value.split(';').next())
        .map(normalize_image_extension)
        .unwrap_or("png");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| AppError::Validation(format!("invalid pasted image: {error}")))?;
    let name = request
        .file_name
        .as_deref()
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "evidencia".into());
    let file_name = format!(
        "{}-{}-{}.{}",
        sanitize_file_stem(&request.question_id),
        chrono::Utc::now().timestamp_millis(),
        name,
        extension
    );
    let path = folder.join(file_name);
    std::fs::write(&path, bytes).map_err(AppError::from)?;

    Ok(SaveEvidenceAttachmentResult {
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn export_provider_review_docx(
    state: State<'_, AppState>,
    request: ExportProviderReviewDocxRequest,
) -> Result<(), CommandError> {
    let (service, _) = state.snapshot()?;
    service.export_provider_review_docx(request).await?;
    Ok(())
}

impl AppState {
    fn snapshot(&self) -> Result<(Arc<AutoEvaluationService>, WorkspaceStatus), AppError> {
        let workspace = self
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        let status = WorkspaceStatus {
            database_path: workspace.database_path.to_string_lossy().to_string(),
            configured_onedrive_path: workspace
                .onedrive_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            microsoft_account: workspace.microsoft_account.clone(),
            microsoft_auth_config: workspace.microsoft_auth_config.clone(),
            editor_profile: workspace.editor_profile.clone(),
            graph_sync_available: workspace.microsoft_access_token.is_some(),
            has_questions: false,
        };
        Ok((Arc::clone(&workspace.service), status))
    }

    fn reopen(
        &self,
        database_path: PathBuf,
        onedrive_path: Option<PathBuf>,
        microsoft_account: Option<MicrosoftAccount>,
        microsoft_auth_config: Option<MicrosoftAuthConfig>,
    ) -> Result<(), AppError> {
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let repository =
            tauri::async_runtime::block_on(LibSqlAutoEvalRepository::open(&database_path))?;
        let service = AutoEvaluationService::new(Arc::new(repository));
        let mut workspace = self
            .workspace
            .write()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.service = Arc::new(service);
        workspace.database_path = database_path;
        workspace.onedrive_path = onedrive_path;
        if microsoft_account.is_some() {
            workspace.microsoft_account = microsoft_account;
        }
        if microsoft_auth_config.is_some() {
            workspace.microsoft_auth_config = microsoft_auth_config;
        }
        self.save_config(&workspace)
    }

    fn save_config(&self, workspace: &RuntimeWorkspace) -> Result<(), AppError> {
        let content = serde_json::json!({
            "databasePath": workspace.database_path,
            "onedrivePath": workspace.onedrive_path,
            "microsoftAccount": workspace.microsoft_account,
            "microsoftAuthConfig": workspace.microsoft_auth_config,
            "microsoftAccessToken": workspace.microsoft_access_token,
            "editorProfile": workspace.editor_profile,
        });
        if let Some(parent) = self.config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.config_file, serde_json::to_vec_pretty(&content)?)?;
        Ok(())
    }

    fn current_editor_name(&self) -> Result<String, AppError> {
        self.workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?
            .editor_profile
            .as_ref()
            .map(|profile| profile.full_name.clone())
            .ok_or_else(|| {
                AppError::Validation("save the editor full name before making changes".into())
            })
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            let config_file = config_dir.join("workspace.json");
            let initial = load_initial_workspace(&config_file, &data_dir)?;

            app.manage(AppState {
                workspace: RwLock::new(initial),
                config_file,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_workspace_status,
            configure_onedrive_workspace,
            open_existing_database,
            export_database_package,
            open_database_package,
            login_with_microsoft,
            save_editor_profile,
            sync_database_to_microsoft_graph,
            sync_database_from_microsoft_graph,
            get_dashboard,
            list_questions,
            create_question,
            update_question,
            list_guideline_aspects,
            create_guideline_aspect,
            update_guideline_aspect,
            delete_guideline_aspect,
            preview_import_workbook,
            import_workbook,
            run_validations,
            get_baseline_status,
            mark_original_baseline,
            export_workbook,
            reset_database_data,
            list_change_logs,
            list_history_snapshots,
            save_manual_history_snapshot,
            delete_history_snapshot,
            restore_history_snapshot,
            list_provider_links,
            record_provider_link,
            list_provider_question_review_items,
            save_provider_question_review,
            reset_provider_question_reviews,
            save_evidence_attachment,
            export_provider_review_docx
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_initial_workspace(
    config_file: &Path,
    data_dir: &Path,
) -> Result<RuntimeWorkspace, AppError> {
    let saved = read_workspace_config(config_file);
    let launch_database_path = database_package_from_process_args();
    let database_path = launch_database_path
        .or_else(|| {
            saved
                .as_ref()
                .and_then(|value| value.get("databasePath"))
                .and_then(|value| value.as_str())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| data_dir.join("autoevaluacion-cna.db"));
    let onedrive_path = saved
        .as_ref()
        .and_then(|value| value.get("onedrivePath"))
        .and_then(|value| value.as_str())
        .map(PathBuf::from);
    let microsoft_account = saved
        .as_ref()
        .and_then(|value| value.get("microsoftAccount").cloned())
        .and_then(|value| serde_json::from_value(value).ok());
    let microsoft_auth_config = saved
        .as_ref()
        .and_then(|value| value.get("microsoftAuthConfig").cloned())
        .and_then(|value| serde_json::from_value(value).ok());
    let microsoft_access_token = saved
        .as_ref()
        .and_then(|value| value.get("microsoftAccessToken"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let editor_profile = saved
        .as_ref()
        .and_then(|value| value.get("editorProfile").cloned())
        .and_then(|value| serde_json::from_value(value).ok());

    if let Some(parent) = database_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let repository =
        tauri::async_runtime::block_on(LibSqlAutoEvalRepository::open(&database_path))?;
    let service = AutoEvaluationService::new(Arc::new(repository));

    Ok(RuntimeWorkspace {
        service: Arc::new(service),
        database_path,
        onedrive_path,
        microsoft_account,
        microsoft_auth_config,
        microsoft_access_token,
        editor_profile,
    })
}

fn read_workspace_config(path: &Path) -> Option<serde_json::Value> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

fn database_package_path(path: &str) -> Result<PathBuf, AppError> {
    let mut path = PathBuf::from(path.trim());
    if path.extension().is_none() {
        path.set_extension(DATABASE_PACKAGE_EXTENSION);
    }
    if is_database_package_path(&path) {
        return Ok(path);
    }
    Err(AppError::Validation(format!(
        "database packages must use the .{DATABASE_PACKAGE_EXTENSION} extension"
    )))
}

fn existing_database_package_path(path: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(path.trim());
    if !is_database_package_path(&path) {
        return Err(AppError::Validation(format!(
            "database packages must use the .{DATABASE_PACKAGE_EXTENSION} extension"
        )));
    }
    Ok(path)
}

fn database_package_from_process_args() -> Option<PathBuf> {
    std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .find(|path| is_database_package_path(path))
}

fn is_database_package_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case(DATABASE_PACKAGE_EXTENSION))
        .unwrap_or(false)
}

fn normalize_image_extension(value: &str) -> &'static str {
    match value {
        "jpeg" | "jpg" => "jpg",
        "gif" => "gif",
        "webp" => "webp",
        _ => "png",
    }
}

fn sanitize_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
