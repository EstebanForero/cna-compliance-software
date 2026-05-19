use chrono::{DateTime, NaiveDate, Utc};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyCycle {
    pub id: String,
    pub name: String,
    pub starts_on: NaiveDate,
    pub application_starts_on: NaiveDate,
    pub status: CycleStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CycleStatus {
    Planning,
    InReview,
    InApplication,
    Closed,
}

impl CycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::InReview => "in_review",
            Self::InApplication => "in_application",
            Self::Closed => "closed",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "in_review" => Self::InReview,
            "in_application" => Self::InApplication,
            "closed" => Self::Closed,
            _ => Self::Planning,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum QuestionScope {
    Institutional,
    Program,
}

impl QuestionScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Institutional => "institutional",
            Self::Program => "program",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "program" => Self::Program,
            _ => Self::Institutional,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CnaFactorCode {
    Factor1,
    Factor2,
    Factor3,
    Factor4,
    Factor5,
    Factor6,
    Factor7,
    Factor8,
    Factor9,
    Factor10,
    Factor11,
    Factor12,
    Custom(String),
}

impl CnaFactorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Factor1 => "1",
            Self::Factor2 => "2",
            Self::Factor3 => "3",
            Self::Factor4 => "4",
            Self::Factor5 => "5",
            Self::Factor6 => "6",
            Self::Factor7 => "7",
            Self::Factor8 => "8",
            Self::Factor9 => "9",
            Self::Factor10 => "10",
            Self::Factor11 => "11",
            Self::Factor12 => "12",
            Self::Custom(value) => value.as_str(),
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let value = value.trim().trim_start_matches("Factor").trim();
        match value {
            "1" => Some(Self::Factor1),
            "2" => Some(Self::Factor2),
            "3" => Some(Self::Factor3),
            "4" => Some(Self::Factor4),
            "5" => Some(Self::Factor5),
            "6" => Some(Self::Factor6),
            "7" => Some(Self::Factor7),
            "8" => Some(Self::Factor8),
            "9" => Some(Self::Factor9),
            "10" => Some(Self::Factor10),
            "11" => Some(Self::Factor11),
            "12" => Some(Self::Factor12),
            "" => None,
            _ => Some(Self::Custom(value.to_string())),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Factor1 => "Proyecto educativo del programa e identidad institucional".into(),
            Self::Factor2 => "Estudiantes".into(),
            Self::Factor3 => "Profesores".into(),
            Self::Factor4 => "Egresados".into(),
            Self::Factor5 => "Aspectos academicos y resultados de aprendizaje".into(),
            Self::Factor6 => "Permanencia y graduacion".into(),
            Self::Factor7 => "Interaccion con el entorno nacional e internacional".into(),
            Self::Factor8 => "Aportes de investigacion, innovacion y creacion".into(),
            Self::Factor9 => "Bienestar de la comunidad academica".into(),
            Self::Factor10 => "Medios educativos y ambientes de aprendizaje".into(),
            Self::Factor11 => "Organizacion, administracion y financiacion".into(),
            Self::Factor12 => "Recursos fisicos y tecnologicos".into(),
            Self::Custom(value) => value.clone(),
        }
    }
}

impl Serialize for CnaFactorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CnaFactorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).ok_or_else(|| {
            de::Error::custom(format!(
                "invalid CNA factor code '{value}', expected a non-empty value"
            ))
        })
    }
}

pub fn normalize_aspect_code(
    factor_code: &CnaFactorCode,
    characteristic_code: &str,
    aspect_code: &str,
    aspect_description: &str,
) -> String {
    let normalized = normalize_identifier(aspect_code);
    if !normalized.is_empty() {
        return normalized;
    }

    let description_key = normalize_identifier(aspect_description);
    let hash = stable_short_hash(&description_key);
    let characteristic = normalize_identifier(characteristic_code);
    if characteristic.is_empty() {
        format!("{}-auto-{hash}", factor_code.as_str())
    } else {
        format!("{}-{characteristic}-auto-{hash}", factor_code.as_str())
    }
}

fn normalize_identifier(value: &str) -> String {
    value
        .replace(['\n', '\r', '.', '/', '\\'], " ")
        .replace('°', "")
        .replace('ú', "u")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('á', "a")
        .replace('é', "e")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn stable_short_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", hash as u32)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum QuestionStatus {
    Keep,
    Modify,
    Add,
    Delete,
}

impl QuestionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Modify => "modify",
            Self::Add => "add",
            Self::Delete => "delete",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "modify" => Self::Modify,
            "add" => Self::Add,
            "delete" => Self::Delete,
            _ => Self::Keep,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: String,
    pub code: String,
    pub text: String,
    pub scope: QuestionScope,
    pub format: String,
    pub convention_code: Option<String>,
    pub status: QuestionStatus,
    pub factor: String,
    pub characteristic: String,
    pub aspect: String,
    pub audiences: Vec<String>,
    pub justification: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewQuestion {
    pub code: String,
    pub text: String,
    pub scope: QuestionScope,
    pub format: String,
    pub convention_code: Option<String>,
    pub status: QuestionStatus,
    pub factor: String,
    pub characteristic: String,
    pub aspect: String,
    pub audiences: Vec<String>,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuestionRequest {
    pub question_id: String,
    pub question: NewQuestion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GuidelineAspect {
    pub id: String,
    pub guideline_title: String,
    pub scope: QuestionScope,
    pub factor_code: CnaFactorCode,
    pub factor_name: String,
    pub characteristic_code: String,
    pub characteristic_name: String,
    pub aspect_code: String,
    pub aspect_description: String,
    pub requires_appreciation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewGuidelineAspect {
    pub guideline_title: String,
    pub scope: QuestionScope,
    pub factor_code: CnaFactorCode,
    pub factor_name: String,
    pub characteristic_code: String,
    pub characteristic_name: String,
    pub aspect_code: String,
    pub aspect_description: String,
    pub requires_appreciation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportWorkbookRequest {
    pub path: String,
    pub cycle_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportWorkbookResult {
    pub source_document_id: String,
    pub file_name: String,
    pub sheet_name: String,
    pub imported_questions: usize,
    pub imported_guideline_aspects: usize,
    pub skipped_rows: usize,
    pub detected_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportWorkbookPreviewResult {
    pub file_name: String,
    pub sheet_name: String,
    pub detected_questions: usize,
    pub detected_guideline_aspects: usize,
    pub skipped_rows: usize,
    pub detected_columns: Vec<String>,
    pub detected_audiences: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewSourceDocument {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub document_type: String,
    pub imported_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceDocument {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub document_type: String,
    pub imported_at: DateTime<Utc>,
    pub imported_rows: usize,
    pub skipped_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OriginalQuestionSnapshot {
    pub id: String,
    pub question_id: String,
    pub source_document_id: String,
    pub code: String,
    pub text: String,
    pub scope: QuestionScope,
    pub format: String,
    pub convention_code: Option<String>,
    pub status: QuestionStatus,
    pub factor: String,
    pub characteristic: String,
    pub aspect: String,
    pub audiences: Vec<String>,
    pub content_hash: String,
    pub marked_by: String,
    pub marked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BaselineStatus {
    pub has_original: bool,
    pub source_document: Option<SourceDocument>,
    pub original_questions: usize,
    pub current_questions: usize,
    pub unchanged_questions: usize,
    pub modified_questions: usize,
    pub added_questions: usize,
    pub removed_questions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MarkOriginalBaselineRequest {
    pub source_document_id: Option<String>,
    pub confirmation_text: String,
    pub acknowledge_replacement: bool,
    pub acknowledge_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportKind {
    Consolidated,
    Instruments,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkbookRequest {
    pub path: String,
    pub kind: ExportKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkbookResult {
    pub path: String,
    pub kind: ExportKind,
    pub exported_questions: usize,
    pub added_questions: usize,
    pub modified_questions: usize,
    pub removed_questions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionDiffKind {
    Unchanged,
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatus {
    pub database_path: String,
    pub configured_onedrive_path: Option<String>,
    pub microsoft_account: Option<MicrosoftAccount>,
    pub microsoft_auth_config: Option<MicrosoftAuthConfig>,
    pub editor_profile: Option<EditorProfile>,
    pub graph_sync_available: bool,
    pub has_questions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditorProfile {
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveEditorProfileRequest {
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftAccount {
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftAuthConfig {
    pub client_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftLoginRequest {
    pub client_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftLoginResult {
    pub account: MicrosoftAccount,
    pub tenant_id: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub method: String,
    pub message: String,
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureWorkspaceRequest {
    pub folder_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenDatabaseRequest {
    pub database_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportDatabasePackageRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenDatabasePackageRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DatabasePackageResult {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetDatabaseRequest {
    pub confirmation_text: String,
    pub acknowledge_backup: bool,
    pub acknowledge_irreversible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetDatabaseResult {
    pub deleted: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteGuidelineAspectRequest {
    pub aspect_id: String,
    pub confirmation_text: String,
    pub acknowledge_related_questions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteGuidelineAspectResult {
    pub deleted_aspect: bool,
    pub affected_questions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuidelineAspectRequest {
    pub aspect_id: String,
    pub aspect: NewGuidelineAspect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuidelineAspectResult {
    pub aspect: GuidelineAspect,
    pub affected_questions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLogEntry {
    pub id: String,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub editor_name: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistorySnapshot {
    pub id: String,
    pub summary: String,
    pub editor_name: String,
    pub snapshot_kind: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoreHistorySnapshotRequest {
    pub snapshot_id: String,
    pub confirmation_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHistorySnapshotRequest {
    pub snapshot_id: String,
    pub confirmation_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ValidationSeverity {
    Blocking,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub id: String,
    pub severity: ValidationSeverity,
    pub entity: String,
    pub entity_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLink {
    pub id: String,
    pub subaudience: String,
    pub url: String,
    pub validation_status: String,
    pub validated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NewProviderLink {
    pub subaudience: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderQuestionReviewStatus {
    Pending,
    Correct,
    NeedsModification,
    Missing,
}

impl ProviderQuestionReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Correct => "correct",
            Self::NeedsModification => "needs_modification",
            Self::Missing => "missing",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "correct" => Self::Correct,
            "needs_modification" => Self::NeedsModification,
            "missing" => Self::Missing,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderQuestionReview {
    pub id: String,
    pub question_id: String,
    pub instrument_audience: String,
    pub status: ProviderQuestionReviewStatus,
    pub observation: String,
    pub evidence_path: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderQuestionReviewItem {
    pub question: Question,
    pub instrument_audience: String,
    pub review: Option<ProviderQuestionReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderQuestionReviewRequest {
    pub question_id: String,
    pub instrument_audience: String,
    pub status: ProviderQuestionReviewStatus,
    pub observation: String,
    pub evidence_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetProviderQuestionReviewsRequest {
    pub confirmation_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResetProviderQuestionReviewsResult {
    pub deleted_reviews: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveEvidenceAttachmentRequest {
    pub question_id: String,
    pub file_name: Option<String>,
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveEvidenceAttachmentResult {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportProviderReviewDocxRequest {
    pub path: String,
    pub instrument_audience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub active_cycle: Option<SurveyCycle>,
    pub workspace: WorkspaceStatus,
    pub total_questions: usize,
    pub pending_changes: usize,
    pub blocking_validations: usize,
    pub provider_links_pending: usize,
    pub questions_by_status: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatusCount {
    pub status: QuestionStatus,
    pub count: usize,
}
