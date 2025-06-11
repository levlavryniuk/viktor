//! A thin Rust wrapper around the `ollama` CLI.
//!
//! Provides methods for every Ollama command by spawning the
//! `ollama` binary under the hood and parsing JSON output where
//! appropriate.

mod client;
mod error;
pub mod types;

pub use client::OllamaClient;
pub use error::OllamaError;
