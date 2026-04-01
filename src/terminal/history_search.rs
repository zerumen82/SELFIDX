// History Search Module - SELFIDEX v3.0
// Búsqueda en historial estilo Ctrl+R de bash

use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use anyhow::Result;

/// Historial de comandos persistente
pub struct CommandHistory {
    history: Vec<String>,
    max_size: usize,
    file_path: PathBuf,
}

impl CommandHistory {
    /// Crear nuevo historial
    pub fn new() -> Result<Self> {
        let file_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("selfidx")
            .join("history.txt");

        // Crear directorio si no existe
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut history = Vec::new();

        // Cargar historial existente
        if file_path.exists() {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(cmd) = line {
                    if !cmd.trim().is_empty() {
                        history.push(cmd);
                    }
                }
            }
        }

        Ok(Self {
            history,
            max_size: 1000,
            file_path,
        })
    }

    /// Agregar comando al historial
    pub fn add(&mut self, command: &str) -> Result<()> {
        // No agregar comandos vacíos o duplicados consecutivos
        if command.trim().is_empty() {
            return Ok(());
        }

        if let Some(last) = self.history.last() {
            if last == command {
                return Ok(());
            }
        }

        self.history.push(command.to_string());

        // Limitar tamaño
        if self.history.len() > self.max_size {
            self.history.remove(0);
        }

        self.save()
    }

    /// Guardar historial en archivo
    fn save(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        for cmd in &self.history {
            writeln!(file, "{}", cmd)?;
        }

        Ok(())
    }

    /// Buscar en el historial (búsqueda substring)
    pub fn search(&self, query: &str) -> Vec<&String> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        self.history
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Buscar hacia atrás desde un índice
    pub fn search_backward(&self, query: &str, from_index: Option<usize>) -> Option<(usize, &String)> {
        if query.is_empty() {
            return None;
        }

        let start = from_index.unwrap_or(self.history.len());
        let query_lower = query.to_lowercase();

        for (i, cmd) in self.history.iter().enumerate().take(start).rev() {
            if cmd.to_lowercase().contains(&query_lower) {
                return Some((i, cmd));
            }
        }

        None
    }

    /// Obtener el último comando
    pub fn last(&self) -> Option<&String> {
        self.history.last()
    }

    /// Obtener comando por índice
    pub fn get(&self, index: usize) -> Option<&String> {
        self.history.get(index)
    }

    /// Obtener longitud del historial
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Verificar si está vacío
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Limpiar historial
    pub fn clear(&mut self) -> Result<()> {
        self.history.clear();
        self.save()
    }

    /// Obtener últimos N comandos
    pub fn recent(&self, n: usize) -> Vec<&String> {
        self.history
            .iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| CommandHistory {
            history: Vec::new(),
            max_size: 1000,
            file_path: PathBuf::from("history.txt"),
        })
    }
}

/// Estado de búsqueda en historial
pub struct HistorySearchState {
    pub active: bool,
    pub query: String,
    pub current_index: Option<usize>,
    pub original_input: String,
}

impl HistorySearchState {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new(),
            current_index: None,
            original_input: String::new(),
        }
    }

    pub fn start(&mut self, current_input: &str) {
        self.active = true;
        self.query.clear();
        self.current_index = None;
        self.original_input = current_input.to_string();
    }

    pub fn stop(&mut self) -> String {
        self.active = false;
        std::mem::take(&mut self.original_input)
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
        self.current_index = None; // Reset search position
    }

    pub fn remove_char(&mut self) {
        self.query.pop();
        self.current_index = None;
    }

    pub fn next_result<'a>(&'a self, history: &'a CommandHistory) -> Option<(usize, &'a String)> {
        let start = self.current_index.map(|i| i - 1);
        history.search_backward(&self.query, start)
    }

    pub fn display_result(&self, result: Option<&String>) -> String {
        if !self.active {
            return String::new();
        }

        match result {
            Some(cmd) => format!(
                "(rev-i-search)`{}': {}",
                self.query,
                cmd
            ),
            None => format!(
                "(failed rev-i-search)`{}'",
                self.query
            ),
        }
    }
}

impl Default for HistorySearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_history() -> CommandHistory {
        // Usar archivo temporal para tests
        let temp_dir = env::temp_dir().join("selfidx_test");
        let file_path = temp_dir.join(format!("history_{}.txt", std::process::id()));
        
        std::fs::create_dir_all(&temp_dir).ok();
        
        CommandHistory {
            history: Vec::new(),
            max_size: 1000,
            file_path,
        }
    }

    #[test]
    fn test_history_add() {
        let mut history = create_test_history();
        history.add("cargo build").unwrap();
        history.add("cargo test").unwrap();
        
        assert_eq!(history.len(), 2);
        assert_eq!(history.last(), Some(&"cargo test".to_string()));
    }

    #[test]
    fn test_history_search() {
        let mut history = create_test_history();
        history.add("cargo build").unwrap();
        history.add("cargo test").unwrap();
        history.add("npm install").unwrap();
        
        let results = history.search("cargo");
        assert_eq!(results.len(), 2);
        
        let results = history.search("npm");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_history_no_duplicates() {
        let mut history = create_test_history();
        history.add("cargo build").unwrap();
        history.add("cargo build").unwrap();
        history.add("cargo build").unwrap();
        
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_history_search_state() {
        let mut state = HistorySearchState::new();
        assert!(!state.active);
        
        state.start("current input");
        assert!(state.active);
        assert_eq!(state.original_input, "current input");
        
        state.add_char('c');
        state.add_char('a');
        assert_eq!(state.query, "ca");
        
        state.remove_char();
        assert_eq!(state.query, "c");
        
        let _ = state.stop();
        assert!(!state.active);
    }
}
