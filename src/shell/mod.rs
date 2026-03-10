// Shell Module - PowerShell Integration

use anyhow::{Context, Result};
use std::process::Stdio;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Shell {
    cwd: PathBuf,
}

impl Shell {
    pub fn new(cwd: Option<PathBuf>) -> Self {
        Self {
            cwd: cwd.unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from("."))),
        }
    }

    /// Get current working directory
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    /// Set working directory
    pub fn cd(&mut self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        if path.exists() && path.is_dir() {
            self.cwd = path;
            Ok(())
        } else {
            anyhow::bail!("Directory not found: {}", path.display())
        }
    }

    /// Execute command
    pub fn exec(&self, cmd: &str) -> Result<CommandOutput> {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", cmd])
            .current_dir(&self.cwd)
            .output()
            .context("Failed to execute command")?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status.code(),
        })
    }

    /// Execute PowerShell script
    pub fn exec_ps(&self, script: &str) -> Result<CommandOutput> {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .current_dir(&self.cwd)
            .output()
            .context("Failed to execute PowerShell script")?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status.code(),
        })
    }

    /// Start interactive PowerShell
    pub fn spawn_interactive(&self) -> Result<()> {
        Command::new("powershell.exe")
            .args(["-NoProfile"])
            .current_dir(&self.cwd)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn PowerShell")?
            .wait()
            .context("PowerShell exited with error")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
}

impl CommandOutput {
    pub fn is_success(&self) -> bool {
        self.status == Some(0)
    }

    pub fn combined(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }
}

/// Parse command string into args
pub fn parse_command(cmd: &str) -> (String, Vec<String>) {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    let cmd_name = args.first().cloned().unwrap_or_default();

    (cmd_name, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("npm install react");
        assert_eq!(cmd, "npm");
        assert_eq!(args, vec!["npm", "install", "react"]);

        let (cmd, args) = parse_command("echo \"hello world\"");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["echo", "hello world"]);
    }
}
