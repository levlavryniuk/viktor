use ollama::types::{ChatMessage, ChatRequest, MessageRole};
use serde_json::json;
use std::error::Error;
use tools::{crawler::Crawler, Tool};

pub async fn handle_tool_calls(
    messages: &mut Vec<ChatMessage>,
    client: &ollama::OllamaClient,
    model_name: &str,
) -> Result<(), Box<dyn Error>> {
    const MAX_INTERACTIVE_TOOL_LOOPS: usize = 5;
    let mut tool_loop_count = 0;

    loop {
        if tool_loop_count >= MAX_INTERACTIVE_TOOL_LOOPS {
            println!("\n⚠️ Max tool call loops reached in interactive mode");
            break;
        }

        let chat_req = ChatRequest {
            model: model_name.to_string(),
            messages: messages.clone(),
            tools: Some(Crawler::get_tool_defs()),
            stream: false,
            format: None,
            think: true,
            options: None,
            keep_alive: None,
        };

        let res = client.chat(&chat_req).await?;
        let assistant_msg = res.message.clone();
        messages.push(assistant_msg.clone());

        if let Some(tool_calls) = assistant_msg.tool_calls {
            if tool_calls.is_empty() {
                println!("\n🧠 Assistant: {}", assistant_msg.content);
                break;
            }

            tool_loop_count += 1;
            println!("\n🔧 Executing {} tool call(s)...", tool_calls.len());

            for call in tool_calls {
                call.log();
                let name = &call.function.name;
                let prefix: Vec<&str> = name.split('.').collect();
                let prefix = prefix.first().expect("Bad tool call name format");

                let tool_output = match *prefix {
                    "crawler" => Crawler::handle_tool_call(call).await,
                    _ => {
                        eprintln!("Error: Unexpected tool call prefix: {}", prefix);
                        json!({"error": format!("Unexpected tool call prefix: {}", prefix)})
                            .to_string()
                    }
                };

                messages.push(ChatMessage {
                    role: MessageRole::Tool,
                    content: tool_output,
                    images: None,
                    tool_calls: None,
                });
            }
        } else {
            println!("\n🧠 Assistant: {}", assistant_msg.content);
            break;
        }
    }

    Ok(())
}
