// LLM Module - Jan.ai Integration
// Uses local Jan.ai server for chat completion

use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

const JAN_AI_DEFAULT_BASE_URL: &str = "http://localhost:1337/v1";
const JAN_AI_DEFAULT_MODEL: &str = "llama2";

#[derive(Debug, Clone)]
pub struct JanClient {
    endpoint: String,
    client: reqwest::Client,
}

impl JanClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        let endpoint = std::env::var("JAN_AI_BASE_URL")
            .unwrap_or_else(|_| JAN_AI_DEFAULT_BASE_URL.to_string());
        Self::new(endpoint)
    }

    /// Get default model from environment or use a sensible default
    pub fn default_model() -> String {
        std::env::var("JAN_AI_MODEL")
            .unwrap_or_else(|_| JAN_AI_DEFAULT_MODEL.to_string())
    }

    /// Check if Jan.ai is available by hitting the health endpoint
    pub async fn is_available(&self) -> bool {
        let base = self.endpoint.trim_end_matches("/v1");
        let health_url = format!("{}/health", base);
        let res = self.client
            .get(health_url)
            .send()
            .await;
        res.is_ok()
    }

    /// List available models from Jan.ai
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let res = self.client
            .get(format!("{}/models", self.endpoint))
            .send()
            .await
            .context("Failed to list models")?;

        if !res.status().is_success() {
            anyhow::bail!("Jan.ai responded with error: {}", res.status());
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

    /// Select a model interactively from available models
    pub async fn select_model_interactively(&self) -> Result<String> {
        println!("📦 Listando modelos disponibles en Jan.ai...\n");
        
        let models = self.list_models().await?;
        
        if models.is_empty() {
            println!("[selfidx] No hay modelos disponibles. Usando modelo predeterminado.");
            return Ok(Self::default_model());
        }

        println!("=== Modelos Disponibles ===\n");
        for (i, model) in models.iter().enumerate() {
            println!("{}. {}", i + 1, model);
        }
        println!("\n0. Usar modelo predeterminado ({})", Self::default_model());
        println!();

        loop {
            use std::io::{self, Write};
            
            print!("Selecciona un modelo (1-{}): ", models.len());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let trimmed = input.trim();
            
            if trimmed == "0" {
                return Ok(Self::default_model());
            }
            
            if let Ok(index) = trimmed.parse::<usize>() {
                if index >= 1 && index <= models.len() {
                    println!("\n✅ Modelo seleccionado: {}\n", models[index - 1]);
                    return Ok(models[index - 1].clone());
                }
            }
            
            println!("\n❌ Selección inválida. Por favor, ingresa un número válido.\n");
        }
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
            anyhow::bail!("Jan.ai error: {}", err);
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
            .expect("No choices in response")
            .message
            .content
            .as_str()
    }
}
