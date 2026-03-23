use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use clap::{Parser, Subcommand};
use selfidx::autonomous::run_autonomous_loop;
use selfidx::llm::{Message, JanClient};
use selfidx::utils::hardware::SystemInfo;
use selfidx::utils::format_size;
use selfidx::agent::Agent;
use selfidx::terminal::capsule::render_capsule;
use std::path::PathBuf;

/// Load project context from current directory
fn load_project_context() -> Result<String> {
    let mut info = String::new();
    let current_dir = std::env::current_dir()?;
    
    // Check for package.json (Node.js projects)
    if let Ok(content) = fs::read_to_string("package.json") {
        info.push_str("\n🔷 Proyecto Node.js detectado");
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                info.push_str(&format!(": {}", name));
            }
            if let Some(version) = data.get("version").and_then(|v| v.as_str()) {
                info.push_str(&format!(" v{}", version));
            }
        }
    } 
    // Check for Cargo.toml (Rust projects)
    else if let Ok(content) = fs::read_to_string("Cargo.toml") {
        info.push_str("\n🦀 Proyecto Rust detectado");
        if let Some(name) = content.lines().find(|l| l.starts_with("name =")) {
            info.push_str(&format!(": {}", name.trim_start_matches("name =").trim_matches('"')));
        }
        if let Some(version) = content.lines().find(|l| l.starts_with("version =")) {
            info.push_str(&format!(" v{}", version.trim_start_matches("version =").trim_matches('"')));
        }
    } 
    // Check for requirements.txt (Python projects)
    else if Path::new("requirements.txt").exists() {
        info.push_str("\n🐍 Proyecto Python detectado");
        let count = fs::read_to_string("requirements.txt")?.lines().count();
        info.push_str(&format!(" con {} dependencias", count));
    } 
    // Check for go.mod (Go projects)
    else if let Ok(content) = fs::read_to_string("go.mod") {
        info.push_str("\n💚 Proyecto Go detectado");
        if let Some(module) = content.lines().find(|l| l.starts_with("module ")) {
            info.push_str(&format!(": {}", module.trim_start_matches("module ")));
        }
    }
    
    // Check for .git directory
    if Path::new(".git").exists() {
        info.push_str("\n📊 Repositorio Git detectado");
    }
    
    // Count files and directories
    let mut file_count = 0;
    let mut dir_count = 0;
    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                dir_count += 1;
            } else {
                file_count += 1;
            }
        }
    }
    info.push_str(&format!("\n📁 {} archivos, {} carpetas", file_count, dir_count));
    
    Ok(info)
}

#[derive(Parser, Debug)]
#[command(name = "selfidx")]
#[command(author = "SELFIDEX")]
#[command(version = "3.0.0")]
#[command(about = "Terminal integrada con Jan.ai - SELFIDEX v3.0", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Modo TUI enriquecido
    #[arg(long)]
    tui: bool,

    /// Modo terminal plano
    #[arg(long)]
    plain: bool,

    /// Instalar selfidx en PATH
    #[arg(long)]
    install: bool,

    /// Modelo LLM a usar (ej: llama2, gpt-4)
    #[arg(long, short = 'm')]
    model: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Modo Agente AI
    Agent {
        #[arg(trailing_var_arg(true))]
        prompt: Vec<String>,
    },

    /// Chat interactivo
    Chat,

    /// Ejecutar comando
    Command {
        cmd: String,
    },

    /// Ejecutar proyecto
    Run,

    /// Crear archivo
    Create {
        path: String,
    },

    /// Editar archivo
    Edit {
        path: String,
    },

    /// Listar archivos
    Files,

    /// Comparar archivos
    Diff {
        file_a: String,
        file_b: String,
    },

    /// Ver versión
    Version,

    /// Listar modelos recomendados
    Models,

    /// Usar un modelo específico
    UseModel {
        model: String,
    },

    /// Descargar modelo
    Download {
        model: String,
    },

    /// Info del sistema
    SysInfo,

    /// Listar modelos instalados
    ListInstalled,

    /// Eliminar modelo
    RemoveModel {
        model: String,
    },

    /// Leer archivo
    Read {
        path: String,
    },

    /// Ejecutar comando del sistema
    Exec {
        cmd: String,
    },

    /// Buscar en archivos
    Search {
        pattern: String,
    },

    /// Ver estructura del proyecto
    Tree,

    /// UI con terminal integrada
    Tui,

    /// Modo autónomo - planificación
    Plan {
        #[arg(trailing_var_arg(true))]
        task: Vec<String>,
    },

    /// Modo autónomo
    Auto {
        #[arg(trailing_var_arg(true))]
        task: Vec<String>,
    },

    /// Modo Vibe Coding - Estilo codificación dinámica
    VibeCode {
        #[arg(trailing_var_arg(true))]
        task: Vec<String>,
    },
}

/// Check if selfidx is in PATH and offer auto-install
fn check_and_offer_install() -> Result<bool> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    
    // Check if we're in a temporary location (downloaded but not installed)
    let is_temp = exe_dir.to_string_lossy().to_lowercase().contains("temp") ||
                  exe_dir.to_string_lossy().to_lowercase().contains("tmp") ||
                  exe_dir.to_string_lossy().to_lowercase().contains("downloads");
    
    if !is_temp {
        return Ok(false); // Already in a good location
    }
    
    // Check if already installed
    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("selfidx");
    
    let installed_exe = install_dir.join("selfidx.exe");
    if installed_exe.exists() {
        return Ok(false); // Already installed
    }
    
    // Offer auto-install
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  SELFIDEX no está instalado en tu sistema                 ║");
    println!("║  ¿Deseas instalarlo automáticamente?                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Ubicación actual: {}", exe_dir.display());
    println!("  Se instalará en: {}", install_dir.display());
    println!();
    print!("  ¿Instalar? (s/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "s" {
        println!();
        println!("[selfidx] Instalando...");
        
        // Create install directory
        fs::create_dir_all(&install_dir)?;
        
        // Copy executable
        let target_exe = install_dir.join("selfidx.exe");
        fs::copy(&exe_path, &target_exe)?;
        println!("[selfidx] ✓ Ejecutable copiado");
        
        // Add to PATH
        #[cfg(windows)]
        {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let install_dir_str = install_dir.to_string_lossy().to_string();
            
            if !current_path.contains(&install_dir_str) {
                let new_path = format!("{};{}", current_path, install_dir_str);
                let _ = std::process::Command::new("setx")
                    .args(["PATH", &new_path])
                    .output();
                println!("[selfidx] ✓ Agregado al PATH");
            }
        }
        
        println!();
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║  ✓ INSTALACIÓN COMPLETADA                                 ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("  SELFIDEX se instaló en: {}", install_dir.display());
        println!();
        println!("  Para usar, abre una NUEVA terminal y ejecuta:");
        println!("    selfidx --help");
        println!();
        println!("  NOTA: Si el comando no funciona, reinicia tu terminal.");
        println!();
        
        // Ask if user wants to continue with the installed version
        print!("  ¿Ejecutar SELFIDEX ahora? (s/n): ");
        io::stdout().flush()?;
        
        let mut input2 = String::new();
        io::stdin().read_line(&mut input2)?;
        
        if input2.trim().to_lowercase() == "s" {
            // Run the installed version
            let _ = std::process::Command::new(&target_exe)
                .args(std::env::args().skip(1))
                .status();
            std::process::exit(0);
        } else {
            std::process::exit(0);
        }
    }
    
    Ok(true)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --install flag
    if args.install {
        install_selfidx()?;
        return Ok(());
    }

    // Check and offer auto-install (only if running from temp location)
    if let Ok(true) = check_and_offer_install() {
        return Ok(());
    }

    // Render cápsula
    println!("{}", render_capsule());
    println!();
    
    // Show project context
    let current_dir = std::env::current_dir().expect("No se puede obtener el directorio actual");
    println!("[selfidx] 📂 Directorio de trabajo: {}", current_dir.display());
    
    // Load and display project context
    if let Ok(project_info) = load_project_context() {
        println!("{}", project_info);
    }
    
    // Verify Jan.ai connectivity
    let client = JanClient::from_env();
    if client.is_available().await {
        println!("[selfidx] ✅ Jan.ai conectado en http://localhost:1337");
        
        // List available models if connected
        match client.list_models().await {
            Ok(models) => {
                println!("[selfidx] 📦 {} modelos disponibles en Jan.ai", models.len());
            }
            Err(e) => {
                println!("[selfidx] ⚠️ No se pueden listar los modelos: {}", e);
            }
        }
    } else {
        println!("[selfidx] ❌ Jan.ai no está disponible en http://localhost:1337");
        println!("[selfidx] ℹ️ Inicia Jan.ai con: ollama serve");
    }
    println!();

    // Determine which model to use
    let mut model = if let Some(m) = args.model {
        m
    } else {
        // If no model specified via command-line, let user select from available models
        println!("[selfidx] ⚙️ No se especificó un modelo. Selecciona uno de la lista:\n");
        let client = JanClient::from_env();
        
        match client.select_model_interactively().await {
            Ok(selected) => selected,
            Err(e) => {
                println!("[selfidx-error] Error al listar modelos: {}. Usando predeterminado.", e);
                JanClient::default_model()
            }
        }
    };

    // Match commands
    match args.command {
        Some(Commands::Agent { prompt }) => {
            let prompt = prompt.join(" ");
            if prompt.is_empty() {
                println!("[selfidx-agent] Modo Agente IA Autonomo");
                println!("================================\n");
                println!("Usa: selfidx agent <tu pregunta o tarea>");
                return Ok(());
            }
            
            println!("[selfidx-agent] Modo Agente IA Autonomo");
            println!("================================\n");
            println!("Tarea: {}\n", prompt);
            println!("Modelo: {}\n", model);
            
            let agent = Agent::new();
            let client = JanClient::from_env();
            
            if !client.is_available().await {
    println!("[selfidx-error] Jan.ai no está disponible.");
    println!("[selfidx-error] Inicia Jan.ai en http://localhost:1337");
    return Ok(());
}
            
            let project_tree = agent.get_project_tree(3).unwrap_or_default();
            
            let full_prompt = format!(
                "Eres un asistente de programación experto. \
                 \n\n=== ESTRUCTURA DEL PROYECTO ===\n{}\n\n=== PETICIÓN DEL USUARIO ===\n{}",
                project_tree, prompt
            );
            
            println!("Enviando petición a IA...");
            
            let messages = vec![
                Message {
                    role: "system".to_string(),
                    content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                    Ejecuta acciones directamente para completar las tareas del usuario. \
                    No pidas permiso - simplemente hazlo. \
                    Cuando debas crear archivos, ejecuta los comandos necesarios. \
                    Cuando debas ejecutar algo, hazlo. \
                    Responde siempre en español de forma clara y concisa.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: full_prompt,
                },
            ];
            
            match client.chat(model, messages).await {
                Ok(response) => {
                    println!("\n=== RESPUESTA ===\n");
                    println!("{}", response.content());
                }
                Err(e) => {
                    println!("[selfidx-error] Error: {}", e);
                }
            }
        }
        
        Some(Commands::Chat) => {
            println!("[selfidx-chat] 🤖Modo chat interactivo - Escribe tus mensajes (escribe 'salir' para terminar)\n");
            
            let client = JanClient::from_env();
            
            println!("[selfidx-chat] Conectando a Jan.ai...");
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible.");
                return Ok(());
            }
            
            println!("[selfidx-chat] Modelo: {}", model);
            println!();
            
            // Chat history
            let mut messages = vec![
                Message {
                    role: "system".to_string(),
                    content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                    Ejecuta acciones directamente para completar las tareas del usuario. \
                    Cuando debas crear archivos, usa los comandos necesarios. \
                    Cuando debas ejecutar algo, hazlo. \
                    Responde siempre en español de forma clara y concisa.".to_string(),
                },
            ];
            
            // Interactive loop
            use std::io::{self, Write};
            
            loop {
                print!("\n> ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    break;
                }
                
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                
                if input == "salir" || input == "exit" || input == "quit" {
                    println!("\n👋 Hasta luego!");
                    break;
                }
                
                if input == "limpiar" || input == "clear" {
                    messages = vec![
                        Message {
                            role: "system".to_string(),
                            content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                            Ejecuta acciones directamente para completar las tareas del usuario. \
                            Cuando debas crear archivos, usa los comandos necesarios. \
                            Cuando debas ejecutar algo, hazlo. \
                            Responde siempre en español de forma clara y concisa.".to_string(),
                        },
                    ];
                    println!("✅ Chat limpiado");
                    continue;
                }
                
                messages.push(Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                });
                
                match client.chat(model.clone(), messages.clone()).await {
                    Ok(response) => {
                        let content = response.content().to_string();
                        println!("\n🤖: {}", content);
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content,
                        });
                    }
                    Err(e) => {
                        println!("[selfidx-error] Error: {}", e);
                    }
                }
            }
        }
        
        Some(Commands::Command { cmd }) => {
            let agent = Agent::new();
            println!("[selfidx] Ejecutando: {}", cmd);
            
            match agent.execute_command(&cmd) {
                Ok(result) => {
                    if !result.stdout.is_empty() {
                        println!("\n=== STDOUT ===\n{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        println!("\n=== STDERR ===\n{}", result.stderr);
                    }
                    println!("\nExit code: {}", result.exit_code);
                }
                Err(e) => {
                    println!("[selfidx-error] Error al ejecutar: {}", e);
                }
            }
        }
        
        Some(Commands::Run) => {
            let agent = Agent::new();
            println!("[selfidx] Ejecutando proyecto...\n");
            
            // Detect project type and run accordingly
            let current_dir = std::env::current_dir()?;
            
            if current_dir.join("package.json").exists() {
                println!("[selfidx] Proyecto Node.js detectado");
                println!("[selfidx] Ejecutando: npm start\n");
                match agent.execute_command("npm start") {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            println!("{}", result.stdout);
                        }
                        if !result.stderr.is_empty() {
                            println!("{}", result.stderr);
                        }
                    }
                    Err(e) => println!("[selfidx-error] {}", e),
                }
            } else if current_dir.join("Cargo.toml").exists() {
                println!("[selfidx] Proyecto Rust detectado");
                println!("[selfidx] Ejecutando: cargo run\n");
                match agent.execute_command("cargo run") {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            println!("{}", result.stdout);
                        }
                        if !result.stderr.is_empty() {
                            println!("{}", result.stderr);
                        }
                    }
                    Err(e) => println!("[selfidx-error] {}", e),
                }
            } else if current_dir.join("requirements.txt").exists() {
                println!("[selfidx] Proyecto Python detectado");
                println!("[selfidx] Ejecutando: python main.py\n");
                match agent.execute_command("python main.py") {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            println!("{}", result.stdout);
                        }
                        if !result.stderr.is_empty() {
                            println!("{}", result.stderr);
                        }
                    }
                    Err(e) => println!("[selfidx-error] {}", e),
                }
            } else if current_dir.join("go.mod").exists() {
                println!("[selfidx] Proyecto Go detectado");
                println!("[selfidx] Ejecutando: go run .\n");
                match agent.execute_command("go run .") {
                    Ok(result) => {
                        if !result.stdout.is_empty() {
                            println!("{}", result.stdout);
                        }
                        if !result.stderr.is_empty() {
                            println!("{}", result.stderr);
                        }
                    }
                    Err(e) => println!("[selfidx-error] {}", e),
                }
            } else {
                println!("[selfidx] No se detectó un proyecto conocido");
                println!("[selfidx] Tipos soportados: Node.js, Rust, Python, Go");
            }
        }
        
        Some(Commands::Create { path }) => {
            let agent = Agent::new();
            println!("[selfidx-create] Creando archivo: {}", path);
            
            let content = "# Nuevo archivo\n";
            
            match agent.write_file(&path, content) {
                Ok(()) => {
                    println!("✅ Archivo '{}' creado correctamente.", path);
                }
                Err(e) => {
                    println!("[selfidx-error] Error al crear archivo: {}", e);
                }
            }
        }
        
        Some(Commands::Edit { path }) => {
            let agent = Agent::new();
            println!("[selfidx] Editando archivo: {}\n", path);
            
            // Check if file exists
            if !std::path::Path::new(&path).exists() {
                println!("[selfidx] El archivo '{}' no existe.", path);
                println!("[selfidx] Usa 'selfidx --create {}' para crearlo.", path);
                return Ok(());
            }
            
            // Read current content
            match agent.read_file(&path) {
                Ok(content) => {
                    println!("=== Contenido actual de {} ===\n", path);
                    println!("{}", content);
                    println!("\n=== Fin del archivo ===\n");
                    
                    // Try to open with system editor
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("notepad")
                            .arg(&path)
                            .spawn();
                        println!("[selfidx] Abriendo con Notepad...");
                    }
                    
                    #[cfg(not(windows))]
                    {
                        let _ = std::process::Command::new("nano")
                            .arg(&path)
                            .spawn();
                        println!("[selfidx] Abriendo con nano...");
                    }
                }
                Err(e) => {
                    println!("[selfidx-error] Error al leer archivo: {}", e);
                }
            }
        }
        
        Some(Commands::Files) => {
            let agent = Agent::new();
            println!("[selfidx] Listando archivos en directorio actual:\n");
            
            match agent.list_files(".") {
                Ok(files) => {
                    let total = files.len();
                    for file in files {
                        let icon = if file.is_dir { "📁" } else { "📄" };
                        let size = if file.is_dir {
                            String::new()
                        } else {
                            format!(" ({})", format_size(file.size))
                        };
                        println!("{} {}{}", icon, file.name, size);
                    }
                    println!("\nTotal: {} elementos", total);
                }
                Err(e) => {
                    println!("[selfidx-error] Error al listar archivos: {}", e);
                }
            }
        }
        
        Some(Commands::Diff { file_a, file_b }) => {
            let agent = Agent::new();
            println!("[selfidx] Comparando archivos:\n");
            println!("  Archivo A: {}", file_a);
            println!("  Archivo B: {}\n", file_b);
            
            // Read both files
            let content_a = match agent.read_file(&file_a) {
                Ok(content) => content,
                Err(e) => {
                    println!("[selfidx-error] Error al leer {}: {}", file_a, e);
                    return Ok(());
                }
            };
            
            let content_b = match agent.read_file(&file_b) {
                Ok(content) => content,
                Err(e) => {
                    println!("[selfidx-error] Error al leer {}: {}", file_b, e);
                    return Ok(());
                }
            };
            
            // Simple diff: compare line by line
            let lines_a: Vec<&str> = content_a.lines().collect();
            let lines_b: Vec<&str> = content_b.lines().collect();
            
            let max_lines = lines_a.len().max(lines_b.len());
            let mut differences = 0;
            
            for i in 0..max_lines {
                let line_a = lines_a.get(i).unwrap_or(&"");
                let line_b = lines_b.get(i).unwrap_or(&"");
                
                if line_a != line_b {
                    differences += 1;
                    println!("Línea {}:", i + 1);
                    println!("  - {}", line_a);
                    println!("  + {}", line_b);
                    println!();
                }
            }
            
            if differences == 0 {
                println!("✅ Los archivos son idénticos");
            } else {
                println!("📊 Total de diferencias: {}", differences);
            }
        }
        
        Some(Commands::Version) => {
            println!("SELFIDEX v3.0.0");
        }
        
        Some(Commands::Models) => {
            println!("=== Modelos Recomendados ===\n");
            let sys = SystemInfo::detect();
            println!("{}", sys.display());
            
            let models = sys.recommended_model_size.recommended_models();
            println!("=== Modelos para tu hardware ===\n");
            for (i, model) in models.iter().enumerate() {
                println!("{}. {} ({} params, ~{}GB)", i + 1, model.name, model.params, model.size_gb);
                println!("   {}", model.description);
                println!("   Repo: {}", model.hf_repo);
                println!();
            }
            println!("Usa: selfidx --download <nombre>");
            println!("Usa: selfidx --use-model <nombre>");
        }
        
        Some(Commands::UseModel { model }) => {
            println!("[selfidx] Cambiando modelo a: {}\n", model);
            
            // Verify Jan.ai is available
            let client = JanClient::from_env();
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible en http://localhost:1337");
                println!("[selfidx] Inicia Jan.ai para cambiar de modelo");
                return Ok(());
            }
            
            // List available models
            match client.list_models().await {
                Ok(models) => {
                    if models.is_empty() {
                        println!("[selfidx] No hay modelos disponibles en Jan.ai");
                        return Ok(());
                    }
                    
                    // Check if model exists
                    if models.contains(&model) {
                        println!("✅ Modelo '{}' encontrado en Jan.ai", model);
                        println!("\n💡 Para usar este modelo, ejecuta:");
                        println!("   selfidx --model {}", model);
                        println!("\nO en modo interactivo:");
                        println!("   /model");
                    } else {
                        println!("[selfidx] Modelo '{}' no encontrado en Jan.ai", model);
                        println!("\nModelos disponibles:");
                        for (i, m) in models.iter().enumerate() {
                            println!("  {}. {}", i + 1, m);
                        }
                    }
                }
                Err(e) => {
                    println!("[selfidx-error] Error al listar modelos: {}", e);
                }
            }
        }
        
        Some(Commands::Download { model }) => {
            println!("[selfidx-download] Descargando modelo: {}", model);
            
            let sys = SystemInfo::detect();
            let models = sys.recommended_model_size.recommended_models();
            
            let model_repo = models.iter().find(|m| m.name.to_lowercase().contains(&model.to_lowercase()));
            
            match model_repo {
                Some(repo) => {
                    println!("\nDescargando {} ...", repo.hf_repo);
                    println!("\n=== INSTRUCCIONES ===");
                    println!("Ejecuta en terminal separada:");
                    println!("  Jan.ai ya incluye modelos locales");
                }
                None => {
                    println!("[selfidx-error] Modelo '{}' no encontrado", model);
                }
            }
        }
        
        Some(Commands::SysInfo) => {
            let sys = SystemInfo::detect();
            println!("{}", sys.display());
        }
        
        Some(Commands::ListInstalled) => {
            use selfidx::utils::hardware::list_installed_models;
            
            println!("[selfidx] Listando modelos instalados...\n");
            
            // First, try to list models from Jan.ai
            let client = JanClient::from_env();
            if client.is_available().await {
                println!("=== Modelos en Jan.ai ===\n");
                match client.list_models().await {
                    Ok(models) => {
                        if models.is_empty() {
                            println!("No hay modelos en Jan.ai.");
                        } else {
                            for (i, model) in models.iter().enumerate() {
                                println!("{}. {}", i + 1, model);
                            }
                            println!("\nTotal: {} modelos en Jan.ai", models.len());
                        }
                    }
                    Err(e) => {
                        println!("[selfidx-error] Error al listar modelos de Jan.ai: {}", e);
                    }
                }
                println!();
            } else {
                println!("[selfidx] ⚠️ Jan.ai no está disponible en http://localhost:1337");
                println!("[selfidx] 💡 Inicia Jan.ai para ver modelos descargados\n");
            }
            
            // Also list models from HuggingFace cache
            println!("=== Modelos en Cache Local (HuggingFace) ===\n");
            let models = list_installed_models();
            
            if models.is_empty() {
                println!("No hay modelos en cache local.");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("{}. {}", i + 1, model.name);
                    println!("   Tamaño: {}", model.size_display());
                    println!();
                }
                println!("Total: {} modelos en cache", models.len());
            }
        }
        
        Some(Commands::RemoveModel { model }) => {
            use selfidx::utils::hardware::remove_model;
            
            println!("[selfidx] Eliminando modelo: {}", model);
            
            match remove_model(&model) {
                Ok(()) => {
                    println!("✅ Modelo '{}' eliminado correctamente.", model);
                }
                Err(e) => {
                    println!("[selfidx-error] {}", e);
                }
            }
        }
        
        Some(Commands::Read { path }) => {
            let agent = Agent::new();
            match agent.read_file(&path) {
                Ok(content) => {
                    println!("=== {} ===\n", path);
                    println!("{}", content);
                }
                Err(e) => {
                    println!("[selfidx-error] Error al leer archivo: {}", e);
                }
            }
        }
        
        Some(Commands::Exec { cmd }) => {
            let agent = Agent::new();
            println!("[selfidx] Ejecutando: {}", cmd);
            
            match agent.execute_command(&cmd) {
                Ok(result) => {
                    if !result.stdout.is_empty() {
                        println!("\n=== STDOUT ===\n{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        println!("\n=== STDERR ===\n{}", result.stderr);
                    }
                    println!("\nExit code: {}", result.exit_code);
                }
                Err(e) => {
                    println!("[selfidx-error] Error al ejecutar: {}", e);
                }
            }
        }
        
        Some(Commands::Search { pattern }) => {
            let agent = Agent::new();
            println!("[selfidx] Buscando: {}\n", pattern);
            
            match agent.search(&pattern, None) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No se encontraron resultados.");
                    } else {
                        println!("=== Resultados ({}) ===\n", results.len());
                        for result in results.iter().take(20) {
                            println!("{}:{}", result.file, result.line);
                            println!("  {}", result.content);
                        }
                    }
                }
                Err(e) => {
                    println!("[selfidx-error] Error al buscar: {}", e);
                }
            }
        }
        
        Some(Commands::Tree) => {
            let agent = Agent::new();
            match agent.get_project_tree(4) {
                Ok(tree) => {
                    println!("=== Estructura del Proyecto ===\n");
                    print!("{}", tree);
                }
                Err(e) => {
                    println!("[selfidx-error] Error: {}", e);
                }
            }
        }
        
        Some(Commands::Tui) => {
            println!("[selfidx] 🎨 Iniciando UI con terminal integrada...");
            
            // Run TUI
            if let Err(e) = selfidx::tui::run_tui() {
                println!("[selfidx-error] Error en TUI: {}", e);
            }
        }
        
        Some(Commands::Auto { task }) => {
            let task_str = task.join(" ");
            if task_str.is_empty() {
                println!("[selfidx-auto] Modo autonomo. Usa: selfidx --auto <tarea>");
                return Ok(());
            }
            
            println!("═══════════════════════════════════════════");
            println!("   🤖 SELFIDEX - MODO AUTÓNOMO");
            println!("═══════════════════════════════════════════\n");
            println!("Tarea: {}\n", task_str);
            println!("Modelo: {}\n", model);
            
            let agent = Agent::new();
            let client = JanClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible.");
                println!("Inicia Jan.ai desde la aplicación");
                return Ok(());
            }
            
            let project_tree = agent.get_project_tree(3).unwrap_or_default();
            
            let auto_prompt = format!(
                "Eres un asistente de programación AUTÓNOMO (estilo Claude Code). Tu objetivo es completar la tarea del usuario de forma independiente.\n\n=== PROYECTO ===\n{}\n\n=== TAREA ===\n{}\n\n{}",
                project_tree, 
                task_str,
                Agent::get_tools_description()
            );
            
            // Interactive autonomous loop
            run_autonomous_loop(&client, &model, &agent, &auto_prompt).await?;
        }
        
        Some(Commands::VibeCode { task }) => {
            let task_str = task.join(" ");
            if task_str.is_empty() {
                println!("[selfidx-vibecode] Modo Vibe Coding. Usa: selfidx vibecode <tarea>");
                return Ok(());
            }
            
            println!("═══════════════════════════════════════════");
            println!("   🎨 SELFIDEX - VIBE CODING");
            println!("═══════════════════════════════════════════\n");
            println!("Tarea: {}\n", task_str);
            println!("Modelo: {}\n", model);
            
            let agent = Agent::new();
            let client = JanClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible.");
                println!("Inicia Jan.ai desde la aplicación");
                return Ok(());
            }
            
            let project_tree = agent.get_project_tree(3).unwrap_or_default();
            
            let vibecode_prompt = format!(
                "Eres un asistente de programación VIBE CODING. Tu estilo es dinámico, creativo y enfocado en resultados rápidos.\n\n=== PROYECTO ===\n{}\n\n=== TAREA ===\n{}\n\n{}",
                project_tree, 
                task_str,
                Agent::get_tools_description()
            );
            
            // Interactive autonomous loop for vibe coding
            run_autonomous_loop(&client, &model, &agent, &vibecode_prompt).await?;
        }
        
        Some(Commands::Plan { task }) => {
            let task_str = task.join(" ");
            if task_str.is_empty() {
                println!("[selfidx-plan] Modo planificación. Usa: selfidx plan <tarea>");
                return Ok(());
            }
            
            println!("═══════════════════════════════════════════");
            println!("   📋 MODO PLANIFICACIÓN");
            println!("═══════════════════════════════════════════\n");
            println!("Tarea: {}\n", task_str);
            println!("Modelo: {}\n", model);
            
            let agent = Agent::new();
            let client = JanClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible.");
                println!("Inicia Jan.ai desde la aplicación");
                return Ok(());
            }
            
            let project_tree = agent.get_project_tree(3).unwrap_or_default();
            
            let plan_prompt = format!(
                "Eres un asistente de planificación de código. Tu tarea es ANALIZAR la tarea y crear un PLAN detallado de los pasos a seguir, SIN ejecutar nada.\n\n=== PROYECTO ===\n{}\n\n=== TAREA ===\n{}\n\nINSTRUCCIONES:\n1. Analiza el código existente\n2. Identifica qué archivos necesitan modificarse\n3. Describe los pasos exactos para completar la tarea\n4. NO escribas código, solo describe qué harías\n\nResponde en español con un plan detallado.",
                project_tree, task_str
            );
            
            let messages = vec![
                Message {
                    role: "system".to_string(),
                    content: "Eres un asistente de planificación. Analiza y planifica, no ejecutes.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: plan_prompt,
                },
            ];
            
            match client.chat(model, messages).await {
                Ok(response) => {
                    println!("\n=== PLAN DE ACCIÓN ===\n");
                    println!("{}", response.content());
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("Para ejecutar este plan, usa:");
                    println!("  selfidx --auto \"{}\"", task_str);
                    println!("═══════════════════════════════════════════");
                }
                Err(e) => {
                    println!("[selfidx-error] Error: {}", e);
                }
            }
        }
        
        None => {
            // Check if Jan.ai is available
            let client = JanClient::from_env();
            
            println!("[selfidx] 🔍 Verificando Jan.ai...");
            
            if !client.is_available().await {
                println!("[selfidx] ⚠️ Jan.ai no está corriendo");
                println!("[selfidx] 💡 Para iniciar Jan.ai, abre la aplicación desde el menú");
                println!("[selfidx]    Jan.ai se ejecuta en http://localhost:1337");
                return Ok(());
            } else {
                println!("[selfidx] ✅ Jan.ai ya está corriendo");
            }
            
            // Default: Start interactive chat mode like Claude/Codex
            println!("═══════════════════════════════════════════");
            println!("   🤖 SELFIDEX - Chat Interactivo");
            println!("═══════════════════════════════════════════\n");
            println!("Comandos: 'salir' para terminar, 'limpiar' para borrar historial\n");
            
            let client = JanClient::from_env();
            
            println!("[selfidx] Conectando a vLLM...");
            if !client.is_available().await {
                println!("[selfidx-error] Jan.ai no está disponible.");
                println!("[selfidx] Inicia Jan.ai desde la aplicación");
                return Ok(());
            }
            
            println!("[selfidx] Modelo: {} ✓\n", model);
            
            // Create logs directory
            let log_dir = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("selfidx")
                .join("logs");
            let _ = std::fs::create_dir_all(&log_dir);
            
            // Log file with timestamp
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let log_file = log_dir.join(format!("session_{}.md", timestamp));
            
            // Chat history
            let mut messages = vec![
                Message {
                    role: "system".to_string(),
                    content: "Eres un asistente de programación AUTÓNOMO llamado SELFIDEX. Tienes acceso completo al sistema de archivos y terminal. \
                    Ejecuta acciones directamente para completar las tareas del usuario. \
                    Cuando debas crear archivos, usa los comandos necesarios. \
                    Cuando debas ejecutar algo, hazlo. \
                    Responde siempre en español de forma clara y concisa. \
                    Mantén un registro de tus acciones en tus respuestas.".to_string(),
                },
            ];
            
            // Save session start to log
            let session_start = format!("# SELFIDEX Session - {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            let _ = std::fs::write(&log_file, &session_start);
            
            println!("📝 Sesión guardada en: {}\n", log_file.display());
            
            // Interactive loop
            use std::io::{self, Write};
            
            loop {
                print!("❯ ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err() {
                    break;
                }
                
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                
                // Autocomplete with --
                if input == "--" {
                    println!("\n📚 Comandos disponibles:");
                    println!("  --agent <prompt>     Modo Agente AI");
                    println!("  --chat               Chat interactivo");
                    println!("  --auto <tarea>       Modo autónomo");
                    println!("  --plan <tarea>       Modo planificación");
                    println!("  --vibecode <tarea>   Modo Vibe Coding");
                    println!("  --tui                Interfaz gráfica");
                    println!("  --models             Ver modelos recomendados");
                    println!("  --use-model <nombre> Usar un modelo específico");
                    println!("  --list-installed     Ver modelos instalados");
                    println!("  --sysinfo            Info del sistema");
                    println!("  --create <ruta>      Crear archivo");
                    println!("  --edit <ruta>        Editar archivo");
                    println!("  --files              Listar archivos");
                    println!("  --diff <a> <b>       Comparar archivos");
                    println!("  --read <archivo>     Leer archivo");
                    println!("  --exec <comando>     Ejecutar comando");
                    println!("  --search <patrón>    Buscar en archivos");
                    println!("  --tree               Ver estructura");
                    println!("  --install            Instalar en PATH");
                    println!("  --version            Ver versión");
                    println!("  --help               Ver ayuda");
                    println!("\n💡 También puedes usar: /model, /clear, /info, /help, /exit");
                    continue;
                }
                
                // Command palette commands
                if input.starts_with("/") {
                    let cmd = input.trim_start_matches("/").to_lowercase();
                    
                    match cmd.as_str() {
                        "model" => {
                            println!("\n🔄 Selecciona un nuevo modelo:");
                            let client = JanClient::from_env();
                            if let Ok(selected) = client.select_model_interactively().await {
                                model = selected;
                                println!("\n✅ Modelo cambiado a: {}", model);
                            }
                            continue;
                        }
                        
                        "clear" | "limpiar" => {
                            messages = vec![
                                Message {
                                    role: "system".to_string(),
                                    content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                                    Ejecuta acciones directamente para completar las tareas del usuario.".to_string(),
                                },
                            ];
                            println!("✅ Chat limpiado");
                            continue;
                        }
                        
                        "info" => {
                            println!("\nℹ️ Información del proyecto:");
                            println!("📂 Directorio: {}", std::env::current_dir().unwrap().display());
                            if let Ok(info) = load_project_context() {
                                println!("{}", info);
                            }
                            println!("🤖 Modelo actual: {}", model);
                            println!("🌐 Jan.ai: http://localhost:1337");
                            continue;
                        }
                        
                        "exit" | "quit" | "salir" => {
                            // Save final log
                            let log_content = format!("{}# Sesión terminada\n", 
                                messages.iter().map(|m| format!("## {}:\n{}\n", m.role, m.content)).collect::<String>());
                            let _ = std::fs::write(&log_file, &log_content);
                            println!("\n📝 Registro guardado en: {}", log_file.display());
                            println!("\n👋 Hasta luego!");
                            break;
                        }
                        
                        "help" => {
                            println!("\n📚 Comandos disponibles:");
                            println!("/model   - Cambiar modelo LLM");
                            println!("/clear   - Limpiar chat");
                            println!("/info    - Ver información del proyecto");
                            println!("/help    - Mostrar esta ayuda");
                            println!("/exit    - Salir de SELFIDEX");
                            continue;
                        }
                        
                        _ => {
                            println!("\n❓ Comando desconocido: {}. Usa /help para ver comandos disponibles.", cmd);
                            continue;
                        }
                    }
                }
                
                // Traditional commands
                if input == "salir" || input == "exit" || input == "quit" {
                    // Save final log
                    let log_content = format!("{}# Sesión terminada\n", 
                        messages.iter().map(|m| format!("## {}:\n{}\n", m.role, m.content)).collect::<String>());
                    let _ = std::fs::write(&log_file, &log_content);
                    println!("\n📝 Registro guardado en: {}", log_file.display());
                    println!("\n👋 Hasta luego!");
                    break;
                }
                
                if input == "limpiar" || input == "clear" {
                    messages = vec![
                        Message {
                            role: "system".to_string(),
                            content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                            Ejecuta acciones directamente para completar las tareas del usuario.".to_string(),
                        },
                    ];
                    println!("✅ Chat limpiado");
                    continue;
                }
                
                // Save user input to log
                let _ = std::fs::write(&log_file, format!("{}## User:\n{}\n", 
                    std::fs::read_to_string(&log_file).unwrap_or_default(), input));
                
                messages.push(Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                });
                
                print!("\n🤖 Thinking...\n");
                
                match client.chat(model.clone(), messages.clone()).await {
                    Ok(response) => {
                        let content = response.content().to_string();
                        println!("\n🤖:\n{}", content);
                        
                        // Save AI response to log
                        let _ = std::fs::write(&log_file, format!("{}## Assistant:\n{}\n", 
                            std::fs::read_to_string(&log_file).unwrap_or_default(), content));
                        
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content,
                        });
                    }
                    Err(e) => {
                        println!("[selfidx-error] Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn print_help() {
    println!(
        r#"
SELFIDEX v3.0 - Asistente de Programación IA

USO:
    selfidx [COMANDO]

MODO AGENTE (IA):
    selfidx --agent [prompt]     Preguntar a la IA sobre el proyecto
    selfidx --chat               Chat interactivo

GESTIÓN DE MODELOS:
    selfidx --models             Ver modelos recomendados
    selfidx --download <modelo>  Descargar modelo
    selfidx --list-installed     Ver modelos instalados
    selfidx --remove-model <nombre>  Eliminar modelo

OPERACIONES DE ARCHIVOS:
    selfidx --read <archivo>     Leer un archivo
    selfidx --tree               Ver estructura del proyecto
    selfidx --search <patrón>    Buscar en archivos
    selfidx --create <ruta>      Crear archivo
    selfidx -c "comando"         Ejecutar comando
    selfidx --exec <comando>     Ejecutar comando del sistema
    selfidx --files              Listar archivos

SISTEMA:
    selfidx --sysinfo           Info del sistema
    selfidx --install            Instalar en PATH
    selfidx --version            Ver versión
    selfidx --help               Ver esta ayuda
"#
    );
}

fn install_selfidx() -> Result<()> {
    use std::fs;

    println!("[selfidx-install] Instalando SELFIDEX v3.0...");

    let exe_path = std::env::current_exe()
        .context("No se pudo obtener la ruta del ejecutable")?;

    let install_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("selfidx");

    fs::create_dir_all(&install_dir)
        .context("No se pudo crear el directorio de instalación")?;

    let target_path = install_dir.join("selfidx.exe");
    fs::copy(&exe_path, &target_path)
        .context("No se pudo copiar el ejecutable")?;

    #[cfg(windows)]
    {
        use std::process::Command;
        
        let current_path = std::env::var("PATH").unwrap_or_default();
        let install_dir_str = install_dir.to_string_lossy().to_string();
        
        if !current_path.contains(&install_dir_str) {
            // Use setx to add to user PATH (not system PATH)
            // setx without /M modifies user environment variables
            let output = Command::new("setx")
                .args(["PATH", &format!("{};{}", current_path, install_dir_str)])
                .output();
            
            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("[selfidx-install] ✓ Añadido al PATH del usuario");
                        println!("[selfidx-install] ⚠️ Reinicia tu terminal para aplicar cambios");
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        println!("[selfidx-install] ⚠️ Error al modificar PATH: {}", stderr);
                        println!("[selfidx-install] 💡 Puedes agregar manualmente: {}", install_dir_str);
                    }
                }
                Err(e) => {
                    println!("[selfidx-install] ⚠️ Error al ejecutar setx: {}", e);
                    println!("[selfidx-install] 💡 Puedes agregar manualmente: {}", install_dir_str);
                }
            }
        } else {
            println!("[selfidx-install] ✓ Ya está en el PATH");
        }
    }

    println!();
    println!("==============================================");
    println!("  ✓ SELFIDEX instalado correctamente!");
    println!("==============================================");
    println!();
    println!("Ubicación: {}", install_dir.display());
    println!();
    println!("Para usar, ejecuta en una nueva terminal:");
    println!("  selfidx --help");
    println!();
    println!("NOTA: Si 'selfidx' no funciona, reinicia tu terminal.");

    Ok(())
}
