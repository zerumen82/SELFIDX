// Agent Module - Full AI Agent capabilities
// Like Claude Code / Codex - can read, edit files and execute commands

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

/// Agent capabilities for file operations and command execution
pub struct Agent {
    pub project_root: PathBuf,
}

impl Agent {
    pub fn new() -> Self {
        let project_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        
        Self { project_root }
    }

    /// Read a file and return its contents
    pub fn read_file(&self, path: &str) -> Result<String> {
        let full_path = self.project_root.join(path);
        Ok(std::fs::read_to_string(full_path)?)
    }

    /// List files in a directory
    pub fn list_files(&self, path: &str) -> Result<Vec<FileInfo>> {
        let full_path = self.project_root.join(path);
        let mut files = Vec::new();
        
        if full_path.is_dir() {
            for entry in std::fs::read_dir(full_path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                let name = entry.file_name().to_string_lossy().to_string();
                
                files.push(FileInfo {
                    name,
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                    path: entry.path().to_string_lossy().to_string().replace("\\", "/"),
                });
            }
        }
        
        // Sort: directories first, then files
        files.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(files)
    }

    /// Execute a command in the shell
    pub fn execute_command(&self, cmd: &str) -> Result<CommandResult> {
        #[cfg(windows)]
        {
            let output = Command::new("cmd")
                .args(["/C", cmd])
                .current_dir(&self.project_root)
                .output()?;
            
            Ok(CommandResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            })
        }
        
        #[cfg(not(windows))]
        {
            let output = Command::new("sh")
                .args(["-c", cmd])
                .current_dir(&self.project_root)
                .output()?;
            
            Ok(CommandResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            })
        }
    }

    /// Write content to a file (create or overwrite)
    pub fn write_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.project_root.join(path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(full_path, content)?;
        Ok(())
    }

    /// Create a directory
    pub fn create_directory(&self, path: &str) -> Result<()> {
        let full_path = self.project_root.join(path);
        std::fs::create_dir_all(full_path)?;
        Ok(())
    }

    /// Delete a file or directory
    pub fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.project_root.join(path);
        
        if full_path.is_dir() {
            std::fs::remove_dir_all(full_path)?;
        } else {
            std::fs::remove_file(full_path)?;
        }
        Ok(())
    }

    /// Get project structure as a tree
    pub fn get_project_tree(&self, max_depth: usize) -> Result<String> {
        let mut output = String::new();
        self.build_tree(&self.project_root, "", max_depth, &mut output);
        Ok(output)
    }

    fn build_tree(&self, path: &PathBuf, prefix: &str, max_depth: usize, output: &mut String) {
        if max_depth == 0 {
            return;
        }

        let entries: Vec<_> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .collect();

        for (i, entry) in entries.iter().enumerate() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files and common ignore patterns
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            let is_last = i == entries.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            
            output.push_str(&format!("{}{}{}\n", prefix, connector, name));

            if entry.path().is_dir() {
                let new_prefix = format!("{}{}   ", prefix, if is_last { "    " } else { "│   " });
                self.build_tree(&entry.path(), &new_prefix, max_depth - 1, output);
            }
        }
    }

    /// Search for a pattern in files
    pub fn search(&self, pattern: &str, file_pattern: Option<&str>) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        // Simple search implementation
        self.search_in_dir(&self.project_root, pattern, file_pattern, &mut results, 0, 3)?;
        
        Ok(results)
    }

    fn search_in_dir(
        &self, 
        path: &PathBuf, 
        pattern: &str, 
        file_pattern: Option<&str>,
        results: &mut Vec<SearchResult>,
        depth: usize,
        max_depth: usize
    ) -> Result<()> {
        if depth > max_depth {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden and ignore patterns
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                self.search_in_dir(&path, pattern, file_pattern, results, depth + 1, max_depth)?;
            } else {
                // Check file pattern filter
                if let Some(ext) = file_pattern {
                    if !name.ends_with(ext) {
                        continue;
                    }
                }

                // Search in file content
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            results.push(SearchResult {
                                file: path.to_string_lossy().replace("\\", "/"),
                                line: line_num + 1,
                                content: line.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub content: String,
}

impl CommandResult {
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Tool definition for Claude Code-like agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl Agent {
    /// Get available tools for the agent (Claude Code style)
    pub fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "read_file".to_string(),
                description: "Read the contents of a file. Use this to read source code, configuration files, or any text file.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "write_file".to_string(),
                description: "Create or overwrite a file with new content. WARNING: This will overwrite existing files!".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path where to create/write the file"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
            Tool {
                name: "execute_command".to_string(),
                description: "Execute a shell command. Use for git operations, running programs, installing packages, etc.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to execute"
                        }
                    },
                    "required": ["command"]
                }),
            },
            Tool {
                name: "list_files".to_string(),
                description: "List files in a directory. Shows files and subdirectories.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The directory path to list (optional, defaults to current)"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "search".to_string(),
                description: "Search for a pattern in files. Returns matching lines with file names and line numbers.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The text pattern to search for"
                        }
                    },
                    "required": ["pattern"]
                }),
            },
            Tool {
                name: "delete".to_string(),
                description: "Delete a file. WARNING: This action is destructive and cannot be undone!".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to delete"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "create_directory".to_string(),
                description: "Create a new directory/folder.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path of the directory to create"
                        }
                    },
                    "required": ["path"]
                }),
            },
        ]
    }

    /// Get tools as JSON string for the system prompt
    pub fn get_tools_description() -> String {
        let tools = Self::get_tools();
        let mut description = String::from("\n=== HERRAMIENTAS DISPONIBLES ===\n\n");
        
        for tool in &tools {
            description.push_str(&format!(
                "## {}\n{}\nParámetros: {}\n\n",
                tool.name,
                tool.description,
                serde_json::to_string_pretty(&tool.parameters).unwrap_or_default()
            ));
        }
        
        description
    }

    /// Execute a tool by name with parameters
    pub fn execute_tool(&self, tool_name: &str, params: &serde_json::Value) -> Result<String> {
        match tool_name {
            "read_file" => {
                let path = params["path"].as_str().unwrap_or("");
                let content = self.read_file(path)?;
                Ok(format!("=== CONTENIDO DE {} ===\n{}", path, content))
            }
            "write_file" => {
                let path = params["path"].as_str().unwrap_or("");
                let content = params["content"].as_str().unwrap_or("");
                self.write_file(path, content)?;
                Ok(format!("✓ Archivo {} creado/escrito correctamente", path))
            }
            "execute_command" => {
                let cmd = params["command"].as_str().unwrap_or("");
                let result = self.execute_command(cmd)?;
                Ok(format!(
                    "=== RESULTADO DEL COMANDO ===\n{}",
                    if result.stdout.is_empty() { result.stderr } else { result.stdout }
                ))
            }
            "list_files" => {
                let path = params["path"].as_str().unwrap_or(".");
                let files = self.list_files(path)?;
                Ok(format!(
                    "=== ARCHIVOS EN {} ===\n{}",
                    path,
                    files.iter()
                        .map(|f| format!("{} {}", if f.is_dir { "[DIR]" } else { "[FILE]" }, f.name))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            "search" => {
                let pattern = params["pattern"].as_str().unwrap_or("");
                let results = self.search(pattern, Some("rs"))?;
                Ok(if results.is_empty() {
                    "No se encontraron resultados".to_string()
                } else {
                    format!(
                        "=== RESULTADOS DE BÚSQUEDA ===\n{}",
                        results.iter()
                            .map(|r| format!("{}:{}: {}", r.file, r.line, r.content))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                })
            }
            "delete" => {
                let path = params["path"].as_str().unwrap_or("");
                self.delete(path)?;
                Ok(format!("✓ Archivo {} eliminado correctamente", path))
            }
            "create_directory" => {
                let path = params["path"].as_str().unwrap_or("");
                self.create_directory(path)?;
                Ok(format!("✓ Directorio {} creado correctamente", path))
            }
            _ => Err(anyhow::anyhow!("Herramienta desconocida: {}", tool_name)),
        }
    }

    /// Check if a tool is destructive (needs confirmation)
    pub fn is_destructive_tool(tool_name: &str) -> bool {
        matches!(tool_name, "delete" | "write_file")
    }

    /// Execute tool with user confirmation for destructive actions
    pub fn execute_tool_with_confirmation(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> Result<(String, bool)> {
        let is_destructive = Self::is_destructive_tool(tool_name);
        
        if is_destructive {
            let path = params["path"].as_str().unwrap_or("unknown");
            println!("\n⚠️  ACCIÓN DESTRUCTIVA DETECTADA");
            println!("   Herramienta: {}", tool_name);
            println!("   Objetivo: {}", path);
            println!("\n¿Confirmar ejecución? (s/n): ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() != "s" {
                return Ok(("Acción cancelada por el usuario".to_string(), false));
            }
        }
        
        let result = self.execute_tool(tool_name, params)?;
        Ok((result, true))
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new();
        assert!(agent.project_root.exists());
    }

    #[test]
    fn test_write_and_read_file() {
        let agent = Agent::new();
        let test_path = "test_file.txt";
        let test_content = "Hello, World!";
        
        // Write file
        let result = agent.write_file(test_path, test_content);
        assert!(result.is_ok());
        
        // Read file
        let content = agent.read_file(test_path);
        assert!(content.is_ok());
        assert_eq!(content.unwrap(), test_content);
        
        // Cleanup
        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_list_files() {
        let agent = Agent::new();
        let result = agent.list_files(".");
        assert!(result.is_ok());
        
        let files = result.unwrap();
        // Should have at least some files
        assert!(!files.is_empty());
    }

    #[test]
    fn test_execute_command() {
        let agent = Agent::new();
        
        #[cfg(windows)]
        let result = agent.execute_command("echo test");
        
        #[cfg(not(windows))]
        let result = agent.execute_command("echo test");
        
        assert!(result.is_ok());
        let cmd_result = result.unwrap();
        assert!(cmd_result.stdout.contains("test") || cmd_result.is_success());
    }

    #[test]
    fn test_create_and_delete_directory() {
        let agent = Agent::new();
        let test_dir = "test_directory";
        
        // Create directory
        let result = agent.create_directory(test_dir);
        assert!(result.is_ok());
        assert!(PathBuf::from(test_dir).exists());
        
        // Delete directory
        let result = agent.delete(test_dir);
        assert!(result.is_ok());
        assert!(!PathBuf::from(test_dir).exists());
    }

    #[test]
    fn test_get_project_tree() {
        let agent = Agent::new();
        let result = agent.get_project_tree(2);
        assert!(result.is_ok());
        
        let tree = result.unwrap();
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_search() {
        let agent = Agent::new();
        let result = agent.search("fn", Some("rs"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_tools_description() {
        let description = Agent::get_tools_description();
        assert!(description.contains("read_file"));
        assert!(description.contains("write_file"));
        assert!(description.contains("execute_command"));
    }

    #[test]
    fn test_is_destructive_tool() {
        assert!(Agent::is_destructive_tool("delete"));
        assert!(Agent::is_destructive_tool("write_file"));
        assert!(!Agent::is_destructive_tool("read_file"));
        assert!(!Agent::is_destructive_tool("list_files"));
    }
}
