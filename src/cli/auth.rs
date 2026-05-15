use std::fs;

pub fn mostrar_ayuda(topic: Option<&str>) {
    match topic.unwrap_or("general") {
        "nuevo" => {
            println!("Uso: neuro nuevo mi_app");
            println!("Crea un proyecto NeuroCode básico con main.neuro, neurocode.json, neurocode.lock y README.md.");
            println!("Compatibilidad: aquila nuevo mi_app sigue funcionando.");
        },
        "crear" => {
            println!("Uso: neuro crear \"idea del proyecto\"");
            println!("El Constructor hace preguntas y genera una primera versión funcional asistida por IA.");
        },
        "paquetes" | "instalar" => {
            println!("Uso: neuro instalar paquete");
            println!("Ejemplos:");
            println!("  neuro instalar web");
            println!("  neuro instalar python:pandas");
            println!("  neuro instalar --solo-registrar python:pandas");
            println!("  neuro paquetes");
            println!("  neuro quitar paquete");
        },
        "depredactor" => {
            println!("Ejemplos de depredactor:");
            println!("  depredactor math como mate");
            println!("  depredactor python:selenium como web");
            println!("  depredactor neuro:json como json");
            println!("  depredactor aquila:json como json  # alias compatible");
            println!("  depredactor \"modulo.aq\" como modulo");
            println!("  depredactor \"modulo.neuro\" como modulo");
        },
        "tipos" => {
            println!("Tipos opcionales:");
            println!("  edad: Entero = 20");
            println!("  funcion sumar(a: Entero, b: Entero) -> Entero {{ retornar a + b }}");
            println!("Tipos: Entero, Decimal, Texto, Booleano, Lista, Diccionario, Nulo, Cualquiera.");
        },
        "revisar" => {
            println!("Uso: neuro revisar [archivo_o_carpeta]");
            println!("Parsea archivos .aq/.neuro, valida símbolos básicos y muestra advertencias de seguridad.");
        },
        _ => {
            println!("NeuroCode - ayuda");
            println!("Comandos principales:");
            println!("  neuro nuevo mi_app");
            println!("  neuro crear \"idea\"");
            println!("  neuro instalar paquete");
            println!("  neuro paquetes");
            println!("  neuro quitar paquete");
            println!("  neuro revisar [archivo_o_carpeta]");
            println!("  neuro test [ruta]");
            println!("  neuro --compilar archivo.aq [salida]");
            println!("  neuro ayuda [nuevo|crear|instalar|depredactor|tipos|revisar]");
            println!("Compatibilidad: el comando legacy 'aquila', archivos .aq y aquila.json siguen funcionando.");
        },
    }
}


pub fn save_auth(args: &[String]) {
    let known_provider = args.get(2)
        .map(|p| matches!(p.to_lowercase().as_str(), "openai" | "groq" | "claude" | "anthropic" | "local" | "ollama"))
        .unwrap_or(false);
    let provider = if known_provider { args[2].clone() } else { "openai".to_string() };
    let key = if known_provider {
        args.get(3).cloned().unwrap_or_default()
    } else {
        args.get(2).cloned().unwrap_or_default()
    };

    let url = match provider.to_lowercase().as_str() {
        "local" | "ollama" => "http://localhost:11434/api/generate",
        "groq" => "https://api.groq.com/openai/v1/chat/completions",
        "claude" | "anthropic" => "https://api.anthropic.com/v1/messages",
        _ => "https://api.openai.com/v1/chat/completions",
    };
    
    let model = match provider.to_lowercase().as_str() {
        "local" | "ollama" => {
            if args.len() > 3 { args[3].as_str() } else { "llama3.2:latest" }
        },
        "groq" => "llama-3.3-70b-versatile",
        "claude" | "anthropic" => "claude-3-opus-20240229",
        _ => "gpt-3.5-turbo",
    };

    let key = match provider.to_lowercase().as_str() {
        "local" | "ollama" => "",
        _ => key.as_str(),
    };

    let config = serde_json::json!({
        "url": url,
        "clave": key,
        "modelo": model
    });

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
    let path = format!("{}/.neurocode_keys", home);
    
    match fs::write(&path, config.to_string()) {
        Ok(_) => {
            println!("✅ ¡Éxito! Clave guardada para '{}'.", provider);
            if provider.to_lowercase() == "local" || provider.to_lowercase() == "ollama" {
                println!("NeuroCode usará Ollama local por defecto para la IA.");
            } else {
                println!("NeuroCode usará este proveedor por defecto y volverá a Ollama local si falla.");
            }
        },
        Err(e) => println!("❌ Error al guardar la clave API: {}", e),
    }
}

