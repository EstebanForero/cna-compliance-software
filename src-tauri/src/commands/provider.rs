use std::path::Path;

use base64::Engine;
use tauri::State;

use crate::domain::{
    ExportProviderReviewDocxRequest, NewProviderLink, ProviderLink, ProviderQuestionReview,
    ProviderQuestionReviewItem, ResetProviderQuestionReviewsRequest,
    ResetProviderQuestionReviewsResult, SaveEvidenceAttachmentRequest,
    SaveEvidenceAttachmentResult, SaveProviderQuestionReviewRequest,
};
use crate::error::{AppError, CommandError};
use crate::file_utils::{normalize_image_extension, sanitize_file_stem};
use crate::workspace_state::AppState;

#[tauri::command]
pub(crate) async fn list_provider_links(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderLink>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_provider_links().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn record_provider_link(
    state: State<'_, AppState>,
    link: NewProviderLink,
) -> Result<ProviderLink, CommandError> {
    let (service, _) = state.snapshot()?;
    service.record_provider_link(link).await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_provider_question_review_items(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderQuestionReviewItem>, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .list_provider_question_review_items()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn save_provider_question_review(
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
pub(crate) async fn reset_provider_question_reviews(
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
pub(crate) async fn save_evidence_attachment(
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
pub(crate) async fn export_provider_review_docx(
    state: State<'_, AppState>,
    request: ExportProviderReviewDocxRequest,
) -> Result<(), CommandError> {
    let (service, _) = state.snapshot()?;
    service.export_provider_review_docx(request).await?;
    Ok(())
}
