// Módulo CLI para interacción directa con la IA de NeuroCode
use std::io::{self, Write};

pub fn ia_cli(args: &[String]) {
    // Si viene con argumento directo: neuro ia "pregunta"
    if let Some(prompt) = args.get(2) {
        let prompt_completo = args[2..].join(" ");
        println!("🧠 Consultando IA...");
        match ia_consulta_directa(&prompt_completo) {
            Some(respuesta) => println!("\n{}", respuesta),
            None => {
                println!("⚠️ No se pudo conectar con la IA.");
                println!("💡 Asegúrate de tener Ollama corriendo: ollama serve");
                println!("   O configura un proveedor: neuro auth ollama");
            }
        }
        return;
    }

    // Modo interactivo
    println!("🧠 NeuroCode IA - Modo Interactivo");
    println!("   Escribe tu pregunta y presiona Enter.");
    println!("   Escribe 'salir' para terminar.\n");

    loop {
        print!("🧠 > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "salir" || input == "exit" || input == "q" {
            println!("👋 ¡Hasta luego!");
            break;
        }

        println!("⏳ Pensando...");
        match ia_consulta_directa(input) {
            Some(respuesta) => println!("\n{}\n", respuesta),
            None => println!("⚠️ Sin respuesta. ¿Está Ollama corriendo?\n"),
        }
    }
}

fn ia_consulta_directa(prompt: &str) -> Option<String> {
    let ollama_urls = crate::ollama_generate_urls();
    let mut url = ollama_urls
        .first()
        .cloned()
        .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string());
    let mut model = "llama3.2:latest".to_string();
    let mut api_key = String::new();
    let mut is_anthropic = false;

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
    let config_str = std::fs::read_to_string(format!("{}/.neurocode_keys", home))
        .or_else(|_| std::fs::read_to_string(format!("{}/.aquila_keys", home)));
    if let Ok(config_str) = config_str {
        if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_str) {
            if let Some(u) = config_json.get("url").and_then(|u| u.as_str()) { url = u.to_string(); is_anthropic = u.contains("anthropic"); }
            if let Some(c) = config_json.get("clave").and_then(|c| c.as_str()) { api_key = c.to_string(); }
            if let Some(m) = config_json.get("modelo").and_then(|m| m.as_str()) { model = m.to_string(); }
        }
    }

    let is_ollama = ollama_urls.iter().any(|candidate| candidate == &url);

    let body = if is_ollama {
        serde_json::json!({"model": model, "prompt": prompt, "stream": false})
    } else if is_anthropic {
        serde_json::json!({"model": model, "max_tokens": 1024, "messages": [{"role": "user", "content": prompt}]})
    } else {
        serde_json::json!({"model": model, "messages": [{"role": "user", "content": prompt}]})
    };

    let mut request = ureq::post(&url).timeout(std::time::Duration::from_secs(60));
    if !api_key.is_empty() {
        if is_anthropic {
            request = request.set("x-api-key", &api_key).set("anthropic-version", "2023-06-01");
        } else {
            request = request.set("Authorization", &format!("Bearer {}", api_key));
        }
    }

    match request.send_json(body) {
        Ok(resp) => {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                if is_ollama {
                    return json.get("response").and_then(|r| r.as_str()).map(|s| s.to_string());
                } else if is_anthropic {
                    return json.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")).and_then(|t| t.as_str()).map(|s| s.to_string());
                } else {
                    return json.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()).map(|s| s.to_string());
                }
            }
        },
        Err(_) => {},
    }

    None
}
