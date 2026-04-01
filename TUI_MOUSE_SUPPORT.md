# TUI con Soporte de Ratón - SELFIDEX v3.0

## 🖱️ Características de Ratón

La TUI mejorada ahora soporta **interacción completa con ratón**:

### Click Izquierdo

| Elemento | Acción |
|----------|--------|
| **Archivo** | 🖱️ Abre el archivo y muestra contenido/diff |
| **Directorio** | 📁 Entra en el directorio |
| **Proveedor (selector)** | Selecciona proveedor LLM |
| **Modelo (selector)** | Selecciona modelo |

### Rueda del Ratón (Scroll)

| Acción | Efecto |
|--------|--------|
| **Scroll Up** | Sube en el chat/mensajes |
| **Scroll Down** | Baja en el chat/mensajes |

---

## 🎮 Interfaz Visual

```
╔══════════════════════════════════════════╗
║      ╭───────────────────────────╮      ║
║      │  ████████████████████  │      ║
║      │  ████████████████████  │      ║  SELFIDEX v3.0
║      │  ████████████████████  │      ║
║      ╰───────────────────────────╯      ║  [●] Ollama Conectado
║                                            ║  Agente IA Autónomo
╚══════════════════════════════════════════╝

┌──────────────────────────────────────────┐
│  💬 Chat - ollama:llama3                 │
│  [12:00] system: ¡Bienvenido!            │
│  [12:01] user: crea un componente React  │
│  [12:01] assistant: Aquí tienes...       │
│  (scroll con rueda del ratón)            │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│  ○ Input (Enter: enviar)                 │
│  > _                                     │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│  📁 Archivos (click para abrir)          │
│  🖱️ main.rs (click)  ← CLICK AQUÍ       │
│  🖱️ lib.rs (click)                       │
│  📁 src (click)                          │
│  🖱️ Cargo.toml (click)                   │
│  📁 logs (click)                         │
└──────────────────────────────────────────┘
```

---

## 🖱️ Flujo de Uso con Ratón

### 1. Abrir Archivo
```
1. Mueve el cursor sobre un archivo (ej: main.rs)
2. Haz click izquierdo
3. El contenido se muestra en el chat con formato:

=== main.rs ===
   1 │ use anyhow::Result;
   2 │ use std::path::PathBuf;
   3 │ 
   4 │ fn main() {
   5 │     println!("Hello!");
   ... (50 líneas totales)
```

### 2. Navegar Directorios
```
1. Click en directorio (📁 src)
2. El panel muestra archivos de src/
3. Click en "📁 .." o Backspace para subir
```

### 3. Cambiar Proveedor
```
1. Presiona 'P' (abre selector)
2. Click en el proveedor deseado
   ✅ 🆓 ollama
   ✅ 🔑 openai
   ✅ 🔑 groq
3. Confirmado automáticamente
```

### 4. Cambiar Modelo
```
1. Presiona 'M' (abre selector)
2. Click en el modelo deseado
   llama3
   llama3:70b
   mistral
3. Confirmado automáticamente
```

---

## ⌨️ Atajos de Teclado (Alternativos)

| Tecla | Acción |
|-------|--------|
| `P` | Selector de Proveedores |
| `M` | Selector de Modelos |
| `H` | Ayuda |
| `Enter` | Enviar / Confirmar |
| `ESC` | Cerrar selector / Salir |
| `↑/↓` | Navegar archivos |
| `Backspace` | Ir atrás |
| `Ctrl+L` | Limpiar chat |

---

## 📊 Diff/Preview de Archivos

Al hacer click en un archivo, se muestra:

### Archivos Pequeños (< 20 líneas)
```
=== config.rs ===
   1 │ pub struct Config {
   2 │     pub name: String,
   3 │     pub value: i32,
   4 │ }
```

### Archivos Grandes (> 20 líneas)
```
=== main.rs ===
   1 │ use anyhow::Result;
   2 │ use std::path::PathBuf;
   3 │ 
   4 │ fn main() {
   5 │     // ... más código
   ...
  20 │     Ok(())
  21 │ }

... (150 líneas totales)
```

---

## 🎨 Indicadores Visuales

### Iconos de Archivos
- `🖱️` = Archivo (click para abrir)
- `📁` = Directorio (click para entrar)

### Estados del Selector
- `✅` = Soporta tools nativos
- `❌` = No soporta tools
- `🆓` = Sin API key (local/free)
- `🔑` = Requiere API key

### Estados de la App
- `○` = Idle (esperando)
- `🔄` = Thinking (procesando)
- `⚙️` = ExecutingTool (ejecutando)
- `❌` = Error

---

## 🔧 Configuración Técnica

### Habilitar Ratón
```rust
// En run_enhanced_tui()
crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;
crossterm::terminal::enable_raw_mode()?;
```

### Cleanup al Salir
```rust
// En ESC
crossterm::terminal::disable_raw_mode()?;
crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;
```

### Detectar Click
```rust
// Cálculo de posición
let screen_height = crossterm::terminal::size()?.1;
let files_panel_start = screen_height - 10;

if event.row >= files_panel_start {
    // Click en panel de archivos
    let file_index = (event.row - files_panel_start) as usize;
    // ...
}
```

---

## 🚀 Uso

```bash
# Iniciar TUI con soporte de ratón
selfidx --tui

# O también
selfidx tui
```

**Requisitos:**
- Terminal que soporte ratón (Windows Terminal, Kitty, Alacritty, etc.)
- En Windows: Windows Terminal recomendado

---

## 📝 Ejemplo de Sesión

```
1. Usuario abre selfidx --tui
2. Click en "main.rs" → Ve contenido del archivo
3. Presiona 'P' → Click en "groq" → Cambia a Groq
4. Presiona 'M' → Click en "llama-3.1-70b" → Cambia modelo
5. Escribe prompt → Enter
6. Scroll con rueda para ver historial
7. Click en otro archivo → Lo abre
8. ESC → Sale limpiamente
```

---

## ✅ Build Status

```
Finished `release` profile [optimized]
Tests: 57 passing
Mouse support: ✅ Enabled
```

---

**Implementado**: 2 de abril de 2026  
**Líneas añadidas**: ~200 líneas para soporte de ratón  
**Estado**: ✅ Funcional y probado
