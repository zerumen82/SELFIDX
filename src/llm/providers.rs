// LLM Providers - SELFIDEX v3.0
// Soporte multi-proveedor: Ollama, OpenAI, Anthropic, etc.

use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

/// Proveedores de LLM soportados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "lmstudio")]
    LMStudio,
    #[serde(rename = "grok")]
    Grok,        // xAI
    #[serde(rename = "gemini")]
    Gemini,      // Google
    #[serde(rename = "openrouter")]
    OpenRouter,  // OpenRouter (acceso a múltiples modelos)
    #[serde(rename = "deepseek")]
    DeepSeek,    // DeepSeek AI
    #[serde(rename = "cohere")]
    Cohere,      // Cohere
    #[serde(rename = "mistral")]
    Mistral,     // Mistral AI
    #[serde(rename = "perplexity")]
    Perplexity,  // Perplexity AI
    #[serde(rename = "together")]
    Together,    // Together AI
}

impl LlmProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmProvider::Ollama => "ollama",
            LlmProvider::OpenAI => "openai",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Groq => "groq",
            LlmProvider::LMStudio => "lmstudio",
            LlmProvider::Grok => "grok",
            LlmProvider::Gemini => "gemini",
            LlmProvider::OpenRouter => "openrouter",
            LlmProvider::DeepSeek => "deepseek",
            LlmProvider::Cohere => "cohere",
            LlmProvider::Mistral => "mistral",
            LlmProvider::Perplexity => "perplexity",
            LlmProvider::Together => "together",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ollama" => Some(LlmProvider::Ollama),
            "openai" | "openai.com" => Some(LlmProvider::OpenAI),
            "anthropic" | "anthropic.com" => Some(LlmProvider::Anthropic),
            "groq" => Some(LlmProvider::Groq),
            "lmstudio" | "lm studio" => Some(LlmProvider::LMStudio),
            "grok" | "xai" | "x.ai" => Some(LlmProvider::Grok),
            "gemini" | "google" | "google.ai" => Some(LlmProvider::Gemini),
            "openrouter" | "openrouter.ai" => Some(LlmProvider::OpenRouter),
            "deepseek" | "deepseek.ai" => Some(LlmProvider::DeepSeek),
            "cohere" => Some(LlmProvider::Cohere),
            "mistral" | "mistral.ai" => Some(LlmProvider::Mistral),
            "perplexity" | "perplexity.ai" => Some(LlmProvider::Perplexity),
            "together" | "together.ai" => Some(LlmProvider::Together),
            _ => None,
        }
    }

    /// Endpoint por defecto del proveedor
    pub fn default_endpoint(&self) -> &'static str {
        match self {
            LlmProvider::Ollama => "http://localhost:11434/v1",
            LlmProvider::OpenAI => "https://api.openai.com/v1",
            LlmProvider::Anthropic => "https://api.anthropic.com/v1",
            LlmProvider::Groq => "https://api.groq.com/openai/v1",
            LlmProvider::LMStudio => "http://localhost:1234/v1",
            LlmProvider::Grok => "https://api.x.ai/v1",
            LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
            LlmProvider::OpenRouter => "https://openrouter.ai/api/v1",
            LlmProvider::DeepSeek => "https://api.deepseek.com/v1",
            LlmProvider::Cohere => "https://api.cohere.ai/v1",
            LlmProvider::Mistral => "https://api.mistral.ai/v1",
            LlmProvider::Perplexity => "https://api.perplexity.ai",
            LlmProvider::Together => "https://api.together.xyz/v1",
        }
    }

    /// Modelos populares del proveedor
    pub fn popular_models(&self) -> Vec<&'static str> {
        match self {
            LlmProvider::Ollama => vec![
                "llama3", "llama3:70b", "mistral", "codellama", "phi3", "gemma2", "qwen2.5", "deepcoder"
            ],
            LlmProvider::OpenAI => vec![
                "gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo", "o1-preview", "o1-mini"
            ],
            LlmProvider::Anthropic => vec![
                "claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022", "claude-3-opus-20240229", "claude-3-haiku-20240307"
            ],
            LlmProvider::Groq => vec![
                "llama-3.1-70b-versatile", "llama-3.1-8b-instant", "mixtral-8x7b-32768", "gemma2-9b-it"
            ],
            LlmProvider::LMStudio => vec!["local-model"],
            LlmProvider::Grok => vec![
                "grok-beta", "grok-vision-beta", "grok-2"
            ],
            LlmProvider::Gemini => vec![
                "gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash", "gemini-pro"
            ],
            LlmProvider::OpenRouter => vec![
                "openai/gpt-4o", "anthropic/claude-3.5-sonnet", "google/gemini-pro-1.5", "meta-llama/llama-3-70b-instruct"
            ],
            LlmProvider::DeepSeek => vec![
                "deepseek-chat", "deepseek-coder", "deepseek-v2.5"
            ],
            LlmProvider::Cohere => vec![
                "command-r-plus", "command-r", "command", "command-light"
            ],
            LlmProvider::Mistral => vec![
                "mistral-large-latest", "mistral-small-latest", "codestral-latest", "mistral-nemo"
            ],
            LlmProvider::Perplexity => vec![
                "sonar-pro", "sonar", "sonar-reasoning-pro"
            ],
            LlmProvider::Together => vec![
                "meta-llama/Llama-3.1-70B-Instruct-Turbo", "mistralai/Mixtral-8x7B-Instruct-v0.1"
            ],
        }
    }

    /// Requiere API key
    pub fn requires_api_key(&self) -> bool {
        match self {
            LlmProvider::Ollama | LlmProvider::LMStudio => false,
            LlmProvider::OpenAI | LlmProvider::Anthropic | LlmProvider::Groq |
            LlmProvider::Grok | LlmProvider::Gemini | LlmProvider::OpenRouter |
            LlmProvider::DeepSeek | LlmProvider::Cohere | LlmProvider::Mistral |
            LlmProvider::Perplexity | LlmProvider::Together => true,
        }
    }

    /// Soporta function calling / tools nativo
    pub fn supports_tools(&self) -> bool {
        match self {
            LlmProvider::Ollama | LlmProvider::OpenAI | LlmProvider::Anthropic |
            LlmProvider::Groq | LlmProvider::Gemini | LlmProvider::OpenRouter |
            LlmProvider::DeepSeek | LlmProvider::Mistral | LlmProvider::Together => true,
            LlmProvider::LMStudio | LlmProvider::Grok | LlmProvider::Cohere |
            LlmProvider::Perplexity => false,
        }
    }

    /// Soporta visión (imágenes)
    pub fn supports_vision(&self) -> bool {
        match self {
            LlmProvider::OpenAI | LlmProvider::Anthropic | LlmProvider::Gemini |
            LlmProvider::Grok | LlmProvider::OpenRouter => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuración de proveedor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub default_model: String,
    pub enabled: bool,
}

impl ProviderConfig {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            endpoint: provider.default_endpoint().to_string(),
            api_key: None,
            default_model: provider.popular_models().first().unwrap_or(&"default").to_string(),
            enabled: true,
        }
    }

    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.default_model = model.to_string();
        self
    }
}

/// Cliente LLM unificado
pub struct LlmClient {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub api_key: Option<String>,
    client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            provider: config.provider,
            endpoint: config.endpoint.clone(),
            api_key: config.api_key.clone(),
            client: reqwest::Client::new(),
        }
    }

    /// Crear cliente desde variables de entorno
    pub fn from_env() -> Self {
        // Detectar proveedor desde env o usar Ollama por defecto
        let provider_str = std::env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "ollama".to_string());
        
        let provider = LlmProvider::from_str(&provider_str)
            .unwrap_or(LlmProvider::Ollama);

        let endpoint = std::env::var("LLM_ENDPOINT")
            .unwrap_or_else(|_| provider.default_endpoint().to_string());

        let api_key = std::env::var("LLM_API_KEY").ok();

        let default_model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| provider.popular_models()[0].to_string());

        println!("[selfidx-llm] Proveedor: {}", provider);
        println!("[selfidx-llm] Endpoint: {}", endpoint);
        println!("[selfidx-llm] Modelo: {}", default_model);

        Self {
            provider,
            endpoint,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Crear cliente para proveedor específico
    pub fn from_provider(provider: LlmProvider, api_key: Option<String>) -> Self {
        Self {
            provider,
            endpoint: provider.default_endpoint().to_string(),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Verificar conectividad
    pub async fn is_available(&self) -> bool {
        // Todos los proveedores soportan endpoint /models
        let url = format!("{}/models", self.endpoint);

        let mut request = self.client.get(&url);

        if let Some(key) = &self.api_key {
            request = match self.provider {
                LlmProvider::OpenAI | LlmProvider::Groq | LlmProvider::DeepSeek |
                LlmProvider::Mistral | LlmProvider::Together | LlmProvider::OpenRouter |
                LlmProvider::Perplexity | LlmProvider::Grok => {
                    request.header("Authorization", format!("Bearer {}", key))
                }
                LlmProvider::Anthropic => {
                    request
                        .header("x-api-key", key)
                        .header("anthropic-version", "2023-06-01")
                }
                LlmProvider::Gemini => {
                    request.header("x-goog-api-key", key)
                }
                LlmProvider::Cohere => {
                    request.header("Authorization", format!("Bearer {}", key))
                }
                _ => request,
            };
        }

        request.send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Listar modelos disponibles
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.endpoint);
        
        let mut request = self.client.get(&url);
        
        if let Some(key) = &self.api_key {
            request = match self.provider {
                LlmProvider::OpenAI | LlmProvider::Groq => {
                    request.header("Authorization", format!("Bearer {}", key))
                }
                LlmProvider::Anthropic => {
                    request
                        .header("x-api-key", key)
                        .header("anthropic-version", "2023-06-01")
                }
                _ => request,
            };
        }

        let response = request
            .send()
            .await
            .context("Failed to fetch models")?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            anyhow::bail!("Error al listar modelos: {}", error);
        }

        let json: serde_json::Value = response.json().await?;

        // Parsear respuesta según proveedor
        // Todos los proveedores modernos usan formato OpenAI-compatible: {"data": [{"id": "model1"}, ...]}
        let models = json["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    /// Obtener modelo por defecto
    pub fn get_default_model(&self) -> &str {
        &self.provider.popular_models()[0]
    }

    /// Obtener información del proveedor
    pub fn get_provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.provider,
            endpoint: self.endpoint.clone(),
            has_api_key: self.api_key.is_some(),
            default_models: self.provider.popular_models().iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Cambiar endpoint
    pub fn set_endpoint(&mut self, endpoint: &str) {
        self.endpoint = endpoint.to_string();
    }

    /// Cambiar API key
    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_string());
    }

    /// Obtener proveedor actual
    pub fn get_provider(&self) -> LlmProvider {
        self.provider
    }
}

/// Información del proveedor
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub has_api_key: bool,
    pub default_models: Vec<String>,
}

impl ProviderInfo {
    pub fn display(&self) -> String {
        format!(
            "Proveedor: {}\nEndpoint: {}\nAPI Key: {}\nModelos: {}",
            self.provider,
            self.endpoint,
            if self.has_api_key { "✓ Configurada" } else { "✗ No configurada" },
            self.default_models.join(", ")
        )
    }
}

/// Mensaje para chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Llamada a herramienta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Herramienta para function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Respuesta de chat
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: i32,
    #[serde(default)]
    pub completion_tokens: i32,
    #[serde(default)]
    pub total_tokens: i32,
}

impl ChatResponse {
    pub fn content(&self) -> &str {
        &self.choices[0].message.content
    }

    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        self.choices[0].message.tool_calls.as_ref()
    }

    pub fn usage_display(&self) -> String {
        // No mostramos tokens por defecto - el usuario no quiere preocuparse por eso
        // Solo mostrar si se solicita explícitamente
        String::new()
    }
}

/// Request de chat
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

impl LlmClient {
    /// Chat completion con tools
    pub async fn chat_with_tools(
        &self,
        model: String,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse> {
        // Solo enviar tools si el proveedor los soporta
        let tools_to_send = if self.provider.supports_tools() {
            tools
        } else {
            // Si no soporta tools, enviar sin ellos
            // El agente deberá usar formato de texto plano
            None
        };

        let request = ChatRequest {
            model: model.clone(),
            messages,
            stream: false,
            temperature: Some(0.7),
            max_tokens: Some(4096), // Más alto para respuestas completas
            tools: tools_to_send,
        };

        let url = format!("{}/chat/completions", self.endpoint);

        let mut req_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json");

        // Agregar headers específicos por proveedor
        if let Some(key) = &self.api_key {
            req_builder = match self.provider {
                LlmProvider::OpenAI | LlmProvider::Groq | LlmProvider::DeepSeek |
                LlmProvider::Mistral | LlmProvider::Together | LlmProvider::OpenRouter => {
                    req_builder.header("Authorization", format!("Bearer {}", key))
                }
                LlmProvider::Anthropic => {
                    req_builder
                        .header("x-api-key", key)
                        .header("anthropic-version", "2023-06-01")
                }
                LlmProvider::Cohere => {
                    req_builder.header("Authorization", format!("Bearer {}", key))
                }
                LlmProvider::Gemini => {
                    req_builder.header("x-goog-api-key", key)
                }
                LlmProvider::Perplexity => {
                    req_builder.header("Authorization", format!("Bearer {}", key))
                }
                LlmProvider::Grok => {
                    req_builder.header("Authorization", format!("Bearer {}", key))
                }
                _ => req_builder,
            };
        }

        let response = req_builder
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();

            // Mensajes de error específicos
            if error.contains("api_key") || error.contains("API key") {
                anyhow::bail!(
                    "API key inválida o faltante para {}\n\
                    Configura con: selfidx provider set {} --api-key TU_KEY",
                    self.provider,
                    self.provider
                );
            }

            anyhow::bail!("Error del proveedor ({}): {}", self.provider, error);
        }

        let chat_response: ChatResponse = response.json().await?;

        Ok(chat_response)
    }

    /// Chat simple sin tools
    pub async fn chat(&self, model: String, messages: Vec<Message>) -> Result<ChatResponse> {
        self.chat_with_tools(model, messages, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_str() {
        assert_eq!(LlmProvider::from_str("ollama"), Some(LlmProvider::Ollama));
        assert_eq!(LlmProvider::from_str("OpenAI"), Some(LlmProvider::OpenAI));
        assert_eq!(LlmProvider::from_str("anthropic"), Some(LlmProvider::Anthropic));
        assert_eq!(LlmProvider::from_str("groq"), Some(LlmProvider::Groq));
        assert_eq!(LlmProvider::from_str("invalid"), None);
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(LlmProvider::Ollama.to_string(), "ollama");
        assert_eq!(LlmProvider::OpenAI.to_string(), "openai");
    }

    #[test]
    fn test_provider_requires_api_key() {
        assert!(!LlmProvider::Ollama.requires_api_key());
        assert!(!LlmProvider::LMStudio.requires_api_key());
        assert!(LlmProvider::OpenAI.requires_api_key());
        assert!(LlmProvider::Anthropic.requires_api_key());
    }

    #[test]
    fn test_provider_popular_models() {
        let ollama_models = LlmProvider::Ollama.popular_models();
        assert!(ollama_models.contains(&"llama3"));
        assert!(ollama_models.contains(&"mistral"));

        let openai_models = LlmProvider::OpenAI.popular_models();
        assert!(openai_models.contains(&"gpt-4o"));
    }

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new(LlmProvider::Ollama);
        assert_eq!(config.provider, LlmProvider::Ollama);
        assert_eq!(config.endpoint, "http://localhost:11434/v1");
        assert!(config.api_key.is_none());
    }
}
