# SELFIDEX v3.0

**Terminal integrada con vLLM** - Una terminal PowerShell moderna con capacidades de IA.

## 🎯 Características

- **Terminal Integrada**: PowerShell nativo con renderizado enriquecido
- **Cápsula Visible**: Diseño minimalista con cápsula SELFIDEX
- **vLLM Integration**: Chat AI con modelos locales (puerto 8000)
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
 ██████████    [●] vLLM Conectado
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
VLLM_PORT=8000          # Puerto de vLLM (por defecto: 8000)
VLLM_ENDPOINT=http://localhost:8000/v1
```

### Modelos vLLM

Modelos recomendados para vLLM:
- `TinyLlama/TinyLlama-1.1B-Chat-v1.0`
- `TheBloke/Llama-2-7B-Chat-GGUF`
- `TheBloke/Mistral-7B-v0.1-GGUF`

## 📁 Estructura

```
selfidx/
├── src/
│   ├── cli/           # Entry point y argumentos
│   ├── terminal/       # Cápsula y terminal emulator
│   ├── shell/          # PowerShell integration
│   ├── llm/           # vLLM client
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

## 📄 Licencia

MIT License

---

**SELFIDEX v3.0** - Terminal Integrada con IA
