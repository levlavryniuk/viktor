use serde::Deserialize;
use serde_json::{json, Value};
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Response {
    tasks: Vec<Task>,
}

#[derive(Deserialize, Debug)]
pub struct Task {
    objective: String,
    affected_files: Vec<String>,
    changes: Changes,
}

#[derive(Deserialize, Debug)]
struct Changes {
    code: String,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Response: {} task(s) ===", self.tasks.len())?;
        for (i, task) in self.tasks.iter().enumerate() {
            writeln!(f, "\n--- Task {}/{} ---", i + 1, self.tasks.len())?;
            write!(f, "{}", task)?;
        }
        Ok(())
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Objective
        writeln!(f, "Objective: {}", self.objective)?;
        // Affected files
        writeln!(f, "Affected files:")?;
        for file in &self.affected_files {
            writeln!(f, "    • {}", file)?;
        }
        // Changes block
        writeln!(f, "Changes:")?;
        for line in self.changes.code.lines() {
            writeln!(f, "    {}", line)?;
        }
        Ok(())
    }
}

pub fn res_format() -> Value {
    json!({
        "type": "object",
        "properties": {
            "tasks": {
                "type": "array",
                "description": "A list of tasks.",
                "items": {
                    "type": "object",
                    "properties": {
                        "objective": {
                            "type": "string",
                            "description": "The main objective of the task."
                        },
                        "affected_files": {
                            "type": "array",
                            "description": "A list of file paths that will be affected by this task.",
                            "items": {
                                "type": "string"
                            }
                        },
                        "changes": {
                            "type": "object",
                            "description": "Details about the changes to be made.",
                            "properties": {
                                "code": {
                                    "type": "string",
                                    "description": "Description of the code changes to be implemented."
                                }
                            },
                            "required": ["code"],
                            "additionalProperties": false
                        }
                    },
                    "required": ["objective", "affected_files", "changes", "dependencies"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["tasks"],
        "additionalProperties": false
    })
}
