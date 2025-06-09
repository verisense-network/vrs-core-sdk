use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vrs_core_sdk::export;

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[export]
pub enum Number {
    I64(i64),
    F64(f64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[export]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

// Helper for `kind` fields often being fixed
macro_rules! kind_field {
    ($name:expr) => {
        #[serde(default = "default_kind")]
        fn default_kind() -> String {
            $name.to_string()
        }
    };
}

// // --- AgentProvider ---
#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct AgentProvider {
    pub organization: String,
    pub url: String,
}

// // --- AgentCapabilities ---
#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct AgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_transition_history: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<AgentExtension>>,
}

// // --- AgentExtension ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct AgentExtension {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<BTreeMap<String, Value>>,
}

// // --- AgentSkill ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modes: Option<Vec<String>>,
}

// // --- SecurityScheme (Simplified, focusing on structure) ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(tag = "type", rename_all = "camelCase")]
#[export]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey(APIKeySecurityScheme),
    Http(HTTPAuthSecurityScheme),
    #[serde(rename = "oauth2")]
    OAuth2(OAuth2SecurityScheme),
    OpenIdConnect(OpenIdConnectSecurityScheme),
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct SecuritySchemeBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct APIKeySecurityScheme {
    #[serde(flatten)]
    pub base: SecuritySchemeBase,
    #[serde(rename = "in")]
    pub location: String, // "query", "header", or "cookie"
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct HTTPAuthSecurityScheme {
    #[serde(flatten)]
    pub base: SecuritySchemeBase,
    pub scheme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct OAuth2SecurityScheme {
    #[serde(flatten)]
    pub base: SecuritySchemeBase,
    pub flows: OAuthFlows, // Simplified
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct OAuthFlows {
    // For brevity, actual flow definitions are omitted but would be here
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeOAuthFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<ClientCredentialsOAuthFlow>, // ... other flows
}
#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[export]
pub struct AuthorizationCodeOAuthFlow {
    /* fields */ pub authorization_url: String,
    pub token_url: String,
    pub scopes: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[export]
pub struct ClientCredentialsOAuthFlow {
    /* fields */ pub token_url: String,
    pub scopes: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct OpenIdConnectSecurityScheme {
    #[serde(flatten)]
    pub base: SecuritySchemeBase,
    pub open_id_connect_url: String,
}

// --- AgentCard ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    pub capabilities: AgentCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<BTreeMap<String, SecurityScheme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<BTreeMap<String, Vec<String>>>>,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
    pub skills: Vec<AgentSkill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_authenticated_extended_card: Option<bool>,
}

// --- TaskState ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Canceled,
    Failed,
    Rejected,
    AuthRequired,
    Unknown,
}

// --- TaskStatus ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>, // Assuming Message type is defined below
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

// --- PartBase (incorporated into Part variants) ---

// --- TextPart ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct TextPartPayload {
    // Renamed to avoid conflict if Part::Text used TextPart struct name
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}

// --- File Variants ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct FileBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct FileWithBytes {
    #[serde(flatten)]
    pub base: FileBase,
    pub bytes: String, // base64 encoded
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct FileWithUri {
    #[serde(flatten)]
    pub base: FileBase,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(untagged)] // Distinguishes by presence of 'bytes' or 'uri'
#[export]
pub enum FileVariant {
    Bytes(FileWithBytes),
    Uri(FileWithUri),
}

// --- FilePart ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct FilePartPayload {
    pub file: FileVariant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}

// --- DataPart ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct DataPartPayload {
    pub data: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}

// --- Part ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[export]
pub enum Part {
    Text(TextPartPayload),
    File(FilePartPayload),
    Data(DataPartPayload),
}

// --- Message ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct Message {
    pub role: String, // "user" | "agent"
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_task_ids: Option<Vec<String>>,
    pub message_id: String, // Uuid or string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    pub kind: String, // "message" - can use macro or default
}

// --- Artifact ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct Artifact {
    pub artifact_id: String, // Uuid or string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

// --- Task ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct Task {
    pub id: String,         // Uuid
    pub context_id: String, // Uuid
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
    pub kind: String, // "task"
}

// --- Push Notification (Simplified) ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct PushNotificationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub url: String,
    // ... other fields
}

// --- Params for RPC methods ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct TaskIdParams {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct TaskQueryParams {
    #[serde(flatten)]
    pub id_params: TaskIdParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct MessageSendConfiguration {
    pub accepted_output_modes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notification_config: Option<PushNotificationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
#[export]
pub struct MessageSendParams {
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<MessageSendConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}

// --- JSON-RPC Structures ---
#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[export]
pub struct JSONRPCRequest<P> {
    pub jsonrpc: String, // "2.0"
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<P>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>, // String, Number, or Null
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[export]
pub struct JSONRPCSuccessResponse<R> {
    pub jsonrpc: String, // "2.0"
    pub result: R,
    pub id: Value, // Should match request ID
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error, Encode, Decode)]
#[error("JSONRPCError code: {code}, message: {message}")]
#[export]
pub struct JSONRPCErrorData {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[export]
pub struct JSONRPCErrorResponse {
    pub jsonrpc: String, // "2.0"
    pub error: JSONRPCErrorData,
    pub id: Option<Value>, // Can be null even if request ID was present for some errors
}

// --- A2A Specific Errors (as constructors for JSONRPCErrorData) ---
// These constants would be used to create JSONRPCErrorData instances
pub mod error_codes {
    pub const JSON_PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST_ERROR: i32 = -32600;
    pub const METHOD_NOT_FOUND_ERROR: i32 = -32601;
    pub const INVALID_PARAMS_ERROR: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    pub const TASK_NOT_FOUND_ERROR: i32 = -32001;
    pub const TASK_NOT_CANCELABLE_ERROR: i32 = -32002;
    // ... other A2A specific error codes
}

impl JSONRPCErrorData {
    pub fn new(code: i32, message: String, data: Option<Value>) -> Self {
        JSONRPCErrorData {
            code,
            message,
            data,
        }
    }
    pub fn task_not_found(task_id: &str) -> Self {
        Self::new(
            error_codes::TASK_NOT_FOUND_ERROR,
            "Task not found".to_string(),
            Some(Value::Object({
                let mut map = BTreeMap::new();
                map.insert("taskId".to_string(), Value::String(task_id.to_string()));
                map
            })),
        )
    }
    pub fn method_not_found(method_name: &str) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND_ERROR,
            "Method not found".to_string(),
            Some(Value::Object({
                let mut map = BTreeMap::new();
                map.insert("method".to_string(), Value::String(method_name.to_string()));
                map
            })),
        )
    }
    pub fn invalid_params(details: Option<Value>) -> Self {
        Self::new(
            error_codes::INVALID_PARAMS_ERROR,
            "Invalid parameters".to_string(),
            details,
        )
    }
    pub fn internal_error(details: Option<Value>) -> Self {
        Self::new(
            error_codes::INTERNAL_ERROR,
            "Internal server error".to_string(),
            details,
        )
    }
    pub fn task_not_cancelable(task_id: &str) -> Self {
        Self::new(
            error_codes::TASK_NOT_CANCELABLE_ERROR,
            "Task not cancelable".to_string(),
            Some(Value::Object({
                let mut map = BTreeMap::new();
                map.insert("taskId".to_string(), Value::String(task_id.to_string()));
                map
            })),
        )
    }
}

#[export]
// Response types for specific methods (Success variants)
pub type SendMessageSuccessResponseResult = Task; // Or Message, simplified to Task for now
#[export]
pub type GetTaskSuccessResponseResult = Task;

#[export]
pub type CancelTaskSuccessResponseResult = Task;

// Top-level response enum for the router
#[derive(Serialize, Debug, Encode, Decode)]
#[serde(untagged)] // Important for sending either success or error structure
#[export]
pub enum JSONRPCHandlerResponse {
    Success(JSONRPCSuccessResponse<Value>),
    Error(JSONRPCErrorResponse),
}
