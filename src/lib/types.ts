export type CycleStatus = "planning" | "inReview" | "inApplication" | "closed";
export type QuestionScope = "institutional" | "program";
export type QuestionStatus = "keep" | "modify" | "add" | "delete";
export type QuestionFormat = "likert" | "open" | "singleChoice" | "multipleChoice";
export type ValidationSeverity = "blocking" | "warning";
export type CnaFactorCode = string;

export interface SurveyCycle {
  id: string;
  name: string;
  startsOn: string;
  applicationStartsOn: string;
  status: CycleStatus;
  notes: string;
}

export interface MicrosoftAccount {
  email: string;
  displayName: string;
}

export interface MicrosoftAuthConfig {
  clientId: string;
  tenantId: string;
}

export interface MicrosoftLoginRequest {
  clientId: string;
  tenantId: string;
}

export interface MicrosoftLoginResult {
  account: MicrosoftAccount;
  tenantId: string;
  scopes: string[];
}

export interface WorkspaceStatus {
  databasePath: string;
  configuredOnedrivePath?: string | null;
  microsoftAccount?: MicrosoftAccount | null;
  microsoftAuthConfig?: MicrosoftAuthConfig | null;
  editorProfile?: EditorProfile | null;
  graphSyncAvailable: boolean;
  hasQuestions: boolean;
}

export interface EditorProfile {
  fullName: string;
}

export interface SaveEditorProfileRequest {
  fullName: string;
}

export interface Question {
  id: string;
  code: string;
  text: string;
  scope: QuestionScope;
  format: QuestionFormat;
  conventionCode?: string | null;
  status: QuestionStatus;
  factor: string;
  characteristic: string;
  aspect: string;
  audiences: string[];
  justification?: string | null;
  updatedAt: string;
}

export interface NewQuestion {
  code: string;
  text: string;
  scope: QuestionScope;
  format: QuestionFormat;
  conventionCode?: string | null;
  status: QuestionStatus;
  factor: string;
  characteristic: string;
  aspect: string;
  audiences: string[];
  justification?: string | null;
}

export interface UpdateQuestionRequest {
  questionId: string;
  question: NewQuestion;
}

export interface GuidelineAspect {
  id: string;
  guidelineTitle: string;
  scope: QuestionScope;
  factorCode: CnaFactorCode;
  factorName: string;
  characteristicCode: string;
  characteristicName: string;
  aspectCode: string;
  aspectDescription: string;
  requiresAppreciation: boolean;
}

export interface DeleteGuidelineAspectRequest {
  aspectId: string;
  confirmationText: string;
  acknowledgeRelatedQuestions: boolean;
}

export interface DeleteGuidelineAspectResult {
  deletedAspect: boolean;
  affectedQuestions: number;
}

export interface UpdateGuidelineAspectRequest {
  aspectId: string;
  aspect: NewGuidelineAspect;
}

export interface UpdateGuidelineAspectResult {
  aspect: GuidelineAspect;
  affectedQuestions: number;
}

export interface ChangeLogEntry {
  id: string;
  entity: string;
  entityId: string;
  action: string;
  editorName: string;
  summary: string;
  createdAt: string;
}

export interface HistorySnapshot {
  id: string;
  summary: string;
  editorName: string;
  snapshotKind: string;
  createdAt: string;
}

export interface RestoreHistorySnapshotRequest {
  snapshotId: string;
  confirmationText: string;
}

export interface DeleteHistorySnapshotRequest {
  snapshotId: string;
  confirmationText: string;
}

export interface NewGuidelineAspect {
  guidelineTitle: string;
  scope: QuestionScope;
  factorCode: CnaFactorCode;
  factorName: string;
  characteristicCode: string;
  characteristicName: string;
  aspectCode: string;
  aspectDescription: string;
  requiresAppreciation: boolean;
}

export interface StatusCount {
  status: QuestionStatus;
  count: number;
}

export interface DashboardSummary {
  activeCycle?: SurveyCycle | null;
  workspace: WorkspaceStatus;
  totalQuestions: number;
  pendingChanges: number;
  blockingValidations: number;
  providerLinksPending: number;
  questionsByStatus: StatusCount[];
}

export interface ImportWorkbookRequest {
  path: string;
  cycleName?: string | null;
}

export interface ImportWorkbookResult {
  sourceDocumentId: string;
  fileName: string;
  sheetName: string;
  importedQuestions: number;
  importedGuidelineAspects: number;
  skippedRows: number;
  detectedColumns: string[];
}

export interface ImportWorkbookPreviewResult {
  fileName: string;
  sheetName: string;
  detectedQuestions: number;
  detectedGuidelineAspects: number;
  skippedRows: number;
  detectedColumns: string[];
  detectedAudiences: string[];
  warnings: string[];
}

export interface SourceDocument {
  id: string;
  fileName: string;
  path: string;
  documentType: string;
  importedAt: string;
  importedRows: number;
  skippedRows: number;
}

export interface BaselineStatus {
  hasOriginal: boolean;
  sourceDocument?: SourceDocument | null;
  originalQuestions: number;
  currentQuestions: number;
  unchangedQuestions: number;
  modifiedQuestions: number;
  addedQuestions: number;
  removedQuestions: number;
}

export interface MarkOriginalBaselineRequest {
  sourceDocumentId?: string | null;
  confirmationText: string;
  acknowledgeReplacement: boolean;
  acknowledgeBackup: boolean;
}

export type ExportKind = "consolidated" | "instruments";

export interface ExportWorkbookRequest {
  path: string;
  kind: ExportKind;
}

export interface ExportWorkbookResult {
  path: string;
  kind: ExportKind;
  exportedQuestions: number;
  addedQuestions: number;
  modifiedQuestions: number;
  removedQuestions: number;
}

export interface ConfigureWorkspaceRequest {
  folderPath: string;
}

export interface SyncResult {
  method: string;
  message: string;
  databasePath: string;
}

export interface OpenDatabaseRequest {
  databasePath: string;
}

export interface ExportDatabasePackageRequest {
  path: string;
}

export interface OpenDatabasePackageRequest {
  path: string;
}

export interface DatabasePackageResult {
  path: string;
  message: string;
}

export interface ResetDatabaseRequest {
  confirmationText: string;
  acknowledgeBackup: boolean;
  acknowledgeIrreversible: boolean;
}

export interface ResetDatabaseResult {
  deleted: boolean;
  message: string;
}

export interface ValidationIssue {
  id: string;
  severity: ValidationSeverity;
  entity: string;
  entityId: string;
  message: string;
}

export interface ProviderLink {
  id: string;
  subaudience: string;
  url: string;
  validationStatus: string;
  validatedAt?: string | null;
}

export interface NewProviderLink {
  subaudience: string;
  url: string;
}

export type ProviderQuestionReviewStatus =
  | "pending"
  | "correct"
  | "needsModification"
  | "missing";

export interface ProviderQuestionReview {
  id: string;
  questionId: string;
  instrumentAudience: string;
  status: ProviderQuestionReviewStatus;
  observation: string;
  evidencePath?: string | null;
  updatedAt: string;
}

export interface ProviderQuestionReviewItem {
  question: Question;
  instrumentAudience: string;
  review?: ProviderQuestionReview | null;
}

export interface SaveProviderQuestionReviewRequest {
  questionId: string;
  instrumentAudience: string;
  status: ProviderQuestionReviewStatus;
  observation: string;
  evidencePath?: string | null;
}

export interface ResetProviderQuestionReviewsRequest {
  confirmationText: string;
}

export interface ResetProviderQuestionReviewsResult {
  deletedReviews: number;
}

export interface SaveEvidenceAttachmentRequest {
  questionId: string;
  fileName?: string | null;
  dataUrl: string;
}

export interface SaveEvidenceAttachmentResult {
  path: string;
}

export interface ExportProviderReviewDocxRequest {
  path: string;
  instrumentAudience?: string | null;
}
