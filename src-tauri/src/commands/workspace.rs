use std::path::{Path, PathBuf};

use tauri::State;

use crate::auth;
use crate::domain::{
    ConfigureTursoWorkspaceRequest, ConfigureWorkspaceRequest, DatabasePackageResult,
    EditorProfile, ExportDatabasePackageRequest, MicrosoftAuthConfig, MicrosoftLoginRequest,
    MicrosoftLoginResult, OpenDatabasePackageRequest, OpenDatabaseRequest,
    SaveEditorProfileRequest, SyncResult, WorkspaceStatus,
};
use crate::error::{AppError, CommandError};
use crate::workspace_state::{default_turso_credentials, AppState};

const DATABASE_PACKAGE_EXTENSION: &str = "acna";
const DATABASE_PACKAGE_DESCRIPTION: &str = "Autoevaluacion CNA database package";

#[tauri::command]
pub(crate) async fn get_workspace_status(
    state: State<'_, AppState>,
) -> Result<WorkspaceStatus, CommandError> {
    let (service, mut status) = state.snapshot()?;
    status.has_questions = !service.list_questions().await?.is_empty();
    Ok(status)
}

#[tauri::command]
pub(crate) async fn configure_onedrive_workspace(
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
pub(crate) async fn open_existing_database(
    state: State<'_, AppState>,
    request: OpenDatabaseRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let database_path = PathBuf::from(request.database_path.trim());
    state.reopen(database_path, None, None, None)?;
    get_workspace_status(state).await
}

#[tauri::command]
pub(crate) async fn configure_turso_workspace(
    state: State<'_, AppState>,
    request: ConfigureTursoWorkspaceRequest,
) -> Result<WorkspaceStatus, CommandError> {
    let database_url = request.database_url.trim();
    let auth_token = request.auth_token.trim();
    if !database_url.starts_with("libsql://") && !database_url.starts_with("https://") {
        return Err(
            AppError::Validation("Turso URL must start with libsql:// or https://".into()).into(),
        );
    }
    if auth_token.len() < 20 {
        return Err(AppError::Validation("Turso auth token is required".into()).into());
    }
    state.reopen_turso(database_url.to_string(), auth_token.to_string())?;
    get_workspace_status(state).await
}

#[tauri::command]
pub(crate) async fn refresh_turso_workspace(
    state: State<'_, AppState>,
) -> Result<WorkspaceStatus, CommandError> {
    let (database_url, auth_token) = {
        let workspace = state
            .workspace
            .read()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        let defaults = default_turso_credentials();
        (
            workspace.turso_database_url.clone().or(defaults.0),
            workspace.turso_auth_token.clone().or(defaults.1),
        )
    };

    match (database_url, auth_token) {
        (Some(database_url), Some(auth_token)) => {
            state.reopen_turso(database_url, auth_token)?;
            get_workspace_status(state).await
        }
        _ => get_workspace_status(state).await,
    }
}

#[tauri::command]
pub(crate) async fn export_database_package(
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
pub(crate) async fn open_database_package(
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
pub(crate) async fn login_with_microsoft(
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
pub(crate) async fn save_editor_profile(
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
pub(crate) async fn sync_database_to_microsoft_graph(
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
pub(crate) async fn sync_database_from_microsoft_graph(
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

fn is_database_package_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case(DATABASE_PACKAGE_EXTENSION))
        .unwrap_or(false)
}
