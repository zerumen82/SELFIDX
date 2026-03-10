use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use selfidx::autonomous::run_autonomous_loop;
use selfidx::llm::{Message, VllmClient};
use selfidx::utils::hardware::SystemInfo;
use selfidx::agent::Agent;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "selfidx")]
#[command(author = "SELFIDEX")]
#[command(version = "3.0.0")]
#[command(about = "Terminal integrada con vLLM - SELFIDEX v3.0", long_about = None)]
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

    /// Modo autónomo
    Auto {
        #[arg(trailing_var_arg(true))]
        task: Vec<String>,
    },
}

/// Render Cápsula SELFIDEX
fn render_capsule() -> String {
    r#"
    █████
 ██████████    SELFIDEX v3.0
 ██████████    [●] vLLM Conectado
    █████
"#.to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --install flag
    if args.install {
        install_selfidx()?;
        return Ok(());
    }

    // Render cápsula
    println!("{}", render_capsule());
    println!();

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
            
            let agent = Agent::new();
            let client = VllmClient::from_env();
            let model = VllmClient::default_model();
            
            if !client.is_available().await {
                println!("[selfidx-error] vLLM no está disponible.");
                println!("[selfidx-error] Inicia vLLM con: vllm serve");
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
            
            let client = VllmClient::from_env();
            let model = VllmClient::default_model();
            
            println!("[selfidx-chat] Conectando a vLLM...");
            if !client.is_available().await {
                println!("[selfidx-error] vLLM no está disponible.");
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
            println!("[selfidx-run] Ejecutando: {}", cmd);
        }
        
        Some(Commands::Run) => {
            println!("[selfidx-run] Ejecutando proyecto");
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
            println!("[selfidx-edit] Editando: {}", path);
        }
        
        Some(Commands::Files) => {
            println!("[selfidx-files] Listando archivos");
        }
        
        Some(Commands::Diff { file_a, file_b }) => {
            println!("[selfidx-diff] Comparando: {} vs {}", file_a, file_b);
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
                    println!("  vllm serve {}", repo.hf_repo);
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
            
            let models = list_installed_models();
            
            if models.is_empty() {
                println!("No hay modelos instalados.");
            } else {
                for (i, model) in models.iter().enumerate() {
                    println!("{}. {}", i + 1, model.name);
                    println!("   Tamaño: {}", model.size_display());
                    println!();
                }
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
            
            let agent = Agent::new();
            let client = VllmClient::from_env();
            let model = VllmClient::default_model();
            
            if !client.is_available().await {
                println!("[selfidx-error] vLLM no está disponible.");
                println!("Inicia vLLM con: vllm serve");
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
        
        None => {
            // Check and start vLLM if needed
            let client = VllmClient::from_env();
            let model = VllmClient::default_model();
            
            println!("[selfidx] 🔍 Verificando vLLM...");
            
            if !client.is_available().await {
                println!("[selfidx] ⚠️ vLLM no está corriendo");
                println!("[selfidx] 💡 Para iniciar vLLM, ejecuta en otra terminal:");
                println!("[selfidx]    vllm serve {}", model);
                println!();
                
                // Ask user if they want to continue anyway
                use std::io::{self, Write};
                print!("[selfidx] ¿Iniciar vLLM ahora? (s/n): ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    if input.trim().to_lowercase() == "s" || input.trim().to_lowercase() == "y" {
                        #[cfg(windows)]
                        {
                            use std::process::Command;
                            
                            // Start vLLM in new window
                            let _ = Command::new("cmd")
                                .args(["/C", "start", "cmd", "/K", &format!("vllm serve {} --host 0.0.0.0 --port 8000", model)])
                                .spawn();
                            
                            println!("[selfidx] ⏳ Esperando a que vLLM inicie (20s)...");
                            for _ in 0..20 {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                print!(".");
                            }
                            println!();
                            
                            if client.is_available().await {
                                println!("[selfidx] ✅ vLLM iniciado correctamente!");
                            } else {
                                println!("[selfidx] ❌ No se pudo iniciar vLLM");
                                return Ok(());
                            }
                        }
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            } else {
                println!("[selfidx] ✅ vLLM ya está corriendo");
            }
            
            // Default: Start interactive chat mode like Claude/Codex
            println!("═══════════════════════════════════════════");
            println!("   🤖 SELFIDEX - Chat Interactivo");
            println!("═══════════════════════════════════════════\n");
            println!("Comandos: 'salir' para terminar, 'limpiar' para borrar historial\n");
            
            let client = VllmClient::from_env();
            let model = VllmClient::default_model();
            
            println!("[selfidx] Conectando a vLLM...");
            if !client.is_available().await {
                println!("[selfidx-error] vLLM no está disponible.");
                println!("[selfidx] Inicia vLLM con: vllm serve");
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
            let new_path = format!("{};{}", current_path, install_dir_str);
            let _ = Command::new("setx")
                .args(["PATH", &new_path])
                .output();
            
            println!("[selfidx-install] ✓ Añadido al PATH del usuario");
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
