use tauri::State;

use crate::domain::{
    DeleteHistorySnapshotRequest, HistorySnapshot, RestoreHistorySnapshotRequest,
};
use crate::error::CommandError;
use crate::workspace_state::AppState;

#[tauri::command]
pub(crate) async fn list_history_snapshots(
    state: State<'_, AppState>,
) -> Result<Vec<HistorySnapshot>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_history_snapshots().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn save_manual_history_snapshot(
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
pub(crate) async fn delete_history_snapshot(
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
pub(crate) async fn restore_history_snapshot(
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
