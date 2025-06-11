//! `file_eyes` is a library providing file system observation capabilities
//! for AI agents, including fuzzy path searching, directory listing,
//! and file content reading.

pub mod error;
mod tool;

use std::env;

use ollama::types::{FunctionDefinition, ToolCall, ToolDefinition};
use serde_json::{json, Value};

use crate::Tool;

pub use self::error::CrawlerError;
pub use self::tool::Crawler;

impl Tool for Crawler {
    fn get_tool_defs() -> Vec<ToolDefinition> {
        let list_dir_tool = ToolDefinition {
            type_: "function".into(),
            function: FunctionDefinition {
                name: "crawler.list_directory_contents".into(),
                description: "Lists direct children (files and directories) within a specified \
                 directory path."
                    .into(),
                parameters: json!({
                    "type": "object",
                    "properties": { "path": { "type": "string", "description": "Directory path to list" } },
                    "required": ["path"]
                }),
            },
        };

        let read_file_tool = ToolDefinition {
            type_: "function".into(),
            function: FunctionDefinition {
                name: "crawler.read_file_contents".into(),
                description: "Reads the entire textual content of a specified file path.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": { "path": { "type": "string", "description": "File path to read" } },
                    "required": ["path"]
                }),
            },
        };

        let fuzzy_search_tool = ToolDefinition {
            type_: "function".into(),
            function: FunctionDefinition {
                name: "crawler.fuzzy_search_paths".into(),
                description: "Recursively searches the codebase for files/directories whose paths \
                 fuzzy-match any of the provided query strings."
                    .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "queries": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of query substrings to match against file paths"
                        }
                    },
                    "required": ["queries"]
                }),
            },
        };

        vec![fuzzy_search_tool, read_file_tool, list_dir_tool]
    }

    async fn handle_tool_call(call: ToolCall) -> String {
        let args: Value = call.function.arguments;
        let name = call
            .function
            .name
            .strip_prefix("crawler.")
            .unwrap_or(&call.function.name);

        let cwd = env::current_dir().unwrap_or("result: {}".into());
        let crawler = Crawler::new(&cwd).await;

        match name {
            "fuzzy_search_paths" => {
                let queries = args
                    .get("queries")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();

                let results = crawler.fuzzy_search_paths(&queries);
                let entries = results
                    .into_iter()
                    .map(|(score, path)| {
                        json!({
                            "path":   path.to_string_lossy(),
                            "score":  score,
                        })
                    })
                    .collect::<Vec<_>>();

                json!({ "results": entries }).to_string()
            }

            "list_directory_contents" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let entries = crawler.list_directory_contents(path).await;
                let paths = entries
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();

                json!({ "entries": paths }).to_string()
            }

            "read_file_contents" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let read_res = crawler.read_file_contents(path).await;
                json!({"content":read_res}).to_string()
            }

            other => {
                eprintln!("⚠️ Unknown tool called: {}", other);
                json!({}).to_string()
            }
        }
    }
}
