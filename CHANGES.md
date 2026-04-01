# Registro de Cambios - SELFIDEX v3.0

## Ultimos Cambios (2026-03-25)

### Cambios Realizados

1. **Eliminación de emoticonos**: Todos los mensajes ahora usan texto simple sin emoticonos
2. **Detección de SO mejorada**: El agente detecta automáticamente Windows, macOS o Linux
3. **Carpeta root del proyecto**: La carpeta donde se abre SELFIDEX es siempre la root del proyecto
4. **MD de cambios**: Se crea y lee .selfidx.md para mantener contexto del proyecto
5. **Agente autónomo mejorado**: Modo autónomo estilo Codex para programación fullstack
6. **Estética verde conservada**: Cápsula verde minimalista mantenida
7. **Lenguaje natural**: Comandos en lenguaje natural soportados
8. **Modo plan**: Planificación de tareas antes de ejecutar
9. **Tools mejoradas**: Detección de SO y ejecución de comandos apropiados
10. **Optimización total**: Código optimizado para mejores resultados
11. **Cambio de modelos**: Permite cambiar entre modelos Ollama
12. **Detección de modelo actual**: Muestra qué modelo está activo

### Archivos Modificados

- `src/main.rs`: Eliminados emoticonos, mejorada detección de SO
- `src/agent/mod.rs`: Agente autónomo estilo Codex
- `src/llm/mod.rs`: Cliente Ollama optimizado
- `src/terminal/capsule.rs`: Estética verde conservada
- `src/config.rs`: Configuración optimizada
- `README.md`: Documentación actualizada

### Características Implementadas

- Terminal sin emoticonos
- Detección automática de SO (Windows/macOS/Linux)
- Contexto de proyecto persistente (.selfidx.md)
- Agente autónomo estilo Codex
- Estética verde minimalista
- Comandos en lenguaje natural
- Modo planificación
- Tools adaptadas al SO
- Optimización de rendimiento
- Cambio dinámico de modelos
- Detección de modelo activo

### Eliminados

- Todos los emoticonos de mensajes
- Referencias obsoletas a Jan.ai
- Código redundante
- Mensajes innecesarios