// Permission Mode - SELFIDEX v3.0
// Modos de operación de permisos inspirados en Claude Code

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::Result;

/// Modos de permiso disponibles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// Modo por defecto - pregunta por cada operación
    #[serde(rename = "default")]
    Default,

    /// Auto-aprobar operaciones de bajo riesgo
    #[serde(rename = "auto")]
    Auto,

    /// No preguntar nunca - auto-aprobar todo
    #[serde(rename = "dontask")]
    DontAsk,

    /// Modo planificación - requiere revisión antes de ejecutar
    #[serde(rename = "plan")]
    Plan,

    /// Bypass - sin restricciones (peligroso)
    #[serde(rename = "bypass")]
    Bypass,

    /// YOLO - denegar todo (irónicamente seguro)
    #[serde(rename = "yolo")]
    Yolo,
}

impl PermissionMode {
    /// Descripción del modo
    pub fn description(&self) -> &'static str {
        match self {
            PermissionMode::Default => "Pregunta por cada operación",
            PermissionMode::Auto => "Auto-aprobar operaciones de bajo riesgo",
            PermissionMode::DontAsk => "No preguntar nunca - auto-aprobar todo",
            PermissionMode::Plan => "Requiere revisión en modo planificación",
            PermissionMode::Bypass => "Sin restricciones (peligroso)",
            PermissionMode::Yolo => "Denegar todo (modo seguro)",
        }
    }

    /// Símbolo visual para el modo
    pub fn symbol(&self) -> &'static str {
        match self {
            PermissionMode::Default => "🔒",
            PermissionMode::Auto => "⚡",
            PermissionMode::DontAsk => "🚀",
            PermissionMode::Plan => "📋",
            PermissionMode::Bypass => "⚠️",
            PermissionMode::Yolo => "🛡️",
        }
    }

    /// Lista de todos los modos disponibles
    pub fn all_modes() -> Vec<PermissionMode> {
        vec![
            PermissionMode::Default,
            PermissionMode::Auto,
            PermissionMode::DontAsk,
            PermissionMode::Plan,
            PermissionMode::Bypass,
            PermissionMode::Yolo,
        ]
    }

    /// Obtener modo desde string (case-insensitive)
    pub fn from_str_lossy(s: &str) -> Option<PermissionMode> {
        match s.to_lowercase().as_str() {
            "default" => Some(PermissionMode::Default),
            "auto" => Some(PermissionMode::Auto),
            "dontask" | "dont_ask" | "dont-ask" => Some(PermissionMode::DontAsk),
            "plan" => Some(PermissionMode::Plan),
            "bypass" => Some(PermissionMode::Bypass),
            "yolo" => Some(PermissionMode::Yolo),
            _ => None,
        }
    }
}

impl FromStr for PermissionMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_lossy(s)
            .ok_or_else(|| anyhow::anyhow!("Modo de permiso inválido: {}", s))
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PermissionMode::Default => "default",
            PermissionMode::Auto => "auto",
            PermissionMode::DontAsk => "dontask",
            PermissionMode::Plan => "plan",
            PermissionMode::Bypass => "bypass",
            PermissionMode::Yolo => "yolo",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mode_from_str() {
        assert_eq!(PermissionMode::from_str("default").unwrap(), PermissionMode::Default);
        assert_eq!(PermissionMode::from_str("AUTO").unwrap(), PermissionMode::Auto);
        assert_eq!(PermissionMode::from_str("yolo").unwrap(), PermissionMode::Yolo);
    }

    #[test]
    fn test_permission_mode_display() {
        assert_eq!(PermissionMode::Default.to_string(), "default");
        assert_eq!(PermissionMode::Auto.to_string(), "auto");
    }

    #[test]
    fn test_permission_mode_description() {
        assert!(!PermissionMode::Default.description().is_empty());
    }
}
