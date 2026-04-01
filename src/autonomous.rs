// Autonomous agent module - estilo Codex fluido

use anyhow::Result;
use crate::llm::{Message, OllamaClient};
use crate::agent::Agent;

/// Ejecutar modo autónomo con interacción fluida estilo Codex
pub async fn run_autonomous_loop(
     client: &OllamaClient,
     model: &str,
     agent: &Agent,
     initial_prompt: &str,
 ) -> Result<()> {
    let max_iterations = 15;
    
    // Prompt del sistema simplificado y directo
    let system_prompt = format!(
        r#"Eres un asistente de programación autónomo especializado en proyectos Flutter y Android. Ejecuta tareas directamente usando las herramientas disponibles.

REGLAS:
1. Ejecuta acciones inmediatamente cuando sea claro qué hacer
2. Si necesitas aclaración, pregunta brevemente
3. Usa las herramientas para leer, escribir archivos y ejecutar comandos
4. Cuando termines, indica que la tarea está completa
5. Analiza el contexto del proyecto antes de actuar

HERRAMIENTAS:
{}"#,
        Agent::get_tools_description()
    );

    let mut messages: Vec<Message> = vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
            tool_calls: None,
        },
        Message {
            role: "user".to_string(),
            content: initial_prompt.to_string(),
            tool_calls: None,
        },
    ];

    println!("\n═══════════════════════════════════════════");
    println!("   🤖 MODO AUTÓNOMO");
    println!("═══════════════════════════════════════════\n");

    for iteration in 0..max_iterations {
        println!("[{}] Procesando...", iteration + 1);

        // Convert agent tools to Ollama format
        let ollama_tools = Some(OllamaClient::convert_agent_tools_to_ollama(Agent::get_tools()));

        match client.chat_with_tools(model.to_string(), messages.clone(), ollama_tools).await {
            Ok(response) => {
                // Check if response has tool calls from Ollama
                if let Some(tool_calls) = response.tool_calls() {
                    println!("→ Ollama solicitó {} herramienta(s)", tool_calls.len());
                    
                    for tool_call in tool_calls {
                        let tool_name = &tool_call.function.name;
                        let params: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                            .unwrap_or(serde_json::json!({}));
                        
                        println!("→ Ejecutando: {} {:?}", tool_name, params);

                        let result = if Agent::is_destructive_tool(tool_name) {
                            let (result, confirmed) = agent.execute_tool_with_confirmation(tool_name, &params)?;
                            if !confirmed {
                                messages.push(Message {
                                    role: "user".to_string(),
                                    content: "Acción cancelada. Continúa con otra tarea.".to_string(),
                                    tool_calls: None,
                                });
                                continue;
                            }
                            result
                        } else {
                            agent.execute_tool(tool_name, &params)?
                        };

                        // Add tool result to messages
                        messages.push(Message {
                            role: "tool".to_string(),
                            content: result,
                            tool_calls: None,
                        });
                    }
                    
                    // Continue conversation with tool results
                    continue;
                }

                let content = response.content();

                // Verificar si la tarea está completa
                if content.contains("[DONE]") || content.contains("tarea completada") || content.contains("listo") {
                    println!("\n✓ Tarea completada\n");
                    break;
                }

                // Fallback: Detectar y ejecutar herramientas desde texto (legacy)
                let tool_calls = parse_tool_calls(content);
                
                if !tool_calls.is_empty() {
                    for (tool_name, params) in &tool_calls {
                        println!("→ {} {:?}", tool_name, params);

                        let result = if Agent::is_destructive_tool(tool_name) {
                            let (result, confirmed) = agent.execute_tool_with_confirmation(tool_name, params)?;
                            if !confirmed {
                                messages.push(Message {
                                    role: "user".to_string(),
                                    content: "Acción cancelada. Continúa con otra tarea.".to_string(),
                                    tool_calls: None,
                                });
                                continue;
                            }
                            result
                        } else {
                            agent.execute_tool(tool_name, params)?
                        };

                        // Agregar resultado al contexto
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: content.to_string(),
                            tool_calls: None,
                        });
                        messages.push(Message {
                            role: "user".to_string(),
                            content: format!("Resultado: {}", result),
                            tool_calls: None,
                        });
                    }
                } else {
                    // Respuesta sin herramientas - mostrar y continuar
                    println!("{}", content);
                    
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: content.to_string(),
                        tool_calls: None,
                    });
                    messages.push(Message {
                        role: "user".to_string(),
                        content: "Continúa.".to_string(),
                        tool_calls: None,
                    });
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    println!("\n═══════════════════════════════════════════");
    println!("✓ Finalizado");
    println!("═══════════════════════════════════════════");

    Ok(())
}

/// Parse tool calls from AI response
fn parse_tool_calls(content: &str) -> Vec<(String, serde_json::Value)> {
    let mut tool_calls = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("[TOOL:") {
            if let Some(tool_name) = line.strip_prefix("[TOOL:") {
                let tool_name = tool_name.trim_end_matches(']').trim().to_string();

                let mut params_json = String::new();
                while let Some(param_line) = lines.peek() {
                    if param_line.starts_with("[PARAMS:") {
                        if let Some(params) = param_line.strip_prefix("[PARAMS:") {
                            params_json = params.trim_end_matches(']').trim().to_string();
                        }
                        break;
                    }
                    if param_line.starts_with("[") {
                        break;
                    }
                    lines.next();
                }

                if let Ok(params) = serde_json::from_str::<serde_json::Value>(&params_json) {
                    tool_calls.push((tool_name, params));
                }
            }
        }
    }

    tool_calls
}
