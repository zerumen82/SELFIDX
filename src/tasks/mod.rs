// Tasks Module - SELFIDEX v3.0
// Sistema de tareas en background inspirado en Claude Code

use anyhow::Result;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Estado de una tarea
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl TaskStatus {
    pub fn display(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pendiente",
            TaskStatus::Running => "Ejecutando",
            TaskStatus::Completed => "Completada",
            TaskStatus::Failed => "Fallida",
            TaskStatus::Cancelled => "Cancelada",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⏳",
            TaskStatus::Running => "🔄",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
            TaskStatus::Cancelled => "⏹️",
        }
    }
}

/// Tarea en background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Local>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
    pub output_path: Option<PathBuf>,
    pub error_message: Option<String>,
    pub exit_code: Option<i32>,
}

impl Task {
    pub fn new(command: String) -> Self {
        Self {
            id: Self::generate_id(),
            command,
            status: TaskStatus::Pending,
            created_at: Local::now(),
            started_at: None,
            completed_at: None,
            output_path: None,
            error_message: None,
            exit_code: None,
        }
    }

    fn generate_id() -> String {
        format!(
            "task_{}",
            Local::now().format("%Y%m%d_%H%M%S_%f")
        )
    }

    pub fn duration(&self) -> Option<String> {
        let start = self.started_at?;
        let end = self.completed_at.unwrap_or(Local::now());
        let duration = end.signed_duration_since(start);
        
        if duration.num_hours() > 0 {
            Some(format!("{}h {}m {}s", 
                duration.num_hours(),
                duration.num_minutes() % 60,
                duration.num_seconds() % 60
            ))
        } else if duration.num_minutes() > 0 {
            Some(format!("{}m {}s", 
                duration.num_minutes(),
                duration.num_seconds() % 60
            ))
        } else {
            Some(format!("{}s", duration.num_seconds()))
        }
    }
}

/// Gestor de tareas
pub struct TaskManager {
    tasks: HashMap<String, Task>,
    tasks_dir: PathBuf,
}

impl TaskManager {
    /// Crear nuevo gestor de tareas
    pub fn new() -> Result<Self> {
        let tasks_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("selfidx")
            .join("tasks");

        // Crear directorio si no existe
        fs::create_dir_all(&tasks_dir)?;

        let mut manager = Self {
            tasks: HashMap::new(),
            tasks_dir,
        };

        // Cargar tareas existentes
        manager.load_tasks()?;

        Ok(manager)
    }

    /// Cargar tareas desde disco
    fn load_tasks(&mut self) -> Result<()> {
        if !self.tasks_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.tasks_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(task) = serde_json::from_str::<Task>(&content) {
                    self.tasks.insert(task.id.clone(), task);
                }
            }
        }

        Ok(())
    }

    /// Guardar tarea en disco
    fn save_task(&self, task: &Task) -> Result<()> {
        let path = self.tasks_dir.join(format!("{}.json", task.id));
        let content = serde_json::to_string_pretty(task)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Crear nueva tarea
    pub fn create_task(&mut self, command: String) -> &Task {
        let task = Task::new(command);
        let id = task.id.clone();
        self.tasks.insert(id, task);
        
        // Obtener referencia a la tarea recién insertada
        let last_id = self.tasks.keys().last().unwrap().clone();
        self.tasks.get(&last_id).unwrap()
    }

    /// Iniciar tarea en background
    pub fn start_task(&mut self, task_id: &str) -> Result<()> {
        // Obtener datos de la tarea antes de mutar
        let (command, output_path, error_path) = {
            let task = self.tasks.get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Tarea no encontrada: {}", task_id))?;

            task.status = TaskStatus::Running;
            task.started_at = Some(Local::now());
            
            let output_path = self.tasks_dir.join(format!("{}.out", task_id));
            let error_path = self.tasks_dir.join(format!("{}.err", task_id));
            (task.command.clone(), output_path, error_path)
        };

        // Guardar tarea actualizada
        self.save_task(self.tasks.get(task_id).unwrap())?;

        // Ejecutar en background
        std::thread::spawn(move || {
            let mut child = Command::new("cmd")
                .args(["/C", &command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut child) => {
                    let output = child.wait_with_output();

                    match output {
                        Ok(output) => {
                            // Guardar stdout
                            if let Ok(mut file) = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&output_path)
                            {
                                file.write_all(&output.stdout).ok();
                            }

                            // Guardar stderr
                            if let Ok(mut file) = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&error_path)
                            {
                                file.write_all(&output.stderr).ok();
                            }
                        }
                        Err(e) => {
                            eprintln!("Error al obtener output: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error al ejecutar comando: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Detener tarea
    pub fn stop_task(&mut self, task_id: &str) -> Result<()> {
        let should_save = {
            let task = self.tasks.get_mut(task_id)
                .ok_or_else(|| anyhow::anyhow!("Tarea no encontrada: {}", task_id))?;

            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(Local::now());
                true
            } else {
                false
            }
        };

        if should_save {
            self.save_task(self.tasks.get(task_id).unwrap())?;
        }

        Ok(())
    }

    /// Obtener estado de tarea
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Listar todas las tareas
    pub fn list_tasks(&self) -> Vec<&Task> {
        let mut tasks: Vec<_> = self.tasks.values().collect();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        tasks
    }

    /// Listar tareas por estado
    pub fn list_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.status == status)
            .collect()
    }

    /// Obtener output de tarea
    pub fn get_task_output(&self, task_id: &str) -> Result<Option<String>> {
        let task = self.tasks.get(task_id)
            .ok_or_else(|| anyhow::anyhow!("Tarea no encontrada: {}", task_id))?;

        if let Some(output_path) = &task.output_path {
            if output_path.exists() {
                let mut content = String::new();
                fs::File::open(output_path)?.read_to_string(&mut content)?;
                return Ok(Some(content));
            }
        }

        // Intentar leer del archivo .out
        let output_path = self.tasks_dir.join(format!("{}.out", task_id));
        if output_path.exists() {
            let mut content = String::new();
            fs::File::open(&output_path)?.read_to_string(&mut content)?;
            return Ok(Some(content));
        }

        Ok(None)
    }

    /// Eliminar tarea
    pub fn remove_task(&mut self, task_id: &str) -> Result<()> {
        // Eliminar archivos asociados
        let output_path = self.tasks_dir.join(format!("{}.out", task_id));
        let error_path = self.tasks_dir.join(format!("{}.err", task_id));
        let json_path = self.tasks_dir.join(format!("{}.json", task_id));

        fs::remove_file(output_path).ok();
        fs::remove_file(error_path).ok();
        fs::remove_file(json_path).ok();

        self.tasks.remove(task_id);
        Ok(())
    }

    /// Limpiar tareas completadas
    pub fn cleanup_completed(&mut self) -> Result<usize> {
        let completed: Vec<_> = self.tasks
            .iter()
            .filter(|(_, t)| t.status == TaskStatus::Completed)
            .map(|(id, _)| id.clone())
            .collect();

        let count = completed.len();
        for id in completed {
            self.remove_task(&id)?;
        }

        Ok(count)
    }

    /// Obtener estadísticas de tareas
    pub fn get_stats(&self) -> TaskStats {
        let mut stats = TaskStats::default();
        
        for task in self.tasks.values() {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Running => stats.running += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Failed => stats.failed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
        }

        stats.total = self.tasks.len();
        stats
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| TaskManager {
            tasks: HashMap::new(),
            tasks_dir: PathBuf::from("tasks"),
        })
    }
}

/// Estadísticas de tareas
#[derive(Debug, Default)]
pub struct TaskStats {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
}

impl TaskStats {
    pub fn display(&self) -> String {
        format!(
            "Total: {} | {} Pendientes | {} Ejecutando | {} Completadas | {} Fallidas | {} Canceladas",
            self.total,
            self.pending,
            self.running,
            self.completed,
            self.failed,
            self.cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_task_creation() {
        let task = Task::new("cargo build".to_string());
        assert_eq!(task.command, "cargo build");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.id.starts_with("task_"));
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Pending.display(), "Pendiente");
        assert_eq!(TaskStatus::Running.display(), "Ejecutando");
        assert_eq!(TaskStatus::Completed.display(), "Completada");
    }

    #[test]
    fn test_task_status_symbol() {
        assert_eq!(TaskStatus::Pending.symbol(), "⏳");
        assert_eq!(TaskStatus::Completed.symbol(), "✅");
        assert_eq!(TaskStatus::Failed.symbol(), "❌");
    }

    #[test]
    fn test_task_manager_creation() {
        let manager = TaskManager::new().unwrap();
        assert!(manager.tasks_dir.exists());
    }

    #[test]
    fn test_task_stats() {
        // Usar directorio temporal para evitar persistencia entre tests
        let temp_dir = std::env::temp_dir().join(format!("selfidx_task_test_{}", std::process::id()));
        let mut manager = TaskManager {
            tasks: HashMap::new(),
            tasks_dir: temp_dir,
        };
        
        // Crear directorio
        fs::create_dir_all(&manager.tasks_dir).ok();
        
        manager.create_task("cargo build".to_string());
        manager.create_task("cargo test".to_string());

        let stats = manager.get_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.pending, 2);
    }
}
