mod audience;
mod auth;
mod commands;
mod db;
mod domain;
mod error;
mod file_utils;
mod importer;
mod repository;
mod service;
mod workspace_state;

use std::sync::RwLock;

use tauri::Manager;
use workspace_state::{load_initial_workspace, AppState};

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
            commands::workspace::get_workspace_status,
            commands::workspace::configure_onedrive_workspace,
            commands::workspace::configure_turso_workspace,
            commands::workspace::refresh_turso_workspace,
            commands::workspace::open_existing_database,
            commands::workspace::export_database_package,
            commands::workspace::open_database_package,
            commands::workspace::login_with_microsoft,
            commands::workspace::save_editor_profile,
            commands::workspace::sync_database_to_microsoft_graph,
            commands::workspace::sync_database_from_microsoft_graph,
            commands::bank::get_dashboard,
            commands::bank::list_questions,
            commands::bank::list_instrument_public_options,
            commands::bank::list_instrument_definitions,
            commands::bank::list_available_instrument_publics,
            commands::bank::save_instrument_definition,
            commands::bank::create_question,
            commands::bank::update_question,
            commands::bank::list_guideline_aspects,
            commands::bank::create_guideline_aspect,
            commands::bank::update_guideline_aspect,
            commands::bank::delete_guideline_aspect,
            commands::bank::preview_import_workbook,
            commands::bank::import_workbook,
            commands::bank::run_validations,
            commands::bank::get_baseline_status,
            commands::bank::mark_original_baseline,
            commands::bank::export_workbook,
            commands::bank::reset_database_data,
            commands::bank::list_change_logs,
            commands::collaboration::list_collaboration_locks,
            commands::collaboration::list_collaboration_locks_for_resources,
            commands::collaboration::heartbeat_collaboration_presence,
            commands::collaboration::acquire_collaboration_lock,
            commands::collaboration::release_collaboration_lock,
            commands::history::list_history_snapshots,
            commands::history::save_manual_history_snapshot,
            commands::history::delete_history_snapshot,
            commands::history::restore_history_snapshot,
            commands::provider::list_provider_links,
            commands::provider::record_provider_link,
            commands::provider::list_provider_question_review_items,
            commands::provider::save_provider_question_review,
            commands::provider::reset_provider_question_reviews,
            commands::provider::save_evidence_attachment,
            commands::provider::export_provider_review_docx
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
