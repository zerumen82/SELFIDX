# Integración Multi-API - SELFIDEX v3.0

## Resumen

SELFIDEX ahora soporta **múltiples proveedores de LLM** además de Ollama, incluyendo OpenAI, Anthropic, Groq y LMStudio.

---

## 🤖 Proveedores Soportados

| Proveedor | Endpoint | API Key | Modelos Populares |
|-----------|----------|---------|-------------------|
| **Ollama** | `http://localhost:11434/v1` | ❌ No requerida | llama3, mistral, codellama, phi3, gemma2, qwen2.5 |
| **OpenAI** | `https://api.openai.com/v1` | ✅ Requerida | gpt-4o, gpt-4o-mini, gpt-4-turbo, gpt-3.5-turbo |
| **Anthropic** | `https://api.anthropic.com/v1` | ✅ Requerida | claude-sonnet-4, claude-3-5-sonnet, claude-3-opus |
| **Groq** | `https://api.groq.com/openai/v1` | ✅ Requerida | llama-3.1-70b, llama-3.1-8b, mixtral-8x7b |
| **LMStudio** | `http://localhost:1234/v1` | ❌ No requerida | local-model (cualquier modelo cargado) |

---

## 🚀 Comandos CLI

### Listar proveedores disponibles

```bash
selfidx provider list
```

**Salida:**
```
═══════════════════════════════════════════
   🤖 PROVEEDORES LLM DISPONIBLES
═══════════════════════════════════════════

Proveedor       Endpoint                            API Key    Modelos Populares
───────────────────────────────────────────────────────────────────────────────
ollama          http://localhost:11434/v1           No requerida llama3, mistral, codellama...
openai          https://api.openai.com/v1           Requerida  gpt-4o, gpt-4o-mini...
anthropic       https://api.anthropic.com/v1        Requerida  claude-sonnet-4...
groq            https://api.groq.com/openai/v1      Requerida  llama-3.1-70b...
lmstudio        http://localhost:1234/v1            No requerida local-model
```

### Ver información del proveedor actual

```bash
selfidx provider info
```

**Salida:**
```
═══════════════════════════════════════════
   🤖 PROVEEDOR ACTUAL
═══════════════════════════════════════════

Proveedor: ollama
Endpoint: http://localhost:11434/v1
API Key: ✗ No configurada
Modelos: llama3, llama3:70b, mistral, codellama, phi3, gemma2, qwen2.5
```

### Cambiar proveedor

```bash
# Cambiar a Ollama (local, sin API key)
selfidx provider set ollama

# Cambiar a OpenAI con API key
selfidx provider set openai --api-key sk-...

# Cambiar a Anthropic con endpoint personalizado
selfidx provider set anthropic --api-key sk-ant-... --endpoint https://api.anthropic.com/v1

# Cambiar a Groq
selfidx provider set groq --api-key gsk_...
```

**Salida:**
```
═══════════════════════════════════════════
   🤖 CONFIGURACIÓN DE PROVEEDOR
═══════════════════════════════════════════

Proveedor: openai
Endpoint: https://api.openai.com/v1
API Key: ✓ Configurada (sk-***)

✅ Configuración guardada en: C:\Users\Tu\AppData\Local\selfidx\provider.env

💡 Para usar esta configuración:
   1. Reinicia tu terminal, O
   2. Ejecuta: source C:\Users\Tu\AppData\Local\selfidx\provider.env
```

### Listar modelos disponibles

```bash
# Listar modelos del proveedor actual
selfidx provider models

# Listar modelos de un proveedor específico
selfidx provider models openai
selfidx provider models ollama
```

**Salida:**
```
═══════════════════════════════════════════
   🤖 MODELOS DISPONIBLES
═══════════════════════════════════════════

Proveedor: ollama
Endpoint: http://localhost:11434/v1

✅ Modelos accesibles: 15 encontrados

  • llama3
  • llama3:70b
  • mistral
  • codellama
  • phi3
  • gemma2
  • qwen2.5
  • ...
```

### Verificar conectividad

```bash
selfidx provider check
```

**Salida:**
```
═══════════════════════════════════════════
   🤖 VERIFICANDO CONECTIVIDAD
═══════════════════════════════════════════

Proveedor: ollama
Endpoint: http://localhost:11434/v1

✅ ¡Proveedor disponible!
✅ Modelos accesibles: 15 encontrados
```

---

## 🔧 Variables de Entorno

| Variable | Descripción | Ejemplo |
|----------|-------------|---------|
| `LLM_PROVIDER` | Proveedor a usar | `ollama`, `openai`, `anthropic` |
| `LLM_ENDPOINT` | Endpoint personalizado | `http://localhost:11434/v1` |
| `LLM_API_KEY` | API key (si es requerida) | `sk-...` |
| `LLM_MODEL` | Modelo por defecto | `llama3`, `gpt-4o` |

### Configurar variables manualmente

**Windows PowerShell:**
```powershell
$env:LLM_PROVIDER="openai"
$env:LLM_API_KEY="sk-..."
$env:LLM_MODEL="gpt-4o"
```

**Linux/macOS:**
```bash
export LLM_PROVIDER=openai
export LLM_API_KEY=sk-...
export LLM_MODEL=gpt-4o
```

### Archivo de configuración persistente

El comando `selfidx provider set` guarda la configuración en:
- **Windows**: `%LOCALAPPDATA%\selfidx\provider.env`
- **Linux/macOS**: `~/.local/share/selfidx/provider.env`

Contenido del archivo:
```bash
LLM_PROVIDER=openai
LLM_ENDPOINT=https://api.openai.com/v1
LLM_API_KEY=sk-...
```

---

## 💡 Casos de Uso

### 1. Desarrollo Local con Ollama

```bash
# Usar Ollama para desarrollo (gratis, local)
selfidx provider set ollama
selfidx agent "crea un componente React"
```

### 2. Producción con OpenAI

```bash
# Usar GPT-4 para tareas críticas
selfidx provider set openai --api-key sk-...
selfidx agent "optimiza esta función compleja"
```

### 3. Claude para Análisis de Código

```bash
# Usar Claude-3.5-Sonnet para análisis profundo
selfidx provider set anthropic --api-key sk-ant-...
selfidx agent "analiza este código en busca de bugs"
```

### 4. Groq para Velocidad

```bash
# Usar Groq para respuestas rápidas
selfidx provider set groq --api-key gsk_...
selfidx chat
```

### 5. LMStudio para Modelos Personalizados

```bash
# Usar modelos GGUF locales con LMStudio
selfidx provider set lmstudio
selfidx agent "usa mi modelo custom-llama-7b"
```

---

## 🔑 Obtener API Keys

### OpenAI
1. Ve a https://platform.openai.com/api-keys
2. Inicia sesión o crea una cuenta
3. Click en "Create new secret key"
4. Copia la key (empieza con `sk-`)

### Anthropic
1. Ve a https://console.anthropic.com/settings/keys
2. Inicia sesión o crea una cuenta
3. Click en "Create Key"
4. Copia la key (empieza con `sk-ant-`)

### Groq
1. Ve a https://console.groq.com/keys
2. Inicia sesión o crea una cuenta
3. Click en "Create API Key"
4. Copia la key (empieza con `gsk_`)

---

## 📊 Comparación de Proveedores

| Característica | Ollama | OpenAI | Anthropic | Groq | LMStudio |
|----------------|--------|--------|-----------|------|----------|
| **Costo** | Gratis | Pago | Pago | Gratis (beta) | Gratis |
| **Velocidad** | ⚡⚡⚡ | ⚡⚡⚡⚡ | ⚡⚡⚡ | ⚡⚡⚡⚡⚡ | ⚡⚡⚡ |
| **Privacidad** | 🔒🔒🔒🔒🔒 | 🔒🔒🔒 | 🔒🔒🔒 | 🔒🔒🔒 | 🔒🔒🔒🔒🔒 |
| **Modelos** | Limitados | Ilimitados | Limitados | Limitados | Ilimitados |
| **Requiere Internet** | ❌ No | ✅ Sí | ✅ Sí | ✅ Sí | ❌ No |

---

## 🛠️ Arquitectura Técnica

### Estructura de Archivos

```
src/llm/
├── mod.rs           # Exportaciones
├── providers.rs     # Multi-provider support (~550 líneas)
└── (existing Ollama code)
```

### Clases Principales

**`LlmProvider`** - Enum de proveedores:
```rust
pub enum LlmProvider {
    Ollama,
    OpenAI,
    Anthropic,
    Groq,
    LMStudio,
}
```

**`LlmClient`** - Cliente unificado:
```rust
pub struct LlmClient {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub api_key: Option<String>,
    client: reqwest::Client,
}
```

**`ProviderConfig`** - Configuración:
```rust
pub struct ProviderConfig {
    pub provider: LlmProvider,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub default_model: String,
    pub enabled: bool,
}
```

### API Unificada

Todos los proveedores usan la misma interfaz:

```rust
// Crear cliente
let client = LlmClient::from_env();

// Verificar conectividad
if client.is_available().await {
    // Listar modelos
    let models = client.list_models().await?;
    
    // Chat con tools
    let response = client.chat_with_tools(
        model,
        messages,
        tools
    ).await?;
}
```

---

## 🧪 Tests

El módulo incluye 5 tests unitarios:

```bash
$ cargo test llm

running 5 tests
test llm::providers::tests::test_provider_config ... ok
test llm::providers::tests::test_provider_display ... ok
test llm::providers::tests::test_provider_popular_models ... ok
test llm::providers::tests::test_provider_from_str ... ok
test llm::providers::tests::test_provider_requires_api_key ... ok

test result: ok. 5 passed
```

---

## 🔮 Futuras Mejoras

- [ ] Soporte para Azure OpenAI
- [ ] Soporte para Google Vertex AI
- [ ] Balanceo de carga entre proveedores
- [ ] Fallback automático si un proveedor falla
- [ ] Caché de respuestas entre proveedores
- [ ] Comparación de costos en tiempo real

---

**Implementado**: 2 de abril de 2026  
**Líneas de código**: ~550 líneas nuevas  
**Tests**: 5 tests passing ✅  
**Proveedores**: 5 soportados  
**Estado**: ✅ Funcional y probado
