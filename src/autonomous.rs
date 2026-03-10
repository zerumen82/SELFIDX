// Autonomous agent module - Claude Code style

use anyhow::Result;
use crate::llm::{Message, VllmClient};
use crate::agent::Agent;

/// Run autonomous loop with tools and confirmation
pub async fn run_autonomous_loop(
    client: &VllmClient,
    model: &str,
    agent: &Agent,
    initial_prompt: &str,
) -> Result<()> {
    let max_iterations = 10;
    let mut messages: Vec<Message> = vec![
        Message {
            role: "system".to_string(),
            content: format!(
                r#"Eres un asistente de programación AUTÓNOMO estilo Claude Code. Tu objetivo es completar las tareas del usuario de forma independiente.

INSTRUCCIONES IMPORTANTES:
1. Cuando el usuario dé instrucciones vagas, PRIMERO pregúntale para aclarar antes de actuar
2. Si no entiendes algo, PREGUNTA en lugar de asumir
3. Usa las herramientas solo cuando tengas claro qué hacer
4. Cuando la tarea esté completa, escribe [DONE]

HERRAMIENTAS DISPONIBLES:
{}

EJEMPLO DE USO:
- Usuario: "haz algo" → Tú: "¿Qué te gustaría que hiciera exactamente?"
- Usuario: "crea un archivo" → Tú: "¿Cómo se llama el archivo y qué contenido debe tener?"

Responde siempre en español de forma clara y útil."#,
                Agent::get_tools_description()
            ),
        },
        Message {
            role: "user".to_string(),
            content: initial_prompt.to_string(),
        },
    ];

    println!("\n═══════════════════════════════════════════");
    println!("   🤖 MODO AUTÓNOMO - CLAUDE CODE STYLE");
    println!("═══════════════════════════════════════════\n");

    for iteration in 0..max_iterations {
        println!("[Iteración {}] Consultando IA...", iteration + 1);

        match client.chat(model.to_string(), messages.clone()).await {
            Ok(response) => {
                let content = response.content();

                if content.contains("[DONE]") {
                    let final_content = content.replace("[DONE]", "");
                    println!("\n=== RESULTADO FINAL ===\n");
                    println!("{}", final_content.trim());
                    break;
                }

                if content.contains("[TOOL:") {
                    println!("\n=== HERRAMIENTAS DETECTADAS ===\n");

                    let tool_calls = parse_tool_calls(content);

                    if tool_calls.is_empty() {
                        println!("{}", content);

                        println!("\n¿Continuar? (s/n): ");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() != "s" {
                            break;
                        }
                    } else {
                        for (tool_name, params) in &tool_calls {
                            println!("→ Ejecutando: {} {:?}", tool_name, params);

                            let result = if Agent::is_destructive_tool(tool_name) {
                                let (result, confirmed) = agent.execute_tool_with_confirmation(tool_name, params)?;
                                if !confirmed {
                                    println!("→ Acción cancelada");
                                    messages.push(Message {
                                        role: "assistant".to_string(),
                                        content: content.to_string(),
                                    });
                                    messages.push(Message {
                                        role: "user".to_string(),
                                        content: "La acción fue cancelada por el usuario. Continúa sin ejecutar esa acción.".to_string(),
                                    });
                                    continue;
                                }
                                result
                            } else {
                                agent.execute_tool(tool_name, params)?
                            };

                            println!("→ Resultado: {}\n", result);

                            messages.push(Message {
                                role: "assistant".to_string(),
                                content: content.to_string(),
                            });
                            messages.push(Message {
                                role: "system".to_string(),
                                content: format!("Resultado de {}: {}", tool_name, result),
                            });
                        }
                    }
                } else {
                    println!("{}", content);

                    println!("\n¿Continuar? (s/n): ");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() != "s" {
                        break;
                    }

                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: content.to_string(),
                    });
                    messages.push(Message {
                        role: "user".to_string(),
                        content: "Continúa con la siguiente acción o responde que has terminado.".to_string(),
                    });
                }
            }
            Err(e) => {
                println!("[selfidx-error] Error: {}", e);
                break;
            }
        }
    }

    println!("\n═══════════════════════════════════════════");
    println!("✓ Modo autónomo finalizado");
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
