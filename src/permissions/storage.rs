// Permission Storage - SELFIDEX v3.0
// Almacenamiento persistente de reglas de permisos

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use super::{PermissionMode, PermissionRule, RuleSource};

/// Configuración de permisos serializable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub mode: PermissionMode,
    pub rules: Vec<StoredRule>,
}

/// Regla almacenada (versión serializable de PermissionRule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRule {
    pub behavior: super::PermissionBehavior,
    pub tool_name: String,
    pub pattern: String,
    pub source: RuleSource,
}

impl StoredRule {
    pub fn from_rule(rule: &PermissionRule) -> Self {
        Self {
            behavior: rule.behavior,
            tool_name: rule.value.tool_name.clone(),
            pattern: rule.value.pattern.clone(),
            source: rule.source,
        }
    }

    pub fn to_rule(&self) -> PermissionRule {
        PermissionRule {
            behavior: self.behavior,
            value: super::PermissionRuleValue {
                tool_name: self.tool_name.clone(),
                pattern: self.pattern.clone(),
            },
            source: self.source,
        }
    }
}

/// Almacenamiento de permisos
pub struct PermissionStorage {
    config_path: PathBuf,
    config: PermissionConfig,
}

impl PermissionStorage {
    /// Obtener path al archivo de configuración
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("selfidx")
            .join("permissions.toml")
    }

    /// Cargar configuración desde archivo o crear default
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        // Crear directorio si no existe
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = if config_path.exists() {
            // Leer archivo existente
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            // Crear configuración default
            PermissionConfig {
                mode: PermissionMode::Default,
                rules: Vec::new(),
            }
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Guardar configuración
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Obtener modo actual
    pub fn get_mode(&self) -> PermissionMode {
        self.config.mode
    }

    /// Establecer modo
    pub fn set_mode(&mut self, mode: PermissionMode) -> Result<()> {
        self.config.mode = mode;
        self.save()
    }

    /// Obtener todas las reglas
    pub fn get_all_rules(&self) -> Vec<PermissionRule> {
        self.config
            .rules
            .iter()
            .map(|r| r.to_rule())
            .collect()
    }

    /// Agregar regla
    pub fn add_rule(&mut self, rule: PermissionRule) -> Result<()> {
        let stored = StoredRule::from_rule(&rule);
        self.config.rules.push(stored);
        self.save()
    }

    /// Remover regla
    pub fn remove_rule(&mut self, tool_name: &str, pattern: &str) -> Result<()> {
        self.config
            .rules
            .retain(|r| r.tool_name != tool_name || r.pattern != pattern);
        self.save()
    }

    /// Limpiar todas las reglas de una fuente
    pub fn clear_rules_from_source(&mut self, source: RuleSource) -> Result<()> {
        self.config
            .rules
            .retain(|r| r.source != source);
        self.save()
    }

    /// Obtener reglas por fuente
    pub fn get_rules_by_source(&self, source: RuleSource) -> Vec<PermissionRule> {
        self.config
            .rules
            .iter()
            .filter(|r| r.source == source)
            .map(|r| r.to_rule())
            .collect()
    }

    /// Buscar regla específica
    pub fn find_rule(&self, tool_name: &str, pattern: &str) -> Option<PermissionRule> {
        self.config
            .rules
            .iter()
            .find(|r| r.tool_name == tool_name && r.pattern == pattern)
            .map(|r| r.to_rule())
    }
}

impl Default for PermissionStorage {
    fn default() -> Self {
        Self {
            config_path: Self::config_path(),
            config: PermissionConfig {
                mode: PermissionMode::Default,
                rules: Vec::new(),
            },
        }
    }
}

/// Reglas predefinidas recomendadas
pub fn get_default_rules() -> Vec<PermissionRule> {
    vec![
        // Auto-aprobar comandos de desarrollo comunes
        PermissionRule::allow("execute_command", "cargo build", RuleSource::User),
        PermissionRule::allow("execute_command", "cargo check", RuleSource::User),
        PermissionRule::allow("execute_command", "cargo test", RuleSource::User),
        PermissionRule::allow("execute_command", "cargo fmt", RuleSource::User),
        PermissionRule::allow("execute_command", "npm run build", RuleSource::User),
        PermissionRule::allow("execute_command", "npm run test", RuleSource::User),
        PermissionRule::allow("execute_command", "npm run dev", RuleSource::User),
        PermissionRule::allow("execute_command", "npm run lint", RuleSource::User),
        
        // Auto-aprobar comandos de git seguros
        PermissionRule::allow("execute_command", "git status", RuleSource::User),
        PermissionRule::allow("execute_command", "git log *", RuleSource::User),
        PermissionRule::allow("execute_command", "git diff *", RuleSource::User),
        PermissionRule::allow("execute_command", "git branch", RuleSource::User),
        
        // Preguntar siempre para operaciones destructivas
        PermissionRule::ask("execute_command", "rm -rf *", RuleSource::User),
        PermissionRule::ask("execute_command", "del *", RuleSource::User),
        
        // Preguntar para escritura de archivos
        PermissionRule::ask("write_file", "*", RuleSource::User),
        
        // Denegar comandos muy peligrosos
        PermissionRule::deny("execute_command", "rm -rf /", RuleSource::User),
        PermissionRule::deny("execute_command", "rm -rf /*", RuleSource::User),
        PermissionRule::deny("execute_command", "rm -rf ~", RuleSource::User),
    ]
}

/// Inicializar con reglas default si no existen
pub fn initialize_with_defaults() -> Result<PermissionStorage> {
    let mut storage = PermissionStorage::load()?;
    
    // Solo agregar defaults si no hay reglas
    if storage.get_all_rules().is_empty() {
        for rule in get_default_rules() {
            storage.add_rule(rule)?;
        }
    }
    
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PermissionBehavior;

    #[test]
    fn test_stored_rule_conversion() {
        let rule = PermissionRule::allow("execute_command", "git *", RuleSource::User);
        let stored = StoredRule::from_rule(&rule);
        let restored = stored.to_rule();

        assert_eq!(restored.behavior, PermissionBehavior::Allow);
        assert_eq!(restored.value.tool_name, "execute_command");
        assert_eq!(restored.value.pattern, "git *");
        assert_eq!(restored.source, RuleSource::User);
    }

    #[test]
    fn test_default_rules() {
        let rules = get_default_rules();
        assert!(!rules.is_empty());

        // Verificar que hay reglas de diferentes tipos
        let allow_count = rules.iter().filter(|r| matches!(r.behavior, PermissionBehavior::Allow)).count();
        let deny_count = rules.iter().filter(|r| matches!(r.behavior, PermissionBehavior::Deny)).count();
        let ask_count = rules.iter().filter(|r| matches!(r.behavior, PermissionBehavior::Ask)).count();

        assert!(allow_count > 0);
        assert!(deny_count > 0);
        assert!(ask_count > 0);
    }
}
