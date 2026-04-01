// Command Risk Classifier - SELFIDEX v3.0
// Clasificador de riesgo de comandos inspirado en Claude Code

use serde::{Deserialize, Serialize};

/// Nivel de riesgo de un comando
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl RiskLevel {
    pub fn display(&self) -> &'static str {
        match self {
            RiskLevel::Low => "BAJO",
            RiskLevel::Medium => "MEDIO",
            RiskLevel::High => "ALTO",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            RiskLevel::Low => "🟢",
            RiskLevel::Medium => "🟡",
            RiskLevel::High => "🔴",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            RiskLevel::Low => "verde",
            RiskLevel::Medium => "amarillo",
            RiskLevel::High => "rojo",
        }
    }
}

/// Comandos de alto riesgo - destrucción de datos o cambios irreversibles
const HIGH_RISK_COMMANDS: &[&str] = &[
    // Eliminación destructiva
    "rm -rf",
    "rm -fr",
    "rm -r",
    "del /F /S",
    "del /S /Q",
    "rmdir /S",
    "rmdir /Q",
    "Remove-Item -Recurse -Force",
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    
    // Formateo/particionado
    "format",
    "mkfs",
    "fdisk",
    "parted",
    "diskpart",
    "newfs",
    "mke2fs",
    "mkntfs",
    
    // Sobrescritura destructiva
    "dd if=/dev/zero",
    "dd if=/dev/null",
    "cipher /w",
    "sdelete",
    
    // Cambios de permisos peligrosos
    "chmod -R 777 /",
    "chmod -R 777 /home",
    "chmod -R 777 /etc",
    "chown -R root:root /",
    
    // Ejecución remota
    "curl * | bash",
    "curl * | sh",
    "wget * | bash",
    "wget * | sh",
    "powershell -c \"iwr",
    "powershell -c \"Invoke-WebRequest",
    
    // Docker destructivo
    "docker system prune -a",
    "docker rm -f $(docker ps -aq)",
    
    // Git destructivo
    "git push --force",
    "git push -f",
    "git reset --hard HEAD~",
    "git clean -fdx",
];

/// Comandos de riesgo medio - pueden causar problemas pero son reversibles
const MEDIUM_RISK_COMMANDS: &[&str] = &[
    // Cambios de permisos
    "chmod",
    "chown",
    "icacls",
    "takeown",
    
    // Ejecución con privilegios
    "sudo",
    "su -",
    "runas",
    "Start-Process -Verb RunAs",
    
    // Cambios en el sistema
    "reg add",
    "reg delete",
    "regedit",
    "systemctl",
    "service",
    "netsh",
    
    // Gestión de procesos
    "kill -9",
    "taskkill /F",
    "Stop-Process -Force",
    "pkill -9",
    
    // Red
    "iptables",
    "ufw",
    "firewall-cmd",
    "netstat",
    "ss",
    
    // Variables de entorno globales
    "setx",
    "export PATH=",
    "$env:PATH =",
    
    // Instalación global
    "npm install -g",
    "pip install --system",
    "gem install",
    
    // Cambios en el sistema de archivos
    "mount",
    "umount",
    "ln -s",
];

/// Comandos de búsqueda/lectura - bajo riesgo
const SEARCH_COMMANDS: &[&str] = &[
    "find",
    "grep",
    "rg",
    "ag",
    "ack",
    "locate",
    "which",
    "whereis",
    "whence",
];

/// Comandos de lectura - bajo riesgo
const READ_COMMANDS: &[&str] = &[
    "cat",
    "head",
    "tail",
    "less",
    "more",
    "bat",
    "type",
    "Get-Content",
];

/// Comandos de listado - bajo riesgo
const LIST_COMMANDS: &[&str] = &[
    "ls",
    "dir",
    "tree",
    "du",
    "Get-ChildItem",
];

/// Comandos seguros - sin efectos secundarios
const SAFE_COMMANDS: &[&str] = &[
    "echo",
    "printf",
    "pwd",
    "cd",
    "pushd",
    "popd",
    "dirs",
    "true",
    "false",
    "date",
    "time",
    "whoami",
    "hostname",
    "uname",
    "env",
    "printenv",
    "Get-Location",
    "Get-Date",
    "Get-ChildItem",
];

/// Comandos de desarrollo - bajo riesgo
const DEV_COMMANDS: &[&str] = &[
    "cargo build",
    "cargo check",
    "cargo test",
    "cargo doc",
    "cargo fmt",
    "cargo clippy",
    "npm run build",
    "npm run test",
    "npm run dev",
    "npm run lint",
    "yarn build",
    "yarn test",
    "yarn dev",
    "pnpm build",
    "pnpm test",
    "go build",
    "go test",
    "go run",
    "go fmt",
    "go vet",
    "python -m build",
    "python -m pytest",
    "python -m unittest",
    "pytest",
    "dotnet build",
    "dotnet test",
    "dotnet run",
    "flutter build",
    "flutter run",
    "flutter test",
];

/// Clasificar el riesgo de un comando
pub fn classify_command_risk(cmd: &str) -> RiskLevel {
    let cmd_lower = cmd.to_lowercase();
    let cmd_trimmed = cmd_lower.trim();

    // Verificar comandos de alto riesgo primero - coincidencia parcial
    for pattern in HIGH_RISK_COMMANDS {
        if cmd_trimmed.contains(pattern) {
            return RiskLevel::High;
        }
    }

    // Verificar comandos de riesgo medio
    for pattern in MEDIUM_RISK_COMMANDS {
        if cmd_trimmed.starts_with(pattern) || cmd_trimmed.contains(&format!("{} ", pattern)) {
            return RiskLevel::Medium;
        }
    }

    // Verificar comandos seguros
    for pattern in SAFE_COMMANDS {
        if cmd_trimmed.starts_with(pattern) || cmd_trimmed.starts_with(&format!("{} ", pattern)) {
            return RiskLevel::Low;
        }
    }

    // Verificar comandos de desarrollo - coincidencia de inicio
    for pattern in DEV_COMMANDS {
        if cmd_trimmed.starts_with(pattern) || cmd_trimmed.starts_with(&format!("{} ", pattern)) {
            return RiskLevel::Low;
        }
    }

    // Verificar comandos de búsqueda/lectura/listado
    for pattern in SEARCH_COMMANDS.iter().chain(READ_COMMANDS.iter()).chain(LIST_COMMANDS.iter()) {
        if cmd_trimmed.starts_with(pattern) || cmd_trimmed.starts_with(&format!("{} ", pattern)) {
            return RiskLevel::Low;
        }
    }

    // Detectar pipes y operadores
    if contains_dangerous_operators(cmd_trimmed) {
        return RiskLevel::Medium;
    }

    // Por defecto - riesgo medio (requiere confirmación)
    RiskLevel::Medium
}

/// Detectar operadores peligrosos en el comando
fn contains_dangerous_operators(cmd: &str) -> bool {
    // Operadores de redirect que pueden sobrescribir archivos
    let dangerous_operators = [">", ">>", ">&", "|&"];
    
    for op in &dangerous_operators {
        if cmd.contains(op) {
            return true;
        }
    }

    false
}

/// Obtener descripción del riesgo
pub fn get_risk_description(cmd: &str, risk_level: RiskLevel) -> String {
    match risk_level {
        RiskLevel::High => {
            if cmd.contains("rm -rf") || cmd.contains("rm -r") {
                "Este comando elimina archivos/directorios permanentemente".to_string()
            } else if cmd.contains("format") || cmd.contains("mkfs") {
                "Este comando puede formatear dispositivos de almacenamiento".to_string()
            } else if cmd.contains("curl") && cmd.contains("| bash") {
                "Este comando descarga y ejecuta código remoto".to_string()
            } else {
                "Este comando puede causar cambios irreversibles en el sistema".to_string()
            }
        }
        RiskLevel::Medium => {
            if cmd.contains("sudo") || cmd.contains("chmod") || cmd.contains("chown") {
                "Este comando requiere privilegios elevados o cambia permisos".to_string()
            } else if cmd.contains("kill") || cmd.contains("taskkill") {
                "Este comando termina procesos en ejecución".to_string()
            } else {
                "Este comando puede modificar el sistema o archivos".to_string()
            }
        }
        RiskLevel::Low => {
            "Este comando es seguro y no realiza cambios destructivos".to_string()
        }
    }
}

/// Extraer archivos afectados por el comando
pub fn extract_affected_files(cmd: &str) -> Vec<String> {
    let mut files = Vec::new();
    
    // Implementación simple para comando rm
    if cmd.starts_with("rm ") {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        for part in parts.iter().skip(1) {
            if !part.starts_with('-') && !part.is_empty() {
                files.push(part.to_string());
            }
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_risk_commands() {
        // rm -rf / está en HIGH_RISK_COMMANDS
        let result = classify_command_risk("rm -rf /");
        assert_eq!(result, RiskLevel::High, "rm -rf / debe ser alto riesgo");
        
        // format está en HIGH_RISK_COMMANDS
        let result = classify_command_risk("format C:");
        assert_eq!(result, RiskLevel::High, "format debe ser alto riesgo");
    }

    #[test]
    fn test_medium_risk_commands() {
        assert_eq!(classify_command_risk("sudo apt update"), RiskLevel::Medium);
        assert_eq!(classify_command_risk("chmod 755 file.sh"), RiskLevel::Medium);
        assert_eq!(classify_command_risk("kill -9 1234"), RiskLevel::Medium);
    }

    #[test]
    fn test_low_risk_commands() {
        // Comandos de lista SAFE_COMMANDS
        assert_eq!(classify_command_risk("pwd"), RiskLevel::Low);
        assert_eq!(classify_command_risk("echo hello"), RiskLevel::Low);
        assert_eq!(classify_command_risk("whoami"), RiskLevel::Low);
        
        // Comandos de lista DEV_COMMANDS
        assert_eq!(classify_command_risk("cargo build"), RiskLevel::Low);
        assert_eq!(classify_command_risk("cargo test"), RiskLevel::Low);
        assert_eq!(classify_command_risk("npm run test"), RiskLevel::Low);
    }

    #[test]
    fn test_risk_display() {
        assert_eq!(RiskLevel::High.display(), "ALTO");
        assert_eq!(RiskLevel::Medium.display(), "MEDIO");
        assert_eq!(RiskLevel::Low.display(), "BAJO");
    }

    #[test]
    fn test_risk_symbol() {
        assert_eq!(RiskLevel::High.symbol(), "🔴");
        assert_eq!(RiskLevel::Medium.symbol(), "🟡");
        assert_eq!(RiskLevel::Low.symbol(), "🟢");
    }
}
