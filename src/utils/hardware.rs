// Hardware Detection Module
// Detects system resources: RAM, VRAM, CPU

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub total_ram_gb: f64,
    pub vram_gb: Option<f64>,
    pub cpu_cores: usize,
    pub os: String,
    pub recommended_model_size: ModelSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModelSize {
    Tiny,      // < 2GB - 1B params
    Small,     // 2-4GB - 3B params  
    Medium,    // 4-8GB - 7B params
    Large,     // 8-16GB - 13B params
    ExtraLarge, // 16GB+ - 34B+ params
}

impl SystemInfo {
    pub fn detect() -> Self {
        let total_ram_gb = Self::get_total_ram();
        let vram_gb = Self::get_vram();
        let cpu_cores = Self::get_cpu_cores();
        let os = Self::get_os();
        let recommended_model_size = Self::calculate_model_size(total_ram_gb, vram_gb);

        Self {
            total_ram_gb,
            vram_gb,
            cpu_cores,
            os,
            recommended_model_size,
        }
    }

    fn get_total_ram() -> f64 {
        #[cfg(windows)]
        {
            use std::process::Command;
            // Try PowerShell first (more reliable on modern Windows)
            let output = Command::new("powershell")
                .args(["-Command", "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(bytes) = stdout.trim().parse::<f64>() {
                    return bytes / (1024.0 * 1024.0 * 1024.0);
                }
            }
            
            // Fallback to wmic
            let output = Command::new("wmic")
                .args(["ComputerSystem", "get", "TotalPhysicalMemory", "/Value"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("TotalPhysicalMemory=") {
                        let bytes: u64 = line
                            .trim_start_matches("TotalPhysicalMemory=")
                            .parse()
                            .unwrap_or(0);
                        return bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    }
                }
            }
            // Default fallback
            16.0
        }

        #[cfg(not(windows))]
        {
            // Unix-like systems
            let output = std::process::Command::new("free")
                .arg("-b")
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(bytes) = parts[1].parse::<u64>() {
                            return bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                        }
                    }
                }
            }
            8.0
        }
    }

    fn get_vram() -> Option<f64> {
        // Allow manual override via environment variable
        if let Ok(vram_str) = std::env::var("SELFIDEX_VRAM") {
            if let Ok(vram) = vram_str.parse::<f64>() {
                return Some(vram);
            }
        }
        
        #[cfg(windows)]
        {
            use std::process::Command;
            
            // Try nvidia-smi first (most accurate for NVIDIA GPUs)
            let output = Command::new("nvidia-smi")
                .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(mb) = stdout.trim().split('\n').next_back() {
                    if let Ok(vram_mb) = mb.trim().parse::<f64>() {
                        return Some(vram_mb / 1024.0); // Convert MB to GB
                    }
                }
            }
            
            // Try PowerShell as fallback
            let output = Command::new("powershell")
                .args(["-Command", "(Get-CimInstance Win32_VideoController).AdapterRAM"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                if let Ok(bytes) = trimmed.parse::<f64>() {
                    if bytes > 100_000_000.0 {
                        return Some(bytes / (1024.0 * 1024.0 * 1024.0));
                    }
                }
            }
            
            // Fallback to wmic
            let output = Command::new("wmic")
                .args(["path", "win32_VideoController", "get", "AdapterRAM", "/Value"])
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.starts_with("AdapterRAM=") {
                        let value = line.trim_start_matches("AdapterRAM=");
                        if let Ok(bytes) = value.parse::<u64>() {
                            if bytes > 0 {
                                return Some(bytes as f64 / (1024.0 * 1024.0 * 1024.0));
                            }
                        }
                    }
                }
            }
            None
        }

        #[cfg(not(windows))]
        {
            None
        }
    }

    fn get_cpu_cores() -> usize {
        env::var("NUMBER_OF_PROCESSORS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(4)
            })
    }

    fn get_os() -> String {
        #[cfg(windows)]
        return "Windows".to_string();
        
        #[cfg(target_os = "linux")]
        return "Linux".to_string();
        
        #[cfg(target_os = "macos")]
        return "macOS".to_string();
        
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        return "Unknown".to_string();
    }

    fn calculate_model_size(ram: f64, vram: Option<f64>) -> ModelSize {
        // Consider both RAM and VRAM for model loading
        // Local inference can use system RAM as backup
        let total_memory = ram + vram.unwrap_or(0.0);
        
        if total_memory >= 36.0 {
            ModelSize::ExtraLarge
        } else if total_memory >= 20.0 {
            ModelSize::Large
        } else if total_memory >= 10.0 {
            ModelSize::Medium
        } else if total_memory >= 4.0 {
            ModelSize::Small
        } else {
            ModelSize::Tiny
        }
    }

    pub fn display(&self) -> String {
        let vram_str = match self.vram_gb {
            Some(v) => format!("{:.1} GB", v),
            None => "No detectada".to_string(),
        };

        let size_str = match self.recommended_model_size {
            ModelSize::Tiny => "Tiny (1-2B) - < 2GB",
            ModelSize::Small => "Small (3B) - 2-4GB",
            ModelSize::Medium => "Medium (7B) - 4-8GB",
            ModelSize::Large => "Large (13B) - 8-16GB",
            ModelSize::ExtraLarge => "ExtraLarge (34B+) - 16GB+",
        };

        format!(
            r#"
=== Hardware Detectado ===
RAM Total:     {:.1} GB
VRAM:          {}
CPU Cores:     {}
OS:            {}
Modelo sugerido: {}
"#,
            self.total_ram_gb, vram_str, self.cpu_cores, self.os, size_str
        )
    }

    pub fn recommended_models(&self) -> Vec<ModelRecommendation> {
        self.recommended_model_size.recommended_models()
    }
}

impl ModelSize {
    pub fn recommended_models(&self) -> Vec<ModelRecommendation> {
        match self {
            ModelSize::Tiny => vec![
                ModelRecommendation {
                    name: "TinyLlama".to_string(),
                    params: "1.1B".to_string(),
                    size_gb: 0.7,
                    description: "Rápido, ideal para CPUs lentas".to_string(),
                    hf_repo: "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
                },
                ModelRecommendation {
                    name: "Phi-2".to_string(),
                    params: "2.7B".to_string(),
                    size_gb: 1.7,
                    description: "Microsoft, buen rendimiento".to_string(),
                    hf_repo: "microsoft/phi-2".to_string(),
                },
            ],
            ModelSize::Small => vec![
                ModelRecommendation {
                    name: "Mistral 3B".to_string(),
                    params: "3B".to_string(),
                    size_gb: 2.5,
                    description: "Excelente calidad/tamaño".to_string(),
                    hf_repo: "mistralai/Mistral-7B-v0.1-GGUF".to_string(),
                },
                ModelRecommendation {
                    name: "Neural Chat".to_string(),
                    params: "7B".to_string(),
                    size_gb: 4.0,
                    description: "Menos censurado, buen código".to_string(),
                    hf_repo: "TheBloke/Neural-Chat-7B-v3-GGUF".to_string(),
                },
            ],
            ModelSize::Medium => vec![
                ModelRecommendation {
                    name: "Mistral 7B".to_string(),
                    params: "7B".to_string(),
                    size_gb: 4.5,
                    description: "Equilibrio perfecto".to_string(),
                    hf_repo: "TheBloke/Mistral-7B-v0.1-GGUF".to_string(),
                },
                ModelRecommendation {
                    name: "Llama 2 7B".to_string(),
                    params: "7B".to_string(),
                    size_gb: 4.0,
                    description: "Clásico, muy estable".to_string(),
                    hf_repo: "TheBloke/Llama-2-7B-Chat-GGUF".to_string(),
                },
            ],
            ModelSize::Large => vec![
                ModelRecommendation {
                    name: "Yi 14B".to_string(),
                    params: "14B".to_string(),
                    size_gb: 8.0,
                    description: "Excelente código y reasoning".to_string(),
                    hf_repo: "TheBloke/Yi-14B-Chat-GGUF".to_string(),
                },
                ModelRecommendation {
                    name: "Mythomist 7B".to_string(),
                    params: "7B".to_string(),
                    size_gb: 4.2,
                    description: "SIN CENSURA, muy bueno".to_string(),
                    hf_repo: "TheBloke/Mythomist-7B-GGUF".to_string(),
                },
            ],
            ModelSize::ExtraLarge => vec![
                ModelRecommendation {
                    name: "Yi 34B".to_string(),
                    params: "34B".to_string(),
                    size_gb: 20.0,
                    description: "El mejor para código".to_string(),
                    hf_repo: "TheBloke/Yi-34B-Chat-GGUF".to_string(),
                },
                ModelRecommendation {
                    name: "Mixtral 8x7B".to_string(),
                    params: "8x7B".to_string(),
                    size_gb: 26.0,
                    description: "Mixture of Experts, excelente".to_string(),
                    hf_repo: "TheBloke/Mixtral-8x7B-Instruct-v0.1-GGUF".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub name: String,
    pub params: String,
    pub size_gb: f64,
    pub description: String,
    pub hf_repo: String,
}

/// Get the HuggingFace cache directory
pub fn get_huggingface_cache_dir() -> Option<PathBuf> {
    // Try multiple possible locations
    let possible_paths = vec![
        // Windows: LocalApplicationData\huggingface\hub
        dirs::data_local_dir().map(|p| p.join("huggingface").join("hub")),
        // Windows alternative: USERPROFILE\.cache\huggingface\hub
        std::env::var("USERPROFILE").ok().map(|h| PathBuf::from(h).join(".cache").join("huggingface").join("hub")),
        // Unix: ~/.cache/huggingface/hub
        dirs::cache_dir().map(|p| p.join("huggingface").join("hub")),
    ];
    
    for p in possible_paths.into_iter().flatten() {
        if p.exists() {
            return Some(p);
        }
    }
    
    // Return the most likely path anyway
    dirs::data_local_dir().map(|p| p.join("huggingface").join("hub"))
}

/// List installed models in HuggingFace cache
pub fn list_installed_models() -> Vec<InstalledModel> {
    let mut models = Vec::new();
    
    if let Some(cache_dir) = get_huggingface_cache_dir() {
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // Only show model directories
                    if name.starts_with("models--") {
                        let display_name = name
                            .trim_start_matches("models--")
                            .replace("--", "/");
                        
                        // Get size
                        let size = get_dir_size(&path);
                        
                        models.push(InstalledModel {
                            id: display_name.clone(),
                            name: display_name,
                            size_bytes: size,
                        });
                    }
                }
            }
        }
    }
    
    models.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    models
}

/// Remove a model by ID
pub fn remove_model(model_id: &str) -> Result<(), String> {
    let model_dir_name = model_id.replace("/", "--");
    let full_name = format!("models--{}", model_dir_name);
    
    if let Some(cache_dir) = get_huggingface_cache_dir() {
        let model_path = cache_dir.join(&full_name);
        
        if model_path.exists() {
            std::fs::remove_dir_all(&model_path)
                .map_err(|e| format!("Error al eliminar: {}", e))?;
            Ok(())
        } else {
            Err(format!("Modelo '{}' no encontrado", model_id))
        }
    } else {
        Err("No se encontró el directorio de cache".to_string())
    }
}

fn get_dir_size(path: &PathBuf) -> u64 {
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    if e.path().is_dir() {
                        get_dir_size(&e.path())
                    } else {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModel {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
}

impl InstalledModel {
    pub fn size_display(&self) -> String {
        let gb = self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if gb >= 1.0 {
            format!("{:.1} GB", gb)
        } else {
            let mb = self.size_bytes as f64 / (1024.0 * 1024.0);
            format!("{:.0} MB", mb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::format_size;

    #[test]
    fn test_system_info_detect() {
        let info = SystemInfo::detect();
        assert!(info.total_ram_gb > 0.0);
        assert!(info.cpu_cores > 0);
        assert!(!info.os.is_empty());
    }

    #[test]
    fn test_model_size_recommendations() {
        let tiny_models = ModelSize::Tiny.recommended_models();
        assert!(!tiny_models.is_empty());
        
        let small_models = ModelSize::Small.recommended_models();
        assert!(!small_models.is_empty());
        
        let medium_models = ModelSize::Medium.recommended_models();
        assert!(!medium_models.is_empty());
        
        let large_models = ModelSize::Large.recommended_models();
        assert!(!large_models.is_empty());
        
        let extra_large_models = ModelSize::ExtraLarge.recommended_models();
        assert!(!extra_large_models.is_empty());
    }

    #[test]
    fn test_calculate_model_size() {
        // Tiny: < 4GB
        assert_eq!(SystemInfo::calculate_model_size(2.0, None), ModelSize::Tiny);
        
        // Small: 4-10GB
        assert_eq!(SystemInfo::calculate_model_size(6.0, None), ModelSize::Small);
        
        // Medium: 10-20GB
        assert_eq!(SystemInfo::calculate_model_size(15.0, None), ModelSize::Medium);
        
        // Large: 20-36GB
        assert_eq!(SystemInfo::calculate_model_size(25.0, None), ModelSize::Large);
        
        // ExtraLarge: >= 36GB
        assert_eq!(SystemInfo::calculate_model_size(40.0, None), ModelSize::ExtraLarge);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_model_recommendation_structure() {
        let models = ModelSize::Tiny.recommended_models();
        let model = &models[0];
        
        assert!(!model.name.is_empty());
        assert!(!model.params.is_empty());
        assert!(model.size_gb > 0.0);
        assert!(!model.description.is_empty());
        assert!(!model.hf_repo.is_empty());
    }
}
