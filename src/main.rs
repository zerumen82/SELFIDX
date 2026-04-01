use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use std::cmp::min;
use clap::{Parser, Subcommand};
use selfidx::autonomous::run_autonomous_loop;
use selfidx::llm::{Message, OllamaClient};
use selfidx::utils::hardware::SystemInfo;
use selfidx::utils::format_size;
use selfidx::agent::Agent;
use selfidx::terminal::capsule::render_capsule;
use selfidx::config::Config;
use selfidx::project::ProjectContext;
use std::path::PathBuf;





/// Load project context from current directory
fn load_project_context() -> Result<String> {
    let mut info = String::new();
    let current_dir = std::env::current_dir()?;
    
    // Check for pubspec.yaml (Flutter projects)
    if let Ok(content) = fs::read_to_string("pubspec.yaml") {
        info.push_str("\n🔷 Proyecto Flutter detectado");
        if let Some(name) = content.lines().find(|l| l.starts_with("name:")) {
            info.push_str(&format!(": {}", name.trim_start_matches("name:").trim()));
        }
        if let Some(version) = content.lines().find(|l| l.starts_with("version:")) {
            info.push_str(&format!(" v{}", version.trim_start_matches("version:").trim()));
        }
        // Check for Android/iOS directories
        if Path::new("android").exists() {
            info.push_str(" (Android)");
        }
        if Path::new("ios").exists() {
            info.push_str(" (iOS)");
        }
    }
    // Check for android/app/build.gradle (Android projects)
    else if Path::new("android").exists() && Path::new("android/app/build.gradle").exists() {
        info.push_str("\n🤖 Proyecto Android detectado");
    }
    // Check for package.json (Node.js projects)
    else if let Ok(content) = fs::read_to_string("package.json") {
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
#[command(about = "Terminal integrada con Ollama - SELFIDEX v3.0", long_about = None)]
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

    /// Gestionar permisos
    Permissions {
        #[command(subcommand)]
        action: Option<PermissionCommands>,
    },

    /// Ver historial de comandos
    History {
        /// Número de comandos a mostrar
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Buscar en el historial
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Gestionar tareas en background
    Task {
        #[command(subcommand)]
        action: Option<TaskCommands>,
    },

    /// Gestionar proveedores LLM
    Provider {
        #[command(subcommand)]
        action: Option<ProviderCommands>,
    },

    /// Gestionar Ollama
    Ollama {
        #[command(subcommand)]
        action: Option<OllamaCommands>,
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

/// Comandos de gestión de permisos
#[derive(Subcommand, Debug)]
enum PermissionCommands {
    /// Mostrar estado de permisos
    Status,

    /// Cambiar modo de permisos
    SetMode {
        /// Modo: default, auto, dontask, plan, bypass, yolo
        mode: String,
    },

    /// Listar reglas de permiso
    List,

    /// Agregar regla de permiso
    Add {
        /// Herramienta (ej: execute_command, write_file, delete)
        tool: String,
        /// Patrón (ej: "git *", "rm -rf *")
        pattern: String,
        /// Comportamiento: allow, deny, ask
        #[arg(short, long)]
        behavior: String,
    },

    /// Eliminar regla de permiso
    Remove {
        /// Herramienta
        tool: String,
        /// Patrón
        pattern: String,
    },

    /// Restaurar reglas por defecto
    Defaults,
}

/// Comandos de gestión de tareas
#[derive(Subcommand, Debug)]
enum TaskCommands {
    /// Crear nueva tarea en background
    Create {
        /// Comando a ejecutar
        command: String,
    },

    /// Listar tareas
    List {
        /// Filtrar por estado: pending, running, completed, failed, cancelled
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Ver estado de una tarea
    Status {
        /// ID de la tarea
        id: String,
    },

    /// Ver output de una tarea
    Output {
        /// ID de la tarea
        id: String,

        /// Seguir output en tiempo real (tail -f)
        #[arg(short, long)]
        follow: bool,
    },

    /// Detener tarea
    Stop {
        /// ID de la tarea
        id: String,
    },

    /// Eliminar tarea
    Remove {
        /// ID de la tarea
        id: String,
    },

    /// Limpiar tareas completadas
    Cleanup,

    /// Ver estadísticas
    Stats,
}

/// Comandos de gestión de proveedores LLM
#[derive(Subcommand, Debug)]
enum ProviderCommands {
    /// Listar proveedores disponibles
    List,

    /// Ver información del proveedor actual
    Info,

    /// Cambiar proveedor
    Set {
        /// Proveedor: ollama, openai, anthropic, groq, lmstudio
        provider: String,

        /// API key (opcional, solo para proveedores cloud)
        #[arg(long)]
        api_key: Option<String>,

        /// Endpoint personalizado (opcional)
        #[arg(long)]
        endpoint: Option<String>,
    },

    /// Listar modelos del proveedor
    Models {
        /// Proveedor específico (opcional, usa el actual si no se especifica)
        provider: Option<String>,
    },

    /// Verificar conectividad
    Check,
}

/// Comandos de gestión de Ollama
#[derive(Subcommand, Debug)]
enum OllamaCommands {
    /// Verificar estado del servidor
    Status,

    /// Listar modelos disponibles
    List,

    /// Descargar modelo
    Pull {
        /// Nombre del modelo (ej: llama3, codellama)
        model: String,
    },

    /// Eliminar modelo
    Remove {
        /// Nombre del modelo
        model: String,
    },

    /// Ver información de un modelo
    Show {
        /// Nombre del modelo
        model: String,
    },

    /// Obtener modelo recomendado para tu hardware
    Recommend,

    /// Iniciar servidor Ollama
    Serve,
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

    // Load configuration (creates config file if it doesn't exist)
    let config = Config::load()?;

    // Initialize project context (creates .selfidx.md if first time)
    let current_dir = std::env::current_dir()?;
    let project = if ProjectContext::is_first_run(&current_dir) {
        // Primera vez en este proyecto - pedir modelo
        println!("[selfidx] Primera ejecución en este proyecto");
        println!("[selfidx] Directorio: {}", current_dir.display());
        println!();
        
        // Mostrar modelos disponibles en Ollama
        let client = OllamaClient::from_env();
        let selected_model = if client.is_available().await {
            println!("[selfidx] Listando modelos en Ollama...\n");
            
            match client.list_models().await {
                Ok(models) => {
                    if models.is_empty() {
                        println!("[selfidx] No hay modelos en Ollama");
                        println!("[selfidx] Descarga modelos con: ollama pull <modelo>");
                        println!();
                        client.get_default_model_from_api().await
                    } else {
                        println!("=== Modelos Disponibles en Ollama ===\n");
                        for (i, model) in models.iter().enumerate() {
                            println!("  {}. {}", i + 1, model);
                        }
                        println!();
                        
                        // Pedir selección
                        print!("Selecciona un modelo (1-{}): ", models.len());
                        io::stdout().flush()?;
                        
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        
                        let trimmed = input.trim();
                        if let Ok(index) = trimmed.parse::<usize>() {
                            if index >= 1 && index <= models.len() {
                                println!("\n✅ Modelo seleccionado: {}\n", models[index - 1]);
                                models[index - 1].clone()
                            } else {
                                println!("[selfidx] Selección inválida, usando modelo por defecto");
                                client.get_default_model_from_api().await
                            }
                        } else {
                                println!("[selfidx] Entrada inválida, usando modelo por defecto");
                            client.get_default_model_from_api().await
                        }
                    }
                }
                Err(e) => {
                    println!("[selfidx] Error al listar modelos: {}", e);
                    println!("[selfidx] Usando modelo por defecto: {}", client.get_default_model_from_api().await);
                    client.get_default_model_from_api().await
                }
            }
        } else {
            println!("[selfidx] Ollama no disponible en http://localhost:11434");
            println!("[selfidx] Usando modelo por defecto: {}", client.get_default_model_from_api().await);
            client.get_default_model_from_api().await
        };
        
        let proj = ProjectContext::init(selected_model)?;
        println!("[selfidx] Archivo .selfidx.md creado");
        proj
    } else {
        // Ya existe archivo - cargar y mostrar contenido
        //优先使用用户提供的模型参数，否则从配置文件读取
        let project_model = if let Some(m) = &args.model {
            m.clone()
        } else {
            config.default_model().to_string()
        };
        let proj = ProjectContext::load(project_model)?;
        println!("[selfidx] Proyecto detectado: {}", proj.project_name());
        
        // Leer y mostrar contenido del archivo
        if let Ok(content) = proj.read_progress() {
            if !content.is_empty() {
                println!("[selfidx] Archivo .selfidx.md encontrado");
            }
        }
        proj
    };

    // Mostrar info del proyecto
    project.display_info();

    // Render cápsula (conservar estética)
    println!("{}", render_capsule());
    println!();
    
    // Show project context
    let current_dir = std::env::current_dir().expect("No se puede obtener el directorio actual");
    println!("[selfidx] Directorio de trabajo: {}", current_dir.display());
    
    // Load and display project context
    if let Ok(project_info) = load_project_context() {
        println!("{}", project_info);
    }
    
    // Verify Ollama connectivity
    let client = OllamaClient::from_env();
    if client.is_available().await {
        println!("[selfidx] Ollama conectado en http://localhost:11434");
        
        // List available models if connected
        match client.list_models().await {
            Ok(models) => {
                println!("[selfidx] {} modelos disponibles en Ollama", models.len());
            }
            Err(e) => {
                println!("[selfidx] No se pueden listar los modelos: {}", e);
            }
        }
    } else {
        println!("[selfidx] Ollama no está disponible en http://localhost:11434");
    }
    println!();

    // Determine which model to use
    // Priority: user argument > config default > API default
    let model = if let Some(m) = args.model {
        m
    } else {
        // Use config default or get from Ollama API
        println!("[selfidx] Usando modelo por defecto...");
        let client = OllamaClient::from_env();
        if let Ok(models) = client.list_models().await {
            if let Some(first_model) = models.first() {
                println!("[selfidx] Modelo disponible: {}", first_model);
                first_model.clone()
            } else {
                config.default_model().to_string()
            }
        } else {
            config.default_model().to_string()
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
            let client = OllamaClient::from_env();
            
            if !client.is_available().await {
    println!("[selfidx-error] Ollama no está disponible.");
    println!("[selfidx-error] Inicia Ollama con: ollama serve");
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
                    content: format!(
                        "Eres un asistente de programación. Ejecuta tareas directamente.\
\
=== HERRAMIENTAS DISPONIBLES ===\
{}\
\
Responde en español.",
                        Agent::get_tools_description()
                    ),
                    tool_calls: None,
                },
                Message {
                    role: "user".to_string(),
                    content: full_prompt,
                    tool_calls: None,
                },
            ];
            
            // Convert agent tools to Ollama format
            let ollama_tools = Some(OllamaClient::convert_agent_tools_to_ollama(Agent::get_tools()));
            match client.chat_with_tools(model, messages, ollama_tools).await {
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
            println!("[selfidx-chat] 🤖 Modo chat interactivo - Escribe tus mensajes (escribe 'salir' para terminar)\n");
            
            // Connection check - only once at startup, not in every iteration
            let client = OllamaClient::from_env();
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible. Asegúrate de que Ollama esté ejecutándose con: ollama serve");
                return Ok(());
            }
            
            println!();
            
            // Load project context for persistence
            let project_ctx = match ProjectContext::load(model.clone()) {
                Ok(ctx) => {
                    println!("[selfidx-chat] 📁 Proyecto: {}", ctx.project_name());
                    Some(ctx)
                }
                Err(e) => {
                    println!("[selfidx-chat] ℹ️ Sin contexto de proyecto: {}", e);
                    None
                }
            };
            
            // Get existing progress for system prompt
            let project_info = if let Some(ref ctx) = project_ctx {
                match ctx.read_progress() {
                    Ok(content) if !content.is_empty() => {
                        format!("\n\n--- CONTEXTO DEL PROYECTO ---\n{}", content)
                    }
                    _ => String::new(),
                }
            } else {
                String::new()
            };
            
            // Chat history with project context
            let mut messages = vec![
                Message {
                    role: "system".to_string(),
                    content: format!(
                        "Eres un asistente de programación AUTÓNOMO. Tienes acceso directo a funciones del sistema. \
                        Cuando el usuario pida algo, EJECUTA la herramienta correspondiente - NO describas qué harías, EJECÚTALA.\
                        \
=== HERRAMIENTAS (USA ESTAS FUNCIONES, NO LAS DESCRIBAS) ===\
{}\
\
{}",
                        Agent::get_tools_description(),
                        project_info
                    ),
                    tool_calls: None,
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
                    // Keep project context when clearing
                    let project_info = if let Some(ref ctx) = project_ctx {
                        match ctx.read_progress() {
                            Ok(content) if !content.is_empty() => {
                                format!("\n\n--- CONTEXTO DEL PROYECTO ---\n{}", content)
                            }
                            _ => String::new(),
                        }
                    } else {
                        String::new()
                    };
                    
                    messages = vec![
                        Message {
                            role: "system".to_string(),
                            content: format!(
                                "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                                Ejecuta acciones directamente para completar las tareas del usuario. \
                                Cuando debas crear archivos, usa los comandos necesarios. \
                                Cuando debas ejecutar algo, hazlo. \
                                Responde siempre en español de forma clara y concisa.\
                                \
=== HERRAMIENTAS DISPONIBLES ===\
{}\
{}",
                                Agent::get_tools_description(),
                                project_info
                            ),
                            tool_calls: None,
                        },
                    ];
                    println!("✅ Chat limpiado (contexto del proyecto preservado)");
                    continue;
                }
                
                messages.push(Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                    tool_calls: None,
                });
                
                // Convert agent tools to Ollama format
                let ollama_tools = Some(OllamaClient::convert_agent_tools_to_ollama(Agent::get_tools()));
                match client.chat_with_tools(model.clone(), messages.clone(), ollama_tools).await {
                    Ok(response) => {
                        // Check if response has tool calls from Ollama (qwen2.5, etc)
                        if let Some(tool_calls) = response.tool_calls() {
                            println!("→ Ollama solicitó {} herramienta(s)", tool_calls.len());
                            
                            let agent = Agent::new();
                            for tool_call in tool_calls {
                                let tool_name = &tool_call.function.name;
                                let params: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                                    .unwrap_or(serde_json::json!({}));
                                
                                println!("→ Ejecutando: {} {:?}", tool_name, params);

                                let result = if Agent::is_destructive_tool(tool_name) {
                                    match agent.execute_tool_with_confirmation(tool_name, &params) {
                                        Ok((result, confirmed)) if confirmed => result,
                                        _ => {
                                            messages.push(Message {
                                                role: "user".to_string(),
                                                content: "Acción cancelada.".to_string(),
                                                tool_calls: None,
                                            });
                                            continue;
                                        }
                                    }
                                } else {
                                    match agent.execute_tool(tool_name, &params) {
                                        Ok(result) => result,
                                        Err(e) => format!("Error: {}", e),
                                    }
                                };

                                // Add tool result to messages
                                messages.push(Message {
                                    role: "tool".to_string(),
                                    content: result,
                                    tool_calls: None,
                                });
                            }
                            
                            // Continue conversation with tool results
                            continue;
                        }

                        // Fallback: Check if content is JSON tool call (for mistral model)
                        let content = response.content().trim().to_string();
                        if content.starts_with('[') {
                            if let Ok(tools) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                                for tool in tools {
                                    if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                                        let args = tool.get("arguments").map(|a| a.clone()).unwrap_or(serde_json::json!({}));
                                        println!("→ Ejecutando (JSON): {} {:?}", name, args);
                                        
                                        let agent = Agent::new();
                                        let result = match agent.execute_tool(name, &args) {
                                            Ok(r) => r,
                                            Err(e) => format!("Error: {}", e),
                                        };
                                        
                                        messages.push(Message {
                                            role: "tool".to_string(),
                                            content: result,
                                            tool_calls: None,
                                        });
                                    }
                                }
                                continue;
                            }
                        }

                        let content = response.content().to_string();
                        println!("\n🤖: {}", content);
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: content.clone(),
                            tool_calls: None,
                        });
                        
                        // Save progress to .selfidx.md
                        if let Some(ref ctx) = project_ctx {
                            if let Err(e) = ctx.add_progress(
                                &input[..input.len().min(50)],
                                "💬",
                                &content[..content.len().min(100)],
                            ) {
                                println!("[selfidx-chat] ℹ️ Nota: No se pudo guardar progreso: {}", e);
                            }
                        }
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
            
            // Verify Ollama is available
            let client = OllamaClient::from_env();
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible en http://localhost:11434");
                println!("[selfidx] Inicia Ollama con: ollama serve");
                return Ok(());
            }
            
            // List available models
            match client.list_models().await {
                Ok(models) => {
                    if models.is_empty() {
                        println!("[selfidx] No hay modelos disponibles en Ollama");
                        return Ok(());
                    }
                    
                    // Check if model exists
                    if models.contains(&model) {
                        println!("✅ Modelo '{}' encontrado en Ollama", model);
                        println!("\n💡 Para usar este modelo, ejecuta:");
                        println!("   selfidx --model {}", model);
                        println!("\nO en modo interactivo:");
                        println!("   /model");
                    } else {
                    println!("[selfidx] Modelo '{}' no encontrado en Ollama", model);
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
                    println!("  ollama pull <modelo>");
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
            
            // First, try to list models from Ollama
            let client = OllamaClient::from_env();
            if client.is_available().await {
                println!("=== Modelos en Ollama ===\n");
                match client.list_models().await {
                    Ok(models) => {
                        if models.is_empty() {
                            println!("No hay modelos en Ollama.");
                        } else {
                            for (i, model) in models.iter().enumerate() {
                                println!("{}. {}", i + 1, model);
                            }
                            println!("\nTotal: {} modelos en Ollama", models.len());
                        }
                    }
                    Err(e) => {
                        println!("[selfidx-error] Error al listar modelos de Ollama: {}", e);
                    }
                }
                println!();
            } else {
                println!("[selfidx] ⚠️ Ollama no está disponible en http://localhost:11434");
                println!("[selfidx] 💡 Inicia Ollama con: ollama serve\n");
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
            println!("[selfidx] 💡 Usa: P=Proveedor, M=Modelo, H=Help, ESC=Salir");

            // Run enhanced TUI with multi-API support
            if let Err(e) = selfidx::terminal::run_enhanced_tui() {
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
            let client = OllamaClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible.");
                println!("Inicia Ollama con: ollama serve");
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
            let client = OllamaClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible.");
                println!("Inicia Ollama con: ollama serve");
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
            let client = OllamaClient::from_env();
            
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible.");
                println!("Inicia Ollama con: ollama serve");
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
                    tool_calls: None,
                },
                Message {
                    role: "user".to_string(),
                    content: plan_prompt,
                    tool_calls: None,
                },
            ];
            
            // Convert agent tools to Ollama format
            let ollama_tools = Some(OllamaClient::convert_agent_tools_to_ollama(Agent::get_tools()));
            match client.chat_with_tools(model, messages, ollama_tools).await {
                Ok(response) => {
                    // Check if response has tool calls from Ollama
                    if let Some(tool_calls) = response.tool_calls() {
                        println!("→ Ollama solicitó {} herramienta(s) (ignorando en modo plan)", tool_calls.len());
                    }
                    
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

        Some(Commands::Permissions { action }) => {
            use selfidx::permissions::{PermissionStorage, PermissionBehavior, RuleSource};
            use selfidx::permissions::storage::initialize_with_defaults;
            
            match action {
                Some(PermissionCommands::Status) => {
                    let storage = PermissionStorage::load()?;
                    println!("\n═══════════════════════════════════════════");
                    println!("   🔒 ESTADO DE PERMISOS");
                    println!("═══════════════════════════════════════════\n");
                    println!("Modo: {} {}", storage.get_mode().symbol(), storage.get_mode());
                    println!("Reglas configuradas: {}", storage.get_all_rules().len());
                    println!();
                }
                
                Some(PermissionCommands::SetMode { mode }) => {
                    let mut storage = PermissionStorage::load()?;
                    let mode = selfidx::PermissionMode::from_str_lossy(&mode)
                        .ok_or_else(|| anyhow::anyhow!("Modo inválido: {}. Modos válidos: default, auto, dontask, plan, bypass, yolo", mode))?;
                    storage.set_mode(mode)?;
                    println!("\n✅ Modo de permisos cambiado a: {} {}", mode.symbol(), mode);
                    println!("   {}", mode.description());
                }
                
                Some(PermissionCommands::List) => {
                    let storage = PermissionStorage::load()?;
                    let rules = storage.get_all_rules();
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   📋 REGLAS DE PERMISO");
                    println!("═══════════════════════════════════════════\n");
                    
                    if rules.is_empty() {
                        println!("No hay reglas configuradas.");
                        println!("Usa 'selfidx permissions defaults' para cargar reglas por defecto.");
                    } else {
                        for rule in &rules {
                            let symbol = match rule.behavior {
                                PermissionBehavior::Allow => "✓",
                                PermissionBehavior::Deny => "✗",
                                PermissionBehavior::Ask => "?",
                            };
                            println!("  [{}] {} {} ({})", symbol, rule.value.tool_name, rule.value.pattern, rule.source);
                        }
                    }
                    println!();
                }
                
                Some(PermissionCommands::Add { tool, pattern, behavior }) => {
                    let mut storage = PermissionStorage::load()?;
                    let behavior = match behavior.to_lowercase().as_str() {
                        "allow" | "a" => PermissionBehavior::Allow,
                        "deny" | "d" => PermissionBehavior::Deny,
                        "ask" => PermissionBehavior::Ask,
                        _ => {
                            anyhow::bail!("Comportamiento inválido: {}. Usa: allow, deny, o ask", behavior);
                        }
                    };
                    
                    let rule = selfidx::permissions::PermissionRule::new(behavior, &tool, &pattern, RuleSource::User);
                    storage.add_rule(rule)?;
                    println!("\n✅ Regla agregada: {} {} {}", 
                        match behavior {
                            PermissionBehavior::Allow => "✓ allow",
                            PermissionBehavior::Deny => "✗ deny",
                            PermissionBehavior::Ask => "? ask",
                        },
                        tool, pattern
                    );
                }
                
                Some(PermissionCommands::Remove { tool, pattern }) => {
                    let mut storage = PermissionStorage::load()?;
                    if storage.find_rule(&tool, &pattern).is_none() {
                        println!("\n⚠️  Regla no encontrada: {} {}", tool, pattern);
                    } else {
                        storage.remove_rule(&tool, &pattern)?;
                        println!("\n✅ Regla eliminada: {} {}", tool, pattern);
                    }
                }
                
                Some(PermissionCommands::Defaults) => {
                    let storage = initialize_with_defaults()?;
                    println!("\n✅ Reglas por defecto cargadas: {}", storage.get_all_rules().len());
                    println!("   Usa 'selfidx permissions list' para ver las reglas.");
                }

                None => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🔒 GESTIÓN DE PERMISOS");
                    println!("═══════════════════════════════════════════\n");
                    println!("Usa: selfidx permissions <comando>\n");
                    println!("Comandos disponibles:");
                    println!("  status              - Mostrar estado actual");
                    println!("  set-mode <modo>     - Cambiar modo (default, auto, dontask, plan, bypass, yolo)");
                    println!("  list                - Listar reglas configuradas");
                    println!("  add <tool> <pattern> - Agregar regla");
                    println!("  remove <tool> <pattern> - Eliminar regla");
                    println!("  defaults            - Cargar reglas por defecto");
                    println!();
                }
            }
        }

        Some(Commands::History { limit, search }) => {
            use selfidx::CommandHistory;
            
            let history = CommandHistory::new()?;
            
            println!("\n═══════════════════════════════════════════");
            println!("   📜 HISTORIAL DE COMANDOS");
            println!("═══════════════════════════════════════════\n");
            
            if let Some(query) = search {
                let results = history.search(&query);
                if results.is_empty() {
                    println!("No se encontraron comandos que coincidan con '{}'", query);
                } else {
                    println!("Comandos que coinciden con '{}':\n", query);
                    for (i, cmd) in results.iter().enumerate() {
                        println!("  {:4}.  {}", results.len() - i, cmd);
                    }
                }
            } else {
                let recent = history.recent(limit);
                if recent.is_empty() {
                    println!("El historial está vacío.");
                } else {
                    println!("Últimos {} comandos:\n", recent.len());
                    for (i, cmd) in recent.iter().enumerate() {
                        println!("  {:4}.  {}", recent.len() - i, cmd);
                    }
                }
            }
            println!();
        }

        Some(Commands::Task { action }) => {
            use selfidx::tasks::{TaskManager, TaskStatus};
            
            let mut manager = TaskManager::new()?;
            
            match action {
                Some(TaskCommands::Create { command }) => {
                    let task = manager.create_task(command.clone());
                    let task_id = task.id.clone();
                    let task_command = task.command.clone();
                    let task_status = task.status;
                    let task_created_at = task.created_at;

                    // Iniciar tarea en background
                    manager.start_task(&task_id)?;

                    println!("\n═══════════════════════════════════════════");
                    println!("   📤 TAREA CREADA");
                    println!("═══════════════════════════════════════════\n");
                    println!("ID:        {}", task_id);
                    println!("Comando:   {}", task_command);
                    println!("Estado:    {} {}", task_status.symbol(), task_status.display());
                    println!("Creada:    {}", task_created_at.format("%Y-%m-%d %H:%M:%S"));
                    println!();
                    println!("💡 Usa 'selfidx task status {}' para ver el estado", task_id);
                    println!("💡 Usa 'selfidx task output {}' para ver el resultado", task_id);
                    println!();
                }
                
                Some(TaskCommands::List { status }) => {
                    let tasks = if let Some(status_filter) = status {
                        let status = match status_filter.to_lowercase().as_str() {
                            "pending" => TaskStatus::Pending,
                            "running" => TaskStatus::Running,
                            "completed" => TaskStatus::Completed,
                            "failed" => TaskStatus::Failed,
                            "cancelled" => TaskStatus::Cancelled,
                            _ => {
                                anyhow::bail!("Estado inválido: {}. Usa: pending, running, completed, failed, cancelled", status_filter);
                            }
                        };
                        manager.list_tasks_by_status(status)
                    } else {
                        manager.list_tasks()
                    };
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   📋 TAREAS");
                    println!("═══════════════════════════════════════════\n");
                    
                    if tasks.is_empty() {
                        println!("No hay tareas.");
                    } else {
                        println!("{:<35} {:<12} {:<20} {}", "ID", "Estado", "Comando", "Creada");
                        println!("{}", "─".repeat(90));
                        
                        for task in tasks {
                            let cmd_preview = if task.command.len() > 30 {
                                format!("{}...", &task.command[..27])
                            } else {
                                task.command.clone()
                            };
                            
                            println!(
                                "{:<35} {} {:<10} {:<20} {}",
                                task.id,
                                task.status.symbol(),
                                task.status.display(),
                                cmd_preview,
                                task.created_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                    println!();
                }
                
                Some(TaskCommands::Status { id }) => {
                    if let Some(task) = manager.get_task(&id) {
                        println!("\n═══════════════════════════════════════════");
                        println!("   📊 ESTADO DE TAREA");
                        println!("═══════════════════════════════════════════\n");
                        println!("ID:            {}", task.id);
                        println!("Comando:       {}", task.command);
                        println!("Estado:        {} {}", task.status.symbol(), task.status.display());
                        println!("Creada:        {}", task.created_at.format("%Y-%m-%d %H:%M:%S"));
                        
                        if let Some(started) = task.started_at {
                            println!("Iniciada:      {}", started.format("%Y-%m-%d %H:%M:%S"));
                        }
                        
                        if let Some(completed) = task.completed_at {
                            println!("Completada:    {}", completed.format("%Y-%m-%d %H:%M:%S"));
                        }
                        
                        if let Some(duration) = task.duration() {
                            println!("Duración:      {}", duration);
                        }
                        
                        if let Some(exit_code) = task.exit_code {
                            println!("Código salida: {}", exit_code);
                        }
                        
                        if let Some(error) = &task.error_message {
                            println!("Error:         {}", error);
                        }
                        
                        println!();
                    } else {
                        println!("\n⚠️  Tarea no encontrada: {}", id);
                        println!("💡 Usa 'selfidx task list' para ver todas las tareas");
                        println!();
                    }
                }
                
                Some(TaskCommands::Output { id, follow }) => {
                    if follow {
                        println!("\n📄 Siguiendo output de {} (Ctrl+C para salir)...\n", id);
                        // En una implementación real, usaríamos notify para watch del archivo
                    }
                    
                    match manager.get_task_output(&id) {
                        Ok(Some(output)) => {
                            println!("\n═══════════════════════════════════════════");
                            println!("   📄 OUTPUT DE TAREA {}", id);
                            println!("═══════════════════════════════════════════\n");
                            println!("{}", output);
                            println!();
                        }
                        Ok(None) => {
                            println!("\n⚠️  No hay output disponible para la tarea: {}", id);
                            println!("La tarea puede estar aún en ejecución.");
                            println!();
                        }
                        Err(e) => {
                            println!("\n❌ Error al leer output: {}", e);
                            println!();
                        }
                    }
                }
                
                Some(TaskCommands::Stop { id }) => {
                    manager.stop_task(&id)?;
                    println!("\n✅ Tarea detenida: {}", id);
                    println!();
                }
                
                Some(TaskCommands::Remove { id }) => {
                    manager.remove_task(&id)?;
                    println!("\n✅ Tarea eliminada: {}", id);
                    println!();
                }
                
                Some(TaskCommands::Cleanup) => {
                    let count = manager.cleanup_completed()?;
                    println!("\n✅ Tareas completadas eliminadas: {}", count);
                    println!();
                }
                
                Some(TaskCommands::Stats) => {
                    let stats = manager.get_stats();
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   📊 ESTADÍSTICAS DE TAREAS");
                    println!("═══════════════════════════════════════════\n");
                    println!("Total:       {}", stats.total);
                    println!();
                    println!("  {} Pendientes:     {}", TaskStatus::Pending.symbol(), stats.pending);
                    println!("  {} Ejecutando:    {}", TaskStatus::Running.symbol(), stats.running);
                    println!("  {} Completadas:   {}", TaskStatus::Completed.symbol(), stats.completed);
                    println!("  {} Fallidas:      {}", TaskStatus::Failed.symbol(), stats.failed);
                    println!("  {} Canceladas:    {}", TaskStatus::Cancelled.symbol(), stats.cancelled);
                    println!();
                }
                
                None => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   📤 GESTIÓN DE TAREAS");
                    println!("═══════════════════════════════════════════\n");
                    println!("Usa: selfidx task <comando>\n");
                    println!("Comandos disponibles:");
                    println!("  create <comando>    - Crear tarea en background");
                    println!("  list [--status X]   - Listar tareas (opcionalmente filtrar)");
                    println!("  status <id>         - Ver estado de tarea");
                    println!("  output <id>         - Ver output de tarea");
                    println!("  stop <id>           - Detener tarea");
                    println!("  remove <id>         - Eliminar tarea");
                    println!("  cleanup             - Limpiar tareas completadas");
                    println!("  stats               - Ver estadísticas");
                    println!();
                }
            }
        }

        Some(Commands::Provider { action }) => {
            use selfidx::llm::{LlmProvider, LlmClient};
            
            match action {
                Some(ProviderCommands::List) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 PROVEEDORES LLM DISPONIBLES");
                    println!("═══════════════════════════════════════════\n");
                    
                    let providers = [
                        LlmProvider::Ollama,
                        LlmProvider::OpenAI,
                        LlmProvider::Anthropic,
                        LlmProvider::Groq,
                        LlmProvider::LMStudio,
                        LlmProvider::Grok,       // xAI
                        LlmProvider::Gemini,     // Google
                        LlmProvider::OpenRouter, // Multi-modelos
                        LlmProvider::DeepSeek,   // DeepSeek AI
                        LlmProvider::Cohere,     // Cohere
                        LlmProvider::Mistral,    // Mistral AI
                        LlmProvider::Perplexity, // Perplexity
                        LlmProvider::Together,   // Together AI
                    ];
                    
                    println!("{:<14} {:<40} {:<12} {}", "Proveedor", "Endpoint", "API Key", "Modelos Populares");
                    println!("{}", "─".repeat(110));
                    
                    for provider in &providers {
                        let api_key_status = if provider.requires_api_key() { "Requerida" } else { "No requerida" };
                        let tools_status = if provider.supports_tools() { "✅" } else { "❌" };
                        let models = provider.popular_models().join(", ");
                        
                        println!(
                            "{:<14} {:<40} {:<12} {} {}",
                            provider.as_str(),
                            provider.default_endpoint(),
                            api_key_status,
                            models,
                            tools_status
                        );
                    }
                    println!("\n✅ = Soporta tools/function calling nativo");
                    println!();
                }
                
                Some(ProviderCommands::Info) => {
                    let client = LlmClient::from_env();
                    let info = client.get_provider_info();
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 PROVEEDOR ACTUAL");
                    println!("═══════════════════════════════════════════\n");
                    println!("{}", info.display());
                    println!();
                }
                
                Some(ProviderCommands::Set { provider: provider_str, api_key, endpoint }) => {
                    let provider = LlmProvider::from_str(&provider_str)
                        .ok_or_else(|| anyhow::anyhow!(
                            "Proveedor inválido: {}\n\
                            Proveedores válidos: ollama, openai, anthropic, groq, lmstudio",
                            provider_str
                        ))?;
                    
                    // Guardar configuración en variables de entorno
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 CONFIGURACIÓN DE PROVEEDOR");
                    println!("═══════════════════════════════════════════\n");
                    println!("Proveedor: {}", provider);
                    let endpoint_str = endpoint.clone().unwrap_or_else(|| provider.default_endpoint().to_string());
                    println!("Endpoint: {}", endpoint_str);
                    
                    if provider.requires_api_key() {
                        if let Some(key) = &api_key {
                            println!("API Key: ✓ Configurada ({}***)", &key[..min(4, key.len())]);
                            
                            // Guardar en archivo de configuración
                            let config_dir = dirs::config_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("selfidx");
                            std::fs::create_dir_all(&config_dir)?;
                            
                            let config_path = config_dir.join("provider.env");
                            let mut content = format!(
                                "LLM_PROVIDER={}\nLLM_ENDPOINT={}\n",
                                provider.as_str(),
                                endpoint.unwrap_or_else(|| provider.default_endpoint().to_string())
                            );
                            
                            if let Some(key) = api_key {
                                content.push_str(&format!("LLM_API_KEY={}\n", key));
                            }
                            
                            std::fs::write(&config_path, &content)?;
                            println!("\n✅ Configuración guardada en: {}", config_path.display());
                            println!("\n💡 Para usar esta configuración:");
                            println!("   1. Reinicia tu terminal, O");
                            println!("   2. Ejecuta: source {}", config_path.display());
                        } else {
                            println!("API Key: ⚠️ No proporcionada");
                            println!("\n💡 Este proveedor requiere API key. Usa --api-key para configurarla.");
                        }
                    } else {
                        println!("API Key: No requerida (servicio local)");
                    }
                    
                    println!();
                }
                
                Some(ProviderCommands::Models { provider: provider_str }) => {
                    let client = if let Some(p_str) = provider_str {
                        let provider = LlmProvider::from_str(&p_str)
                            .ok_or_else(|| anyhow::anyhow!("Proveedor inválido: {}", p_str))?;
                        
                        LlmClient::from_provider(provider, None)
                    } else {
                        LlmClient::from_env()
                    };
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 MODELOS DISPONIBLES");
                    println!("═══════════════════════════════════════════\n");
                    
                    println!("Proveedor: {}", client.provider);
                    println!("Endpoint: {}", client.endpoint);
                    println!();
                    
                    // Verificar conectividad
                    if client.is_available().await {
                        match client.list_models().await {
                            Ok(models) => {
                                if models.is_empty() {
                                    println!("No se encontraron modelos.");
                                    println!();
                                    println!("Modelos populares para {}:", client.provider);
                                    for model in client.provider.popular_models() {
                                        println!("  - {}", model);
                                    }
                                } else {
                                    println!("Modelos encontrados: {}\n", models.len());
                                    for model in &models {
                                        println!("  • {}", model);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("⚠️  Error al listar modelos: {}", e);
                                println!();
                                println!("Modelos populares para {}:", client.provider);
                                for model in client.provider.popular_models() {
                                    println!("  - {}", model);
                                }
                            }
                        }
                    } else {
                        println!("⚠️  Proveedor no disponible.");
                        println!();
                        println!("Modelos populares para {}:", client.provider);
                        for model in client.provider.popular_models() {
                            println!("  - {}", model);
                        }
                    }
                    println!();
                }
                
                Some(ProviderCommands::Check) => {
                    let client = LlmClient::from_env();
                    
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 VERIFICANDO CONECTIVIDAD");
                    println!("═══════════════════════════════════════════\n");
                    println!("Proveedor: {}", client.provider);
                    println!("Endpoint: {}", client.endpoint);
                    println!();
                    
                    if client.is_available().await {
                        println!("✅ ¡Proveedor disponible!");
                        
                        // Intentar listar modelos
                        match client.list_models().await {
                            Ok(models) => {
                                println!("✅ Modelos accesibles: {} encontrados", models.len());
                            }
                            Err(e) => {
                                println!("⚠️  No se pudieron listar modelos: {}", e);
                            }
                        }
                    } else {
                        println!("❌ Proveedor no disponible");
                        println!();
                        
                        if client.provider.requires_api_key() && client.api_key.is_none() {
                            println!("💡 Este proveedor requiere API key.");
                            println!("   Configura con: selfidx provider set {} --api-key TU_KEY", client.provider);
                        } else {
                            println!("💡 Verifica que el servicio esté corriendo en: {}", client.endpoint);
                        }
                    }
                    println!();
                }
                
                None => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🤖 GESTIÓN DE PROVEEDORES LLM");
                    println!("═══════════════════════════════════════════\n");
                    println!("Usa: selfidx provider <comando>\n");
                    println!("Comandos disponibles:");
                    println!("  list                - Listar proveedores disponibles");
                    println!("  info                - Ver información del proveedor actual");
                    println!("  set <provider>      - Cambiar proveedor (ollama, openai, anthropic, groq, lmstudio)");
                    println!("  models [provider]   - Listar modelos disponibles");
                    println!("  check               - Verificar conectividad");
                    println!();
                }
            }
        }

        Some(Commands::Ollama { action }) => {
            use selfidx::{OllamaManager, ollama};
            
            let ollama = OllamaManager::from_env();
            
            match action {
                Some(OllamaCommands::Status) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🦙 ESTADO DE OLLAMA");
                    println!("═══════════════════════════════════════════\n");
                    
                    match ollama.get_server_info().await {
                        Ok(info) => {
                            if info.is_running {
                                println!("✅ Ollama está corriendo en: {}", info.url);
                                if let Some(version) = info.version {
                                    println!("   Versión: {}", version);
                                }
                                
                                // Listar modelos
                                match ollama.list_models().await {
                                    Ok(models) => {
                                        println!("\n📦 Modelos instalados: {}", models.len());
                                        for model in &models {
                                            println!("   • {} ({})", model.name, model.size);
                                        }
                                    }
                                    Err(e) => println!("⚠️  Error al listar modelos: {}", e),
                                }
                            } else {
                                println!("❌ Ollama NO está corriendo");
                                println!("\n💡 Para iniciar Ollama:");
                                println!("   1. Descarga desde: https://ollama.ai");
                                println!("   2. Ejecuta: ollama serve");
                                println!("   3. O usa: selfidx ollama serve");
                            }
                        }
                        Err(e) => println!("❌ Error: {}", e),
                    }
                    println!();
                }
                
                Some(OllamaCommands::List) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   📦 MODELOS OLLAMA DISPONIBLES");
                    println!("═══════════════════════════════════════════\n");
                    
                    match ollama.list_models().await {
                        Ok(models) => {
                            if models.is_empty() {
                                println!("No hay modelos instalados.");
                                println!("\n💡 Modelos recomendados para código:");
                                for model in OllamaManager::get_recommended_models().iter().take(5) {
                                    println!("   • {}", model);
                                }
                                println!("\n   Para instalar: selfidx ollama pull <modelo>");
                            } else {
                                println!("{:<30} {:<12} {:<15} {}", "Modelo", "Tamaño", "Familia", "Params");
                                println!("{}", "─".repeat(75));
                                
                                for model in &models {
                                    println!(
                                        "{:<30} {:<12} {:<15} {}",
                                        model.name,
                                        model.size,
                                        model.details.family,
                                        model.details.parameter_size
                                    );
                                }
                            }
                        }
                        Err(e) => println!("❌ Error: {}", e),
                    }
                    println!();
                }
                
                Some(OllamaCommands::Pull { model }) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   📥 DESCARGANDO MODELO: {}", model.to_uppercase());
                    println!("═══════════════════════════════════════════\n");
                    
                    match ollama.get_or_pull_model(&model).await {
                        Ok(msg) => {
                            println!("{}", msg);
                            
                            // Verificar si está disponible
                            if ollama.model_exists(&model).await {
                                println!("\n✅ Modelo {} listo para usar", model);
                                println!("   Usa: selfidx --model {}", model);
                            }
                        }
                        Err(e) => println!("❌ Error: {}", e),
                    }
                    println!();
                }
                
                Some(OllamaCommands::Remove { model }) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🗑️  ELIMINANDO MODELO: {}", model.to_uppercase());
                    println!("═══════════════════════════════════════════\n");
                    
                    match ollama.delete_model(&model).await {
                        Ok(_) => println!("✅ Modelo {} eliminado", model),
                        Err(e) => println!("❌ Error: {}", e),
                    }
                    println!();
                }
                
                Some(OllamaCommands::Show { model }) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   📊 INFORMACIÓN DEL MODELO: {}", model.to_uppercase());
                    println!("═══════════════════════════════════════════\n");
                    
                    match ollama.show_model(&model).await {
                        Ok(info) => {
                            println!("Modelo: {}", model);
                            if let Some(details) = info.get("details") {
                                println!("Detalles: {:?}", details);
                            }
                        }
                        Err(e) => println!("❌ Error: {}", e),
                    }
                    println!();
                }
                
                Some(OllamaCommands::Recommend) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   💡 MODELO RECOMENDADO");
                    println!("═══════════════════════════════════════════\n");
                    
                    let recommended = OllamaManager::recommend_model_for_hardware();
                    let ram_gb = ollama::get_total_ram_gb();
                    
                    println!("Tu sistema tiene: {:.1} GB RAM", ram_gb);
                    println!();
                    println!("Modelo recomendado: {}", recommended);
                    println!();
                    println!("Otros modelos según tu hardware:");
                    
                    if ram_gb >= 64.0 {
                        println!("  • llama3:70b (máxima calidad)");
                        println!("  • codellama:34b (código)");
                        println!("  • mixtral:8x7b (rápido)");
                    } else if ram_gb >= 32.0 {
                        println!("  • codellama:34b (código)");
                        println!("  • llama3 (general)");
                        println!("  • mistral (rápido)");
                    } else if ram_gb >= 16.0 {
                        println!("  • codellama:13b (código)");
                        println!("  • llama3:8b (general)");
                        println!("  • mistral:7b (rápido)");
                    } else {
                        println!("  • phi3 (ligero)");
                        println!("  • codellama:7b (código)");
                        println!("  • tinyllama (muy ligero)");
                    }
                    
                    println!();
                    println!("💡 Para instalar: selfidx ollama pull {}", recommended);
                    println!();
                }
                
                Some(OllamaCommands::Serve) => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🚀 INICIANDO OLLAMA SERVER");
                    println!("═══════════════════════════════════════════\n");
                    
                    match OllamaManager::start_server() {
                        Ok(_) => {
                            println!("✅ Ollama server iniciado");
                            println!();
                            println!("💡 El servidor se está iniciando en segundo plano.");
                            println!("   Espera unos segundos y verifica con:");
                            println!("   selfidx ollama status");
                        }
                        Err(e) => {
                            println!("❌ Error al iniciar: {}", e);
                            println!();
                            println!("💡 Asegúrate de tener Ollama instalado:");
                            println!("   https://ollama.ai");
                        }
                    }
                    println!();
                }
                
                None => {
                    println!("\n═══════════════════════════════════════════");
                    println!("   🦙 GESTIÓN DE OLLAMA");
                    println!("═══════════════════════════════════════════\n");
                    println!("Usa: selfidx ollama <comando>\n");
                    println!("Comandos disponibles:");
                    println!("  status              - Verificar estado del servidor");
                    println!("  list                - Listar modelos instalados");
                    println!("  pull <modelo>       - Descargar modelo");
                    println!("  remove <modelo>     - Eliminar modelo");
                    println!("  show <modelo>       - Ver información de modelo");
                    println!("  recommend           - Obtener modelo recomendado");
                    println!("  serve               - Iniciar servidor Ollama");
                    println!();
                    println!("Modelos recomendados para código:");
                    for model in OllamaManager::get_recommended_models().iter().take(5) {
                        println!("  • {}", model);
                    }
                    println!();
                }
            }
        }

        None => {
            // Check if Ollama is available
            let client = OllamaClient::from_env();
            
            println!("[selfidx] 🔍 Verificando Ollama...");
            
            if !client.is_available().await {
                println!("[selfidx] ⚠️ Ollama no está corriendo");
                println!("[selfidx] 💡 Inicia Ollama con: ollama serve");
                return Ok(());
            } else {
                println!("[selfidx] ✅ Ollama ya está corriendo");
            }
            
            // Default: Start interactive chat mode like Claude/Codex
            println!("═══════════════════════════════════════════");
            println!("   🤖 SELFIDEX - Chat Interactivo");
            println!("═══════════════════════════════════════════\n");
            
            // Show current working directory clearly
            println!("📂 Directorio de trabajo: {}", current_dir.display());
            println!();
            
            // Show current model
            println!("🤖 Modelo actual: {}", model);
            println!();
            
            // Show .selfidx.md status
            let selfidx_md_path = current_dir.join(".selfidx.md");
            if selfidx_md_path.exists() {
                println!("📄 Archivo .selfidx.md: ✅ Encontrado");
                if let Ok(content) = fs::read_to_string(&selfidx_md_path) {
                    if !content.is_empty() {
                        println!("   ({} caracteres)", content.len());
                    }
                }
            } else {
                println!("📄 Archivo .selfidx.md: ❌ No encontrado");
                println!("   💡 Se creará automáticamente al usar el chat");
            }
            println!();
            
            // Show available commands
            println!("📚 Comandos disponibles:");
            println!("  /model   - Cambiar modelo LLM");
            println!("  /clear   - Limpiar chat");
            println!("  /info    - Ver información del proyecto");
            println!("  /help    - Mostrar ayuda");
            println!("  /exit    - Salir de SELFIDEX");
            println!();
            println!("  salir    - Terminar sesión");
            println!("  limpiar  - Borrar historial");
            println!("  --       - Ver todos los comandos");
            println!();
            
            // Ask if user wants to change model
            print!("¿Quieres cambiar el modelo? (s/n): ");
            io::stdout().flush()?;
            
            let mut change_model_input = String::new();
            io::stdin().read_line(&mut change_model_input)?;
            
            if change_model_input.trim().to_lowercase() == "s" {
                // Show available models
                println!("\n📦 Listando modelos disponibles en Ollama...\n");
                
                match client.list_models().await {
                    Ok(models) => {
                        if models.is_empty() {
                            println!("[selfidx] No hay modelos disponibles en Ollama");
                            println!("[selfidx] Descarga modelos con: ollama pull <modelo>");
                            println!();
                        } else {
                            println!("=== Modelos Disponibles ===\n");
                            for (i, m) in models.iter().enumerate() {
                                let marker = if *m == model { " ⭐ (ACTUAL)" } else { "" };
                                println!("{}. {}{}", i + 1, m, marker);
                            }
                            println!();
                            
                            print!("Selecciona un modelo (1-{}): ", models.len());
                            io::stdout().flush()?;
                            
                            let mut model_input = String::new();
                            io::stdin().read_line(&mut model_input)?;
                            
                            if let Ok(index) = model_input.trim().parse::<usize>() {
                                if index >= 1 && index <= models.len() {
                                    let new_model = models[index - 1].clone();
                                    println!("\n✅ Modelo seleccionado: {}\n", new_model);
                                    
                                    // Use new model for this session
                                    // Note: This only affects the current session
                                    println!("[selfidx] Usando modelo: {} para esta sesión", new_model);
                                    println!("[selfidx] Para cambiar permanentemente, usa: selfidx --model {}", new_model);
                                    println!();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("[selfidx] Error al listar modelos: {}", e);
                        println!();
                    }
                }
            }
            
            println!("═══════════════════════════════════════════");
            println!("   Iniciando chat interactivo...");
            println!("═══════════════════════════════════════════\n");
            
            let client = OllamaClient::from_env();
            
            println!("[selfidx] Conectando a Ollama...");
            if !client.is_available().await {
                println!("[selfidx-error] Ollama no está disponible.");
                println!("[selfidx] Inicia Ollama con: ollama serve");
                return Ok(());
            }
            
            println!("[selfidx] Modelo: {} ✓\n", model);
            
            // Use .selfidx.md in project root for session logging
            let selfidx_md_path = current_dir.join(".selfidx.md");
            
            // Chat history with simplified tool instructions
            let mut messages = vec![
                Message {
                    role: "system".to_string(),
                    content: format!(
                        "Eres SELFIDEX, un asistente de programación AUTÓNOMO.\n\n\
                        HERRAMIENTAS DISP:\n {}\n\n\
                        INSTRUCCIONES IMPORTANTES:\n\
                        1. Cuando el usuario pide algo, DEBES ejecutar una herramienta\n\
                        2. Para leer archivos: EJECUTA read_file con path=\"archivo\"\n\
                        3. Para ejecutar comandos: EJECUTA execute_command con command=\"comando\"\n\
                        4. Para listar archivos: EJECUTA list_files con path=\".\"\n\
                        5. Para escribir archivos: EJECUTA write_file con path=\"archivo\" y content=\"texto\"\n\
                        6. Responde en español\n\
                        7. NO expliques, EJECUTA directamente\n\n\
                        EJEMPLO:\n\
                        Usuario: lee el archivo main.rs\n\
                        Tú: EJECUTA read_file con path=\"src/main.rs\"\n\n\
                        Usuario: lista los archivos\n\
                        Tú: EJECUTA list_files con path=\".\"\n\n\
                        Cuando el usuario pide algo, EJECUTA la herramienta inmediatamente.",
                        Agent::get_tools_description()
                    ),
                    tool_calls: None,
                },
            ];
            
            // Append session start to .selfidx.md
            let session_header = format!("\n\n---\n## 📝 Sesión: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&selfidx_md_path) {
                let _ = file.write_all(session_header.as_bytes());
            }
            
            println!("📝 Registro en: {}\n", selfidx_md_path.display());
            
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

                        
                        "clear" | "limpiar" => {
                            messages = vec![
                                Message {
                                    role: "system".to_string(),
                                    content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                                    Ejecuta acciones directamente para completar las tareas del usuario.".to_string(),
                                    tool_calls: None,
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
            println!("🌐 Ollama: http://localhost:11434");
                            continue;
                        }
                        
                        "exit" | "quit" | "salir" => {
                            // Save final log to .selfidx.md
                            let session_end = format!("\n## 🏁 Fin de sesión: {}\n\n---\n", 
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&selfidx_md_path) {
                                let _ = file.write_all(session_end.as_bytes());
                            }
                            println!("\n📝 Registro guardado en: {}", selfidx_md_path.display());
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
                    // Save final log to .selfidx.md
                    let session_end = format!("\n## 🏁 Fin de sesión: {}\n\n---\n", 
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&selfidx_md_path) {
                        let _ = file.write_all(session_end.as_bytes());
                    }
                    println!("\n📝 Registro guardado en: {}", selfidx_md_path.display());
                    println!("\n👋 Hasta luego!");
                    break;
                }
                
                if input == "limpiar" || input == "clear" {
                    messages = vec![
                        Message {
                            role: "system".to_string(),
                            content: "Eres un asistente de programación AUTÓNOMO. Tienes acceso completo al sistema de archivos y terminal. \
                            Ejecuta acciones directamente para completar las tareas del usuario.".to_string(),
                            tool_calls: None,
                        },
                    ];
                    println!("✅ Chat limpiado");
                    continue;
                }
                
                // Save user input to .selfidx.md
                let user_entry = format!("\n### 👤 Usuario:\n{}\n", input);
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&selfidx_md_path) {
                    let _ = file.write_all(user_entry.as_bytes());
                }
                
                messages.push(Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                    tool_calls: None,
                });
                
                // Animated thinking indicator
                print!("\n🤖 Pensando");
                io::stdout().flush().unwrap();
                for _ in 0..3 {
                    print!(".");
                    io::stdout().flush().unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
                println!();
                
                // Convert agent tools to Ollama format
                let ollama_tools = Some(OllamaClient::convert_agent_tools_to_ollama(Agent::get_tools()));
                match client.chat_with_tools(model.clone(), messages.clone(), ollama_tools).await {
                    Ok(response) => {
                        // Check if response has tool calls from Ollama
                        if let Some(tool_calls) = response.tool_calls() {
                            println!("→ Ollama solicitó {} herramienta(s)", tool_calls.len());
                            
                            let agent = Agent::new();
                            for tool_call in tool_calls {
                                let tool_name = &tool_call.function.name;
                                let params: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                                    .unwrap_or(serde_json::json!({}));
                                
                                println!("→ Ejecutando: {} {:?}", tool_name, params);

                                let result = if Agent::is_destructive_tool(tool_name) {
                                    match agent.execute_tool_with_confirmation(tool_name, &params) {
                                        Ok((result, confirmed)) if confirmed => result,
                                        _ => {
                                            messages.push(Message {
                                                role: "user".to_string(),
                                                content: "Acción cancelada.".to_string(),
                                                tool_calls: None,
                                            });
                                            continue;
                                        }
                                    }
                                } else {
                                    match agent.execute_tool(tool_name, &params) {
                                        Ok(result) => result,
                                        Err(e) => format!("Error: {}", e),
                                    }
                                };

                                // Add tool result to messages
                                messages.push(Message {
                                    role: "tool".to_string(),
                                    content: result,
                                    tool_calls: None,
                                });
                            }
                            
                            // Continue conversation with tool results
                            continue;
                        }

                        let content = response.content().to_string();
                        println!("\n🤖:\n{}", content);
                        
                        // Save AI response to .selfidx.md
                        let ai_entry = format!("\n### 🤖 SELFIDEX:\n{}\n", content);
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&selfidx_md_path) {
                            let _ = file.write_all(ai_entry.as_bytes());
                        }
                        
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content,
                            tool_calls: None,
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
