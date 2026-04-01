// TUI Module - SELFIDEX v3.0 Multi-API Enhanced
// Terminal UI con soporte para 13 proveedores LLM

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear},
    Terminal, Frame,
};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender, Receiver};
use crate::agent::Agent;
use crate::llm::providers::{LlmProvider, LlmClient, Message, Tool, GenerationConfig};
use crate::terminal::capsule::render_capsule;

/// Mensajes asíncronos para la TUI
pub enum TuiMessage {
    LlmResponse(Result<String, anyhow::Error>),
    LlmChunk(String),  // Chunk de streaming
    ToolResult(String),
    Error(String),
}

/// Aplicación TUI principal
pub struct TuiApp {
    pub agent: Agent,
    pub llm_client: LlmClient,
    pub files: Vec<FileEntry>,
    pub selected_file: Option<usize>,
    pub file_content: String,
    pub messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub cursor_position: usize,
    pub terminal_output: Vec<String>,
    pub selected_provider: usize,
    pub selected_model: usize,
    pub show_provider_selector: bool,
    pub show_model_selector: bool,
    pub providers: Vec<LlmProvider>,
    pub models: Vec<String>,
    pub scroll_offset: usize,
    pub status: AppStatus,
    pub message_tx: Option<Sender<TuiMessage>>,
    pub message_rx: Option<Receiver<TuiMessage>>,
    pub pending_response: bool,
    pub streaming_buffer: String,  // Buffer para streaming
    pub is_streaming: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Idle,
    Thinking,
    ExecutingTool,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let agent = Agent::new();
        let current_dir = agent.project_root.clone();
        let files = Self::load_directory(&current_dir);
        let llm_client = LlmClient::from_env();
        
        let providers = vec![
            LlmProvider::Ollama,
            LlmProvider::OpenAI,
            LlmProvider::Anthropic,
            LlmProvider::Groq,
            LlmProvider::LMStudio,
            LlmProvider::Grok,
            LlmProvider::Gemini,
            LlmProvider::OpenRouter,
            LlmProvider::DeepSeek,
            LlmProvider::Mistral,
        ];

        let models = llm_client.provider.popular_models()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Crear canal para mensajes
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            agent,
            llm_client,
            files,
            selected_file: None,
            file_content: String::new(),
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "¡Bienvenido a SELFIDEX v3.0! Presiona H para ayuda.".to_string(),
                timestamp: chrono::Local::now().format("%H:%M").to_string(),
            }],
            input_buffer: String::new(),
            cursor_position: 0,
            terminal_output: vec![],
            selected_provider: 0,
            selected_model: 0,
            show_provider_selector: false,
            show_model_selector: false,
            providers,
            models,
            scroll_offset: 0,
            status: AppStatus::Idle,
            message_tx: Some(tx),
            message_rx: Some(rx),
            pending_response: false,
            streaming_buffer: String::new(),
            is_streaming: false,
        })
    }

    fn load_directory(path: &PathBuf) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        if let Ok(dir) = std::fs::read_dir(path) {
            for entry in dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('$') {
                    continue;
                }
                if let Ok(metadata) = entry.metadata() {
                    entries.push(FileEntry {
                        name,
                        path: path.clone(),
                        is_dir: metadata.is_dir(),
                    });
                }
            }
        }
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        entries
    }

    pub fn refresh_directory(&mut self) {
        self.files = Self::load_directory(&self.agent.project_root);
        if let Some(idx) = self.selected_file {
            if idx >= self.files.len() {
                self.selected_file = None;
            }
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
        self.scroll_offset = self.messages.len();
    }

    pub fn set_status(&mut self, status: AppStatus) {
        self.status = status;
    }

    pub fn cycle_provider(&mut self) {
        self.selected_provider = (self.selected_provider + 1) % self.providers.len();
        self.llm_client.provider = self.providers[self.selected_provider];
        self.llm_client.endpoint = self.llm_client.provider.default_endpoint().to_string();
        self.add_message("system", &format!("Proveedor cambiado a: {}", self.llm_client.provider));
    }

    pub fn toggle_provider_selector(&mut self) {
        self.show_provider_selector = !self.show_provider_selector;
    }

    pub fn toggle_model_selector(&mut self) {
        self.show_model_selector = !self.show_model_selector;
    }

    pub fn select_provider(&mut self, index: usize) {
        if index < self.providers.len() {
            self.selected_provider = index;
            self.llm_client.provider = self.providers[index];
            self.llm_client.endpoint = self.llm_client.provider.default_endpoint().to_string();
            self.models = self.llm_client.provider.popular_models()
                .iter()
                .map(|s| s.to_string())
                .collect();
            self.selected_model = 0;
            self.show_provider_selector = false;
            self.add_message("system", &format!("✅ Proveedor: {}", self.llm_client.provider));
        }
    }

    pub fn select_model(&mut self, index: usize) {
        if index < self.models.len() {
            self.selected_model = index;
            self.show_model_selector = false;
            self.add_message("system", &format!("✅ Modelo: {}", self.models[index]));
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Función principal para ejecutar la TUI
pub fn run_enhanced_tui() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new()?;

    // Habilitar ratón
    crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;

    // Habilitar modo raw para mejor soporte de terminal
    crossterm::terminal::enable_raw_mode()?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Check for LLM responses
        if let Some(rx) = &app.message_rx {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    TuiMessage::LlmChunk(chunk) => {
                        // Streaming: agregar al buffer y mostrar
                        app.streaming_buffer.push_str(&chunk);
                        app.is_streaming = true;
                        app.set_status(AppStatus::Thinking);
                        // Actualizar último mensaje con el buffer
                        if let Some(last) = app.messages.last_mut() {
                            if last.role == "assistant" && last.content.starts_with("🔄 ") {
                                last.content = app.streaming_buffer.clone();
                            }
                        }
                    }
                    TuiMessage::LlmResponse(Ok(content)) => {
                        // Respuesta completa (fallback si no hay streaming)
                        if !app.is_streaming {
                            app.add_message("assistant", &content);
                        } else {
                            // Finalizar streaming
                            app.is_streaming = false;
                        }
                        app.set_status(AppStatus::Idle);
                        app.pending_response = false;
                    }
                    TuiMessage::LlmResponse(Err(e)) => {
                        app.add_message("system", &format!("❌ Error: {}", e));
                        app.set_status(AppStatus::Error(e.to_string()));
                        app.pending_response = false;
                        app.is_streaming = false;
                    }
                    TuiMessage::ToolResult(result) => {
                        app.add_message("assistant", &format!("🛠️ Resultado: {}", result));
                    }
                    TuiMessage::Error(e) => {
                        app.add_message("system", &format!("❌ {}", e));
                        app.set_status(AppStatus::Error(e));
                        app.pending_response = false;
                        app.is_streaming = false;
                    }
                }
            }
        }

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                // Eventos de ratón
                crossterm::event::Event::Mouse(mouse_event) => {
                    handle_mouse_event(&mut app, mouse_event)?;
                }
                
                // Eventos de teclado
                crossterm::event::Event::Key(key) => {
                    handle_key_event(&mut app, key)?;
                }
                
                // Eventos de redimensionamiento
                crossterm::event::Event::Resize(_, _) => {
                    // La TUI se redimensiona automáticamente
                }
                
                _ => {}
            }
        }
    }
}

/// Manejar eventos de ratón
fn handle_mouse_event(app: &mut TuiApp, event: crossterm::event::MouseEvent) -> Result<()> {
    use crossterm::event::{MouseEventKind, MouseButton};
    
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Click izquierdo - seleccionar/abrir archivo
            // Calculamos si el click está en el panel de archivos
            // El panel de archivos ocupa los últimos 10 rows
            let screen_height = crossterm::terminal::size()?.1;
            let files_panel_start = screen_height - 10;
            
            if event.row >= files_panel_start {
                // Click en panel de archivos
                let file_index = (event.row - files_panel_start) as usize;
                if file_index < app.files.len() {
                    app.selected_file = Some(file_index);
                    
                    // Abrir archivo o entrar en directorio
                    let file = app.files[file_index].clone();
                    if file.is_dir {
                        if std::env::set_current_dir(&file.path).is_ok() {
                            app.agent.project_root = file.path.clone();
                            app.refresh_directory();
                            app.selected_file = None;
                            app.add_message("system", &format!("📁 cd {}", file.name));
                        }
                    } else {
                        // Mostrar diff/contenido del archivo
                        show_file_diff(app, &file)?;
                    }
                }
            }
        }
        
        MouseEventKind::ScrollUp => {
            // Scroll up en mensajes
            if app.scroll_offset > 0 {
                app.scroll_offset -= 1;
            }
        }
        
        MouseEventKind::ScrollDown => {
            // Scroll down en mensajes
            if app.scroll_offset < app.messages.len() {
                app.scroll_offset += 1;
            }
        }
        
        _ => {}
    }
    
    Ok(())
}

/// Mostrar diff/contenido de archivo
fn show_file_diff(app: &mut TuiApp, file: &FileEntry) -> Result<()> {
    match app.agent.read_file(&file.name) {
        Ok(content) => {
            app.file_content = content.clone();
            app.add_message("system", &format!("📄 {}", file.name));
            
            // Mostrar primeras líneas como preview
            let preview: String = content
                .lines()
                .take(20)
                .enumerate()
                .map(|(i, line)| format!("{:4} │ {}", i + 1, line))
                .collect::<Vec<_>>()
                .join("\n");
            
            app.add_message("assistant", &format!(
                "=== {} ===\n{}\n{}",
                file.name,
                preview,
                if content.lines().count() > 20 {
                    format!("\n... ({} líneas totales)", content.lines().count())
                } else {
                    String::new()
                }
            ));
        }
        Err(e) => {
            app.add_message("system", &format!("❌ Error: {}", e));
        }
    }
    Ok(())
}

/// Enviar mensaje al LLM en background con streaming
fn send_to_llm(app: &mut TuiApp, prompt: &str) -> Result<()> {
    let provider = app.llm_client.provider;
    let endpoint = app.llm_client.endpoint.clone();
    let api_key = app.llm_client.api_key.clone();
    let model = app.models.get(app.selected_model).cloned().unwrap_or_else(|| "llama3".to_string());
    let tx = app.message_tx.clone().unwrap();

    // Detectar tipo de tarea y obtener configuración óptima
    let config = GenerationConfig::from_prompt(prompt);

    // Crear mensajes del chat (últimos 10 para contexto)
    let recent_messages: Vec<Message> = app.messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .take(10)
        .map(|m| Message {
            role: m.role.clone(),
            content: m.content.clone(),
            tool_calls: None,
        })
        .collect();

    // Spawn thread para llamada LLM
    std::thread::spawn(move || {
        let client = LlmClient::from_provider(provider, api_key.clone());
        
        // Crear system prompt con contexto del proyecto
        let system_prompt = format!(
            "Eres un asistente de programación experto en SELFIDEX. \
             Proveedor: {} | Modelo: {} | Endpoint: {}",
            provider, model, endpoint
        );

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: system_prompt,
            tool_calls: None,
        }];
        messages.extend(recent_messages);

        // Añadir mensaje placeholder para streaming
        let _ = tx.send(TuiMessage::LlmChunk("🔄 Respondiendo...".to_string()));

        // Enviar request con configuración optimizada
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                client.chat_with_config(model, messages, None, config).await
            });

        match result {
            Ok(response) => {
                let full_content = response.content().to_string();
                
                // Simular streaming carácter por carácter
                for chunk in full_content.chars() {
                    let _ = tx.send(TuiMessage::LlmChunk(chunk.to_string()));
                    // Pequeño delay para simular streaming real
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                
                // Enviar señal de finalización
                let _ = tx.send(TuiMessage::LlmResponse(Ok(full_content)));
            }
            Err(e) => {
                let _ = tx.send(TuiMessage::LlmResponse(Err(e)));
            }
        }
    });

    Ok(())
}

/// Manejar eventos de teclado
fn handle_key_event(app: &mut TuiApp, key: crossterm::event::KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;
    
    match key.code {
        // Salir
        KeyCode::Esc => {
            if app.show_provider_selector || app.show_model_selector {
                app.show_provider_selector = false;
                app.show_model_selector = false;
            } else {
                // Cleanup
                crossterm::terminal::disable_raw_mode()?;
                crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;
                std::process::exit(0);
            }
        }
        
        // Navegación archivos
        KeyCode::Up => {
            if !app.show_provider_selector && !app.show_model_selector {
                if let Some(idx) = app.selected_file {
                    if idx > 0 {
                        app.selected_file = Some(idx - 1);
                    }
                } else if !app.files.is_empty() {
                    app.selected_file = Some(0);
                }
            }
        }
        KeyCode::Down => {
            if !app.show_provider_selector && !app.show_model_selector {
                if let Some(idx) = app.selected_file {
                    if idx < app.files.len().saturating_sub(1) {
                        app.selected_file = Some(idx + 1);
                    }
                } else if !app.files.is_empty() {
                    app.selected_file = Some(0);
                }
            }
        }
        
        // Seleccionar archivo
        KeyCode::Enter => {
            if app.show_provider_selector {
                app.select_provider(app.selected_provider);
            } else if app.show_model_selector {
                app.select_model(app.selected_model);
            } else if let Some(idx) = app.selected_file {
                if idx < app.files.len() {
                    let file = app.files[idx].clone();
                    if file.is_dir {
                        if std::env::set_current_dir(&file.path).is_ok() {
                            app.agent.project_root = file.path.clone();
                            app.refresh_directory();
                            app.selected_file = None;
                            app.add_message("system", &format!("📁 cd {}", file.name));
                        }
                    } else {
                        show_file_diff(app, &file)?;
                    }
                }
            }
        }
        
        // Ir atrás
        KeyCode::Backspace => {
            if let Some(parent) = app.agent.project_root.parent() {
                if std::env::set_current_dir(parent).is_ok() {
                    app.agent.project_root = parent.to_path_buf();
                    app.refresh_directory();
                    app.selected_file = None;
                    app.add_message("system", "📁 cd ..");
                }
            }
        }
        
        // Cambiar proveedor (P)
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.toggle_provider_selector();
        }
        
        // Cambiar modelo (M)
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.toggle_model_selector();
        }
        
        // Input de texto
        KeyCode::Char(c) => {
            if !app.show_provider_selector && !app.show_model_selector {
                app.input_buffer.insert(app.cursor_position, c);
                app.cursor_position += 1;
            }
        }
        
        // Borrar
        KeyCode::Backspace => {
            if !app.show_provider_selector && !app.show_model_selector {
                if app.cursor_position > 0 {
                    app.input_buffer.remove(app.cursor_position - 1);
                    app.cursor_position -= 1;
                }
            }
        }
        
        // Enviar mensaje (Enter en input)
        KeyCode::Enter => {
            if !app.show_provider_selector && !app.show_model_selector && !app.input_buffer.is_empty() {
                let input = app.input_buffer.clone();
                app.add_message("user", &input);
                app.input_buffer.clear();
                app.cursor_position = 0;
                
                // Añadir placeholder para streaming
                app.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: "🔄 Respondiendo...".to_string(),
                    timestamp: chrono::Local::now().format("%H:%M").to_string(),
                });
                app.scroll_offset = app.messages.len();
                
                app.set_status(AppStatus::Thinking);
                app.pending_response = true;
                
                // Enviar al LLM en background
                if let Err(e) = send_to_llm(app, &input) {
                    app.add_message("system", &format!("❌ Error: {}", e));
                    app.set_status(AppStatus::Idle);
                    app.pending_response = false;
                }
            }
        }
        
        // Limpiar chat
        KeyCode::Char('l') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            app.messages.clear();
            app.add_message("system", "Chat limpiado");
        }
        
        // Help
        KeyCode::Char('h') | KeyCode::F(1) => {
            app.add_message("system", "🖱️ Click: Abrir archivo | P: Proveedores | M: Modelo | H: Help | ESC: Salir | Enter: Enviar");
        }
        
        _ => {}
    }
    
    Ok(())
}

/// Función de renderizado principal
fn ui(f: &mut Frame, app: &mut TuiApp) {
    let size = f.size();
    
    // Layout principal
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Chat/Messages
            Constraint::Length(3),  // Input
            Constraint::Length(10), // Files
        ])
        .split(size);

    // Header
    render_header(f, app, chunks[0]);
    
    // Messages/Chat area
    render_messages(f, app, chunks[1]);
    
    // Input
    render_input(f, app, chunks[2]);
    
    // Files
    render_files(f, app, chunks[3]);
    
    // Provider selector overlay
    if app.show_provider_selector {
        render_provider_selector(f, app);
    }
    
    // Model selector overlay
    if app.show_model_selector {
        render_model_selector(f, app);
    }
}

fn render_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let header = Paragraph::new(render_capsule())
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    f.render_widget(header, area);
}

fn render_messages(f: &mut Frame, app: &TuiApp, area: Rect) {
    let messages: Vec<Line> = app.messages.iter().map(|msg| {
        let style = match msg.role.as_str() {
            "user" => Style::default().fg(Color::Cyan),
            "assistant" => Style::default().fg(Color::Green),
            "system" => Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM),
            _ => Style::default(),
        };
        Line::from(vec![
            Span::styled(format!("[{}] ", msg.timestamp), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}: ", msg.role), style),
            Span::raw(&msg.content),
        ])
    }).collect();

    let messages_widget = Paragraph::new(messages)
        .block(Block::default()
            .title(format!(" 💬 Chat - {}:{} ", app.llm_client.provider, app.models.get(app.selected_model).unwrap_or(&"model".to_string())))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));
    
    f.render_widget(messages_widget, area);
}

fn render_input(f: &mut Frame, app: &TuiApp, area: Rect) {
    let status_symbol = match &app.status {
        AppStatus::Idle => "○",
        AppStatus::Thinking => "🔄",
        AppStatus::ExecutingTool => "⚙️",
        AppStatus::Error(_) => "❌",
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .block(Block::default()
            .title(format!(" {} Input (Enter: enviar) ", status_symbol))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(input, area);
}

fn render_files(f: &mut Frame, app: &TuiApp, area: Rect) {
    let files: Vec<ListItem> = app.files.iter().enumerate().map(|(i, file)| {
        let icon = if file.is_dir { "📁" } else { "🖱️" };
        let style = if Some(i) == app.selected_file {
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(format!("{} {} {}", icon, file.name, if !file.is_dir { "(click)" } else { "" })).style(style)
    }).collect();

    let files_widget = List::new(files)
        .block(Block::default()
            .title(" 📁 Archivos (click para abrir) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    
    f.render_widget(files_widget, area);
}

fn render_provider_selector(f: &mut Frame, app: &TuiApp) {
    let area = centered_rect(60, 50, f.size());
    f.render_widget(Clear, area);

    let providers: Vec<ListItem> = app.providers.iter().enumerate().map(|(i, p)| {
        let style = if i == app.selected_provider {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        let tools = if p.supports_tools() { "✅" } else { "❌" };
        let api = if p.requires_api_key() { "🔑" } else { "🆓" };
        ListItem::new(format!("{} {} {} {}", tools, api, p.as_str(), p.default_endpoint())).style(style)
    }).collect();

    let selector = List::new(providers)
        .block(Block::default()
            .title(" 🤖 Seleccionar Proveedor (Enter: confirmar, ESC: cancelar) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    
    f.render_widget(selector, area);
}

fn render_model_selector(f: &mut Frame, app: &TuiApp) {
    let area = centered_rect(60, 50, f.size());
    f.render_widget(Clear, area);

    let models: Vec<ListItem> = app.models.iter().enumerate().map(|(i, m)| {
        let style = if i == app.selected_model {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(m.clone()).style(style)
    }).collect();

    let selector = List::new(models)
        .block(Block::default()
            .title(" 📦 Seleccionar Modelo (Enter: confirmar, ESC: cancelar) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    
    f.render_widget(selector, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
