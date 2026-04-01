# SELFIDEX v3.0 - Estado Técnico del Proyecto

**Fecha**: 1 de abril de 2026  
**Versión**: 3.0.0  
**Estado**: ✅ Funcional - En desarrollo activo

---

## Resumen Ejecutivo

SELFIDEX v3.0 es una terminal integrada con capacidades de IA construida en Rust. El proyecto está **funcional** con todas las características principales implementadas: terminal TUI, integración con Ollama, agente autónomo tipo Codex, y gestión de contexto de proyecto.

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI (main.rs)                          │
│  Argument parsing • Command routing • Main loop             │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌───────────────┐
│    Terminal   │   │     Agent       │   │     LLM       │
│   (TUI/Caps)  │   │  (Tool exec)    │   │  (Ollama)     │
└───────────────┘   └─────────────────┘   └───────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌───────────────┐
│   ratatui     │   │  File/Command   │   │   reqwest     │
│   crossterm   │   │  Operations     │   │   serde_json  │
└───────────────┘   └─────────────────┘   └───────────────┘
```

---

## Estado por Módulo

### ✅ `src/main.rs` - Entry Point
**Estado**: Completo  
**Líneas**: ~1867  
**Funcionalidades**:
- [x] Parseo de argumentos CLI con `clap`
- [x] Detección automática de tipo de proyecto (Flutter, Node, Rust, Python, Go)
- [x] Gestión de contexto de proyecto (.selfidx.md)
- [x] Verificación de conexión con Ollama
- [x] Routing a todos los comandos (agent, chat, tui, auto, etc.)
- [x] Instalación automática en PATH desde ubicaciones temporales

**Comandos implementados**:
| Comando | Estado | Descripción |
|---------|--------|-------------|
| `agent` | ✅ | Agente IA autónomo |
| `chat` | ✅ | Chat interactivo |
| `command` | ✅ | Ejecutar comando |
| `run` | ✅ | Ejecutar proyecto |
| `create` | ✅ | Crear archivo |
| `edit` | ✅ | Editar archivo |
| `files` | ✅ | Listar archivos |
| `diff` | ✅ | Comparar archivos |
| `tree` | ✅ | Ver estructura |
| `sysinfo` | ✅ | Info del sistema |
| `models` | ✅ | Listar modelos |
| `use-model` | ✅ | Cambiar modelo |
| `plan` | ✅ | Modo planificación |
| `auto` | ✅ | Modo autónomo |
| `vibecode` | ✅ | Modo vibe coding |
| `tui` | ✅ | UI con terminal |

---

### ✅ `src/agent/mod.rs` - Agente Autónomo
**Estado**: Completo  
**Líneas**: ~550  
**Funcionalidades**:
- [x] Lectura de archivos (`read_file`)
- [x] Escritura de archivos (`write_file`)
- [x] Ejecución de comandos (`execute_command`)
- [x] Listado de archivos (`list_files`)
- [x] Búsqueda en archivos (`search`)
- [x] Eliminación de archivos (`delete`) - con confirmación
- [x] Creación de directorios (`create_directory`)
- [x] Generación de árbol de proyecto (`get_project_tree`)
- [x] Detección automática de SO (Windows/macOS/Linux)
- [x] Ejecución de herramientas con confirmación para operaciones destructivas

**Shell por SO**:
```rust
Windows  → PowerShell (powershell.exe -NoProfile -Command)
macOS    → zsh (zsh -c)
Linux    → bash (bash -c)
```

**Tests**: 9 tests unitarios implementados ✅

---

### ✅ `src/llm/mod.rs` - Cliente Ollama
**Estado**: Completo  
**Líneas**: ~736  
**Funcionalidades**:
- [x] Conexión a Ollama (puerto 11434)
- [x] Listado de modelos disponibles
- [x] Chat completion con `chat_with_tools`
- [x] Conversión de agent_tools a formato Ollama
- [x] Detección de modelos locales vs cloud
- [x] Selección interactiva de modelos
- [x] Verificación de disponibilidad de modelos
- [x] Espera activa para carga de modelos (`wait_for_model_ready`)
- [x] Listado de modelos desde múltiples fuentes (Ollama, Jan, HuggingFace)

**Endpoints soportados**:
- `/v1/models` - OpenAI standard
- `/api/tags` - Ollama local
- `/v1/chat/completions` - Chat con herramientas

**Tests**: Múltiples tests de integración ✅

---

### ✅ `src/terminal/` - Terminal UI
**Estado**: Completo  
**Archivos**:
- `mod.rs` - Exportaciones
- `capsule.rs` - Renderizado de cápsula minimalista verde
- `tui.rs` - TUI completa con ratatui (~400 líneas)

**Funcionalidades TUI**:
- [x] Navegación de archivos (UP/DOWN/ENTER/BACKSPACE)
- [x] Vista previa de contenido de archivos
- [x] Terminal integrada
- [x] Detección de directorios protegidos
- [x] Límite de tamaño de archivo (50KB) para prevenir problemas de UI
- [x] Manejo robusto de errores de renderizado

**Cápsula visual**:
```
    █████
 ██████████    SELFIDEX v3.0
 ██████████    [●] Ollama Conectado
    █████
```

---

### ✅ `src/config.rs` - Configuración
**Estado**: Completo  
**Líneas**: ~130  
**Funcionalidades**:
- [x] Archivo de configuración TOML
- [x] Ubicación: `%APPDATA%\selfidx\config.toml`
- [x] Creación automática si no existe
- [x] Configuración de endpoint Ollama
- [x] Configuración de modelo por defecto
- [x] Configuración de directorio de logs
- [x] Soporte para múltiples idiomas

**Estructura de configuración**:
```toml
[jan_ai]
endpoint = "http://localhost:11434/v1"
default_model = "mistral:latest"
auto_connect = true

[general]
language = "es"
log_sessions = true
log_dir = "<project>/logs"
```

**Tests**: 2 tests unitarios ✅

---

### ✅ `src/project.rs` - Contexto de Proyecto
**Estado**: Completo  
**Líneas**: ~130  
**Funcionalidades**:
- [x] Gestión de `.selfidx.md` en directorio actual
- [x] Creación automática en primera ejecución
- [x] Lectura de contexto para el agente
- [x] Registro de progreso en tabla markdown
- [x] Detección de tipo de proyecto
- [x] Información de hardware y modelo activo

**Tests**: 1 test unitario ✅

---

### ✅ `src/autonomous.rs` - Modo Autónomo
**Estado**: Completo  
**Líneas**: ~200  
**Funcionalidades**:
- [x] Loop autónomo con límite de iteraciones (15)
- [x] Ejecución de herramientas desde tool_calls de Ollama
- [x] Confirmación para herramientas destructivas
- [x] Parser legacy para tool_calls en texto
- [x] Detección de finalización de tarea
- [x] Acumulación de contexto en mensajes

**Prompt del sistema**:
```
Eres un asistente de programación autónomo especializado en 
proyectos Flutter y Android. Ejecuta tareas directamente usando 
las herramientas disponibles.
```

---

### ✅ `src/shell/mod.rs` - Integración PowerShell
**Estado**: Completo  
**Líneas**: ~150  
**Funcionalidades**:
- [x] Ejecución de comandos PowerShell
- [x] Ejecución de scripts (.ps1)
- [x] Shell interactivo
- [x] Cambio de directorio
- [x] Parseo de comandos con soporte para quotes

**Tests**: 1 test unitario ✅

---

### ✅ `src/utils/` - Utilidades
**Estado**: Completo  
**Archivos**:
- `mod.rs` - Funciones utilitarias
- `hardware.rs` - Detección de hardware (~450 líneas)

**Funcionalidades hardware**:
- [x] Detección de RAM total
- [x] Detección de VRAM (nvidia-smi, PowerShell, wmic)
- [x] Detección de cores de CPU
- [x] Detección de sistema operativo
- [x] Recomendación de tamaño de modelo según hardware
- [x] Listado de modelos instalados en HuggingFace cache
- [x] Eliminación de modelos
- [x] Cálculo de tamaño de directorios

**Tests**: 6 tests unitarios ✅

---

## Dependencias

| Dependencia | Versión | Propósito | Estado |
|-------------|---------|-----------|--------|
| `clap` | 4.4 | CLI parsing | ✅ |
| `anyhow` | 1.0 | Error handling | ✅ |
| `tokio` | 1 | Async runtime | ✅ |
| `async-trait` | 0.1 | Async traits | ✅ |
| `tokio-stream` | 0.1 | Stream utilities | ✅ |
| `reqwest` | 0.11 | HTTP client | ✅ |
| `serde` | 1 | Serialization | ✅ |
| `serde_json` | 1 | JSON handling | ✅ |
| `toml` | 0.8 | Config files | ✅ |
| `crossterm` | 0.28 | Terminal ops | ✅ |
| `ratatui` | 0.24 | TUI rendering | ✅ |
| `fs4` | 0.8 | File operations | ✅ |
| `notify` | 6 | File watching | ✅ |
| `portable-pty` | 0.8 | PTY management | ✅ |
| `url` | 2 | URL parsing | ✅ |
| `humantime` | 2 | Time formatting | ✅ |
| `dirs` | 5 | Directory paths | ✅ |
| `chrono` | 0.4 | Date/time | ✅ |

**Total dependencias**: 18  
**Estado**: Todas actualizadas a versiones estables

---

## Sistema de Build

### Scripts Disponibles

| Script | Propósito | Estado |
|--------|-----------|--------|
| `scripts/build-bundle.bat` | Build + cargo-bundle (MSI) | ✅ |
| `scripts/build-installer.bat` | Build + Inno Setup | ✅ |
| `scripts/install.bat` | Instalación simple | ✅ |
| `scripts/installer.iss` | Script Inno Setup | ✅ |

### Proceso de Build

```bash
# Desarrollo
cargo run

# Release
cargo build --release

# Con instalador MSI
scripts\build-bundle.bat
```

### Output de Build

```
dist/
├── selfidx.exe              # Ejecutable principal
├── *.msi                    # Instalador MSI (cargo-bundle)
└── SELFIDEX-Setup-3.0.0.exe # Instalador Inno Setup
```

---

## Testing

### Tests Unitarios

| Módulo | Tests | Estado |
|--------|-------|--------|
| `agent` | 9 | ✅ |
| `config` | 2 | ✅ |
| `project` | 1 | ✅ |
| `shell` | 1 | ✅ |
| `utils/hardware` | 6 | ✅ |
| **Total** | **19** | ✅ |

### Ejecutar Tests

```bash
cargo test
```

---

## Configuración y Archivos

### Archivos de Configuración

| Archivo | Ubicación | Propósito |
|---------|-----------|-----------|
| `config.toml` | `%APPDATA%\selfidx\` | Configuración global |
| `.selfidx.md` | `<project_root>/` | Contexto del proyecto |

### Variables de Entorno

| Variable | Valor por defecto | Propósito |
|----------|-------------------|-----------|
| `OLLAMA_BASE_URL` | `http://localhost:11434/v1` | Endpoint Ollama |
| `OLLAMA_MODEL` | `llama3` | Modelo por defecto |
| `SELFIDEX_VRAM` | (auto-detect) | VRAM manual override |

---

## Integración con Ollama

### Modelos Soportados

| Categoría | Modelos |
|-----------|---------|
| Recomendados | `llama3`, `mistral`, `codellama`, `phi3` |
| Grandes | `llama3:70b`, `yi:34b`, `mixtral:8x7b` |
| Pequeños | `tinyllama`, `phi:2.7b`, `mistral:3b` |

### Comandos Ollama

```bash
# Listar modelos disponibles
selfidx --models

# Cambiar modelo
selfidx --use-model codellama

# Descargar modelo
selfidx --download llama3

# Eliminar modelo
selfidx --remove-model modelo-viejo
```

---

## Características de IA

### Herramientas del Agente

| Herramienta | Destructiva | Confirmación | Descripción |
|-------------|-------------|--------------|-------------|
| `read_file` | ❌ | No | Leer contenido de archivo |
| `write_file` | ✅ | Sí | Crear/sobrescribir archivo |
| `execute_command` | ⚠️ | No | Ejecutar comando shell |
| `list_files` | ❌ | No | Listar directorio |
| `search` | ❌ | No | Buscar patrón en archivos |
| `delete` | ✅ | Sí | Eliminar archivo/directorio |
| `create_directory` | ❌ | No | Crear directorio |

### Modos de Operación

| Modo | Comando | Descripción |
|------|---------|-------------|
| **Agente** | `agent <prompt>` | Ejecución autónoma de tareas |
| **Chat** | `chat` | Conversación interactiva |
| **Plan** | `plan <task>` | Planificación antes de ejecutar |
| **Auto** | `auto <task>` | Modo autónomo completo |
| **VibeCode** | `vibecode <task>` | Codificación dinámica |

---

## Detección de Hardware

### Sistema de Recomendación

| RAM+VRAM | Tamaño | Modelos Recomendados |
|----------|--------|---------------------|
| < 4GB | Tiny | TinyLlama (1.1B), Phi-2 (2.7B) |
| 4-10GB | Small | Mistral 3B, Neural Chat 7B |
| 10-20GB | Medium | Mistral 7B, Llama 2 7B |
| 20-36GB | Large | Yi 14B, Mythomist 7B |
| ≥ 36GB | ExtraLarge | Yi 34B, Mixtral 8x7B |

### Métodos de Detección

**VRAM (Windows)**:
1. `nvidia-smi` (primario)
2. PowerShell `Get-CimInstance Win32_VideoController`
3. `wmic path win32_VideoController get AdapterRAM`

**RAM**:
1. PowerShell `Get-CimInstance Win32_ComputerSystem`
2. `wmic ComputerSystem get TotalPhysicalMemory`

---

## Seguridad

### Protecciones Implementadas

- [x] Confirmación para operaciones destructivas (`delete`, `write_file`)
- [x] Validación de comandos no vacíos
- [x] Límite de tamaño de archivo en TUI (50KB)
- [x] Manejo robusto de errores de directorios protegidos
- [x] Filtrado de archivos ocultos y system files
- [x] Skip de directorios: `node_modules`, `target`, `dist`, `.git`

### Consideraciones de Seguridad

| Riesgo | Mitigación |
|--------|------------|
| Ejecución de comandos | Validación de entrada, logs |
| Eliminación de archivos | Confirmación requerida |
| Lectura de archivos sensibles | Filtrado de paths ocultos |
| Overflows de memoria | Límites de tamaño de contenido |

---

## Rendimiento

### Optimizaciones

- [x] Async/await con tokio para operaciones I/O
- [x] Streaming de respuestas LLM
- [x] Cache de contexto de proyecto
- [x] Lectura diferida de archivos grandes
- [x] Reintentos con backoff para conexión Ollama

### Límites Configurados

| Límite | Valor | Propósito |
|--------|-------|-----------|
| Iteraciones modo autónomo | 15 | Prevenir loops infinitos |
| Tamaño máximo archivo TUI | 50KB | Prevenir lag de render |
| Profundidad de búsqueda | 3 niveles | Prevenir scans lentos |
| Reintentos de modelo | 5 | Esperar carga de modelo |

---

## Problemas Conocidos

### Issues Menores

| Issue | Severidad | Workaround |
|-------|-----------|------------|
| Render TUI puede fallar en terminals muy pequeños | Baja | Redimensionar terminal |
| Detección de VRAM puede fallar en GPUs AMD/Intel | Media | Usar `SELFIDEX_VRAM` env var |
| Modelos cloud pueden tener latencia alta | Baja | Usar modelos locales |

### No Implementado (Future Work)

- [ ] Soporte para múltiples proveedores cloud (OpenAI, Anthropic)
- [ ] Edición de archivos en TUI
- [ ] Historial de comandos persistente
- [ ] Plugins/extensions
- [ ] Modo servidor/headless

---

## Métricas del Código

| Métrica | Valor |
|---------|-------|
| Líneas totales (src/) | ~5,000+ |
| Módulos principales | 8 |
| Tests unitarios | 19 |
| Dependencias | 18 |
| Comandos CLI | 15+ |
| Herramientas de agente | 7 |

---

## Próximos Pasos (Roadmap)

### Corto Plazo
- [ ] Mejorar detección de GPUs no-NVIDIA
- [ ] Agregar más tests de integración
- [ ] Optimizar memoria en modo autónomo

### Medio Plazo
- [ ] Soporte para proveedores cloud
- [ ] Editor de archivos en TUI
- [ ] Sistema de plugins

### Largo Plazo
- [ ] Modo colaborativo multi-usuario
- [ ] Integración con IDEs
- [ ] Marketplace de herramientas

---

## Conclusión

**SELFIDEX v3.0** está en estado **funcional y estable**. Todas las características principales están implementadas y probadas:

✅ Terminal TUI operativa  
✅ Agente autónomo tipo Codex  
✅ Integración completa con Ollama  
✅ Gestión de contexto de proyecto  
✅ Detección de hardware  
✅ Sistema de build e instalación  

El proyecto es adecuado para uso diario en desarrollo de software, especialmente para proyectos Flutter, Android, Node.js, Rust, y Python.

---

**Última actualización**: 1 de abril de 2026  
**Maintainer**: SELFIDEX Team  
**Licencia**: MIT
