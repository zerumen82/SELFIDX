// TUI Module - Retro terminal UI

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::path::PathBuf;
use crate::agent::Agent;

pub struct TuiApp {
    pub agent: Agent,
    pub files: Vec<FileEntry>,
    pub selected_file: Option<usize>,
    pub file_content: String,
    pub terminal_output: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        let agent = Agent::new();
        let current_dir = agent.project_root.clone();
        let files = Self::load_directory(&current_dir);
        
        Self {
            agent,
            files,
            selected_file: None,
            file_content: String::new(),
            terminal_output: vec!["SELFIDEX TUI - v0.1 - TAB: panel, ESC: salir, ENTER: abrir, BACKSPACE: ir atras".to_string()],
        }
    }
    
    fn load_directory(path: &PathBuf) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        
        if let Ok(dir) = std::fs::read_dir(path) {
            for entry in dir.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                if name.starts_with('.') {
                    continue;
                }
                
                entries.push(FileEntry {
                    name,
                    path: path.clone(),
                    is_dir: path.is_dir(),
                });
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
    }
    
    pub fn select_file(&mut self, index: usize) {
        if index < self.files.len() {
            let file = self.files[index].clone();
            let path = file.path.clone();
            let name = file.name.clone();
            let is_dir = file.is_dir;
            
            if is_dir {
                // Change to directory using agent
                if std::env::set_current_dir(&path).is_ok() {
                    self.agent.project_root = path;
                    self.refresh_directory();
                    self.selected_file = None;
                    self.terminal_output.push(format!(" cd {}", name));
                }
            } else {
                self.selected_file = Some(index);
                // Read file using agent
                match self.agent.read_file(&name) {
                    Ok(content) => {
                        self.file_content = content;
                        self.terminal_output.push(format!("Opened: {}", name));
                    }
                    Err(e) => {
                        self.file_content = format!("[Error: {}]", e);
                    }
                }
            }
        }
    }
    
    pub fn go_up(&mut self) {
        if let Some(parent) = self.agent.project_root.parent() {
            if std::env::set_current_dir(parent).is_ok() {
                self.agent.project_root = parent.to_path_buf();
                self.refresh_directory();
                self.selected_file = None;
                self.terminal_output.push(" cd ..".to_string());
            }
        }
    }
    
    pub fn execute_command(&mut self, cmd: &str) {
        match self.agent.execute_command(cmd) {
            Ok(result) => {
                if !result.stdout.is_empty() {
                    self.terminal_output.push(format!("$ {}", cmd));
                    self.terminal_output.push(result.stdout);
                }
                if !result.stderr.is_empty() {
                    self.terminal_output.push(format!("ERR: {}", result.stderr));
                }
            }
            Err(e) => {
                self.terminal_output.push(format!("Error: {}", e));
            }
        }
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_tui() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = TuiApp::new();
    
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.size());
            
            let title = Paragraph::new(" SELFIDEX ")
                .style(Style::default().fg(Color::Green).bold())
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
            f.render_widget(title, chunks[0]);
            
            let main = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(45), Constraint::Percentage(35)])
                .split(chunks[1]);
            
            // File list
            let items: Vec<ListItem> = app.files.iter().enumerate().map(|(i, f)| {
                let icon = if f.is_dir { "[DIR]" } else { "[FILE]" };
                let name = format!("{} {}", icon, f.name);
                if app.selected_file == Some(i) {
                    ListItem::new(name).style(Style::default().fg(Color::Black).bg(Color::Green))
                } else {
                    ListItem::new(name).style(Style::default().fg(Color::Green))
                }
            }).collect();
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" ARCHIVOS ").border_style(Style::default().fg(Color::Green)));
            f.render_widget(list, main[0]);
            
            // Editor
            let editor = Paragraph::new(app.file_content.as_str())
                .block(Block::default().borders(Borders::ALL).title(" EDITOR ").border_style(Style::default().fg(Color::Green)));
            f.render_widget(editor, main[1]);
            
            // Terminal
            let term_output = app.terminal_output.join("\n");
            let term = Paragraph::new(term_output.as_str())
                .block(Block::default().borders(Borders::ALL).title(" TERMINAL ").border_style(Style::default().fg(Color::Green)));
            f.render_widget(term, main[2]);
        })?;
        
        if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
            match key.code {
                    crossterm::event::KeyCode::Esc => break,
                    crossterm::event::KeyCode::Up => {
                        if let Some(idx) = app.selected_file {
                            if idx > 0 { app.select_file(idx - 1); }
                        } else if !app.files.is_empty() { app.select_file(0); }
                    }
                    crossterm::event::KeyCode::Down => {
                        if let Some(idx) = app.selected_file {
                            if idx < app.files.len() - 1 { app.select_file(idx + 1); }
                        } else if !app.files.is_empty() { app.select_file(0); }
                    }
                    crossterm::event::KeyCode::Enter => {
                        if let Some(idx) = app.selected_file { app.select_file(idx); }
                        else if !app.files.is_empty() { app.select_file(0); }
                    }
                    crossterm::event::KeyCode::Backspace => app.go_up(),
                    _ => {}
                }
        }
    }
    
    Ok(())
}
