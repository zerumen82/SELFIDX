// Utils Module

pub mod hardware;

use std::path::PathBuf;

/// Get project root directory
pub fn get_project_root() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

/// Get config directory
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("selfidx")
}

/// Read file content
pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

/// Write file content
pub fn write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = PathBuf::from(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// List files in directory
pub fn list_files(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let entries = std::fs::read_dir(dir)?;
    Ok(entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect())
}

/// Check if file exists
pub fn file_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

/// Get file extension
pub fn get_extension(path: &str) -> Option<String> {
    PathBuf::from(path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
}

/// Format file size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get current timestamp
pub fn timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
