// Project Module - SelfIDX v3.0
// Gestión de contexto de proyecto estilo Codex

use anyhow::Result;
use std::path::PathBuf;
use chrono::Local;

/// Contexto de proyecto - SIEMPRE usa el directorio actual como root
pub struct ProjectContext {
    pub root: PathBuf,
    pub progress_file: PathBuf,
    pub model: String,
}

impl ProjectContext {
    /// Inicializar contexto de proyecto en directorio actual
    /// El directorio actual SIEMPRE es el root del proyecto
    pub fn init(model: String) -> Result<Self> {
        let root = std::env::current_dir()?;
        let progress_file = root.join(".selfidx.md");

        // Crear archivo de contexto si no existe
        if !progress_file.exists() {
            let project_name = root.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            
            let initial_content = format!(
                r#"# {}
**Modelo**: {}
**Creado**: {}

## Contexto del Proyecto

Este archivo contiene el contexto y progreso del proyecto.
SELFIDEX usará este archivo para entender el estado actual.

## Estructura

<!-- SELFIDEX actualizará esta sección automáticamente -->

## Progreso

| Fecha | Tarea | Estado |
|-------|-------|--------|
| {} | Proyecto inicializado | ✅ |
"#,
                project_name,
                model,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                Local::now().format("%Y-%m-%d %H:%M")
            );
            std::fs::write(&progress_file, initial_content)?;
        }

        Ok(Self {
            root,
            progress_file,
            model,
        })
    }

    /// Cargar contexto de proyecto existente
    pub fn load(model: String) -> Result<Self> {
        let root = std::env::current_dir()?;
        let progress_file = root.join(".selfidx.md");

        Ok(Self {
            root,
            progress_file,
            model,
        })
    }

    /// Leer contenido del archivo de progreso
    pub fn read_progress(&self) -> Result<String> {
        if self.progress_file.exists() {
            Ok(std::fs::read_to_string(&self.progress_file)?)
        } else {
            Ok(String::new())
        }
    }

    /// Agregar entrada de progreso
    pub fn add_progress(&self, task: &str, status: &str, notes: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M");
        let entry = format!("| {} | {} | {} | {} |\n", timestamp, task, status, notes);

        // Leer contenido actual
        let content = std::fs::read_to_string(&self.progress_file)?;

        // Buscar la tabla y agregar
        if let Some(pos) = content.rfind("|-------|") {
            let new_content = format!(
                "{}{}{}",
                &content[..pos + 42], // Después del separador de encabezado
                entry,
                &content[pos + 42..]
            );
            std::fs::write(&self.progress_file, new_content)?;
        }

        Ok(())
    }

    /// Mostrar información del proyecto (estilo Codex)
    pub fn display_info(&self) {
        println!("  📁 Raíz:   {}", self.root.display());
        println!("  🤖 Modelo: {}", self.model);
        println!("  📝 Log:    {}", self.progress_file.display());
        println!();

        // Verificar si existe archivo de progreso
        if self.progress_file.exists() {
            println!("  ✓ Archivo de progreso encontrado");
        } else {
            println!("  ⚠ Se creará archivo de progreso");
        }
        println!();
    }

    /// Obtener nombre del proyecto
    pub fn project_name(&self) -> String {
        self.root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Verificar si es primera ejecución en este proyecto (método estático)
    pub fn is_first_run(project_root: &PathBuf) -> bool {
        !project_root.join(".selfidx.md").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_context() {
        let ctx = ProjectContext::load("test-model".to_string()).unwrap();
        assert!(ctx.root.exists());
        assert_eq!(ctx.model, "test-model");
    }
}
