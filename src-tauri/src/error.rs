use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] libsql::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("spreadsheet error: {0}")]
    Spreadsheet(#[from] calamine::Error),
    #[error("excel writer error: {0}")]
    ExcelWriter(#[from] rust_xlsxwriter::XlsxError),
    #[error("word writer error: {0}")]
    WordWriter(#[from] docx_rs::DocxError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tauri path error: {0}")]
    TauriPath(#[from] tauri::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("browser open error: {0}")]
    BrowserOpen(String),
    #[error("invalid date in persisted data: {0}")]
    InvalidDate(String),
    #[error("validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
