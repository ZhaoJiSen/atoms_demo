use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct App {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) idea: String,
    pub(crate) status: AppStatus,
    pub(crate) steps: Vec<AgentStep>,
    pub(crate) messages: Vec<AgentMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<GeneratedResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AppStatus {
    Pending,
    Generating,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentStep {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) status: AgentStepStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentStepStatus {
    Waiting,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentMessage {
    pub(crate) id: String,
    pub(crate) role: AgentMessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent_name: Option<String>,
    pub(crate) content: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AgentMessageRole {
    User,
    Agent,
    System,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneratedResult {
    pub(crate) product_spec: ProductSpec,
    pub(crate) pages: Vec<GeneratedPage>,
    pub(crate) apis: Vec<GeneratedApi>,
    pub(crate) data_models: Vec<DataModel>,
    pub(crate) file_structure: Vec<FileNode>,
    pub(crate) preview: PreviewData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSpec {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) target_users: Vec<String>,
    pub(crate) core_features: Vec<String>,
    pub(crate) user_flow: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneratedPage {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeneratedApi {
    pub(crate) method: ApiMethod,
    pub(crate) path: String,
    pub(crate) description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) request_example: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response_example: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum ApiMethod {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataModel {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) fields: Vec<DataModelField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataModelField {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileNode {
    pub(crate) path: String,
    #[serde(rename = "type")]
    pub(crate) node_type: FileNodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FileNodeType {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewData {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) theme: PreviewTheme,
    pub(crate) sections: Vec<PreviewSection>,
    pub(crate) actions: Vec<PreviewAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PreviewTheme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewSection {
    pub(crate) id: String,
    pub(crate) title: String,
    #[serde(rename = "type")]
    pub(crate) section_type: PreviewSectionType,
    pub(crate) content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) items: Option<Vec<PreviewItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PreviewSectionType {
    Hero,
    Stats,
    List,
    Form,
    Cards,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewItem {
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewAction {
    pub(crate) label: String,
    #[serde(rename = "type")]
    pub(crate) action_type: PreviewActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum PreviewActionType {
    Primary,
    Secondary,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAppRequest {
    pub(crate) idea: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAppMessageRequest {
    pub(crate) content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateAppRequest {
    pub(crate) title: Option<String>,
    pub(crate) idea: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DemoInitState {
    pub(crate) status: InitStatus,
    pub(crate) api_healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) completed_at: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InitStatus {
    NotInitialized,
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OAuthProviderSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) mode: OAuthProviderMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OAuthProviderMode {
    Demo,
    Google,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OAuthUser {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) email: String,
    pub(crate) provider: OAuthProviderSummary,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthSession {
    pub(crate) authenticated: bool,
    pub(crate) provider: OAuthProviderSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) user: Option<OAuthUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthLoginQuery {
    pub(crate) redirect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerConnection {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) auth_type: ServerAuthType,
    pub(crate) status: ServerConnectionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_connected_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ServerAuthType {
    Password,
    PrivateKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ServerConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateServerConnectionRequest {
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: Option<u16>,
    pub(crate) username: String,
    pub(crate) auth_type: ServerAuthType,
    pub(crate) password: Option<String>,
    pub(crate) private_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateServerConnectionRequest {
    pub(crate) name: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) username: Option<String>,
    pub(crate) auth_type: Option<ServerAuthType>,
    pub(crate) password: Option<String>,
    pub(crate) private_key: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum ServerCredential {
    Password(String),
    PrivateKey(String),
}

impl ServerCredential {
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Self::Password(value) | Self::PrivateKey(value) => value.trim().is_empty(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SuccessResponse {
    pub(crate) success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Note {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) app_id: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateNoteRequest {
    pub(crate) title: String,
    pub(crate) content: Option<String>,
    pub(crate) app_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateNoteRequest {
    pub(crate) title: Option<String>,
    pub(crate) content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AiProvider {
    Gpt,
    Mimo,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Gpt => write!(f, "gpt"),
            AiProvider::Mimo => write!(f, "mimo"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiSettings {
    pub(crate) provider: AiProvider,
    pub(crate) api_key: String,
    pub(crate) model: String,
    pub(crate) base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateAiSettingsRequest {
    pub(crate) provider: AiProvider,
    pub(crate) api_key: String,
    pub(crate) model: Option<String>,
    pub(crate) base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) enum TerminalClientMessage {
    Input { data: String },
    Resize { cols: u16, rows: u16 },
    Disconnect,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) enum TerminalServerMessage {
    Output { data: String },
    Status { status: ServerConnectionStatus },
    Error { error: String },
}
