import { invoke } from "@tauri-apps/api/core";

import type {
  DashboardSummary,
  BaselineStatus,
  ChangeLogEntry,
  CollaborationLock,
  CollaborationLocksForResourcesRequest,
  CollaborationPresence,
  AcquireCollaborationLockRequest,
  ReleaseCollaborationLockRequest,
  ConfigureTursoWorkspaceRequest,
  ConfigureWorkspaceRequest,
  DatabasePackageResult,
  ExportWorkbookRequest,
  ExportWorkbookResult,
  DeleteGuidelineAspectRequest,
  DeleteGuidelineAspectResult,
  DeleteHistorySnapshotRequest,
  HistorySnapshot,
  GuidelineAspect,
  ImportWorkbookRequest,
  ImportWorkbookPreviewResult,
  ImportWorkbookResult,
  AvailableInstrumentPublic,
  InstrumentDefinition,
  InstrumentPublicOption,
  MicrosoftLoginRequest,
  MicrosoftLoginResult,
  MarkOriginalBaselineRequest,
  NewProviderLink,
  NewQuestion,
  NewGuidelineAspect,
  OpenDatabasePackageRequest,
  OpenDatabaseRequest,
  ProviderLink,
  ProviderQuestionReview,
  ProviderQuestionReviewItem,
  Question,
  ResetDatabaseRequest,
  ResetDatabaseResult,
  ResetProviderQuestionReviewsRequest,
  ResetProviderQuestionReviewsResult,
  RestoreHistorySnapshotRequest,
  SaveProviderQuestionReviewRequest,
  ExportDatabasePackageRequest,
  SaveEvidenceAttachmentRequest,
  SaveEvidenceAttachmentResult,
  ExportProviderReviewDocxRequest,
  SaveEditorProfileRequest,
  SaveInstrumentDefinitionRequest,
  SyncResult,
  UpdateQuestionRequest,
  UpdateGuidelineAspectRequest,
  UpdateGuidelineAspectResult,
  ValidationIssue,
  WorkspaceStatus,
} from "@/lib/types";

export const api = {
  workspace: () => invoke<WorkspaceStatus>("get_workspace_status"),
  configureWorkspace: (request: ConfigureWorkspaceRequest) =>
    invoke<WorkspaceStatus>("configure_onedrive_workspace", { request }),
  configureTursoWorkspace: (request: ConfigureTursoWorkspaceRequest) =>
    invoke<WorkspaceStatus>("configure_turso_workspace", { request }),
  refreshTursoWorkspace: () => invoke<WorkspaceStatus>("refresh_turso_workspace"),
  openDatabase: (request: OpenDatabaseRequest) =>
    invoke<WorkspaceStatus>("open_existing_database", { request }),
  exportDatabasePackage: (request: ExportDatabasePackageRequest) =>
    invoke<DatabasePackageResult>("export_database_package", { request }),
  openDatabasePackage: (request: OpenDatabasePackageRequest) =>
    invoke<WorkspaceStatus>("open_database_package", { request }),
  loginWithMicrosoft: (request: MicrosoftLoginRequest) =>
    invoke<MicrosoftLoginResult>("login_with_microsoft", { request }),
  saveEditorProfile: (request: SaveEditorProfileRequest) =>
    invoke<WorkspaceStatus>("save_editor_profile", { request }),
  syncToGraph: () => invoke<SyncResult>("sync_database_to_microsoft_graph"),
  syncFromGraph: () => invoke<SyncResult>("sync_database_from_microsoft_graph"),
  dashboard: () => invoke<DashboardSummary>("get_dashboard"),
  questions: () => invoke<Question[]>("list_questions"),
  createQuestion: (question: NewQuestion) =>
    invoke<Question>("create_question", { question }),
  updateQuestion: (request: UpdateQuestionRequest) =>
    invoke<Question>("update_question", { request }),
  guidelineAspects: () => invoke<GuidelineAspect[]>("list_guideline_aspects"),
  createGuidelineAspect: (aspect: NewGuidelineAspect) =>
    invoke<GuidelineAspect>("create_guideline_aspect", { aspect }),
  updateGuidelineAspect: (request: UpdateGuidelineAspectRequest) =>
    invoke<UpdateGuidelineAspectResult>("update_guideline_aspect", { request }),
  deleteGuidelineAspect: (request: DeleteGuidelineAspectRequest) =>
    invoke<DeleteGuidelineAspectResult>("delete_guideline_aspect", { request }),
  previewImportWorkbook: (request: ImportWorkbookRequest) =>
    invoke<ImportWorkbookPreviewResult>("preview_import_workbook", { request }),
  importWorkbook: (request: ImportWorkbookRequest) =>
    invoke<ImportWorkbookResult>("import_workbook", { request }),
  validations: () => invoke<ValidationIssue[]>("run_validations"),
  baselineStatus: () => invoke<BaselineStatus>("get_baseline_status"),
  markOriginalBaseline: (request: MarkOriginalBaselineRequest) =>
    invoke<BaselineStatus>("mark_original_baseline", { request }),
  exportWorkbook: (request: ExportWorkbookRequest) =>
    invoke<ExportWorkbookResult>("export_workbook", { request }),
  instrumentPublicOptions: () =>
    invoke<InstrumentPublicOption[]>("list_instrument_public_options"),
  instrumentDefinitions: () =>
    invoke<InstrumentDefinition[]>("list_instrument_definitions"),
  availableInstrumentPublics: () =>
    invoke<AvailableInstrumentPublic[]>("list_available_instrument_publics"),
  saveInstrumentDefinition: (request: SaveInstrumentDefinitionRequest) =>
    invoke<InstrumentDefinition>("save_instrument_definition", { request }),
  providerLinks: () => invoke<ProviderLink[]>("list_provider_links"),
  recordProviderLink: (link: NewProviderLink) =>
    invoke<ProviderLink>("record_provider_link", { link }),
  providerQuestionReviewItems: () =>
    invoke<ProviderQuestionReviewItem[]>("list_provider_question_review_items"),
  saveProviderQuestionReview: (review: SaveProviderQuestionReviewRequest) =>
    invoke<ProviderQuestionReview>("save_provider_question_review", { review }),
  resetProviderQuestionReviews: (request: ResetProviderQuestionReviewsRequest) =>
    invoke<ResetProviderQuestionReviewsResult>("reset_provider_question_reviews", { request }),
  saveEvidenceAttachment: (request: SaveEvidenceAttachmentRequest) =>
    invoke<SaveEvidenceAttachmentResult>("save_evidence_attachment", { request }),
  exportProviderReviewDocx: (request: ExportProviderReviewDocxRequest) =>
    invoke<void>("export_provider_review_docx", { request }),
  resetDatabaseData: (request: ResetDatabaseRequest) =>
    invoke<ResetDatabaseResult>("reset_database_data", { request }),
  changeLogs: () => invoke<ChangeLogEntry[]>("list_change_logs"),
  collaborationLocks: () => invoke<CollaborationLock[]>("list_collaboration_locks"),
  collaborationLocksForResources: (request: CollaborationLocksForResourcesRequest) =>
    invoke<CollaborationLock[]>("list_collaboration_locks_for_resources", { request }),
  heartbeatCollaborationPresence: () =>
    invoke<CollaborationPresence[]>("heartbeat_collaboration_presence"),
  acquireCollaborationLock: (request: AcquireCollaborationLockRequest) =>
    invoke<CollaborationLock>("acquire_collaboration_lock", { request }),
  releaseCollaborationLock: (request: ReleaseCollaborationLockRequest) =>
    invoke<void>("release_collaboration_lock", { request }),
  historySnapshots: () => invoke<HistorySnapshot[]>("list_history_snapshots"),
  saveManualHistorySnapshot: () =>
    invoke<HistorySnapshot>("save_manual_history_snapshot"),
  deleteHistorySnapshot: (request: DeleteHistorySnapshotRequest) =>
    invoke<void>("delete_history_snapshot", { request }),
  restoreHistorySnapshot: (request: RestoreHistorySnapshotRequest) =>
    invoke<void>("restore_history_snapshot", { request }),
};
