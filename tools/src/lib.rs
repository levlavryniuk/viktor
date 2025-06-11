use ollama::types::{ToolCall, ToolDefinition};

pub mod crawler;

pub trait Tool {
    fn get_tool_defs() -> Vec<ToolDefinition>;
    async fn handle_tool_call(call: ToolCall) -> String;
}
