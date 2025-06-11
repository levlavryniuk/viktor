use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::error::OllamaError;
use crate::types::{ChatRequest, ChatResponse, GenerateRequest, GenerateResponse};

#[derive(Clone)]
pub struct OllamaClient {
    base: Url,
    http: HttpClient,
}

impl OllamaClient {
    /// Connect to an existing server, e.g. `"http://localhost:11434"`.
    pub fn new(base_url: &str) -> Result<Self, OllamaError> {
        let base = Url::parse(base_url)?;
        let http = HttpClient::builder().build()?;
        Ok(Self { base, http })
    }

    /// Build `/api/<path>` URL.
    fn api_path(&self, path: &str) -> Result<Url, OllamaError> {
        Ok(self.base.join(&format!("/api/{}", path))?)
    }

    /// Generic POST → typed JSON.
    async fn post_json<Q: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        q: &Q,
    ) -> Result<R, OllamaError> {
        let url = self.api_path(endpoint)?;
        let resp = self.http.post(url).json(q).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            Ok(serde_json::from_str(&body)?)
        } else {
            Err(OllamaError::ServerError { status, body })
        }
    }

    /// POST /api/generate (non-streaming)
    pub async fn generate(&self, req: &GenerateRequest) -> Result<GenerateResponse, OllamaError> {
        self.post_json("generate", req).await
    }

    /// POST /api/chat (non-streaming)
    pub async fn chat(&self, req: &ChatRequest) -> Result<ChatResponse, OllamaError> {
        self.post_json("chat", req).await
    }
}
