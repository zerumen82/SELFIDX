// Cápsula SELFIDEX v3.0
// Visible pero moderada

pub fn render_capsule() -> String {
    r#"
    █████
   ██████████    SELFIDEX v3.0
   ██████████    [●] Jan.ai Conectado
     █████
    "#.to_string()
}

pub fn render_header() -> String {
    "SELFIDEX v3.0 | ".to_string()
}

pub fn render_status(connected: bool) -> String {
    if connected {
        "[●] Jan.ai Conectado".to_string()
    } else {
        "[○] Jan.ai Desconectado".to_string()
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
    }
}
