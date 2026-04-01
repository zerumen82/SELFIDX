// Cápsula SELFIDEX v3.0 - Enhanced Green Edition
// Diseño mejorado con múltiples tonos de verde y efectos visuales

/// ANSI color codes for terminal styling
mod colors {
    // Verdes principales
    pub const GREEN: &str = "\x1b[32m";
    #[allow(dead_code)]
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const DARK_GREEN: &str = "\x1b[2;32m";
    pub const LIME_GREEN: &str = "\x1b[38;5;82m";
    pub const FOREST_GREEN: &str = "\x1b[38;5;22m";
    pub const NEON_GREEN: &str = "\x1b[38;5;10m";
    
    // Efectos
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    #[allow(dead_code)]
    pub const UNDERLINE: &str = "\x1b[4m";
    #[allow(dead_code)]
    pub const BLINK: &str = "\x1b[5m";
    
    // Fondo verde
    #[allow(dead_code)]
    pub const BG_GREEN: &str = "\x1b[42m";
    #[allow(dead_code)]
    pub const BG_DARK_GREEN: &str = "\x1b[48;5;22m";
}

/// Render the enhanced SELFIDEX capsule with gradient effect
pub fn render_capsule() -> String {
    format!(
        r#"
{fg}╔══════════════════════════════════════════╗{reset}
{fg}║{reset}      {lg}{bold}╭───────────────────────────╮{reset}      {fg}║{reset}
{fg}║{reset}      {lg}{bold}│  ████████████████████  │{reset}      {fg}║{reset}
{fg}║{reset}      {lg}{bold}│  ████████████████████  │{reset}      {fg}║{reset}  {ng}{bold}SELFIDEX{reset} {g}v3.0{reset}
{fg}║{reset}      {lg}{bold}│  ████████████████████  │{reset}      {fg}║{reset}
{fg}║{reset}      {lg}{bold}╰───────────────────────────╯{reset}      {fg}║{reset}  {g}[●]{reset} {g}Ollama Conectado{reset}
{fg}║{reset}                                            {fg}║{reset}  {dg}Agente IA Autónomo{reset}
{fg}╚══════════════════════════════════════════╝{reset}
"#,
        fg = colors::FOREST_GREEN,
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        g = colors::GREEN,
        dg = colors::DARK_GREEN,
        bold = colors::BOLD,
        reset = colors::RESET
    )
}

/// Render a compact horizontal capsule
pub fn render_capsule_horizontal() -> String {
    format!(
        r#"
{lg}╔═══════════════════════════════════════════════════════╗{reset}
{lg}║{reset}  {ng}{bold}█████{reset} {g}{bold}SELFIDEX v3.0{reset}  {g}[●]{reset} {g}Ollama Conectado{reset}  {lg}║{reset}
{lg}║{reset}  {dg}Terminal Integrada con IA Autónoma{reset}            {lg}║{reset}
{lg}╚═══════════════════════════════════════════════════════╝{reset}
"#,
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        g = colors::GREEN,
        dg = colors::DARK_GREEN,
        bold = colors::BOLD,
        reset = colors::RESET
    )
}

/// Render mini capsule for inline display
pub fn render_mini_capsule() -> String {
    format!(
        "{lg}[{reset}{ng}{bold}█▇█{reset}{lg}]{reset} {g}{bold}SELFIDEX{reset} {g}v3.0{reset}  {g}●{reset} {dg}Ollama Ready{reset}",
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        g = colors::GREEN,
        dg = colors::DARK_GREEN,
        bold = colors::BOLD,
        reset = colors::RESET
    )
}

pub fn render_header() -> String {
    format!(
        "{lg}╔══════════════════════════════════════════╗{reset}\n{lg}║{reset}  {ng}{bold}SELFIDEX{reset} {g}v3.0{reset}  {g}Terminal con IA{reset}      {lg}║{reset}\n{lg}╚══════════════════════════════════════════╝{reset}",
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        g = colors::GREEN,
        bold = colors::BOLD,
        reset = colors::RESET
    )
}

pub fn render_status(connected: bool) -> String {
    if connected {
        format!(
            "{lg}[{ng}●{lg}]{reset} {g}{bold}Ollama Conectado{reset}",
            lg = colors::LIME_GREEN,
            ng = colors::NEON_GREEN,
            g = colors::GREEN,
            bold = colors::BOLD,
            reset = colors::RESET
        )
    } else {
        format!(
            "{dg}[○]{reset} {dim}Ollama Desconectado{reset}",
            dg = colors::DARK_GREEN,
            dim = colors::DIM,
            reset = colors::RESET
        )
    }
}

/// Render a decorative separator line with gradient
pub fn render_separator() -> String {
    format!(
        "{lg}════════════════════════════════════════════════════════{reset}",
        lg = colors::LIME_GREEN,
        reset = colors::RESET
    )
}

/// Render a double-line separator
pub fn render_double_separator() -> String {
    format!(
        "{lg}╔════════════════════════════════════════════════════════╗{reset}\n{lg}╚════════════════════════════════════════════════════════╝{reset}",
        lg = colors::LIME_GREEN,
        reset = colors::RESET
    )
}

/// Render section header
pub fn render_section_header(title: &str) -> String {
    format!(
        "\n{lg}┌──────────────────────────────────────────────────────┐{reset}\n{lg}│{reset}  {ng}{bold}{title}{reset}                                  {lg}│{reset}\n{lg}└──────────────────────────────────────────────────────┘{reset}",
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        bold = colors::BOLD,
        reset = colors::RESET,
        title = title
    )
}

/// Render success message with green highlight
pub fn render_success(message: &str) -> String {
    format!(
        "{lg}[✓]{reset} {g}{message}{reset}",
        lg = colors::LIME_GREEN,
        g = colors::GREEN,
        reset = colors::RESET,
        message = message
    )
}

/// Render error message
pub fn render_error(message: &str) -> String {
    format!(
        "{r}[✗]{reset} {dim}{message}{reset}",
        r = "\x1b[31m",
        dim = colors::DIM,
        reset = colors::RESET,
        message = message
    )
}

/// Render warning message
pub fn render_warning(message: &str) -> String {
    format!(
        "{y}[!]{reset} {y}{message}{reset}",
        y = "\x1b[33m",
        reset = colors::RESET,
        message = message
    )
}

/// Render permission status with color
pub fn render_permission_status(mode: &str, symbol: &str) -> String {
    format!(
        "{lg}[{ng}{symbol}{lg}]{reset} {g}Modo: {mode}{reset}",
        lg = colors::LIME_GREEN,
        ng = colors::NEON_GREEN,
        g = colors::GREEN,
        reset = colors::RESET,
        symbol = symbol,
        mode = mode
    )
}

/// Render risk level indicator
pub fn render_risk_indicator(level: &str) -> String {
    match level {
        "LOW" => format!("{g}[●]{reset} {g}Riesgo: BAJO{reset}", g = colors::GREEN, reset = colors::RESET),
        "MEDIUM" => format!("{y}[●]{reset} {y}Riesgo: MEDIO{reset}", y = "\x1b[33m", reset = colors::RESET),
        "HIGH" => format!("{r}[●]{reset} {r}Riesgo: ALTO{reset}", r = "\x1b[31m", reset = colors::RESET),
        _ => format!("{dim}[○]{reset} {dim}Riesgo: DESCONOCIDO{reset}", dim = colors::DIM, reset = colors::RESET),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_render() {
        let capsule = render_capsule();
        assert!(capsule.contains("SELFIDEX"));
        assert!(capsule.contains("v3.0"));
        assert!(capsule.contains("████"));
    }

    #[test]
    fn test_horizontal_capsule() {
        let capsule = render_capsule_horizontal();
        assert!(capsule.contains("SELFIDEX"));
        assert!(capsule.contains("Ollama"));
    }

    #[test]
    fn test_mini_capsule() {
        let capsule = render_mini_capsule();
        assert!(capsule.contains("SELFIDEX"));
        assert!(capsule.contains("█▇█"));
    }

    #[test]
    fn test_status_rendering() {
        let connected = render_status(true);
        assert!(connected.contains("Conectado"));
        
        let disconnected = render_status(false);
        assert!(disconnected.contains("Desconectado"));
    }

    #[test]
    fn test_section_header() {
        let header = render_section_header("TEST");
        assert!(header.contains("TEST"));
        assert!(header.contains("┌──"));
    }
}
