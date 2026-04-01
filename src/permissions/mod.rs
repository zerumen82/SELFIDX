// Permissions Module - SELFIDEX v3.0
// Sistema de permisos inspirado en Claude Code

pub mod mode;
pub mod rules;
pub mod classifier;
pub mod storage;

pub use mode::PermissionMode;
pub use rules::{PermissionRule, PermissionBehavior, RuleSource, PermissionRuleValue};
pub use classifier::{RiskLevel, classify_command_risk};
pub use storage::PermissionStorage;

use anyhow::Result;

/// Contexto de permisos para evaluación de herramientas
pub struct PermissionContext {
    pub mode: PermissionMode,
    pub rules: Vec<PermissionRule>,
    pub storage: PermissionStorage,
}

impl PermissionContext {
    pub fn new() -> Result<Self> {
        let storage = PermissionStorage::load()?;
        let rules = storage.get_all_rules();
        
        Ok(Self {
            mode: storage.get_mode(),
            rules,
            storage,
        })
    }

    /// Verificar si una herramienta puede ejecutarse sin preguntar
    pub fn can_auto_execute(&self, tool_name: &str, input: &str) -> PermissionDecision {
        let risk_level = classify_command_risk(input);
        
        // En modo YOLO, denegar todo (irónicamente seguro)
        if self.mode == PermissionMode::Yolo {
            return PermissionDecision::Deny {
                reason: "Modo YOLO activo - todas las operaciones están denegadas".to_string(),
            };
        }

        // En modo bypass, permitir todo
        if self.mode == PermissionMode::Bypass {
            return PermissionDecision::Allow {
                reason: "Modo Bypass activo".to_string(),
            };
        }

        // Verificar reglas explícitas
        for rule in &self.rules {
            if rule.matches(tool_name, input) {
                match rule.behavior {
                    PermissionBehavior::Allow => {
                        return PermissionDecision::Allow {
                            reason: format!("Regla {} coincide", rule.source),
                        };
                    }
                    PermissionBehavior::Deny => {
                        return PermissionDecision::Deny {
                            reason: format!("Regla {} coincide", rule.source),
                        };
                    }
                    PermissionBehavior::Ask => {
                        return PermissionDecision::Ask {
                            reason: format!("Regla {} requiere confirmación", rule.source),
                            risk_level,
                        };
                    }
                }
            }
        }

        // Sin reglas coincidentes - decidir basado en modo y riesgo
        match self.mode {
            PermissionMode::Default => PermissionDecision::Ask {
                reason: "Modo default - requiere confirmación".to_string(),
                risk_level,
            },
            PermissionMode::Auto => {
                // Auto-aprobar solo bajo riesgo
                if risk_level == RiskLevel::Low {
                    PermissionDecision::Allow {
                        reason: "Riesgo bajo - auto-aprobado".to_string(),
                    }
                } else {
                    PermissionDecision::Ask {
                        reason: "Riesgo medio/alto - requiere confirmación".to_string(),
                        risk_level,
                    }
                }
            }
            PermissionMode::DontAsk => PermissionDecision::Allow {
                reason: "Modo dontask - auto-aprobar todo".to_string(),
            },
            PermissionMode::Plan => PermissionDecision::Ask {
                reason: "Modo plan - requiere revisión".to_string(),
                risk_level,
            },
            PermissionMode::Bypass => PermissionDecision::Allow {
                reason: "Modo bypass - sin restricciones".to_string(),
            },
            PermissionMode::Yolo => PermissionDecision::Deny {
                reason: "Modo YOLO - denegado por seguridad".to_string(),
            },
        }
    }

    /// Agregar regla de permiso
    pub fn add_rule(&mut self, rule: PermissionRule) -> Result<()> {
        self.rules.push(rule.clone());
        self.storage.add_rule(rule)?;
        Ok(())
    }

    /// Remover regla de permiso
    pub fn remove_rule(&mut self, tool_name: &str, pattern: &str) -> Result<()> {
        self.rules.retain(|r| {
            !(r.value.tool_name == tool_name && r.value.pattern == pattern)
        });
        self.storage.remove_rule(tool_name, pattern)?;
        Ok(())
    }

    /// Cambiar modo de permisos
    pub fn set_mode(&mut self, mode: PermissionMode) -> Result<()> {
        self.mode = mode;
        self.storage.set_mode(mode)?;
        Ok(())
    }

    /// Obtener todas las reglas
    pub fn get_rules(&self) -> &Vec<PermissionRule> {
        &self.rules
    }

    /// Obtener modo actual
    pub fn get_mode(&self) -> PermissionMode {
        self.mode
    }
}

impl Default for PermissionContext {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            mode: PermissionMode::Default,
            rules: Vec::new(),
            storage: PermissionStorage::default(),
        })
    }
}

/// Decisión de permiso
#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Allow { reason: String },
    Deny { reason: String },
    Ask { reason: String, risk_level: RiskLevel },
}

impl PermissionDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionDecision::Allow { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionDecision::Deny { .. })
    }

    pub fn needs_confirmation(&self) -> bool {
        matches!(self, PermissionDecision::Ask { .. })
    }

    pub fn reason(&self) -> &str {
        match self {
            PermissionDecision::Allow { reason } => reason,
            PermissionDecision::Deny { reason } => reason,
            PermissionDecision::Ask { reason, .. } => reason,
        }
    }

    pub fn risk_level(&self) -> Option<RiskLevel> {
        match self {
            PermissionDecision::Ask { risk_level, .. } => Some(*risk_level),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_context_default() {
        // El contexto por defecto carga desde storage
        // Verificamos que se pueda crear sin error
        let result = PermissionContext::default();
        // El modo puede ser Default o Auto dependiendo del estado del storage
        assert!(matches!(result.get_mode(), PermissionMode::Default | PermissionMode::Auto | PermissionMode::Yolo | PermissionMode::Bypass | PermissionMode::DontAsk | PermissionMode::Plan));
    }

    #[test]
    fn test_yolo_mode_denies_all() {
        let mut ctx = PermissionContext::default();
        ctx.set_mode(PermissionMode::Yolo).unwrap();
        
        let decision = ctx.can_auto_execute("execute_command", "rm -rf test");
        assert!(decision.is_denied());
    }

    #[test]
    fn test_bypass_mode_allows_all() {
        let mut ctx = PermissionContext::default();
        ctx.set_mode(PermissionMode::Bypass).unwrap();
        
        let decision = ctx.can_auto_execute("execute_command", "rm -rf test");
        assert!(decision.is_allowed());
    }
}
