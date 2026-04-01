// Configuration Module - SelfIDX v3.0
// Handles configuration file creation and reading

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub jan_ai: JanAiConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JanAiConfig {
    pub endpoint: String,
    pub default_model: String,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub language: String,
    pub log_sessions: bool,
    pub log_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jan_ai: JanAiConfig {
                endpoint: "http://localhost:11434/v1".to_string(), // Ollama default
                default_model: "mistral:latest".to_string(), // Best model with tools support
                auto_connect: true,
            },
            general: GeneralConfig {
                language: "es".to_string(),
                log_sessions: true,
                log_dir: Self::default_log_dir(),
            },
        }
    }
}

impl Config {
    /// Get the config directory
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("selfidx")
    }

    /// Get the config file path
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Get default log directory - SIEMPRE en la carpeta del proyecto
    fn default_log_dir() -> String {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("logs")
            .to_string_lossy()
            .to_string()
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file();

        // Create config directory if it doesn't exist
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // If config file doesn't exist, create it with defaults
        if !config_file.exists() {
            let default_config = Config::default();
            default_config.save()?;
            println!("[selfidx] ✓ Archivo de configuración creado: {}", config_file.display());
            return Ok(default_config);
        }

        // Read existing config
        let content = std::fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file();
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_file, content)?;
        Ok(())
    }

    /// Get Jan.ai endpoint
    pub fn jan_ai_endpoint(&self) -> &str {
        &self.jan_ai.endpoint
    }

    /// Get default model
    pub fn default_model(&self) -> &str {
        &self.jan_ai.default_model
    }

    /// Get log directory
    pub fn log_dir(&self) -> &str {
        &self.general.log_dir
    }

    /// Check if sessions should be logged
    pub fn should_log_sessions(&self) -> bool {
        self.general.log_sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        // El endpoint por defecto es Ollama, no Jan
        assert_eq!(config.jan_ai.endpoint, "http://localhost:11434/v1");
        // El modelo por defecto es mistral:latest
        assert_eq!(config.jan_ai.default_model, "mistral:latest");
        assert_eq!(config.general.language, "es");
    }

    #[test]
    fn test_config_paths() {
        let config_dir = Config::config_dir();
        assert!(config_dir.to_string_lossy().contains("selfidx"));

        let config_file = Config::config_file();
        assert!(config_file.to_string_lossy().contains("config.toml"));
    }
}
