use tauri::State;

use crate::domain::{
    AvailableInstrumentPublic, BaselineStatus, ChangeLogEntry, DashboardSummary,
    DeleteGuidelineAspectRequest, DeleteGuidelineAspectResult, ExportWorkbookRequest,
    ExportWorkbookResult, GuidelineAspect, ImportWorkbookPreviewResult, ImportWorkbookRequest,
    ImportWorkbookResult, InstrumentDefinition, InstrumentPublicOption, MarkOriginalBaselineRequest,
    NewGuidelineAspect, NewQuestion, Question, ResetDatabaseRequest, ResetDatabaseResult,
    SaveInstrumentDefinitionRequest, UpdateGuidelineAspectRequest, UpdateGuidelineAspectResult,
    UpdateQuestionRequest, ValidationIssue,
};
use crate::error::CommandError;
use crate::workspace_state::AppState;

#[tauri::command]
pub(crate) async fn get_dashboard(
    state: State<'_, AppState>,
) -> Result<DashboardSummary, CommandError> {
    let (service, workspace) = state.snapshot()?;
    service.dashboard(workspace).await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_questions(state: State<'_, AppState>) -> Result<Vec<Question>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_questions().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_instrument_public_options(
    state: State<'_, AppState>,
) -> Result<Vec<InstrumentPublicOption>, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .list_instrument_public_options()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_instrument_definitions(
    state: State<'_, AppState>,
) -> Result<Vec<InstrumentDefinition>, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .list_instrument_definitions()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_available_instrument_publics(
    state: State<'_, AppState>,
) -> Result<Vec<AvailableInstrumentPublic>, CommandError> {
    let (service, _) = state.snapshot()?;
    service
        .list_available_instrument_publics()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn save_instrument_definition(
    state: State<'_, AppState>,
    request: SaveInstrumentDefinitionRequest,
) -> Result<InstrumentDefinition, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let instrument = service.save_instrument_definition(request).await?;
    service
        .record_change(
            "instrument",
            &instrument.id,
            "save_instrument_definition",
            &editor,
            "Instrument definition updated from the desktop app",
        )
        .await?;
    Ok(instrument)
}

#[tauri::command]
pub(crate) async fn create_question(
    state: State<'_, AppState>,
    question: NewQuestion,
) -> Result<Question, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let created = service.create_question(question).await?;
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
pub(crate) async fn update_question(
    state: State<'_, AppState>,
    request: UpdateQuestionRequest,
) -> Result<Question, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    let updated = service.update_question(request, &editor).await?;
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
pub(crate) async fn list_guideline_aspects(
    state: State<'_, AppState>,
) -> Result<Vec<GuidelineAspect>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_guideline_aspects().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn create_guideline_aspect(
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
pub(crate) async fn update_guideline_aspect(
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
pub(crate) async fn delete_guideline_aspect(
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
pub(crate) async fn preview_import_workbook(
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
pub(crate) async fn import_workbook(
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
pub(crate) async fn run_validations(
    state: State<'_, AppState>,
) -> Result<Vec<ValidationIssue>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.run_validations().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn get_baseline_status(
    state: State<'_, AppState>,
) -> Result<BaselineStatus, CommandError> {
    let (service, _) = state.snapshot()?;
    service.baseline_status().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn mark_original_baseline(
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
pub(crate) async fn export_workbook(
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
pub(crate) async fn reset_database_data(
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
pub(crate) async fn list_change_logs(
    state: State<'_, AppState>,
) -> Result<Vec<ChangeLogEntry>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_change_logs().await.map_err(Into::into)
}
