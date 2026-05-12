pub mod ast;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod servidor;
pub mod base_datos;

use std::fs;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// use serde_json::json; (Removido para evitar advertencias)

fn get_marker() -> Vec<u8> {
    // Ofuscamos el marcador para que no aparezca como literal en el binario
    let part1 = b"---AQUILA_";
    let part2 = b"PAYLOAD---";
    let mut m = Vec::new();
    m.push(b'\n');
    m.extend_from_slice(part1);
    m.extend_from_slice(part2);
    m.push(b'\n');
    m
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let marker = get_marker();
    
    // 1. Intento detectar si somos un binario empaquetado
    if let Ok(mut file) = std::fs::File::open(std::env::current_exe().unwrap()) {
        use std::io::Read;
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);
        
        if let Some(pos) = buffer.windows(marker.len()).rposition(|window| window == marker.as_slice()) {
            // El marcador debe estar "lejos" del inicio para no ser el de la función get_marker
            if pos > 1000000 { 
                let script_data = &buffer[pos + marker.len()..];
                if let Ok(script_code) = std::str::from_utf8(script_data) {
                    run_embedded_script(script_code);
                    return;
                }
            }
        }
    }

    if args.len() > 1 {
        if args[1] == "--compilar" && args.len() >= 3 {
            compilar_script(&args[2], args.get(3).map(|s| s.as_str()));
        } else if args[1] == "crear" {
            if args.len() > 2 {
                crear_proyecto(&args[2]);
            } else {
                println!("❌ Uso: aquila crear \"descripción del proyecto\"");
            }
        } else if args[1] == "auth" {
            if args.len() > 2 {
                save_auth(&args);
            } else {
                println!("❌ Uso: aquila auth [proveedor] \"tu-clave-api\"");
            }
        } else {
            run_file(&args[1]);
        }
    } else {
        println!(r#"
    
       / \
      /   \
     / / \ \      A Q U I L A   v 2.1
    / /   \ \     El Lenguaje Consciente
   / /     \ \    -----------------------
  /_/       \_\   [ IA | ASYNC | VISUAL ]
        "#);
        println!("🦅 Bienvenido a Aquila, el futuro de la programación reactiva.");
        println!("👨‍💻 Creado por: Juan Manuel Peralta");
        println!("💡 Tip: Usa 'aquila crear \"tu idea\"' para que el Arquitecto diseñe tu proyecto.");
        println!("📚 Comandos: --compilar, crear, [archivo.aq]");
        run_repl();
    }
}

fn run_embedded_script(code: &str) {
    let tokens = lexer::tokenize(code);
    match parser::parse(tokens) {
        Ok(ast) => {
            let mut interpreter = interpreter::Interpreter::new();
            if let Err(e) = interpreter.interpret(ast) {
                eprintln!("❌ [Error de Ejecución] {}", e);
            }
        },
        Err(e) => {
            eprintln!("❌ [Error de Sintaxis en Binario] {}", e);
            if let Some(ex) = explain_error_with_ai(code, &e) {
                println!("\n💡 Sugerencia del Águila:\n{}", ex);
            }
        }
    }
}

fn compilar_script(input_path: &str, output_name: Option<&str>) {
    let out_name = output_name.unwrap_or("salida_aquila");
    println!("📦 Compilando '{}' -> '{}'...", input_path, out_name);

    let script_code = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(_) => {
            println!(">> ERROR: No se pudo leer el script '{}'.", input_path);
            return;
        }
    };

    let exe_path = std::env::current_exe().unwrap();
    let mut exe_data = fs::read(&exe_path).unwrap();

    let marker = get_marker();

    // Limpiar payloads previos si existen
    if let Some(pos) = exe_data.windows(marker.len()).rposition(|window| window == marker.as_slice()) {
        if pos > 1000000 {
            exe_data.truncate(pos);
        }
    }
    
    let mut out_data = exe_data;
    out_data.extend_from_slice(&marker);
    out_data.extend_from_slice(script_code.as_bytes());

    match fs::write(out_name, out_data) {
        Ok(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(out_name).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(out_name, perms).unwrap();
            }
            println!("✅ ¡Éxito! Binario independiente generado: '{}'", out_name);
        },
        Err(e) => println!("❌ Error al escribir el binario: {}", e),
    }
}

fn run_file(filepath: &str) {
    println!("🚀 Aquila v1.0 🦅 - Ejecutando Script: {}", filepath);
    
    let code = match fs::read_to_string(&filepath) {
        Ok(c) => c,
        Err(_) => {
            println!(">> ERROR: No se encontró el archivo '{}' en tu disco.", filepath);
            return;
        }
    };
    
    let tokens = lexer::tokenize(&code);
    match parser::parse(tokens) {
        Ok(ast) => {
            let mut interpreter = interpreter::Interpreter::new();
            if let Err(e) = interpreter.interpret(ast) {
                eprintln!("❌ [Error de Ejecución] {}", e);
                solicitar_reparacion(filepath, &code, &e);
            } else {
                println!("✅ Ejecución finalizada con éxito.");
            }
        },
        Err(e) => {
            eprintln!("❌ [Error de Sintaxis] {}", e);
            solicitar_reparacion(filepath, &code, &e);
        }
    }
}

fn solicitar_reparacion(filepath: &str, code: &str, error: &str) {
    if let Some((explicacion, codigo_reparado)) = explain_and_fix_error_with_ai(code, error) {
        println!("\n💡 Sugerencia del Águila:\n{}", explicacion);
        
        print!("\n🔧 ¿Deseas que el Guardián repare tu archivo automáticamente? (s/n): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if input.trim().to_lowercase() == "s" {
                if fs::write(filepath, codigo_reparado).is_ok() {
                    println!("✅ Archivo sobrescrito exitosamente.");
                    println!("Relanzando el script reparado...\n------------------------------------------------");
                    run_file(filepath);
                } else {
                    println!("❌ Error al intentar sobrescribir el archivo.");
                }
            } else {
                println!("Abortado. Arregla el archivo manualmente.");
            }
        }
    }
}

fn explain_error_with_ai(_code: &str, _error: &str) -> Option<String> {
    explain_and_fix_error_with_ai(_code, _error).map(|t| t.0)
}

fn explain_and_fix_error_with_ai(_code: &str, _error: &str) -> Option<(String, String)> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✨"])
        .template("{spinner:.blue} {msg}").unwrap());
    pb.set_message("🔍 El Águila (Guardián) está analizando el error y preparando un Fix...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let prompt = format!(
        "Eres el 'Guardián' de 'Aquila'.\n\
         REGLAS DE ORO:\n\
         1. NO uses 'importar' ni 'usar' para la clase 'BaseDatos'. Es NATIVA, ya está disponible.\n\
         2. La clase se llama 'BaseDatos' (sin 'De').\n\
         3. Usa llaves {{ }} en funciones y bucles.\n\
         4. IA se consulta con 'esperar ia(\"...\")'.\n\
         5. Al final del código llama con 'esperar principal()'.\n\n\
         Código Original Roto:\n{}\n\n\
         Error en consola: {}\n\n\
         Debes proporcionar DOS SECCIONES separadas por '---AQUILA_DIVISOR---'.\n\
         SECCION 1: Explicación corta.\n\
         SECCION 2: Código reparado (SIN IMPORTACIONES INNECESARIAS) entre ---INICIO--- y ---FIN---.",
        _code, _error
    );

    // Cargar credenciales universales tal como lo hace interpreter.rs
    let mut url = "http://localhost:11434/api/generate".to_string();
    let mut model = "llama3.2:latest".to_string();
    let mut api_key = String::new();
    let mut is_anthropic = false;

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
    if let Ok(config_str) = std::fs::read_to_string(format!("{}/.aquila_keys", home)) {
        if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_str) {
            if let Some(u) = config_json.get("url").and_then(|u| u.as_str()) { url = u.to_string(); is_anthropic = u.contains("anthropic"); }
            if let Some(c) = config_json.get("clave").and_then(|c| c.as_str()) { api_key = c.to_string(); }
            if let Some(m) = config_json.get("modelo").and_then(|m| m.as_str()) { model = m.to_string(); }
        }
    }

    let is_ollama = url.contains("localhost:11434");

    let body = if is_ollama {
        serde_json::json!({"model": model, "prompt": prompt, "stream": false})
    } else if is_anthropic {
        serde_json::json!({"model": model, "max_tokens": 1024, "messages": [{"role": "user", "content": prompt}]})
    } else {
        serde_json::json!({"model": model, "messages": [{"role": "user", "content": prompt}]})
    };

    let mut request = ureq::post(&url).timeout(std::time::Duration::from_secs(60));
    if !api_key.is_empty() {
        if is_anthropic { request = request.set("x-api-key", &api_key).set("anthropic-version", "2023-06-01"); } 
        else { request = request.set("Authorization", &format!("Bearer {}", api_key)); }
    }

    match request.send_json(body) {
        Ok(resp) => {
            pb.finish_and_clear();
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                let mut content_opt = None;
                if is_ollama { content_opt = json.get("response").and_then(|r| r.as_str()); }
                else if is_anthropic { content_opt = json.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")).and_then(|t| t.as_str()); }
                else { content_opt = json.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()); }
                
                if let Some(r) = content_opt {
                    let parts: Vec<&str> = r.split("---AQUILA_DIVISOR---").collect();
                    if parts.len() == 2 {
                        let mut explain = parts[0].trim().to_string();
                        explain = explain.replace("SECCION 1:", "").trim().to_string();
                        
                        let code_part = parts[1];
                        if let Some(start_idx) = code_part.find("---INICIO---") {
                            if let Some(end_idx) = code_part.find("---FIN---") {
                                let fixed_code = code_part[start_idx + 12..end_idx].trim().to_string();
                                return Some((explain, fixed_code));
                            }
                        }
                        
                        return Some((explain, parts[1].trim().to_string())); // Fallback
                    }
                }
            }
            None
        },
        Err(_) => {
            pb.finish_and_clear();
            None
        },
    }
}

fn crear_proyecto(descripcion: &str) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["🥚", "🐣", "🐤", "🐥", "🦅", "✨"])
        .template("{spinner} {msg}").unwrap());
    pb.set_message("🏗️ El Arquitecto de Aquila está diseñando tu proyecto...");
    pb.enable_steady_tick(Duration::from_millis(150));

    // println!("🏗️ El Arquitecto de Aquila está diseñando tu proyecto...");
    println!("🔍 Descripción: \"{}\"", descripcion);

    let prompt = format!(
        "Eres un Arquitecto Senior y Master del lenguaje Aquila v2.0. Tu misión es generar el código fuente para un proyecto completo basado en esta descripción: \"{}\".\n\n\
         Debes proporcionar el código principal en un bloque ```aquila y una breve guía en un bloque ```markdown.\n\n\
         REGLAS DE ORO DE AQUILA:\n\
         1. Todo código debe iniciar con `asincrono funcion main()` y terminar con `esperar main()`.\n\
         2. Usa `esperar` para llamadas a `ia()`, `http_get()`, y métodos de `BaseDatos`.\n\
         3. Incluye comentarios profesionales en español.",
        descripcion
    );

    let body = serde_json::json!({
        "model": "llama3.2:latest",
        "prompt": prompt,
        "stream": false
    });

    // Intentamos obtener el diseño de la IA, pero tenemos un respaldo de oro
    let mut files_created = 0;
    let dir_name = "proyecto_aquila";
    let _ = std::fs::create_dir_all(dir_name);

    match ureq::post("http://127.0.0.1:11434/api/generate")
        .timeout(std::time::Duration::from_secs(45)) // Menos tiempo para disparar el respaldo rápido
        .send_json(body) 
    {
        Ok(resp) => {
            pb.finish_and_clear();
            if let Ok(json_resp) = resp.into_json::<serde_json::Value>() {
                if let Some(r_str) = json_resp.get("response").and_then(|r| r.as_str()) {
                    println!("📂 Generando archivos desde el Arquitecto...");
                    
                    if let Some(start) = r_str.find("```aquila") {
                        let sub = &r_str[start + 9..];
                        if let Some(end) = sub.find("```") {
                            let content = &sub[..end];
                            let _ = std::fs::write(format!("{}/main.aq", dir_name), content.trim());
                            println!("  + [Creado] main.aq");
                            files_created += 1;
                        }
                    }
                    
                    if let Some(start) = r_str.find("```markdown") {
                        let sub = &r_str[start + 11..];
                        if let Some(end) = sub.find("```") {
                            let content = &sub[..end];
                            let _ = std::fs::write(format!("{}/README.md", dir_name), content.trim());
                            println!("  + [Creado] README.md");
                            files_created += 1;
                        }
                    }
                }
            }
        },
        Err(_) => {
            pb.finish_and_clear();
            println!("💡 El Arquitecto está saturado, usando Plantilla Maestra de Alta Performance...");
        }
    }

    // SI LA IA FALLÓ O NO DIO BLOQUES, USAMOS EL RESPALDO PROFESIONAL
    if files_created == 0 {
        let is_dashboard = descripcion.to_lowercase().contains("dashboard") || descripcion.to_lowercase().contains("monitor");
        
        let (main_code, readme_content, extra_files) = if is_dashboard {
            (
                r#"// --- AQUILA 2.0 VISUAL DASHBOARD: SALES MONITOR ---
asincrono funcion main() {
    imprimir("🛰️ Iniciando Servidor de Monitoreo Visual (Aquila 8080)...")
    
    db = nuevo BaseDatos("ventas.db")
    esperar db.ejecutar("CREATE TABLE IF NOT EXISTS ventas (id INTEGER PRIMARY KEY, producto TEXT, total REAL, fecha TEXT)")
    
    // Insertamos datos de prueba si está vacía
    conteo = esperar db.consultar("SELECT COUNT(*) as total FROM ventas")
    // (Lógica simplificada para demo)
    esperar db.ejecutar("INSERT INTO ventas (producto, total, fecha) VALUES ('Nexus Pro', 1200.50, '2026-05-11')")
    
    servidor = nuevo ServidorWeb(8080)
    
    asincrono funcion api_datos(peticion) {
        datos = esperar db.consultar("SELECT * FROM ventas ORDER BY id DESC LIMIT 10")
        responder(datos)
    }
    
    servidor.ruta("GET", "/api/datos", api_datos)
    servidor.estatico("/", "index.html")
    
    imprimir("🚀 Dashboard disponible en http://localhost:8080")
    esperar servidor.iniciar()
}

esperar main()
"#,
                "# 🎨 Dashboard Reactivo de Aquila\nMonitor de ventas generado automáticamente con visualización en tiempo real.",
                vec![("index.html", r#"<!DOCTYPE html>
<html>
<head>
    <title>Aquila Vision Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body { background: #0f172a; color: white; font-family: 'Inter', sans-serif; }
        .glass { background: rgba(255, 255, 255, 0.05); backdrop-filter: blur(10px); border: 1px solid rgba(255, 255, 255, 0.1); }
    </style>
</head>
<body class="p-8">
    <div class="max-w-6xl mx-auto">
        <header class="flex justify-between items-center mb-12">
            <h1 class="text-4xl font-bold text-blue-400">🦅 Aquila <span class="text-white">Vision</span></h1>
            <div class="px-4 py-2 glass rounded-full text-sm font-mono text-emerald-400">● Sistema Activo</div>
        </header>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
            <div class="glass p-6 rounded-2xl">
                <p class="text-slate-400 text-sm mb-1">Total Ventas</p>
                <h2 class="text-3xl font-bold">$1,200.50</h2>
            </div>
            <div class="glass p-6 rounded-2xl">
                <p class="text-slate-400 text-sm mb-1">Nodos Activos</p>
                <h2 class="text-3xl font-bold text-blue-400">12</h2>
            </div>
            <div class="glass p-6 rounded-2xl">
                <p class="text-slate-400 text-sm mb-1">Latencia IA</p>
                <h2 class="text-3xl font-bold text-emerald-400">120ms</h2>
            </div>
        </div>

        <div class="glass p-8 rounded-3xl mb-12">
            <h3 class="text-xl font-bold mb-6">Tendencia de Datos en Tiempo Real</h3>
            <canvas id="ventasChart" height="100"></canvas>
        </div>
        <footer class=\"text-center text-slate-500 text-sm mt-8\">
            Creado por <span class=\"text-blue-400 font-semibold\">Juan Manuel Peralta</span> | Aquila v2.1 🦅
        </footer>
    </div>

    <script>
        const ctx = document.getElementById('ventasChart').getContext('2d');
        const chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: ['10:00', '11:00', '12:00', '13:00', '14:00', '15:00'],
                datasets: [{
                    label: 'Ventas (USD)',
                    data: [400, 600, 800, 500, 1000, 1200],
                    borderColor: '#60a5fa',
                    backgroundColor: 'rgba(96, 165, 250, 0.1)',
                    fill: true,
                    tension: 0.4
                }]
            },
            options: { plugins: { legend: { display: false } }, scales: { y: { grid: { color: '#1e293b' } }, x: { grid: { display: false } } } }
        });

        async function updateData() {
            try {
                const res = await fetch('/api/datos');
                const data = await res.json();
                console.log("Datos de Aquila:", data);
            } catch (e) {
                console.warn("API de Aquila no disponible aún.");
            }
        }
        setInterval(updateData, 5000);
    </script>
</body>
</html>"#)]
            )
        } else {
            (
                r#"// --- AQUILA 2.0 MASTER PROJECT: SMART NOTES ---
asincrono funcion main() {
    imprimir("🛡️ Iniciando Servicio de Notas Inteligente (Aquila v2.0)...")
    db = nuevo BaseDatos("notas.db")
    esperar db.ejecutar("CREATE TABLE IF NOT EXISTS notas (id INTEGER PRIMARY KEY, contenido TEXT, categoria TEXT)")
    
    imprimir("💾 Base de Datos conectada.")
    contenido = "Necesito comprar café y huevos para el desayuno."
    imprimir("📝 Nueva nota recibida: " + contenido)
    
    prompt = "Categoriza esta nota en una sola palabra: " + contenido
    categoria = esperar ia(prompt)
    
    imprimir("🧠 IA Categorizó como: " + categoria)
    esperar db.ejecutar("INSERT INTO notas (contenido, categoria) VALUES (?, ?)", [contenido, categoria])
    
    imprimir("✅ Nota guardada con éxito.")
    notas = esperar db.consultar("SELECT * FROM notas")
    imprimir(notas)
}

esperar main()
"#,
                "# 🦅 Proyecto Aquila - Servicio de Notas Inteligente\nEste proyecto demuestra la superioridad de Aquila v2.0 frente a Python.",
                vec![]
            )
        };

        let _ = std::fs::write(format!("{}/main.aq", dir_name), main_code.trim());
        let _ = std::fs::write(format!("{}/README.md", dir_name), readme_content.trim());
        println!("  + [Creado] main.aq (Plantilla de Oro)");
        println!("  + [Creado] README.md (Plantilla de Oro)");
        
        for (ruta, contenido) in extra_files {
            let _ = std::fs::write(format!("{}/{}", dir_name, ruta), contenido.trim());
            println!("  + [Creado] {} (Plantilla de Oro)", ruta);
        }
    }

    println!("---------------------------------------------------");
    println!("✅ ¡Proyecto generado con éxito en '{}'!", dir_name);
    println!("💡 Tip: Compila tu app con: aquila --compilar {}/main.aq mi_app", dir_name);
}

fn run_repl() {
    println!("🚀 Aquila v1.0 REPL 🦅 — Escribe 'salir' para terminar");
    let mut interpreter = interpreter::Interpreter::new();
    
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error al leer entrada. Saliendo...");
            break;
        }
        
        let input = input.trim();
        if input == "salir" {
            break;
        }
        if input.is_empty() {
            continue;
        }
        
        let tokens = lexer::tokenize(input);
        match parser::parse(tokens) {
            Ok(ast) => {
                match interpreter.interpret(ast) {
                    Ok(val) => {
                        if val != interpreter::RuntimeValue::Null {
                            println!("{}", val);
                        }
                    },
                    Err(e) => eprintln!("❌ {}", e),
                }
            },
            Err(e) => {
                eprintln!("❌ {}", e);
                if let Some(ex) = explain_error_with_ai(input, &e) {
                    println!("💡 {}", ex);
                }
            },
        }
    }
    println!("Adiós!");
}

fn save_auth(args: &[String]) {
    let provider = if args.len() > 3 { args[2].clone() } else { "openai".to_string() };
    let key = if args.len() > 3 { args[3].clone() } else { args[2].clone() };

    let url = match provider.to_lowercase().as_str() {
        "groq" => "https://api.groq.com/openai/v1/chat/completions",
        "claude" | "anthropic" => "https://api.anthropic.com/v1/messages",
        _ => "https://api.openai.com/v1/chat/completions",
    };
    
    let model = match provider.to_lowercase().as_str() {
        "groq" => "llama-3.3-70b-versatile",
        "claude" | "anthropic" => "claude-3-opus-20240229",
        _ => "gpt-3.5-turbo",
    };

    let config = serde_json::json!({
        "url": url,
        "clave": key,
        "modelo": model
    });

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
    let path = format!("{}/.aquila_keys", home);
    
    match fs::write(&path, config.to_string()) {
        Ok(_) => {
            println!("✅ ¡Éxito! Clave guardada para '{}'.", provider);
            println!("Aquila usará este proveedor por defecto para la IA en todos tus scripts.");
        },
        Err(e) => println!("❌ Error al guardar la clave API: {}", e),
    }
}
