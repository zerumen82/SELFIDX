# SELFIDEX v3.0

**Terminal integrada con Ollama** - Una terminal PowerShell moderna con capacidades de IA.

## 🎯 Características

- **Terminal Integrada**: PowerShell nativo con renderizado enriquecido
- **Cápsula Visible**: Diseño minimalista con cápsula SELFIDEX
- **Ollama Integration**: Chat AI con modelos locales (puerto 11434)
- **Agente IA**: Asistente de programación en español
- **Navegación Bidireccional**: UI y terminal sincronizadas
- **Instalación en PATH**: Comando `selfidx` global

## 📦 Instalación

```bash
# Clonar repositorio
git clone https://github.com/selfidex/selfidx.git
cd selfidx

# Compilar
cargo build --release

# Instalar en PATH
.\target\release.selfidx.exe --install
```

## 🚀 Uso

```bash
# Iniciar terminal
selfidx

# Modo Agente AI
selfidx --agent "crea un componente React"

# Chat interactivo
selfidx --chat

# Ejecutar comando
selfidx -c "npm install"

# Crear archivo
selfidx --create src/App.tsx

# Editar archivo
selfidx --edit src/App.tsx

# Listar archivos
selfidx --files

# Comparar archivos
selfidx --diff file1.txt file2.txt
```

## 🎨 Cápsula

```
    █████
 ██████████    SELFIDEX v3.0
 ██████████    [●] Ollama Conectado
    █████
```

## ⌨️ Atajos

| Tecla | Acción |
|-------|--------|
| `F1` | Help |
| `F2` | File List |
| `F3` | Editor |
| `F4` | Run Project |
| `Ctrl+C` | Cancelar/Salir |
| `Ctrl+L` | Limpiar |

## 🔧 Configuración

### Variables de Entorno

```bash
OLLAMA_BASE_URL=http://localhost:11434/v1  # Puerto de Ollama (por defecto: 11434)
OLLAMA_MODEL=llama3                        # Modelo por defecto
```

### Modelos Ollama

Modelos recomendados para Ollama:
- `llama3` - Meta Llama 3 (8B)
- `llama3:70b` - Meta Llama 3 (70B)
- `codellama` - Code Llama (7B-34B)
- `mistral` - Mistral 7B
- `phi3` - Microsoft Phi-3 (3.8B)

## 📁 Estructura

```
selfidx/
├── src/
│   ├── cli/           # Entry point y argumentos
│   ├── terminal/       # Cápsula y terminal emulator
│   ├── shell/          # PowerShell integration
│   ├── llm/           # Ollama client
│   └── utils/         # Utilidades
├── assets/            # Recursos
└── scripts/           # Scripts de instalación
```

## 🛠️ Desarrollo

```bash
# Modo desarrollo
cargo run

# Compilar release
cargo build --release

# Tests
cargo test
```

## 📦 Generar Instalador

### Opción 1: cargo-bundle (Recomendado - Nativo Rust)

```bash
# Ejecutar script de build con cargo-bundle
scripts\build-bundle.bat
```

O manualmente:

```bash
# 1. Instalar cargo-bundle (solo una vez)
cargo install cargo-bundle

# 2. Compilar y generar instalador
cargo build --release
cargo bundle --release --format msi
```

**Ventajas de cargo-bundle:**
- ✅ Nativo de Rust (sin dependencias externas)
- ✅ Genera instalador MSI para Windows
- ✅ No requiere permisos de administrador
- ✅ Configuración en Cargo.toml

### Opción 2: Inno Setup (Alternativa)

```bash
# Ejecutar script de build con Inno Setup
scripts\build-installer.bat
```

O manualmente:

```bash
# 1. Compilar proyecto
cargo build --release

# 2. Generar instalador con Inno Setup
# Abre scripts\installer.iss en Inno Setup Compiler
# O ejecuta desde línea de comandos:
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" scripts\installer.iss
```

### Opción 3: Solo Ejecutable (Sin Instalador)

```bash
# Compilar y copiar ejecutable
cargo build --release
copy target\release\selfidx.exe dist\
```

### Requisitos

- **Rust**: https://rustup.rs/
- **cargo-bundle**: Se instala automáticamente con el script
- **Inno Setup 6** (opcional): https://jrsoftware.org/isinfo.php

### Archivos Generados

Después de ejecutar el build:
- `dist\selfidx.exe` - Ejecutable principal
- `dist\*.msi` - Instalador MSI (con cargo-bundle)
- `dist\SELFIDEX-Setup-3.0.0.exe` - Instalador (con Inno Setup)

### Instalación

**Con instalador MSI (cargo-bundle):**
1. Ejecuta `dist\*.msi`
2. Sigue las instrucciones del asistente
3. Abre una NUEVA terminal y ejecuta `selfidx --help`

**Sin instalador:**
1. Copia `dist\selfidx.exe` a cualquier ubicación
2. Ejecuta `selfidx.exe --install` para agregar al PATH
3. Abre una NUEVA terminal y ejecuta `selfidx --help`

**Nota**: Los instaladores no requieren permisos de administrador y agregan SELFIDEX al PATH del usuario.

## 📄 Licencia

MIT License

---

**SELFIDEX v3.0** - Terminal Integrada con Ollama
