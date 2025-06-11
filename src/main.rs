mod consts;

use consts::system_prompt;
use ollama::{
    types::{ChatMessage, ChatRequest, MessageRole},
    OllamaClient,
};
use serde_json::{json, Value};
use std::{
    env,
    error::Error,
    io::{self, Write},
    process,
};
use tools::{crawler::Crawler, Tool};

pub trait ResponseSchema {
    fn schema() -> &'static str;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    // This res_format remains the same, it defines the desired final output structure.
    let res_format = json!({
        "type": "object",
        "properties": {
            "tasks": {
                "type": "array",
                "description": "A list of tasks.",
                "items": {
                    "type": "object",
                    "properties": {
                        "related_files": {
                            "type": "array",
                            "description": "A list of file paths related to the task.",
                            "items": {
                                "type": "string"
                            }
                        },
                        "description": {
                            "type": "string",
                            "description": "A description of the task."
                        }
                    },
                    "required": ["related_files", "description"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["tasks"],
        "additionalProperties": false
    });

    if args.is_empty() {
        eprintln!("Sir, a prompt is required to begin the conversation.");
        eprintln!("Usage: cargo run -- \"<your initial question>\"");
        process::exit(1);
    }
    let initial_prompt = args.join(" ");

    let cwd = env::current_dir()?;
    let client = OllamaClient::new("http://127.0.0.1:11434")?;
    let model_name = "qwen3:latest".to_string(); // Assuming qwen3:latest supports tool calling

    let mut messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: system_prompt(cwd),
            images: None,
            tool_calls: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: initial_prompt,
            images: None,
            tool_calls: None,
        },
    ];

    const MAX_TOOL_CALL_LOOPS: usize = 7; // Max turns for tool calls before asking for final output
    let mut current_tool_loop = 0;
    let mut final_output_requested = false;

    // --- Main Loop: Tool Calling and Reasoning Phase ---
    while current_tool_loop < MAX_TOOL_CALL_LOOPS && !final_output_requested {
        current_tool_loop += 1;
        println!(
            "\n=== Reasoning Step {}/{} ===",
            current_tool_loop, MAX_TOOL_CALL_LOOPS
        );

        let chat_req = ChatRequest {
            model: model_name.clone(),
            messages: messages.clone(),
            tools: Some(Crawler::get_tool_defs()),
            stream: false,
            format: None, // <-- Crucial: No format here during tool calls and reasoning
            think: true,  // Let the model think and use tools
            options: None,
            keep_alive: None,
        };

        let res = client.chat(&chat_req).await?;
        let assistant_msg = res.message.clone();
        messages.push(assistant_msg.clone()); // Add model's response to history

        if let Some(tool_calls) = assistant_msg.tool_calls {
            if tool_calls.is_empty() {
                // If the model explicitly returned an empty tool_calls array,
                // it might be done with tools and is providing a content message.
                // In this case, we'll break and proceed to request the final format.
                println!(
                    "\n🧠 Assistant (reasoning complete, preparing final output): {}",
                    assistant_msg.content
                );
                final_output_requested = true; // Signal to break this loop and request final output
                break;
            }
            // Execute tool calls
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
            final_output_requested = true;
            break;
        }
    }

    if final_output_requested || current_tool_loop >= MAX_TOOL_CALL_LOOPS {
        println!("\n=== Requesting Final Structured Output ===");
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: "Based on all the information gathered and your reasoning, please provide the complete task breakdown in the precise JSON format you were instructed to use. Ensure the output is a valid JSON object matching the `tasks` schema.".to_string(),
            images: None,
            tool_calls: None,
        });

        let chat_req_final = ChatRequest {
            model: model_name.clone(),
            messages: messages.clone(),
            tools: None,
            stream: false,
            format: Some(res_format.clone()),
            think: false,
            options: None,
            keep_alive: None,
        };

        match client.chat(&chat_req_final).await {
            Ok(res_final) => {
                let final_message_content = res_final.message.content;
                println!("\n Final Structured Output:\n{}", final_message_content);
                match serde_json::from_str::<Value>(&final_message_content) {
                    Ok(parsed_json) => {
                        println!("\n✅ JSON parsing successful!");
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
        println!("\n Exiting without final formatted output (tool loop limit reached or unexpected state).");
    }

    // --- Interactive loop (optional, keeping it in for continued interaction) ---
    // If you want to keep the interactive loop *after* the initial task breakdown:
    // You might move this into a separate function or only enable it after the
    // initial "task breakdown" is complete. For this example, I'll keep it
    // simple and assume the main task ends after the formatted output.
    // If you want to continue a general chat, you'd need to re-evaluate the `format`
    // and `tools` parameters based on the new user prompt.

    println!("\n--- Conversation concluded. ---");

    // Original interactive loop, modified to be an ongoing chat AFTER the main task if needed
    // If you want this to continue a general chat, you'd need to re-think how `format` and `tools`
    // are applied in subsequent turns. For now, it will just exit after the main task.
    loop {
        print!("\n> ");
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

        // For subsequent interactive turns, you'd generally not apply `format`
        // unless explicitly asking for structured data again.
        // And you'd likely keep tools available.
        messages.push(ChatMessage {
            role: MessageRole::User,
            content: trimmed_input.to_string(),
            images: None,
            tool_calls: None,
        });

        // Re-run chat request for general interaction
        let chat_req_interactive = ChatRequest {
            model: model_name.clone(),
            messages: messages.clone(),
            tools: Some(Crawler::get_tool_defs()), // Keep tools available for general chat
            stream: false,
            format: None, // No format for general chat, unless specifically prompted
            think: true,
            options: None,
            keep_alive: None,
        };

        match client.chat(&chat_req_interactive).await {
            Ok(res_interactive) => {
                let assistant_msg_interactive = res_interactive.message.clone();
                messages.push(assistant_msg_interactive.clone());
                println!("\n🧠 Assistant: {}", assistant_msg_interactive.content);

                // If tool calls are made in interactive mode, you'd handle them here too
                if let Some(tool_calls) = assistant_msg_interactive.tool_calls {
                    if !tool_calls.is_empty() {
                        println!("\n(Assistant made tool calls in interactive mode - not fully handled in this simple loop)");
                        // In a real application, you'd need to loop and execute these too.
                        // For simplicity, this example just prints.
                    }
                }
            }
            Err(e) => {
                eprintln!("\n❌ Error during interactive chat: {}", e);
            }
        }
    }

    Ok(())
}
