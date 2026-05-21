use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::db::LibSqlAutoEvalRepository;
use crate::domain::{
    EditorProfile, MicrosoftAccount, MicrosoftAuthConfig, WorkspaceStatus,
};
use crate::error::AppError;
use crate::service::AutoEvaluationService;

pub(crate) struct RuntimeWorkspace {
    pub(crate) service: Arc<AutoEvaluationService>,
    pub(crate) database_path: PathBuf,
    pub(crate) onedrive_path: Option<PathBuf>,
    pub(crate) turso_database_url: Option<String>,
    pub(crate) turso_auth_token: Option<String>,
    pub(crate) microsoft_account: Option<MicrosoftAccount>,
    pub(crate) microsoft_auth_config: Option<MicrosoftAuthConfig>,
    pub(crate) microsoft_access_token: Option<String>,
    pub(crate) editor_profile: Option<EditorProfile>,
}

pub(crate) struct AppState {
    pub(crate) workspace: RwLock<RuntimeWorkspace>,
    pub(crate) config_file: PathBuf,
}

impl AppState {
    pub(crate) fn snapshot(
        &self,
    ) -> Result<(Arc<AutoEvaluationService>, WorkspaceStatus), AppError> {
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
            turso_database_url: workspace.turso_database_url.clone(),
            microsoft_account: workspace.microsoft_account.clone(),
            microsoft_auth_config: workspace.microsoft_auth_config.clone(),
            editor_profile: workspace.editor_profile.clone(),
            graph_sync_available: workspace.microsoft_access_token.is_some(),
            turso_connected: workspace.turso_database_url.is_some(),
            has_questions: false,
        };
        Ok((Arc::clone(&workspace.service), status))
    }

    pub(crate) fn reopen(
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
        workspace.turso_database_url = None;
        workspace.turso_auth_token = None;
        if microsoft_account.is_some() {
            workspace.microsoft_account = microsoft_account;
        }
        if microsoft_auth_config.is_some() {
            workspace.microsoft_auth_config = microsoft_auth_config;
        }
        self.save_config(&workspace)
    }

    pub(crate) fn reopen_turso(
        &self,
        database_url: String,
        auth_token: String,
    ) -> Result<(), AppError> {
        let repository = tauri::async_runtime::block_on(LibSqlAutoEvalRepository::open_remote(
            &database_url,
            &auth_token,
        ))?;
        let service = AutoEvaluationService::new(Arc::new(repository));
        let mut workspace = self
            .workspace
            .write()
            .map_err(|_| AppError::Validation("workspace lock is poisoned".into()))?;
        workspace.service = Arc::new(service);
        workspace.turso_database_url = Some(database_url);
        workspace.turso_auth_token = Some(auth_token);
        workspace.onedrive_path = None;
        self.save_config(&workspace)
    }

    pub(crate) fn save_config(&self, workspace: &RuntimeWorkspace) -> Result<(), AppError> {
        let content = serde_json::json!({
            "databasePath": workspace.database_path,
            "onedrivePath": workspace.onedrive_path,
            "tursoDatabaseUrl": workspace.turso_database_url,
            "tursoAuthToken": workspace.turso_auth_token,
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

    pub(crate) fn current_editor_name(&self) -> Result<String, AppError> {
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

pub(crate) fn load_initial_workspace(
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
    let turso_database_url = saved_turso_value(&saved, "tursoDatabaseUrl")
        .or_else(|| default_env_value("AUTOCNA_TURSO_DATABASE_URL", "TURSO_DATABASE_URL"));
    let turso_auth_token = saved_turso_value(&saved, "tursoAuthToken")
        .or_else(|| default_env_value("AUTOCNA_TURSO_AUTH_TOKEN", "TURSO_AUTH_TOKEN"));
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

    let repository = if let (Some(database_url), Some(auth_token)) =
        (turso_database_url.as_deref(), turso_auth_token.as_deref())
    {
        tauri::async_runtime::block_on(LibSqlAutoEvalRepository::open_remote(
            database_url,
            auth_token,
        ))?
    } else {
        tauri::async_runtime::block_on(LibSqlAutoEvalRepository::open(&database_path))?
    };
    let service = AutoEvaluationService::new(Arc::new(repository));

    Ok(RuntimeWorkspace {
        service: Arc::new(service),
        database_path,
        onedrive_path,
        turso_database_url,
        turso_auth_token,
        microsoft_account,
        microsoft_auth_config,
        microsoft_access_token,
        editor_profile,
    })
}

pub(crate) fn default_turso_credentials() -> (Option<String>, Option<String>) {
    (
        default_env_value("AUTOCNA_TURSO_DATABASE_URL", "TURSO_DATABASE_URL"),
        default_env_value("AUTOCNA_TURSO_AUTH_TOKEN", "TURSO_AUTH_TOKEN"),
    )
}

fn saved_turso_value(saved: &Option<serde_json::Value>, key: &str) -> Option<String> {
    saved
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

fn default_env_value(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(fallback).ok())
        .or_else(|| embedded_env(primary))
        .or_else(|| embedded_env(fallback))
}

fn embedded_env(name: &str) -> Option<String> {
    match name {
        "AUTOCNA_TURSO_DATABASE_URL" => option_env!("AUTOCNA_TURSO_DATABASE_URL"),
        "TURSO_DATABASE_URL" => option_env!("TURSO_DATABASE_URL"),
        "AUTOCNA_TURSO_AUTH_TOKEN" => option_env!("AUTOCNA_TURSO_AUTH_TOKEN"),
        "TURSO_AUTH_TOKEN" => option_env!("TURSO_AUTH_TOKEN"),
        _ => None,
    }
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToOwned::to_owned)
}

fn read_workspace_config(path: &Path) -> Option<serde_json::Value> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
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
        .map(|extension| extension.eq_ignore_ascii_case("acna"))
        .unwrap_or(false)
}
