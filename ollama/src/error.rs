use reqwest::StatusCode;
use std::io;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse URL: {0}")]
    Url(#[from] ParseError),

    #[error("JSON (de)serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("unexpected server response [{status}]: {body}")]
    ServerError { status: StatusCode, body: String },
}
