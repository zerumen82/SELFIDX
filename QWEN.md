# SELFIDEX v3.0 - Project Context

## Project Overview

**SELFIDEX v3.0** is a modern terminal application with integrated AI capabilities, built in Rust. It provides a PowerShell-based terminal interface with native Ollama integration for local AI model interaction.

### Core Features

- **Integrated Terminal**: Native PowerShell terminal with rich UI rendering using `ratatui` and `crossterm`
- **Ollama Integration**: AI chat with local models (default port 11434)
- **Autonomous Agent**: Codex-style programming assistant with tool execution capabilities
- **File Operations**: Read, write, search, and navigate project files
- **Project Context**: Persistent project context via `.selfidx.md` files
- **Global Installation**: `selfidx` command available in PATH after installation

### Technology Stack

- **Language**: Rust (Edition 2021)
- **CLI Framework**: `clap` with derive macros
- **Async Runtime**: `tokio` with full features
- **Terminal UI**: `ratatui` + `crossterm`
- **HTTP Client**: `reqwest` for Ollama API communication
- **PTY**: `portable-pty` for PowerShell integration
- **Configuration**: TOML-based config files

## Project Structure

```
selfidx/
├── src/
│   ├── main.rs           # CLI entry point, argument parsing, main loop
│   ├── lib.rs            # Module exports
│   ├── config.rs         # Configuration management (TOML)
│   ├── project.rs        # Project context handling
│   ├── autonomous.rs     # Autonomous mode implementation
│   ├── agent/
│   │   └── mod.rs        # AI agent with tool execution (read/write files, commands)
│   ├── cli/              # CLI-specific modules
│   ├── llm/
│   │   └── mod.rs        # Ollama client, model management, tool conversion
│   ├── shell/            # PowerShell integration
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── capsule.rs    # Minimalist green capsule UI rendering
│   │   └── tui.rs        # Terminal UI implementation
│   └── utils/            # Hardware detection, utilities
├── assets/               # Static resources
├── scripts/
│   ├── build-bundle.bat  # Build with cargo-bundle (MSI installer)
│   ├── build-installer.bat
│   ├── install.bat
│   └── installer.iss     # Inno Setup script
├── logs/                 # Session logs
├── dist/                 # Build output
└── Cargo.toml            # Dependencies and bundle metadata
```

## Building and Running

### Prerequisites

- **Rust**: Install from https://rustup.rs/
- **Ollama**: Running at `http://localhost:11434` (for AI features)

### Development

```bash
# Run in development mode
cargo run

# Run with specific command
cargo run -- --help
cargo run -- agent "create a React component"
cargo run -- chat
```

### Building Release

```bash
# Build release binary
cargo build --release

# Build with cargo-bundle (generates MSI installer)
scripts\build-bundle.bat

# Or manually
cargo install cargo-bundle
cargo bundle --release --format msi
```

### Installation

```bash
# Install to PATH
selfidx --install

# Or use the build script output
# Executable: dist\selfidx.exe
# Installer: dist\*.msi (if cargo-bundle succeeded)
```

### Testing

```bash
cargo test
```

## Usage Commands

| Command | Description |
|---------|-------------|
| `selfidx` | Launch terminal UI |
| `selfidx agent <prompt>` | Autonomous AI agent mode |
| `selfidx chat` | Interactive chat mode |
| `selfidx --model <name>` | Specify model to use |
| `selfidx --create <path>` | Create a file |
| `selfidx --edit <path>` | Edit a file |
| `selfidx --files` | List files |
| `selfidx --diff <a> <b>` | Compare files |
| `selfidx --tree` | Show project structure |
| `selfidx --sysinfo` | System information |
| `selfidx --models` | List available models |
| `selfidx --use-model <name>` | Switch model |

## Configuration

### Environment Variables

```bash
OLLAMA_BASE_URL=http://localhost:11434/v1  # Ollama endpoint
OLLAMA_MODEL=llama3                        # Default model
```

### Config File Location

- **Windows**: `%APPDATA%\selfidx\config.toml`

### Recommended Ollama Models

- `llama3` - Meta Llama 3 (8B)
- `llama3:70b` - Meta Llama 3 (70B)
- `codellama` - Code Llama (7B-34B)
- `mistral` - Mistral 7B
- `phi3` - Microsoft Phi-3 (3.8B)

## Agent Capabilities

The AI agent has access to these tools:

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents |
| `write_file` | Create/overwrite files |
| `execute_command` | Run shell commands (PowerShell on Windows) |
| `list_files` | List directory contents |
| `search` | Search for patterns in files |
| `delete` | Delete files (requires confirmation) |
| `create_directory` | Create directories |

### OS Detection

The agent automatically detects the operating system and uses appropriate shells:
- **Windows**: PowerShell
- **macOS**: zsh
- **Linux**: bash

## Development Conventions

### Code Style

- Spanish language for user-facing messages and prompts
- English for code identifiers, comments, and technical documentation
- Minimal emoji usage (removed in v3.0, text-only messages)
- Green aesthetic for capsule UI

### Project Context

- `.selfidx.md` file tracks project progress and context
- Created automatically on first run in a directory
- Read by the agent for persistent context across sessions

### Session Logging

- Logs stored in `logs/` directory
- Session files named: `session_YYYYMMDD_HHMMSS.md`

## Key Files Reference

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies, package metadata, bundle configuration |
| `src/main.rs` | CLI parsing, main entry point, command routing |
| `src/agent/mod.rs` | Agent tools, file operations, command execution |
| `src/llm/mod.rs` | Ollama client, model management, chat completion |
| `src/terminal/capsule.rs` | Visual capsule rendering |
| `src/config.rs` | TOML configuration handling |
| `scripts/build-bundle.bat` | Windows build script for MSI generation |

## Common Workflows

### First Run in New Project

1. Run `selfidx` in project directory
2. Select an Ollama model from available options
3. `.selfidx.md` is created automatically
4. Project context is displayed

### Using Agent Mode

```bash
# Ask the agent to create something
selfidx agent "create a React button component"

# The agent will:
# 1. Analyze project structure
# 2. Execute tools to create files
# 3. Report results
```

### Switching Models

```bash
# List available models
selfidx --models

# Switch to a different model
selfidx --use-model codellama
```

## Troubleshooting

### Ollama Not Available

```bash
# Start Ollama server
ollama serve

# Pull a model if none available
ollama pull llama3
```

### Model Not Found

```bash
# List installed models
ollama list

# Install required model
ollama pull <model-name>
```

### PATH Issues After Install

- Close and reopen terminal after `selfidx --install`
- Verify installation: `where selfidx` (Windows)

## License

MIT License
