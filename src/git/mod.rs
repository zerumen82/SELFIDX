// Git Module - SELFIDEX v3.0
// Integración con Git para operaciones de versión

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Estado del repositorio Git
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub is_clean: bool,
    pub staged: Vec<GitFile>,
    pub unstaged: Vec<GitFile>,
    pub untracked: Vec<String>,
}

/// Archivo con cambios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFile {
    pub path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Unmerged,
}

/// Commit en el historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
}

/// Gestor de operaciones Git
pub struct GitManager {
    repo_path: PathBuf,
}

impl GitManager {
    /// Crear nuevo gestor Git
    pub fn new() -> Result<Self> {
        let repo_path = std::env::current_dir()
            .context("No se pudo obtener el directorio actual")?;

        // Verificar que es un repositorio Git
        if !repo_path.join(".git").exists() {
            anyhow::bail!("No es un repositorio Git: {}", repo_path.display());
        }

        Ok(Self { repo_path })
    }

    /// Ejecutar comando Git
    fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .context("Error al ejecutar git")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git error: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Obtener estado del repositorio
    pub fn status(&self) -> Result<GitStatus> {
        // Obtener rama actual
        let branch = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?
            .trim().to_string();

        // Obtener estado de ahead/behind
        let (ahead, behind) = self.get_ahead_behind(&branch)?;

        // Obtener archivos modificados
        let unstaged = self.get_unstaged_changes()?;
        let staged = self.get_staged_changes()?;
        let untracked = self.get_untracked_files()?;

        let is_clean = unstaged.is_empty() && staged.is_empty() && untracked.is_empty();

        Ok(GitStatus {
            branch,
            ahead,
            behind,
            is_clean,
            staged,
            unstaged,
            untracked,
        })
    }

    /// Obtener ahead/behind del remote
    fn get_ahead_behind(&self, branch: &str) -> Result<(usize, usize)> {
        let output = self.run_git(&[
            "rev-list",
            "--left-right",
            "--count",
            &format!("origin/{}...{}", branch, branch),
        ])?;

        let parts: Vec<&str> = output.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            Ok((
                parts[0].parse().unwrap_or(0),
                parts[1].parse().unwrap_or(0),
            ))
        } else {
            Ok((0, 0))
        }
    }

    /// Obtener archivos sin stagear
    fn get_unstaged_changes(&self) -> Result<Vec<GitFile>> {
        let output = self.run_git(&["diff", "--name-status"])?;
        self.parse_diff_output(&output)
    }

    /// Obtener archivos con stage
    fn get_staged_changes(&self) -> Result<Vec<GitFile>> {
        let output = self.run_git(&["diff", "--cached", "--name-status"])?;
        self.parse_diff_output(&output)
    }

    /// Obtener archivos sin trackear
    fn get_untracked_files(&self) -> Result<Vec<String>> {
        let output = self.run_git(&["ls-files", "--others", "--exclude-standard"])?;
        Ok(output
            .lines()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect())
    }

    /// Parsear output de diff
    fn parse_diff_output(&self, output: &str) -> Result<Vec<GitFile>> {
        let mut files = Vec::new();

        for line in output.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let status_char = parts[0].chars().next().unwrap_or('M');
            let path = parts[1].to_string();

            let status = match status_char {
                'A' => FileStatus::Added,
                'M' => FileStatus::Modified,
                'D' => FileStatus::Deleted,
                'R' => FileStatus::Renamed,
                'C' => FileStatus::Copied,
                'U' => FileStatus::Unmerged,
                _ => FileStatus::Modified,
            };

            files.push(GitFile {
                path,
                status,
                additions: 0,
                deletions: 0,
            });
        }

        Ok(files)
    }

    /// Agregar archivo al stage
    pub fn add(&self, path: &str) -> Result<()> {
        self.run_git(&["add", path])?;
        Ok(())
    }

    /// Agregar todos los archivos
    pub fn add_all(&self) -> Result<()> {
        self.run_git(&["add", "-A"])?;
        Ok(())
    }

    /// Hacer commit
    pub fn commit(&self, message: &str) -> Result<String> {
        self.run_git(&["commit", "-m", message])?;
        self.get_latest_commit()
    }

    /// Obtener último commit
    fn get_latest_commit(&self) -> Result<String> {
        self.run_git(&["log", "-1", "--format=%h"])
            .map(|s| s.trim().to_string())
    }

    /// Hacer push
    pub fn push(&self) -> Result<String> {
        self.run_git(&["push"])
    }

    /// Hacer pull
    pub fn pull(&self) -> Result<String> {
        self.run_git(&["pull"])
    }

    /// Obtener historial de commits
    pub fn log(&self, limit: usize) -> Result<Vec<GitCommit>> {
        let output = self.run_git(&[
            "log",
            &format!("-{}", limit),
            "--format=%H|%h|%an|%ae|%ad|%s",
            "--date=short",
        ])?;

        let mut commits = Vec::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    author: parts[2].to_string(),
                    email: parts[3].to_string(),
                    date: parts[4].to_string(),
                    message: parts[5..].join("|"),
                });
            }
        }

        Ok(commits)
    }

    /// Obtener diff de un archivo
    pub fn diff(&self, path: &str) -> Result<String> {
        self.run_git(&["diff", "--", path])
    }

    /// Obtener diff con stage
    pub fn diff_staged(&self, path: &str) -> Result<String> {
        self.run_git(&["diff", "--cached", "--", path])
    }

    /// Deshacer cambios en archivo
    pub fn checkout(&self, path: &str) -> Result<()> {
        self.run_git(&["checkout", "--", path])?;
        Ok(())
    }

    /// Deshacer stage
    pub fn reset(&self, path: &str) -> Result<()> {
        self.run_git(&["reset", "HEAD", "--", path])?;
        Ok(())
    }

    /// Crear nueva rama
    pub fn checkout_branch(&self, branch: &str) -> Result<()> {
        self.run_git(&["checkout", "-b", branch])?;
        Ok(())
    }

    /// Cambiar de rama
    pub fn switch_branch(&self, branch: &str) -> Result<()> {
        self.run_git(&["checkout", branch])?;
        Ok(())
    }

    /// Listar ramas
    pub fn branches(&self) -> Result<Vec<String>> {
        let output = self.run_git(&["branch", "--list"])?;
        Ok(output
            .lines()
            .map(|s| s.trim_start_matches("* ").trim().to_string())
            .collect())
    }

    /// Verificar si hay cambios sin commitear
    pub fn has_uncommitted_changes(&self) -> bool {
        match self.status() {
            Ok(status) => !status.is_clean,
            Err(_) => false,
        }
    }
}

impl Default for GitManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| GitManager {
            repo_path: PathBuf::from("."),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_manager_creation() {
        // Solo funciona si estamos en un repo git
        let result = GitManager::new();
        // Puede fallar si no hay git instalado
        if result.is_ok() {
            let manager = result.unwrap();
            assert!(manager.repo_path.exists());
        }
    }

    #[test]
    fn test_file_status_enum() {
        let status = FileStatus::Modified;
        assert_eq!(status, FileStatus::Modified);
    }
}
