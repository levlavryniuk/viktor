use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request for POST /api/generate
#[derive(Debug, Serialize, Default)]
pub struct GenerateRequest {
    /// e.g. "llama3:8b"
    pub model: String,

    /// The prompt to complete. `None` loads/unloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Text appended after the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// For multimodal models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,

    /// Non-streaming by default
    #[serde(default)]
    pub stream: bool,

    /// JSON mode or schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Value>,

    /// Model runtime options (seed, top_k, …)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,

    /// Override system prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Override template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Bypass templating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,

    /// Seconds to keep model alive (`0` to unload)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<u64>,

    /// Conversation context tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<u32>>,
}

/// Response from POST /api/generate
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub response: String,
    pub done: bool,
    #[serde(rename = "done_reason")]
    pub done_reason: Option<String>,
    pub context: Option<Vec<u32>>,
    #[serde(rename = "total_duration")]
    pub total_duration: Option<u64>,
    #[serde(rename = "load_duration")]
    pub load_duration: Option<u64>,
    #[serde(rename = "prompt_eval_count")]
    pub prompt_eval_count: Option<u32>,
    #[serde(rename = "prompt_eval_duration")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(rename = "eval_count")]
    pub eval_count: Option<u32>,
    #[serde(rename = "eval_duration")]
    pub eval_duration: Option<u64>,
}

/// Reference to a function in a tool-call
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionRef {
    pub name: String,
    pub arguments: Value,
}

/// A single tool‐call in a chat response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub function: FunctionRef,
}

/// Definition of a function for `/api/chat`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Wraps a function into a "tool"
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub type_: String, // always "function"
    pub function: FunctionDefinition,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "user")]
    User,
}
/// A chat message (may carry images or tool_calls)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Request for POST /api/chat
#[derive(Debug, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    /// Optional tools (functions) the model can call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<u64>,
    pub think: bool,
}

/// Response from POST /api/chat
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(rename = "done_reason")]
    pub done_reason: Option<String>,
    #[serde(rename = "total_duration")]
    pub total_duration: Option<u64>,
    #[serde(rename = "load_duration")]
    pub load_duration: Option<u64>,
    #[serde(rename = "prompt_eval_count")]
    pub prompt_eval_count: Option<u32>,
    #[serde(rename = "prompt_eval_duration")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(rename = "eval_count")]
    pub eval_count: Option<u32>,
    #[serde(rename = "eval_duration")]
    pub eval_duration: Option<u64>,
}

impl ToolCall {
    pub fn log(&self) {
        println!(
            "ToolCall → function: {} | arguments: {}",
            self.function.name, self.function.arguments
        );
    }
}
