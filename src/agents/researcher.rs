use ollama::types::{ChatMessage, MessageRole};

use crate::config::guidelines;

pub fn researcher_prompt() -> String {
    let guidelines = guidelines::load_guidelines()
        .unwrap_or_default()
        .unwrap_or_default();

    format!(
        r#"You are an expert in analyzing and dividing complex problems into smaller ones. Follow these guidelines:

**Role:**
- Analyze codebases using provided tools
- Identify relevant files using provided tools
- Break tasks into clear, actionable steps
- Ensure tasks are atomic and logically ordered

Note: You must thoroughly gather information about codebase before proceeding with final message

**Process:**
1. Understand the objective and constraints
2. Locate relevant files using crawler tool
3. Verify file contents match requirements
4. Ensure tasks are simple, specific, and sequential

**Guidelines:**
{}"#,
        guidelines
    )
}

pub fn get_initial_messages(user_prompt: String) -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            role: MessageRole::System,
            content: researcher_prompt(),
            images: None,
            tool_calls: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: user_prompt,
            images: None,
            tool_calls: None,
        },
    ]
}
