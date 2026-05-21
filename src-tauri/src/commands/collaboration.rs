use tauri::State;

use crate::domain::{
    AcquireCollaborationLockRequest, CollaborationLock, CollaborationLocksForResourcesRequest,
    CollaborationPresence, ReleaseCollaborationLockRequest,
};
use crate::error::CommandError;
use crate::workspace_state::AppState;

#[tauri::command]
pub(crate) async fn list_collaboration_locks(
    state: State<'_, AppState>,
) -> Result<Vec<CollaborationLock>, CommandError> {
    let (service, _) = state.snapshot()?;
    service.list_collaboration_locks().await.map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn list_collaboration_locks_for_resources(
    state: State<'_, AppState>,
    request: CollaborationLocksForResourcesRequest,
) -> Result<Vec<CollaborationLock>, CommandError> {
    let (service, status) = state.snapshot()?;
    if !status.turso_connected {
        return Ok(vec![]);
    }
    service
        .list_collaboration_locks_for_resources(request.resource_type, &request.resource_ids)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn heartbeat_collaboration_presence(
    state: State<'_, AppState>,
) -> Result<Vec<CollaborationPresence>, CommandError> {
    let (service, status) = state.snapshot()?;
    if !status.turso_connected {
        return Ok(vec![]);
    }
    let editor = state.current_editor_name()?;
    service.heartbeat_collaboration_presence(&editor).await?;
    service
        .list_collaboration_presence()
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn acquire_collaboration_lock(
    state: State<'_, AppState>,
    request: AcquireCollaborationLockRequest,
) -> Result<CollaborationLock, CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    service
        .acquire_collaboration_lock(request.resource_type, &request.resource_id, &editor)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub(crate) async fn release_collaboration_lock(
    state: State<'_, AppState>,
    request: ReleaseCollaborationLockRequest,
) -> Result<(), CommandError> {
    let (service, _) = state.snapshot()?;
    let editor = state.current_editor_name()?;
    service
        .release_collaboration_lock(request.resource_type, &request.resource_id, &editor)
        .await?;
    Ok(())
}
