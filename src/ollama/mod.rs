// Ollama Management Module - SELFIDEX v3.0
// Gestión de servidor Ollama y modelos

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use reqwest;

/// Información del servidor Ollama
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaServerInfo {
    pub is_running: bool,
    pub url: String,
    pub version: Option<String>,
}

/// Información de un modelo Ollama
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: String,
    pub digest: String,
    pub modified_at: String,
    pub details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

/// Gestor de Ollama
pub struct OllamaManager {
    pub base_url: String,
    client: reqwest::Client,
}

impl OllamaManager {
    /// Crear nuevo gestor
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Crear desde variables de entorno
    pub fn from_env() -> Self {
        let url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self::new(&url)
    }

    /// Verificar si Ollama está corriendo
    pub async fn is_running(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Obtener información del servidor
    pub async fn get_server_info(&self) -> Result<OllamaServerInfo> {
        let is_running = self.is_running().await;
        
        let version = if is_running {
            // Intentar obtener versión
            let url = format!("{}/api/version", self.base_url);
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        response.json::<serde_json::Value>().await
                            .ok()
                            .and_then(|v| v["version"].as_str().map(String::from))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(OllamaServerInfo {
            is_running,
            url: self.base_url.clone(),
            version,
        })
    }

    /// Listar modelos disponibles
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client.get(&url).send().await
            .context("Error al conectar con Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Error al listar modelos: {}", response.status());
        }

        let json: serde_json::Value = response.json().await?;
        
        let mut models = Vec::new();
        
        if let Some(models_array) = json["models"].as_array() {
            for model_json in models_array {
                if let Some(name) = model_json["name"].as_str() {
                    models.push(OllamaModel {
                        name: name.to_string(),
                        size: format_bytes(model_json["size"].as_u64().unwrap_or(0)),
                        digest: model_json["digest"].as_str().unwrap_or("").to_string(),
                        modified_at: model_json["modified_at"].as_str().unwrap_or("").to_string(),
                        details: ModelDetails {
                            parent_model: model_json["details"]["parent_model"].as_str().unwrap_or("").to_string(),
                            format: model_json["details"]["format"].as_str().unwrap_or("").to_string(),
                            family: model_json["details"]["family"].as_str().unwrap_or("").to_string(),
                            families: model_json["details"]["families"]
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                            parameter_size: model_json["details"]["parameter_size"].as_str().unwrap_or("").to_string(),
                            quantization_level: model_json["details"]["quantization_level"].as_str().unwrap_or("").to_string(),
                        },
                    });
                }
            }
        }

        Ok(models)
    }

    /// Descargar modelo
    pub async fn pull_model(&self, name: &str) -> Result<String> {
        let url = format!("{}/api/pull", self.base_url);
        
        let body = serde_json::json!({
            "name": name,
            "stream": false
        });

        let response = self.client.post(&url).json(&body).send().await
            .context("Error al descargar modelo")?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            anyhow::bail!("Error al descargar {}: {}", name, error);
        }

        Ok(format!("✓ Modelo {} descargado exitosamente", name))
    }

    /// Eliminar modelo
    pub async fn delete_model(&self, name: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.base_url);
        
        let body = serde_json::json!({
            "name": name
        });

        let response = self.client.delete(&url).json(&body).send().await
            .context("Error al eliminar modelo")?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            anyhow::bail!("Error al eliminar {}: {}", name, error);
        }

        Ok(())
    }

    /// Obtener información de un modelo
    pub async fn show_model(&self, name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/show", self.base_url);
        
        let body = serde_json::json!({
            "name": name
        });

        let response = self.client.post(&url).json(&body).send().await
            .context("Error al obtener información del modelo")?;

        if !response.status().is_success() {
            anyhow::bail!("Error al obtener información de {}", name);
        }

        response.json().await.context("Error al parsear respuesta")
    }

    /// Verificar si un modelo existe
    pub async fn model_exists(&self, name: &str) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.name == name || m.name.starts_with(&format!("{}:", name))),
            Err(_) => false,
        }
    }

    /// Obtener o descargar modelo
    pub async fn get_or_pull_model(&self, name: &str) -> Result<String> {
        if self.model_exists(name).await {
            Ok(format!("✓ Modelo {} ya está disponible", name))
        } else {
            self.pull_model(name).await
        }
    }

    /// Iniciar servidor Ollama (si está instalado)
    pub fn start_server() -> Result<()> {
        #[cfg(windows)]
        {
            Command::new("ollama")
                .arg("serve")
                .spawn()
                .context("Error al iniciar Ollama server")?;
        }

        #[cfg(not(windows))]
        {
            Command::new("ollama")
                .arg("serve")
                .spawn()
                .context("Error al iniciar Ollama server")?;
        }

        Ok(())
    }

    /// Obtener modelos recomendados para coding
    pub fn get_recommended_models() -> Vec<&'static str> {
        vec![
            "codellama",      // Especializado en código
            "codellama:7b",   // Ligero
            "codellama:13b",  // Balanceado
            "codellama:34b",  // Potente
            "llama3",         // Propósito general
            "llama3:70b",     // Máxima calidad
            "mistral",        // Rápido y bueno
            "deepseek-coder", // Especializado código
            "qwen2.5-coder",  // Especializado código
            "phi3",           // Ligero Microsoft
        ]
    }

    /// Obtener modelo recomendado por hardware
    pub fn recommend_model_for_hardware() -> &'static str {
        // Simplificado - en producción usar detección real de hardware
        let ram_gb = get_total_ram_gb();
        
        if ram_gb >= 64.0 {
            "llama3:70b"  // 70B con 64GB+ RAM
        } else if ram_gb >= 32.0 {
            "codellama:34b"  // 34B con 32GB RAM
        } else if ram_gb >= 16.0 {
            "codellama:13b"  // 13B con 16GB RAM
        } else if ram_gb >= 8.0 {
            "codellama:7b"  // 7B con 8GB RAM
        } else {
            "phi3"  // Ligero para <8GB
        }
    }
}

/// Formatear bytes a tamaño legible
fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Obtener RAM total del sistema en GB
pub fn get_total_ram_gb() -> f64 {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args(["-Command", "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(bytes) = stdout.trim().parse::<f64>() {
                return bytes / (1024.0 * 1024.0 * 1024.0);
            }
        }
        16.0  // Default
    }

    #[cfg(not(windows))]
    {
        16.0  // Default para Linux/Mac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_recommended_models() {
        let models = OllamaManager::get_recommended_models();
        assert!(models.contains(&"codellama"));
        assert!(models.contains(&"llama3"));
    }
}
