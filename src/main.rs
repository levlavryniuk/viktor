mod agents;
mod config;
mod response;
mod system_prompt;
mod tool_handling;

use agents::researcher::get_initial_messages;
use config::init::ViktorInit;

use ollama::{
    types::{ChatMessage, ChatRequest, MessageRole},
    OllamaClient,
};
use response::{res_format, Response};
use serde_json::{json, Value};
use std::{
    env,
    error::Error,
    io::{self, Write},
    process,
};
use tool_handling::handle_tool_calls;
use tools::{crawler::Crawler, Tool};

const MODEL: &str = "qwen3:latest";

fn get_user_prompt() -> String {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.first().map(|s| s.as_str()) == Some("init") {
        let init = ViktorInit::new().expect("Unable to init");
        init.execute().expect("Unable to init")
    }

    if args.is_empty() {
        eprintln!("Sir, a prompt is required to begin the conversation.");
        eprintln!("Usage: cargo run -- \"<your initial question>\"");
        eprintln!("       cargo run -- init");
        process::exit(1);
    }
    args.join(" ")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = OllamaClient::new("http://127.0.0.1:11434")?;

    let initial_prompt = get_user_prompt();

    let mut messages = get_initial_messages(initial_prompt);
    const MAX_TOOL_CALL_LOOPS: usize = 10;
    let mut current_tool_loop = 0;
    let mut final_output_requested = false;

    while current_tool_loop < MAX_TOOL_CALL_LOOPS && !final_output_requested {
        current_tool_loop += 1;
        println!(
            "\n=== Reasoning Step {}/{} ===",
            current_tool_loop, MAX_TOOL_CALL_LOOPS
        );

        let chat_req = ChatRequest {
            model: MODEL.to_string().clone(),
            messages: messages.clone(),
            tools: Some(Crawler::get_tool_defs()),
            stream: false,
            format: None,
            think: false,
            options: None,
            keep_alive: None,
        };

        let res = client.chat(&chat_req).await?;
        let assistant_msg = res.message.clone();
        messages.push(assistant_msg.clone());

        if let Some(tool_calls) = assistant_msg.tool_calls {
            if assistant_msg.content.contains("<FINAL>") {
                println!(
                    "\n🧠 Assistant (reasoning complete, preparing final output): {}",
                    assistant_msg.content
                );
                final_output_requested = true;
                break;
            }
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
        }
    }

    if final_output_requested || current_tool_loop >= MAX_TOOL_CALL_LOOPS {
        println!("\n=== Requesting Final Structured Output ===");
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: "Based on all the information gathered and your reasoning, please provide the complete task breakdown in the precise JSON format you were instructed to use. Ensure the output is a valid JSON object matching the updated `tasks` schema with objective, affected_files, changes fields.".to_string(),
            images: None,
            tool_calls: None,
        });

        let chat_req_final = ChatRequest {
            model: MODEL.to_string().clone(),
            messages: messages.clone(),
            tools: None,
            stream: false,
            format: Some(res_format().clone()),
            think: false,
            options: None,
            keep_alive: None,
        };

        match client.chat(&chat_req_final).await {
            Ok(res_final) => {
                let final_message_content = res_final.message.content;
                match serde_json::from_str::<Value>(&final_message_content) {
                    Ok(parsed_json) => {
                        let res: Response =
                            serde_json::from_value(parsed_json).expect("Bad response structure");
                        println!("{res}");
                    }
                    Err(e) => {
                        eprintln!("\n❌ Error parsing final JSON output: {}", e);
                        eprintln!("Raw output:\n{}", final_message_content);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ Error during final output request: {}", e);
            }
        }
    } else {
        println!("\nExiting without final formatted output (tool loop limit reached or unexpected state).");
    }

    println!("\n--- Task completed. Entering interactive mode ---");

    print!("\n> ");
    loop {
        io::stdout().flush()?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let trimmed_input = user_input.trim();

        if trimmed_input.is_empty()
            || trimmed_input.eq_ignore_ascii_case("exit")
            || trimmed_input.eq_ignore_ascii_case("quit")
        {
            println!("Very well. Concluding session.");
            break;
        }

        messages.push(ChatMessage {
            role: MessageRole::User,
            content: trimmed_input.to_string(),
            images: None,
            tool_calls: None,
        });

        match handle_tool_calls(&mut messages, &client, MODEL).await {
            Ok(_) => {} // Tool calls handled successfully
            Err(e) => {
                eprintln!("\n❌ Error during interactive chat: {}", e);
            }
        }
    }

    Ok(())
}
