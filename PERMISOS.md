# Sistema de Permisos - SELFIDEX v3.0

## Resumen de la Implementación

Hemos implementado un **sistema de permisos inspirado en Claude Code** que proporciona control granular sobre las operaciones del agente de IA.

---

## 🎯 Características Principales

### 1. **Modos de Permiso**

| Modo | Símbolo | Descripción |
|------|---------|-------------|
| `default` | 🔒 | Pregunta por cada operación |
| `auto` | ⚡ | Auto-aprobar operaciones de bajo riesgo |
| `dontask` | 🚀 | No preguntar nunca - auto-aprobar todo |
| `plan` | 📋 | Requiere revisión en modo planificación |
| `bypass` | ⚠️ | Sin restricciones (peligroso) |
| `yolo` | 🛡️ | Denegar todo (irónicamente seguro) |

### 2. **Clasificación de Riesgo**

El sistema clasifica automáticamente los comandos por nivel de riesgo:

- **🟢 BAJO**: Comandos seguros (ls, cat, cargo build, npm test)
- **🟡 MEDIO**: Comandos que pueden modificar el sistema (chmod, sudo, npm install -g)
- **🔴 ALTO**: Comandos destructivos (rm -rf, format, curl | bash)

### 3. **Reglas de Permiso**

Cada regla tiene:
- **Herramienta**: `execute_command`, `write_file`, `delete`, etc.
- **Patrón**: Patrón con wildcard (ej: `git *`, `rm -rf *`)
- **Comportamiento**: `allow`, `deny`, `ask`
- **Fuente**: `user`, `project`, `local`, `cli`, `session`

---

## 📁 Estructura de Archivos

```
src/permissions/
├── mod.rs           # Módulo principal, PermissionContext
├── mode.rs          # PermissionMode enum
├── rules.rs         # PermissionRule, PermissionBehavior
├── classifier.rs    # classify_command_risk(), RiskLevel
└── storage.rs       # PermissionStorage (TOML persistente)
```

---

## 🛠️ Comandos CLI

### Ver estado
```bash
selfidx permissions status
```

### Cambiar modo
```bash
selfidx permissions set-mode auto
selfidx permissions set-mode yolo
```

### Listar reglas
```bash
selfidx permissions list
```

### Agregar regla
```bash
# Auto-aprobar todos los comandos git
selfidx permissions add execute_command "git *" --behavior allow

# Preguntar siempre para escritura de archivos
selfidx permissions add write_file "*" --behavior ask

# Denegar comando específico
selfidx permissions add execute_command "rm -rf /" --behavior deny
```

### Eliminar regla
```bash
selfidx permissions remove execute_command "git *"
```

### Cargar reglas por defecto
```bash
selfidx permissions defaults
```

---

## 🔧 Integración con el Agente

El agente ahora verifica permisos antes de ejecutar cualquier herramienta:

```rust
// En agent/mod.rs
pub fn execute_command(&self, cmd: &str) -> Result<CommandResult> {
    // Verificar permisos
    let decision = self.permission_context.can_auto_execute("execute_command", cmd);
    
    if decision.is_denied() {
        anyhow::bail!("Comando denegado por permisos: {}", decision.reason());
    }
    
    // Ejecutar comando...
}

pub fn execute_tool_with_confirmation(
    &self,
    tool_name: &str,
    params: &serde_json::Value,
) -> Result<(String, bool)> {
    // Verificar permisos
    let decision = self.permission_context.can_auto_execute(tool_name, &input_for_check);
    
    if decision.is_denied() {
        return Ok(("❌ Herramienta denegada".to_string(), false));
    }
    
    // Si necesita confirmación, preguntar al usuario
    if decision.needs_confirmation() {
        println!("🟡 REQUERIR CONFIRMACIÓN - Riesgo: {}", risk_level.display());
        // ...
    }
}
```

---

## 📋 Reglas por Defecto

El sistema incluye 18 reglas predefinidas:

### Auto-aprobar (✓)
- `cargo build`, `cargo check`, `cargo test`, `cargo fmt`
- `npm run build`, `npm run test`, `npm run dev`, `npm run lint`
- `git status`, `git log *`, `git diff *`, `git branch`

### Preguntar (?)
- `rm -rf *`, `del *` (eliminación de archivos)
- `write_file *` (escritura de archivos)

### Denegar (✗)
- `rm -rf /`, `rm -rf /*`, `rm -rf ~` (comandos muy peligrosos)

---

## 💾 Almacenamiento

Las reglas se guardan en:
- **Windows**: `%APPDATA%\selfidx\permissions.toml`

Formato TOML:
```toml
mode = "auto"

[[rules]]
behavior = "allow"
tool_name = "execute_command"
pattern = "git *"
source = "user"

[[rules]]
behavior = "ask"
tool_name = "write_file"
pattern = "*"
source = "user"
```

---

## 🎨 UI de Confirmación

Cuando un comando requiere confirmación:

```
🟡 REQUERIR CONFIRMACIÓN - Riesgo: MEDIO
   Herramienta: execute_command
   Objetivo: npm install -g typescript
   Razón: Modo default - requiere confirmación

¿Confirmar ejecución? (s/n): 
```

Para comandos de alto riesgo:

```
🔴 REQUERIR CONFIRMACIÓN - Riesgo: ALTO
   Herramienta: execute_command
   Objetivo: rm -rf ./temp
   Razón: Este comando elimina archivos permanentemente

¿Confirmar ejecución? (s/n): 
```

---

## 🧪 Tests

El módulo incluye tests unitarios para:
- `PermissionMode` parsing y display
- `match_pattern()` con wildcards
- `classify_command_risk()` para varios comandos
- `PermissionRule` matching
- `PermissionStorage` load/save

Ejecutar tests:
```bash
cargo test permissions
```

---

## 📈 Próximas Mejoras (Fase 2)

1. **UI colapsable** para output de comandos search/read
2. **History search** con Ctrl+R
3. **Prompt input mejorado** con vim mode
4. **ML classifier** para auto-aprobación inteligente

---

## 🔗 Referencias

Inspirado en el sistema de permisos de **Claude Code v2.1.88**:
- `src/tools/permissions/permissions.ts`
- `src/utils/permissions/PermissionResult.ts`
- `src/tools/BashTool/bashPermissions.ts`

---

**Implementado**: 1 de abril de 2026  
**Líneas de código**: ~900 líneas Rust  
**Estado**: ✅ Funcional y probado
