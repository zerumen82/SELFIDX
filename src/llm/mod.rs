// LLM Module - Multi-Provider Support
// Ollama, OpenAI, Anthropic, Groq, LMStudio

pub mod providers;

pub use providers::*;

// Keep existing OllamaClient for backwards compatibility
use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use dirs;

const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434/v1";
const OLLAMA_DEFAULT_MODEL: &str = "llama3"; // Default model for Ollama

#[derive(Debug, Clone)]
pub struct OllamaClient {
    endpoint: String,
    client: reqwest::Client,
}

/// Model type enum to distinguish between local and cloud models
#[derive(Debug, Clone, PartialEq)]
pub enum ModelType {
    Local,  // Modelos locales (llama.cpp, etc.)
    Cloud(String),  // Modelos cloud con proveedor (OpenAI, Anthropic, etc.)
}

impl ModelType {
    /// Get a display string for the model type
    pub fn display(&self) -> String {
        match self {
            ModelType::Local => "LOCAL".to_string(),
            ModelType::Cloud(provider) => format!("CLOUD ({})", provider),
        }
    }
    
    /// Check if it's a cloud model
    pub fn is_cloud(&self) -> bool {
        matches!(self, ModelType::Cloud(_))
    }
}

/// Model info struct to hold model name and type
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
}

impl OllamaClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        let endpoint = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| OLLAMA_DEFAULT_BASE_URL.to_string());
        Self::new(endpoint)
    }

    /// Get default model from environment or try to get from Ollama API
    pub async fn get_default_model() -> String {
        // First try to get from environment
        if let Ok(env_model) = std::env::var("OLLAMA_MODEL") {
            return env_model;
        }
        
        // Try to get a model from Ollama API
        let temp_client = Self::new(OLLAMA_DEFAULT_BASE_URL);
        temp_client.get_default_model_from_api().await
    }
    
    /// Get default model from Ollama API or use fallback
    pub async fn get_default_model_from_api(&self) -> String {
        // Try to get the first available model from the models list
        if let Ok(models) = self.list_models().await {
            if let Some(first_model) = models.first() {
                return first_model.clone();
            }
        }
        
        // Fallback to hardcoded default
        OLLAMA_DEFAULT_MODEL.to_string()
    }
    
    /// Get the currently active/loaded model from Ollama
    pub async fn get_active_model(&self) -> Option<String> {
        // Try to get the first available model as a fallback
        if let Ok(models) = self.list_models().await {
            if let Some(first_model) = models.first() {
                return Some(first_model.clone());
            }
        }
        
        None
    }
    
    /// Get default model with callback support
    pub async fn get_default_model_with_callback(&self) -> String {
        // Try to get the first available model from the models list
        if let Ok(models) = self.list_models().await {
            if let Some(first_model) = models.first() {
                println!("[selfidx] 📦 Primer modelo disponible en Ollama: {}", first_model);
                return first_model.clone();
            }
        }
        
        // Final fallback
        Self::get_default_model().await
    }

    /// Set active model in Ollama
    pub async fn set_active_model(&self, model: &str) -> Result<()> {
        // Verify the model is available
        if !self.is_model_available(model).await {
            anyhow::bail!("El modelo '{}' no está disponible en Ollama", model);
        }
        
        // Ollama loads models automatically when you use them in /chat/completions
        println!("[selfidx] ✅ Modelo '{}' verificado como disponible", model);
        println!("[selfidx] 💡 Ollama cargará el modelo automáticamente al primer uso");
        Ok(())
    }
    
    /// Check if a specific model is available in Ollama
    pub async fn is_model_available(&self, model: &str) -> bool {
        let models = self.list_models().await.unwrap_or_default();
        models.iter().any(|m| m == model)
    }
    
    /// Switch to a different model with validation
    pub async fn switch_model(&self, new_model: &str) -> Result<String> {
        println!("[selfidx] 🔄 Cambiando a modelo '{}'...", new_model);
        
        // Check if model is available
        if !self.is_model_available(new_model).await {
            anyhow::bail!(
                "El modelo '{}' no está disponible en Ollama.\n\
                \n\
                Modelos disponibles:\n{}",
                new_model,
                self.list_models().await.unwrap_or_default()
                    .iter()
                    .map(|m| format!("  - {}", m))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        
        // Ollama loads models automatically when you use them in /chat/completions
        println!("[selfidx] ✅ Modelo '{}' está listo para usar", new_model);
        Ok(new_model.to_string())
    }
    
    /// Get model information from Ollama
    pub async fn get_model_info(&self, model: &str) -> Result<serde_json::Value> {
        // Try to get the model info from the models list
        if let Ok(models) = self.list_models().await {
            if models.contains(&model.to_string()) {
                // Return a simple JSON with the model name
                return Ok(serde_json::json!({
                    "id": model,
                    "name": model,
                    "available": true
                }));
            }
        }
        
        anyhow::bail!("No se pudo obtener información del modelo '{}'", model)
    }

    /// Check if Ollama is available by hitting the health endpoint
    pub async fn is_available(&self) -> bool {
        // Try to check if the models list endpoint is available
        let res = self.client
            .get(format!("{}/models", self.endpoint))
            .send()
            .await;
        res.is_ok()
    }

    /// Load a model in Ollama using the API
    pub async fn load_model(&self, model: &str) -> Result<()> {
        // Verify the model is available
        if !self.is_model_available(model).await {
            anyhow::bail!("El modelo '{}' no está disponible en Ollama", model);
        }
        
        // Ollama loads models automatically when you use them in /chat/completions
        println!("[selfidx] ✅ Modelo '{}' verificado como disponible", model);
        println!("[selfidx] 💡 Ollama cargará el modelo automáticamente cuando lo uses en /chat/completions");
        Ok(())
    }

    /// Wait for model to be ready by making test chat requests
    pub async fn wait_for_model_ready(&self, model: &str, max_retries: u32) -> Result<()> {
        use std::time::Duration;
        
        println!("🔄 Esperando que el modelo '{}' se cargue automáticamente en Ollama...", model);
        println!("💡 Ollama cargará el modelo automáticamente cuando lo uses en /chat/completions");
        
        // Initial wait to give Ollama time to prepare
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Now try the actual request with retries
        for attempt in 1..=max_retries {
            // Try a simple chat request to check if model is ready
            let test_request = ChatRequest {
                model: model.to_string(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "test".to_string(),
                    tool_calls: None,
                }],
                stream: false,
                temperature: Some(0.7),
                max_tokens: Some(10),
                tools: None,
            };
            
            let res = self.client
                .post(format!("{}/chat/completions", self.endpoint))
                .json(&test_request)
                .send()
                .await;
            
            match res {
                Ok(response) if response.status().is_success() => {
                    // Model is ready
                    println!("✅ Modelo '{}' cargado y listo para usar", model);
                    return Ok(());
                }
                Ok(response) => {
                    let err = response.text().await.unwrap_or_default();
                    if err.contains("model not found") || err.contains("not loaded") || err.contains("not running") {
                        // Model not ready yet, wait and retry
                        if attempt < max_retries {
                            let wait_time = 2 + (attempt - 1) * 2;
                            println!("⏳ Esperando que el modelo se cargue... (intento {}/{})", attempt, max_retries);
                            tokio::time::sleep(Duration::from_secs(wait_time as u64)).await;
                            continue;
                        }
                    }
                    // For other errors or max retries reached, return the error
                    anyhow::bail!("Ollama error: {}", err);
                }
                Err(e) => {
                    // Network error, retry
                    if attempt < max_retries {
                        let wait_time = 2 + (attempt - 1) * 2;
                        println!("⏳ Esperando conexión... (intento {}/{} - esperando {}s)", attempt, max_retries, wait_time);
                        tokio::time::sleep(Duration::from_secs(wait_time as u64)).await;
                        continue;
                    }
                    anyhow::bail!("Failed to connect to Ollama: {}", e);
                }
            }
        }
        
        anyhow::bail!(
            "El modelo '{}' no se cargó después de {} intentos.\n\
            Por favor, verifica que el modelo esté instalado en Ollama.",
            model, max_retries
        )
    }

    /// List available models from Ollama with their type (local/cloud)
    pub async fn list_models_with_type(&self) -> Result<Vec<ModelInfo>> {
        // First, try to list local models from filesystem
        let mut local_names = std::collections::HashSet::new();
        if let Ok(local_models) = list_local_models() {
            for model in &local_models {
                local_names.insert(model.clone());
            }
        }
        
        // If no local models found, try API endpoints
        let base = self.endpoint.trim_end_matches("/v1");
        
        // Try multiple endpoints to find models
        let endpoints = vec![
            format!("{}/models", self.endpoint),           // OpenAI standard
            format!("{}/api/tags", base),                  // Ollama local
            format!("{}/v1/models", base),                 // Alternative
        ];
        
        for url in endpoints {
            if let Ok(res) = self.client.get(&url).send().await {
                if res.status().is_success() {
                    if let Ok(json) = res.json::<serde_json::Value>().await {
                        // Try different response formats
                        let mut models: Vec<ModelInfo> = Vec::new();
                        
                        if let Some(data) = json["data"].as_array() {
                            // Format: {"data": [{"id": "model1", "source": "local"}, ...]}
                            for m in data {
                                let name = m["id"].as_str().map(|s| s.to_string());
                                if let Some(model_name) = name {
                                    // Determine if local or cloud based on source or name
                                    let source = m["source"].as_str().unwrap_or("remote");
                                    let model_type = if source == "local" || local_names.contains(&model_name) {
                                        ModelType::Local
                                    } else {
                                        ModelType::Cloud("unknown".to_string())
                                    };
                                    models.push(ModelInfo {
                                        name: model_name,
                                        model_type,
                                    });
                                }
                            }
                        } else if let Some(models_arr) = json["models"].as_array() {
                            // Format: {"models": [{"name": "model1", "source": "local"}, ...]}
                            for m in models_arr {
                                let name = m["name"].as_str().map(|s| s.to_string());
                                if let Some(model_name) = name {
                                    let source = m["source"].as_str().unwrap_or("remote");
                                    let model_type = if source == "local" || local_names.contains(&model_name) {
                                        ModelType::Local
                                    } else {
                                        ModelType::Cloud("unknown".to_string())
                                    };
                                    models.push(ModelInfo {
                                        name: model_name,
                                        model_type,
                                    });
                                }
                            }
                        } else if let Some(obj) = json.as_object() {
                            // Format: {"model1": {...}, "model2": {...}}
                            for (key, value) in obj {
                                let source = value.get("source").and_then(|s| s.as_str()).unwrap_or("remote");
                                let model_type = if source == "local" || local_names.contains(key) {
                                    ModelType::Local
                                } else {
                                    ModelType::Cloud("unknown".to_string())
                                };
                                models.push(ModelInfo {
                                    name: key.clone(),
                                    model_type,
                                });
                            }
                        }
                        
                        if !models.is_empty() {
                            return Ok(models);
                        }
                    }
                }
            }
        }
        
        // Fallback: return local models from filesystem
        if !local_names.is_empty() {
            return Ok(local_names.into_iter().map(|name| ModelInfo {
                name,
                model_type: ModelType::Local,
            }).collect());
        }
        
        Ok(vec![])
    }
    
    /// List available models from Ollama (legacy function for compatibility)
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let models_with_type = self.list_models_with_type().await?;
        Ok(models_with_type.into_iter().map(|m| m.name).collect())
    }
    
    /// Get the currently active/loaded model in Ollama (by checking which model responds)
    pub async fn get_currently_loaded_model(&self) -> Option<String> {
        // Try to find which model is currently loaded by testing each model
        // Start with the first available model
        if let Ok(models) = self.list_models().await {
            for model in models {
                // Try a simple request to see if this model is loaded
                let test_request = ChatRequest {
                    model: model.clone(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: "test".to_string(),
                        tool_calls: None,
                    }],
                    stream: false,
                    temperature: Some(0.7),
                    max_tokens: Some(5),
                    tools: None,
                };
                
                if let Ok(res) = self.client
                    .post(format!("{}/chat/completions", self.endpoint))
                    .json(&test_request)
                    .send()
                    .await
                {
                    if res.status().is_success() {
                        return Some(model);
                    }
                }
            }
        }
        None
    }

    /// Select a model interactively from available models with local/cloud labels
    pub async fn select_model_interactively(&self) -> Result<String> {
        println!("📦 Listando modelos disponibles en Ollama...\n");
        
        let models = self.list_models_with_type().await?;
        
        if models.is_empty() {
            println!("[selfidx] No hay modelos disponibles. Usando modelo predeterminado.");
            return Ok(Self::get_default_model().await);
        }

        // Get the currently loaded model (if any)
        let loaded_model = self.get_currently_loaded_model().await;
        println!("📦 Modelo actualmente cargado: {:?}\n", loaded_model);

        println!("=== Modelos Disponibles ===\n");
        
        for (i, model_info) in models.iter().enumerate() {
            let type_label = match &model_info.model_type {
                ModelType::Local => "🟢 LOCAL".to_string(),
                ModelType::Cloud(provider) => format!("🔵 CLOUD ({})", provider),
            };
            // Mark the currently loaded model
            let loaded_marker = if loaded_model.as_ref() == Some(&model_info.name) {
                " ⭐ (ACTIVO)"
            } else {
                ""
            };
            println!("{}. {} [{}]{}", i + 1, model_info.name, type_label, loaded_marker);
        }
        println!("\n0. Usar modelo predeterminado ({})", Self::get_default_model().await);
        println!();

        loop {
            use std::io::{self, Write};
            
            print!("Selecciona un modelo (1-{}): ", models.len());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let trimmed = input.trim();
            
            if trimmed == "0" {
                return Ok(Self::get_default_model().await);
            }
            
            if let Ok(index) = trimmed.parse::<usize>() {
                if index >= 1 && index <= models.len() {
                    let selected_model = &models[index - 1];
                    
                    // Cloud model or same local model - can use directly
                    let type_str = match selected_model.model_type {
                        ModelType::Local => "LOCAL",
                        ModelType::Cloud(_) => "CLOUD",
                    };
                    println!("\n✅ Modelo seleccionado: {} [{}]\n", selected_model.name, type_str);
                    
                    return Ok(selected_model.name.clone());
                }
            }
            
            println!("\n❌ Selección inválida. Por favor, ingresa un número válido.\n");
        }
    }

    /// Chat completion
    pub async fn chat(&self, model: String, messages: Vec<Message>) -> Result<ChatResponse> {
        self.chat_with_tools(model, messages, None).await
    }

    /// Chat completion with tools support
    pub async fn chat_with_tools(
        &self,
        model: String,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse> {
        // Verify the model is available (silent check)
        if !self.is_model_available(&model).await {
            anyhow::bail!("El modelo '{}' no está disponible en Ollama", model);
        }
        
        let request = ChatRequest {
            model: model.clone(),
            messages,
            stream: false,
            temperature: Some(0.7),
            max_tokens: Some(2048),
            tools,
        };

        let res = self.client
            .post(format!("{}/chat/completions", self.endpoint))
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            
            // Provide helpful error messages for common issues
            if err.contains("model not found") {
                anyhow::bail!(
                    "El modelo '{}' no fue encontrado en Ollama.\n\
                    \n\
                    Solución:\n\
                    1. Verifica que el modelo esté instalado en Ollama\n\
                    2. Revisa la lista de modelos disponibles con: selfidx --chat\n\
                    3. Instala el modelo con: ollama pull {}",
                    model, model
                );
            } else {
                anyhow::bail!("Ollama error: {}", err);
            }
        } else {
            let response: ChatResponse = res.json().await?;
            Ok(response)
        }
    }

    /// Convert agent tools to Ollama format
    pub fn convert_agent_tools_to_ollama(agent_tools: Vec<crate::agent::Tool>) -> Vec<Tool> {
        agent_tools
            .into_iter()
            .map(|t| Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                },
            })
            .collect()
    }
}

/// List local models from Jan.ai/Ollama/HuggingFace directories
pub fn list_local_models() -> Result<Vec<String>> {
    let mut models = Vec::new();
    
    // Common model directories
    let possible_dirs = vec![
        // Jan.ai directories (actual data directory)
        dirs::data_dir().map(|d| d.join("Jan").join("data").join("llamacpp").join("models")),
        dirs::data_local_dir().map(|d| d.join("Jan").join("data").join("llamacpp").join("models")),
        // Legacy Jan.ai directories
        dirs::home_dir().map(|h| h.join(".jan").join("models")),
        dirs::data_local_dir().map(|d| d.join("jan").join("models")),
        dirs::data_dir().map(|d| d.join("jan").join("models")),
        // Ollama directories
        dirs::home_dir().map(|h| h.join(".ollama").join("models")),
        dirs::data_local_dir().map(|d| d.join("ollama").join("models")),
        dirs::data_dir().map(|d| d.join("ollama").join("models")),
        // HuggingFace cache directories
        dirs::home_dir().map(|h| h.join(".cache").join("huggingface").join("hub")),
        dirs::data_local_dir().map(|d| d.join("huggingface").join("hub")),
        dirs::cache_dir().map(|d| d.join("huggingface").join("hub")),
    ];
    
    for dir_opt in possible_dirs {
        if let Some(models_dir) = dir_opt {
            if models_dir.exists() {
                // Recursively search for models in subdirectories
                fn search_dir(dir: &std::path::Path, models: &mut Vec<String>, depth: usize) {
                    if depth > 3 {
                        return; // Limit recursion depth
                    }
                    
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if path.is_dir() {
                                // Check if this directory contains a model file
                                let has_model = path.join("model.gguf").exists() ||
                                               path.join("model.bin").exists() ||
                                               path.join("pytorch_model.bin").exists() ||
                                               path.join("model.safetensors").exists();
                                
                                if has_model {
                                    // This is a model directory
                                    if let Some(name) = path.file_name() {
                                        let name_str = name.to_string_lossy().to_string();
                                        // Filter out non-model directories
                                        if !name_str.starts_with('.') &&
                                           !name_str.starts_with('_') &&
                                           name_str != "blobs" &&
                                           name_str != "manifests" &&
                                           name_str != "registry" &&
                                           !name_str.starts_with("models--") {
                                            models.push(name_str);
                                        }
                                    }
                                } else {
                                    // Recurse into subdirectory (e.g., janhq/, ollama/, etc.)
                                    search_dir(&path, models, depth + 1);
                                }
                            } else if path.is_file() {
                                // Check for GGUF files at root level
                                if let Some(ext) = path.extension() {
                                    if ext == "gguf" {
                                        if let Some(name) = path.file_stem() {
                                            models.push(name.to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                search_dir(&models_dir, &mut models, 0);
            }
        }
    }
    
    // Remove duplicates and sort
    models.sort();
    models.dedup();
    
    Ok(models)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
    #[serde(default)]
    pub index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
    #[serde(default)]
    pub index: Option<u32>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
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

    /// Get tool calls from the response (checks both choice.tool_calls and message.tool_calls)
    pub fn tool_calls(&self) -> Option<Vec<ToolCall>> {
        // First check choice-level tool_calls
        if let Some(tool_calls) = self.choices.first().and_then(|c| c.tool_calls.as_ref()) {
            return Some(tool_calls.clone());
        }
        // Then check message-level tool_calls
        self.choices.first().and_then(|c| c.message.tool_calls.clone())
    }

    /// Check if response has tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls().is_some()
    }
}
