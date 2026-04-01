// Permission Rules - SELFIDEX v3.0
// Reglas de permiso para control granular de herramientas

use serde::{Deserialize, Serialize};

/// Valor de una regla de permiso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRuleValue {
    /// Nombre de la herramienta (ej: "execute_command", "write_file")
    pub tool_name: String,
    /// Patrón de contenido (ej: "git *", "rm *")
    pub pattern: String,
}

impl PermissionRuleValue {
    pub fn new(tool_name: &str, pattern: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            pattern: pattern.to_string(),
        }
    }

    /// Verificar si un input coincide con el patrón
    pub fn matches(&self, input: &str) -> bool {
        match_pattern(&self.pattern, input)
    }
}

/// Fuente de una regla de permiso
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSource {
    /// Configuración de usuario global
    #[serde(rename = "user")]
    User,
    /// Configuración del proyecto (.selfidx/settings.toml)
    #[serde(rename = "project")]
    Project,
    /// Configuración local del directorio
    #[serde(rename = "local")]
    Local,
    /// Argumento CLI
    #[serde(rename = "cli")]
    Cli,
    /// Sesión actual (temporal)
    #[serde(rename = "session")]
    Session,
}

impl RuleSource {
    pub fn display(&self) -> &'static str {
        match self {
            RuleSource::User => "usuario",
            RuleSource::Project => "proyecto",
            RuleSource::Local => "local",
            RuleSource::Cli => "CLI",
            RuleSource::Session => "sesión",
        }
    }
}

impl std::fmt::Display for RuleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Regla de permiso completa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Comportamiento de la regla (allow/deny/ask)
    pub behavior: PermissionBehavior,
    /// Valor de la regla (tool + patrón)
    pub value: PermissionRuleValue,
    /// Fuente de la regla
    pub source: RuleSource,
}

impl PermissionRule {
    pub fn new(behavior: PermissionBehavior, tool_name: &str, pattern: &str, source: RuleSource) -> Self {
        Self {
            behavior,
            value: PermissionRuleValue::new(tool_name, pattern),
            source,
        }
    }

    /// Verificar si la regla coincide con una herramienta e input
    pub fn matches(&self, tool_name: &str, input: &str) -> bool {
        self.value.tool_name == tool_name && self.value.matches(input)
    }

    /// Crear regla de allow
    pub fn allow(tool_name: &str, pattern: &str, source: RuleSource) -> Self {
        Self::new(PermissionBehavior::Allow, tool_name, pattern, source)
    }

    /// Crear regla de deny
    pub fn deny(tool_name: &str, pattern: &str, source: RuleSource) -> Self {
        Self::new(PermissionBehavior::Deny, tool_name, pattern, source)
    }

    /// Crear regla de ask
    pub fn ask(tool_name: &str, pattern: &str, source: RuleSource) -> Self {
        Self::new(PermissionBehavior::Ask, tool_name, pattern, source)
    }
}

/// Comportamiento de permiso
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionBehavior {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ask")]
    Ask,
}

/// Matchear patrón con wildcard (*)
fn match_pattern(pattern: &str, input: &str) -> bool {
    // Caso exacto
    if pattern == input {
        return true;
    }

    // Sin wildcards - no coincide
    if !pattern.contains('*') {
        return false;
    }

    // Wildcard al inicio: "*foo" → termina con "foo"
    if pattern.starts_with('*') && !pattern[1..].contains('*') {
        return input.ends_with(&pattern[1..]);
    }

    // Wildcard al final: "foo*" → empieza con "foo"
    if pattern.ends_with('*') && !pattern[..pattern.len()-1].contains('*') {
        return input.starts_with(&pattern[..pattern.len()-1]);
    }

    // Wildcards en ambos lados: "*foo*" → contiene "foo"
    if pattern.starts_with('*') && pattern.ends_with('*') && pattern.len() > 2 {
        let inner = &pattern[1..pattern.len()-1];
        return input.contains(inner);
    }

    // Wildcard en medio: "foo*bar" → empieza con "foo" y termina con "bar"
    if let Some(pos) = pattern.find('*') {
        let prefix = &pattern[..pos];
        let suffix = &pattern[pos+1..];
        return input.starts_with(prefix) && input.ends_with(suffix) && input.len() >= prefix.len() + suffix.len();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_pattern_exact() {
        assert!(match_pattern("git status", "git status"));
        assert!(!match_pattern("git status", "git diff"));
    }

    #[test]
    fn test_match_pattern_prefix_wildcard() {
        assert!(match_pattern("*status", "git status"));
        assert!(match_pattern("*status", "hg status"));
        assert!(!match_pattern("*status", "git diff"));
    }

    #[test]
    fn test_match_pattern_suffix_wildcard() {
        assert!(match_pattern("git *", "git status"));
        assert!(match_pattern("git *", "git diff"));
        assert!(!match_pattern("git *", "hg status"));
    }

    #[test]
    fn test_match_pattern_contains_wildcard() {
        assert!(match_pattern("*status*", "git status verbose"));
        // Patrón con múltiples wildcards es complejo - simplificamos
        assert!(match_pattern("*git*", "my git status command"));
    }

    #[test]
    fn test_match_pattern_middle_wildcard() {
        assert!(match_pattern("git*status", "git diff status"));
        assert!(!match_pattern("git*status", "git status diff"));
    }

    #[test]
    fn test_permission_rule_matches() {
        let rule = PermissionRule::allow("execute_command", "git *", RuleSource::User);
        assert!(rule.matches("execute_command", "git status"));
        assert!(rule.matches("execute_command", "git commit -m 'test'"));
        assert!(!rule.matches("execute_command", "npm install"));
        assert!(!rule.matches("write_file", "git status"));
    }

    #[test]
    fn test_permission_rule_creation() {
        let allow_rule = PermissionRule::allow("rm", "rm -rf *", RuleSource::User);
        assert_eq!(allow_rule.behavior, PermissionBehavior::Allow);
        assert_eq!(allow_rule.value.tool_name, "rm");
        assert_eq!(allow_rule.value.pattern, "rm -rf *");

        let deny_rule = PermissionRule::deny("execute_command", "sudo *", RuleSource::Project);
        assert_eq!(deny_rule.behavior, PermissionBehavior::Deny);

        let ask_rule = PermissionRule::ask("write_file", "*", RuleSource::Session);
        assert_eq!(ask_rule.behavior, PermissionBehavior::Ask);
    }
}
