// LLM Module - vLLM Integration

use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

const VLLM_DEFAULT_PORT: u16 = 8000;

#[derive(Debug, Clone)]
pub struct VllmClient {
    endpoint: String,
    client: reqwest::Client,
}

impl VllmClient {
    pub fn new(port: u16) -> Self {
        let endpoint = format!("http://localhost:{}/v1", port);
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        let port = std::env::var("VLLM_PORT")
            .unwrap_or_else(|_| VLLM_DEFAULT_PORT.to_string())
            .parse()
            .unwrap_or(VLLM_DEFAULT_PORT);
        Self::new(port)
    }

    /// Get default model from environment or use a sensible default
    pub fn default_model() -> String {
        std::env::var("VLLM_MODEL")
            .unwrap_or_else(|_| "llama2".to_string())
    }

    /// Check if vLLM is available
    pub async fn is_available(&self) -> bool {
        // vLLM health endpoint is at /health, not /v1/health
        let port = std::env::var("VLLM_PORT")
            .unwrap_or_else(|_| VLLM_DEFAULT_PORT.to_string())
            .parse()
            .unwrap_or(VLLM_DEFAULT_PORT);
        let health_url = format!("http://localhost:{}/health", port);
        let res = self.client
            .get(health_url)
            .send()
            .await;
        res.is_ok()
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let res = self.client
            .get(format!("{}/models", self.endpoint.replace("/v1", "")))
            .send()
            .await
            .context("Failed to list models")?;

        if !res.status().is_success() {
            anyhow::bail!("vLLM responded with error: {}", res.status());
        }

        let json: serde_json::Value = res.json().await?;
        let models = json["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }

    /// Chat completion
    pub async fn chat(&self, model: String, messages: Vec<Message>) -> Result<ChatResponse> {
        let request = ChatRequest {
            model,
            messages,
            stream: false,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        };

        let res = self.client
            .post(format!("{}/chat/completions", self.endpoint))
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            anyhow::bail!("vLLM error: {}", err);
        }

        let response: ChatResponse = res.json().await?;
        Ok(response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl ChatResponse {
    pub fn content(&self) -> &str {
        self.choices
            .first()
            .map(|c| c.message.content.as_str())
            .unwrap_or("")
    }
}
