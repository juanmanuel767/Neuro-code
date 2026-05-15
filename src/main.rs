pub mod ast;
pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod servidor;
pub mod base_datos;
pub mod depredactor;

pub mod cli;
use cli::tests::*;
use cli::auth::*;
use cli::paquetes::*;
use cli::proyectos::*;
use std::fs;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
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

pub fn ollama_generate_urls() -> Vec<String> {
    let mut urls = Vec::new();

    if let Ok(host) = std::env::var("OLLAMA_HOST") {
        let host = host.trim().trim_end_matches('/');
        if !host.is_empty() {
            let base = if host.starts_with("http://") || host.starts_with("https://") {
                host.to_string()
            } else {
                format!("http://{}", host)
            };
            urls.push(format!("{}/api/generate", base));
        }
    }

    urls.push("http://localhost:11434/api/generate".to_string());
    urls.push("http://127.0.0.1:11434/api/generate".to_string());
    urls.dedup();
    urls
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
            let idea_inicial = if args.len() > 2 {
                Some(args[2..].join(" "))
            } else {
                None
            };
            crear_proyecto_guiado(idea_inicial);
        } else if args[1] == "nuevo" {
            nuevo_proyecto_cli(&args);
        } else if args[1] == "init" {
            init_proyecto_cli();
        } else if args[1] == "ayuda" || args[1] == "--ayuda" || args[1] == "-h" {
            mostrar_ayuda(args.get(2).map(|s| s.as_str()));
        } else if args[1] == "auth" {
            if args.len() > 2 {
                save_auth(&args);
            } else {
                println!("❌ Uso: neuro auth [proveedor] \"tu-clave-api\"");
            }
        } else if args[1] == "instalar" {
            instalar_paquete_cli(&args);
        } else if args[1] == "paquetes" {
            listar_paquetes_cli();
        } else if args[1] == "quitar" {
            quitar_paquete_cli(&args);
        } else if args[1] == "revisar" {
            let target = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            revisar_cli(target);
        } else if args[1] == "test" {
            let target = args.get(2).map(|s| s.as_str()).unwrap_or("tests");
            run_tests(target);
        } else if args[1] == "ia" {
            cli::ia::ia_cli(&args);
        } else if args[1] == "servidor" {
            cli::servidor::servidor_cli(&args);
        } else {
            run_file(&args[1]);
        }
    } else {
        println!(r#"
    
       / \
      /   \
     / / \ \      N E U R O C O D E   v 2.1
    / /   \ \     El Lenguaje Consciente
   / /     \ \    -----------------------
  /_/       \_\   [ IA | ASYNC | VISUAL ]
        "#);
        println!("🧠 Bienvenido a NeuroCode, el lenguaje inteligente para construir con menos errores.");
        println!("👨‍💻 Creado por: Juan Manuel Peralta");
        println!("💡 Tip: Usa 'neuro crear \"tu idea\"' para que el Arquitecto diseñe tu proyecto.");
        println!("📚 Comandos: nuevo, crear, instalar, paquetes, quitar, revisar, ayuda, auth, test, --compilar, [archivo.aq|archivo.neuro]");
        println!("🧩 Compatibilidad: 'aquila', '.neuro' y 'aquila.json' siguen funcionando.");
        run_repl();
    }
}

fn run_embedded_script(code: &str) {
    let (tokens, positions) = lexer::tokenize_with_positions(code);
    match parser::parse_with_positions(tokens, positions) {
        Ok(ast) => {
            let mut interpreter = interpreter::Interpreter::new();
            if let Err(e) = interpreter.interpret(ast) {
                eprintln!("❌ [Error de Ejecución] {}", e);
            }
        },
        Err(e) => {
            eprintln!("❌ [Error de Sintaxis en Binario] {}", e);
            if let Some(ex) = explain_error_with_ai(code, &e) {
                println!("\n💡 Sugerencia del Cerebro:\n{}", ex);
            }
        }
    }
}

fn compilar_script(input_path: &str, output_name: Option<&str>) {
    let out_name = output_name.unwrap_or("salida_neurocode");
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

pub fn run_file(filepath: &str) {
    println!("🚀 NeuroCode v2.1 - Ejecutando Script: {}", filepath);
    let ok = execute_file(filepath, true);
    if ok {
        println!("✅ Ejecución finalizada con éxito.");
    }
}

pub fn execute_file(filepath: &str, repair_on_error: bool) -> bool {
    let code = match fs::read_to_string(&filepath) {
        Ok(c) => c,
        Err(_) => {
            println!(">> ERROR: No se encontró el archivo '{}' en tu disco.", filepath);
            return false;
        }
    };

    let base_dir = Path::new(filepath)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    
    let (tokens, positions) = lexer::tokenize_with_positions(&code);
    match parser::parse_with_positions(tokens, positions) {
        Ok(ast) => {
            let mut interpreter = interpreter::Interpreter::with_base_dir(base_dir);
            if let Err(e) = interpreter.interpret(ast) {
                eprintln!("❌ [Error de Ejecución] {}", e);
                if repair_on_error {
                    solicitar_reparacion(filepath, &code, &e);
                }
                false
            } else {
                true
            }
        },
        Err(e) => {
            eprintln!("❌ [Error de Sintaxis] {}", e);
            if repair_on_error {
                solicitar_reparacion(filepath, &code, &e);
            }
            false
        }
    }
}


fn solicitar_reparacion(filepath: &str, code: &str, error: &str) {
    if let Some((explicacion, codigo_reparado)) = explain_and_fix_error_with_ai(code, error) {
        println!("\n[Error Inteligente] 🧠");
        println!("{}", explicacion);
        
        print!("\n🔧 ¿Deseas que el Guardián repare tu archivo automáticamente? (s/n): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if input.trim().to_lowercase() == "s" {
                if let Err(validation_error) = validar_reparacion_guardian(&codigo_reparado) {
                    println!("❌ El Guardián generó una reparación inválida.");
                    println!("   No se sobrescribió el archivo. {}", validation_error);
                    return;
                }

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

fn validar_reparacion_guardian(code: &str) -> Result<(), String> {
    let (tokens, positions) = lexer::tokenize_with_positions(code);
    let ast = parser::parse_with_positions(tokens, positions)
        .map_err(|e| format!("Error del parche: {}", e))?;

    let mut symbols = guardian_builtin_symbols();
    collect_declared_symbols(&ast, &mut symbols);

    for stmt in &ast {
        validate_statement_symbols(stmt, &symbols)?;
    }

    Ok(())
}

pub fn guardian_builtin_symbols() -> HashSet<String> {
    [
        "imprimir",
        "rango",
        "tipo",
        "longitud",
        "entrada",
        "a_numero",
        "a_texto",
        "entero",
        "decimal",
        "texto",
        "json_parsear",
        "json_texto",
        "mayusculas",
        "minusculas",
        "contiene",
        "dividir",
        "unir",
        "agregar",
        "quitar",
        "http_get",
        "http_post",
        "ia",
        "leer_archivo",
        "archivo_existe",
        "escribir_archivo",
        "timestamp",
        "dormir",
        "BaseDatos",
        "ServidorWeb",
    ]
    .iter()
    .map(|name| name.to_string())
    .collect()
}

pub fn collect_declared_symbols(statements: &[ast::Statement], symbols: &mut HashSet<String>) {
    for stmt in statements {
        match stmt {
            ast::Statement::Function(name, _, _, body) |
            ast::Statement::AsyncFunction(name, _, _, body) => {
                symbols.insert(name.clone());
                collect_declared_symbols(body, symbols);
            },
            ast::Statement::Class(name, methods) => {
                symbols.insert(name.clone());
                collect_declared_symbols(methods, symbols);
            },
            ast::Statement::If(_, then_branch, else_branch) => {
                collect_declared_symbols(then_branch, symbols);
                collect_declared_symbols(else_branch, symbols);
            },
            ast::Statement::While(_, body) |
            ast::Statement::For(_, _, body) => collect_declared_symbols(body, symbols),
            ast::Statement::TryCatch(try_block, _, catch_block) => {
                collect_declared_symbols(try_block, symbols);
                collect_declared_symbols(catch_block, symbols);
            },
            _ => {},
        }
    }
}

pub fn validate_statement_symbols(stmt: &ast::Statement, symbols: &HashSet<String>) -> Result<(), String> {
    match stmt {
        ast::Statement::Assign(_, expr) |
        ast::Statement::AssignTyped(_, _, expr) |
        ast::Statement::Return(expr) |
        ast::Statement::Expression(expr) |
        ast::Statement::Throw(expr) => validate_expression_symbols(expr, symbols),
        ast::Statement::AssignProperty(callee, _, value) => {
            validate_expression_symbols(callee, symbols)?;
            validate_expression_symbols(value, symbols)
        },
        ast::Statement::AssignIndex(callee, index, value) => {
            validate_expression_symbols(callee, symbols)?;
            validate_expression_symbols(index, symbols)?;
            validate_expression_symbols(value, symbols)
        },
        ast::Statement::If(cond, then_branch, else_branch) => {
            validate_expression_symbols(cond, symbols)?;
            validate_statement_list_symbols(then_branch, symbols)?;
            validate_statement_list_symbols(else_branch, symbols)
        },
        ast::Statement::While(cond, body) => {
            validate_expression_symbols(cond, symbols)?;
            validate_statement_list_symbols(body, symbols)
        },
        ast::Statement::For(_, iterable, body) => {
            validate_expression_symbols(iterable, symbols)?;
            validate_statement_list_symbols(body, symbols)
        },
        ast::Statement::Function(_, _, _, body) |
        ast::Statement::AsyncFunction(_, _, _, body) |
        ast::Statement::Class(_, body) => validate_statement_list_symbols(body, symbols),
        ast::Statement::TryCatch(try_block, _, catch_block) => {
            validate_statement_list_symbols(try_block, symbols)?;
            validate_statement_list_symbols(catch_block, symbols)
        },
        ast::Statement::Usar(_, _) |
        ast::Statement::Export(_) |
        ast::Statement::Break => Ok(()),
        ast::Statement::Parallel(stmts) |
        ast::Statement::Block(stmts) => validate_statement_list_symbols(stmts, symbols),
        ast::Statement::Task(stmt) => validate_statement_symbols(stmt, symbols),
        ast::Statement::Reactive(_, expr) => validate_expression_symbols(expr, symbols),
        ast::Statement::ReactObserve(_, body) |
        ast::Statement::Api(body) => validate_statement_list_symbols(body, symbols),
        ast::Statement::ApiRoute(_, body) => validate_statement_list_symbols(body, symbols),
    }
}

fn validate_statement_list_symbols(statements: &[ast::Statement], symbols: &HashSet<String>) -> Result<(), String> {
    for stmt in statements {
        validate_statement_symbols(stmt, symbols)?;
    }
    Ok(())
}

fn validate_expression_symbols(expr: &ast::Expression, symbols: &HashSet<String>) -> Result<(), String> {
    match expr {
        ast::Expression::BinaryOp(left, _, right) |
        ast::Expression::LogicalOp(left, _, right) => {
            validate_expression_symbols(left, symbols)?;
            validate_expression_symbols(right, symbols)
        },
        ast::Expression::UnaryOp(_, inner) |
        ast::Expression::Await(inner) => validate_expression_symbols(inner, symbols),
        ast::Expression::FunctionCall(name, args) => {
            for arg in args {
                validate_expression_symbols(arg, symbols)?;
            }
            if symbols.contains(name) {
                Ok(())
            } else {
                Err(format!("Error semántico del parche: llamada a función desconocida '{}'.", name))
            }
        },
        ast::Expression::MethodCall(callee, _, args) => {
            validate_expression_symbols(callee, symbols)?;
            for arg in args {
                validate_expression_symbols(arg, symbols)?;
            }
            Ok(())
        },
        ast::Expression::IndexAccess(callee, index) => {
            validate_expression_symbols(callee, symbols)?;
            validate_expression_symbols(index, symbols)
        },
        ast::Expression::NewInstance(class_name, args) => {
            for arg in args {
                validate_expression_symbols(arg, symbols)?;
            }
            if symbols.contains(class_name) {
                Ok(())
            } else {
                Err(format!("Error semántico del parche: clase desconocida '{}'.", class_name))
            }
        },
        ast::Expression::List(items) => {
            for item in items {
                validate_expression_symbols(item, symbols)?;
            }
            Ok(())
        },
        ast::Expression::Dictionary(pairs) => {
            for (key, value) in pairs {
                validate_expression_symbols(key, symbols)?;
                validate_expression_symbols(value, symbols)?;
            }
            Ok(())
        },
        ast::Expression::LambdaFunction(_, _, body) => validate_statement_list_symbols(body, symbols),
        ast::Expression::Number(_) |
        ast::Expression::Int(_) |
        ast::Expression::Text(_) |
        ast::Expression::Boolean(_) |
        ast::Expression::Null |
        ast::Expression::Identifier(_) => Ok(()),
    }
}

pub fn collect_review_warnings(statements: &[ast::Statement], inside_try: bool, warnings: &mut Vec<String>) {
    for stmt in statements {
        match stmt {
            ast::Statement::Assign(_, expr) |
            ast::Statement::AssignTyped(_, _, expr) |
            ast::Statement::Return(expr) |
            ast::Statement::Expression(expr) |
            ast::Statement::Throw(expr) => collect_expression_warnings(expr, inside_try, warnings),
            ast::Statement::AssignProperty(callee, _, value) => {
                collect_expression_warnings(callee, inside_try, warnings);
                collect_expression_warnings(value, inside_try, warnings);
            },
            ast::Statement::AssignIndex(callee, index, value) => {
                collect_expression_warnings(callee, inside_try, warnings);
                collect_expression_warnings(index, inside_try, warnings);
                collect_expression_warnings(value, inside_try, warnings);
            },
            ast::Statement::If(cond, then_branch, else_branch) => {
                collect_expression_warnings(cond, inside_try, warnings);
                collect_review_warnings(then_branch, inside_try, warnings);
                collect_review_warnings(else_branch, inside_try, warnings);
            },
            ast::Statement::While(cond, body) => {
                collect_expression_warnings(cond, inside_try, warnings);
                collect_review_warnings(body, inside_try, warnings);
            },
            ast::Statement::For(_, iterable, body) => {
                collect_expression_warnings(iterable, inside_try, warnings);
                collect_review_warnings(body, inside_try, warnings);
            },
            ast::Statement::Function(_, _, _, body) |
            ast::Statement::AsyncFunction(_, _, _, body) |
            ast::Statement::Class(_, body) => collect_review_warnings(body, inside_try, warnings),
            ast::Statement::TryCatch(try_block, _, catch_block) => {
                collect_review_warnings(try_block, true, warnings);
                collect_review_warnings(catch_block, inside_try, warnings);
            },
            ast::Statement::Usar(_, _) |
            ast::Statement::Export(_) |
            ast::Statement::Break => {},
            ast::Statement::Parallel(stmts) |
            ast::Statement::Block(stmts) => collect_review_warnings(stmts, inside_try, warnings),
            ast::Statement::Task(stmt) => collect_review_warnings(std::slice::from_ref(stmt), inside_try, warnings),
            ast::Statement::Reactive(_, expr) => collect_expression_warnings(expr, inside_try, warnings),
            ast::Statement::ReactObserve(_, body) |
            ast::Statement::Api(body) => collect_review_warnings(body, inside_try, warnings),
            ast::Statement::ApiRoute(_, body) => collect_review_warnings(body, inside_try, warnings),
        }
    }
}

fn collect_expression_warnings(expr: &ast::Expression, inside_try: bool, warnings: &mut Vec<String>) {
    match expr {
        ast::Expression::FunctionCall(name, args) => {
            if name == "ia" && !inside_try {
                warnings.push("ia() se está llamando fuera de intentar/capturar; considera agregar fallback local.".to_string());
            }
            for arg in args {
                collect_expression_warnings(arg, inside_try, warnings);
            }
        },
        ast::Expression::BinaryOp(left, _, right) |
        ast::Expression::LogicalOp(left, _, right) => {
            collect_expression_warnings(left, inside_try, warnings);
            collect_expression_warnings(right, inside_try, warnings);
        },
        ast::Expression::UnaryOp(_, inner) |
        ast::Expression::Await(inner) => collect_expression_warnings(inner, inside_try, warnings),
        ast::Expression::MethodCall(callee, _, args) => {
            collect_expression_warnings(callee, inside_try, warnings);
            for arg in args {
                collect_expression_warnings(arg, inside_try, warnings);
            }
        },
        ast::Expression::IndexAccess(callee, index) => {
            collect_expression_warnings(callee, inside_try, warnings);
            collect_expression_warnings(index, inside_try, warnings);
        },
        ast::Expression::NewInstance(_, args) |
        ast::Expression::List(args) => {
            for arg in args {
                collect_expression_warnings(arg, inside_try, warnings);
            }
        },
        ast::Expression::Dictionary(pairs) => {
            for (key, value) in pairs {
                collect_expression_warnings(key, inside_try, warnings);
                collect_expression_warnings(value, inside_try, warnings);
            }
        },
        ast::Expression::LambdaFunction(_, _, body) => collect_review_warnings(body, inside_try, warnings),
        ast::Expression::Number(_) |
        ast::Expression::Int(_) |
        ast::Expression::Text(_) |
        ast::Expression::Boolean(_) |
        ast::Expression::Null |
        ast::Expression::Identifier(_) => {},
    }
}

fn explain_error_with_ai(_code: &str, _error: &str) -> Option<String> {
    explain_and_fix_error_with_ai(_code, _error).map(|t| t.0)
}

fn parse_guardian_fix_response(response: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = response.split("---NEUROCODE_DIVISOR---").collect();
    if parts.len() != 2 {
        return None;
    }

    let mut explain = parts[0].trim().to_string();
    explain = explain.replace("SECCION 1:", "").trim().to_string();

    let code_part = parts[1];
    if let Some(start_idx) = code_part.find("---INICIO---") {
        if let Some(end_idx) = code_part.find("---FIN---") {
            let fixed_code = code_part[start_idx + 12..end_idx].trim().to_string();
            return Some((explain, fixed_code));
        }
    }

    Some((explain, parts[1].trim().to_string()))
}

fn explain_and_fix_error_with_ai(_code: &str, _error: &str) -> Option<(String, String)> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✨"])
        .template("{spinner:.blue} {msg}").unwrap());
    pb.set_message("🔍 NeuroCode Guardián está analizando el error y preparando un fix...");
    pb.enable_steady_tick(Duration::from_millis(120));

    let prompt = format!(
        "Eres el 'Guardián' de 'NeuroCode' v2.1. Tu objetivo es explicar errores y reparar código.\n\
         REGLAS DE EXPLICACION:\n\
         1. Inicia con una frase clara como: 'La variable [nombre] esperaba un tipo [tipo] pero recibió [tipo].'\n\
         2. Explica brevemente por qué es un error (ej: 'No puedes sumar texto con números').\n\
         3. Sé profesional y directo.\n\n\
         REGLAS DE REPARACION:\n\
         1. NO uses 'importar', 'usar' ni 'depredactor' para la clase 'BaseDatos'. Es NATIVA.\n\
         2. La clase se llama 'BaseDatos'.\n\
         3. Usa llaves {{ }} en funciones y bucles.\n\
         4. IA se consulta con 'esperar ia(\"...\")'.\n\
         5. Al final del código llama con 'esperar principal()' si existe una función principal asíncrona.\n\
         6. NO envuelvas todo el archivo reparado en llaves {{ }}.\n\
         7. La SECCION 2 debe contener SOLO código NeuroCode ejecutable entre ---INICIO--- y ---FIN---.\n\n\
         Código Original Roto:\n{}\n\n\
         Error de NeuroCode: {}\n\n\
         Debes proporcionar DOS SECCIONES separadas por '---NEUROCODE_DIVISOR---'.\n\
         SECCION 1: Explicación detallada pero concisa.\n\
         SECCION 2: Código reparado entre ---INICIO--- y ---FIN---.",
        _code, _error
    );

    // Cargar credenciales universales tal como lo hace interpreter.rs
    let ollama_urls = ollama_generate_urls();
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
        if is_anthropic { request = request.set("x-api-key", &api_key).set("anthropic-version", "2023-06-01"); } 
        else { request = request.set("Authorization", &format!("Bearer {}", api_key)); }
    }

    match request.send_json(body) {
        Ok(resp) => {
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                let mut content_opt = None;
                if is_ollama { content_opt = json.get("response").and_then(|r| r.as_str()); }
                else if is_anthropic { content_opt = json.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")).and_then(|t| t.as_str()); }
                else { content_opt = json.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()); }
                
                if let Some(r) = content_opt {
                    if let Some(parsed) = parse_guardian_fix_response(r) {
                        pb.finish_and_clear();
                        return Some(parsed);
                    }
                }
            }
        },
        Err(_) => {
        },
    }

    if !is_ollama {
        pb.set_message("🔁 La IA primaria no dio un parche válido. Probando Guardián local...");
        let fallback_body = serde_json::json!({"model": "llama3.2:latest", "prompt": prompt, "stream": false});
        for ollama_url in &ollama_urls {
            if let Ok(resp) = ureq::post(ollama_url)
                .timeout(std::time::Duration::from_secs(60))
                .send_json(fallback_body.clone())
            {
                if let Ok(json) = resp.into_json::<serde_json::Value>() {
                    if let Some(r) = json.get("response").and_then(|r| r.as_str()) {
                        if let Some(parsed) = parse_guardian_fix_response(r) {
                            pb.finish_and_clear();
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    } else {
        pb.set_message("🔁 Ollama no dio un parche válido. Probando rutas locales alternativas...");
        let fallback_body = serde_json::json!({"model": "llama3.2:latest", "prompt": prompt, "stream": false});
        for ollama_url in ollama_urls.iter().filter(|candidate| *candidate != &url) {
            if let Ok(resp) = ureq::post(ollama_url)
                .timeout(std::time::Duration::from_secs(60))
                .send_json(fallback_body.clone())
            {
                if let Ok(json) = resp.into_json::<serde_json::Value>() {
                    if let Some(r) = json.get("response").and_then(|r| r.as_str()) {
                        if let Some(parsed) = parse_guardian_fix_response(r) {
                            pb.finish_and_clear();
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    }

    pb.finish_and_clear();
    None
}

fn preguntar_constructor(pregunta: &str) -> String {
    print!("{}", pregunta);
    io::stdout().flush().unwrap();

    let mut respuesta = String::new();
    if io::stdin().read_line(&mut respuesta).is_ok() {
        respuesta.trim().to_string()
    } else {
        String::new()
    }
}

fn construir_descripcion_guiada(idea_base: &str, usuario_objetivo: &str, funciones_clave: &str, integraciones: &str) -> String {
    format!(
        "IDEA PRINCIPAL:\n{}\n\n\
         USUARIO Y OBJETIVO:\n{}\n\n\
         FUNCIONES CLAVE:\n{}\n\n\
         DATOS, INTEGRACIONES Y RESTRICCIONES:\n{}\n\n\
         INSTRUCCION PARA NEUROCODE:\n\
         Construye una primera version funcional, simple de ejecutar, con codigo NeuroCode claro, pruebas basicas y README. \
         Prioriza que funcione antes que agregar complejidad innecesaria.",
        idea_base.trim(),
        usuario_objetivo.trim(),
        funciones_clave.trim(),
        integraciones.trim()
    )
}

fn crear_proyecto_guiado(idea_inicial: Option<String>) {
    println!("🧭 Constructor NeuroCode: antes de crear, necesito entender el proyecto.");
    if let Some(idea) = &idea_inicial {
        if !idea.trim().is_empty() {
            println!("💡 Idea inicial: {}", idea.trim());
        }
    }

    let mut idea_base = preguntar_constructor("1. ¿Qué quieres construir o qué problema debe resolver? ");
    if idea_base.is_empty() {
        idea_base = idea_inicial.unwrap_or_else(|| "Una app NeuroCode funcional".to_string());
    }

    let usuario_objetivo = preguntar_constructor("2. ¿Quién lo va a usar y cuál es el resultado principal que espera? ");
    let funciones_clave = preguntar_constructor("3. ¿Qué funciones, datos o integraciones son indispensables? ");

    let descripcion = construir_descripcion_guiada(
        &idea_base,
        if usuario_objetivo.is_empty() { "Usuario general; quiere una solucion facil de usar." } else { &usuario_objetivo },
        if funciones_clave.is_empty() { "Interfaz o flujo principal funcional, datos de prueba, validacion basica." } else { &funciones_clave },
        "Usar capacidades nativas de NeuroCode cuando sea posible; usar depredactor solo si aporta valor claro."
    );

    crear_proyecto(&descripcion);
}

fn crear_proyecto(descripcion: &str) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["🌑", "🌘", "🌗", "🌖", "🌕", "🧠", "✨"])
        .template("{spinner} {msg}").unwrap());
    pb.set_message("🏗️ El Arquitecto de NeuroCode está diseñando tu proyecto...");
    pb.enable_steady_tick(Duration::from_millis(150));

    // println!("🏗️ El Arquitecto de NeuroCode está diseñando tu proyecto...");
    println!("🔍 Descripción: \"{}\"", descripcion);

    let prompt = format!(
        "Eres un Arquitecto Senior y Master del lenguaje NeuroCode v2.1. Tu misión es generar el código fuente para un proyecto completo basado en esta descripción: \"{}\".\n\n\
         Debes proporcionar el código principal en un bloque ```aquila y una breve guía en un bloque ```markdown.\n\n\
         REGLAS DE ORO DE NEUROCODE:\n\
         1. Todo código debe iniciar con `asincrono funcion main()` y terminar con `esperar main()`.\n\
         2. Usa `esperar` para llamadas a `ia()`, `http_get()`, y métodos de `BaseDatos`.\n\
         3. Toda llamada a `ia()` debe estar dentro de `intentar/capturar` y tener un valor de respaldo útil si la IA no responde.\n\
         4. Incluye comentarios profesionales en español.",
        descripcion
    );

    let body = serde_json::json!({
        "model": "llama3.2:latest",
        "prompt": prompt,
        "stream": false
    });

    // Intentamos obtener el diseño de la IA, pero tenemos un respaldo de oro
    let mut files_created = 0;
    let dir_name = "proyecto_neurocode";
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
                            let _ = std::fs::write(format!("{}/main.neuro", dir_name), content.trim());
                            println!("  + [Creado] main.neuro");
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
                r#"// --- NEUROCODE VISUAL DASHBOARD: SALES MONITOR ---
asincrono funcion main() {
    imprimir("🛰️ Iniciando Servidor de Monitoreo Visual (NeuroCode 8080)...")
    
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
                "# 🎨 Dashboard Reactivo de NeuroCode\nMonitor de ventas generado automáticamente con visualización en tiempo real.",
                vec![("index.html", r#"<!DOCTYPE html>
<html>
<head>
    <title>NeuroCode Vision Dashboard</title>
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
            <h1 class="text-4xl font-bold text-blue-400">NeuroCode <span class="text-white">Vision</span></h1>
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
            Creado por <span class=\"text-blue-400 font-semibold\">Juan Manuel Peralta</span> | NeuroCode v2.1
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
                console.log("Datos de NeuroCode:", data);
            } catch (e) {
                console.warn("API de NeuroCode no disponible aún.");
            }
        }
        setInterval(updateData, 5000);
    </script>
</body>
</html>"#)]
            )
        } else {
            (
                r#"// --- NEUROCODE MASTER PROJECT: SMART NOTES ---
asincrono funcion main() {
    imprimir("🛡️ Iniciando Servicio de Notas Inteligente (NeuroCode v2.1)...")
    db = nuevo BaseDatos("notas.db")
    esperar db.ejecutar("CREATE TABLE IF NOT EXISTS notas (id INTEGER PRIMARY KEY, contenido TEXT, categoria TEXT)")
    
    imprimir("💾 Base de Datos conectada.")
    contenido = "Necesito comprar café y huevos para el desayuno."
    imprimir("📝 Nueva nota recibida: " + contenido)
    
    prompt = "Categoriza esta nota en una sola palabra: " + contenido
    categoria = "General"
    intentar {
        categoria = esperar ia(prompt)
    } capturar e {
        imprimir("⚠️ IA no disponible. Usando categoría por defecto: " + categoria)
    }
    
    imprimir("🧠 IA Categorizó como: " + categoria)
    esperar db.ejecutar("INSERT INTO notas (contenido, categoria) VALUES (?, ?)", [contenido, categoria])
    
    imprimir("✅ Nota guardada con éxito.")
    notas = esperar db.consultar("SELECT * FROM notas")
    imprimir(notas)
}

esperar main()
"#,
                "# Proyecto NeuroCode - Servicio de Notas Inteligente\nEste proyecto demuestra el enfoque inteligente de NeuroCode.",
                vec![]
            )
        };

        let _ = std::fs::write(format!("{}/main.neuro", dir_name), main_code.trim());
        let _ = std::fs::write(format!("{}/README.md", dir_name), readme_content.trim());
        println!("  + [Creado] main.neuro (Plantilla de Oro)");
        println!("  + [Creado] README.md (Plantilla de Oro)");
        
        for (ruta, contenido) in extra_files {
            let _ = std::fs::write(format!("{}/{}", dir_name, ruta), contenido.trim());
            println!("  + [Creado] {} (Plantilla de Oro)", ruta);
        }
    }

    println!("---------------------------------------------------");
    println!("✅ ¡Proyecto generado con éxito en '{}'!", dir_name);
    println!("💡 Tip: Compila tu app con: neuro --compilar {}/main.neuro mi_app", dir_name);
}

fn run_repl() {
    println!("🚀 NeuroCode v2.1 REPL — Escribe 'salir' para terminar");
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
        
        let (tokens, positions) = lexer::tokenize_with_positions(input);
        match parser::parse_with_positions(tokens, positions) {
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

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test]
    fn descripcion_guiada_incluye_respuestas_clave() {
        let descripcion = construir_descripcion_guiada(
            "dashboard de ventas",
            "equipo comercial",
            "login, reportes y base de datos",
            "usar SQLite nativo",
        );

        assert!(descripcion.contains("dashboard de ventas"));
        assert!(descripcion.contains("equipo comercial"));
        assert!(descripcion.contains("login, reportes y base de datos"));
        assert!(descripcion.contains("usar SQLite nativo"));
        assert!(descripcion.contains("funcione antes"));
    }

    #[test]
    fn paquete_resuelve_origenes_basicos() {
        assert_eq!(cli::paquetes::resolve_package_spec("selenium"), cli::paquetes::PackageSpec {
            origin: "python".to_string(),
            package: "selenium".to_string(),
        });
        assert_eq!(cli::paquetes::resolve_package_spec("python:pandas"), cli::paquetes::PackageSpec {
            origin: "python".to_string(),
            package: "pandas".to_string(),
        });
        assert_eq!(cli::paquetes::resolve_package_spec("py:requests"), cli::paquetes::PackageSpec {
            origin: "python".to_string(),
            package: "requests".to_string(),
        });
        assert_eq!(cli::paquetes::resolve_package_spec("web"), cli::paquetes::PackageSpec {
            origin: "neurocode".to_string(),
            package: "web".to_string(),
        });
        assert_eq!(cli::paquetes::resolve_package_spec("aq:calendario"), cli::paquetes::PackageSpec {
            origin: "neurocode".to_string(),
            package: "calendario".to_string(),
        });
    }

    #[test]
    fn instalar_cli_acepta_solo_registrar_en_cualquier_posicion() {
        let args = vec![
            "aquila".to_string(),
            "instalar".to_string(),
            "--solo-registrar".to_string(),
            "python:pandas".to_string(),
        ];
        let solo_registrar = args.iter().skip(2).any(|arg| arg == "--solo-registrar" || arg == "--solo_registrar");
        let raw_package = args.iter().skip(2).find(|arg| !arg.starts_with("--"));
        assert!(solo_registrar);
        assert_eq!(raw_package.map(|s| s.as_str()), Some("python:pandas"));

        let args = vec![
            "aquila".to_string(),
            "instalar".to_string(),
            "python:pandas".to_string(),
            "--solo_registrar".to_string(),
        ];
        let solo_registrar = args.iter().skip(2).any(|arg| arg == "--solo-registrar" || arg == "--solo_registrar");
        let raw_package = args.iter().skip(2).find(|arg| !arg.starts_with("--"));
        assert!(solo_registrar);
        assert_eq!(raw_package.map(|s| s.as_str()), Some("python:pandas"));
    }

    #[test]
    fn nuevo_proyecto_crea_estructura_minima() {
        let base = std::env::temp_dir().join(format!("aquila_nuevo_test_{}", std::process::id()));
        let project = base.join("demo_app");
        let _ = fs::remove_file(project.join("main.neuro"));
        let _ = fs::remove_file(project.join("neurocode.json"));
        let _ = fs::remove_file(project.join("neurocode.lock"));
        let _ = fs::remove_file(project.join("README.md"));
        let _ = fs::remove_dir(project.join("src"));
        let _ = fs::remove_dir(&project);
        let _ = fs::create_dir_all(&base);

        let created = cli::proyectos::write_new_project("demo_app", &base).unwrap();
        assert_eq!(created, project);
        assert!(created.join("main.neuro").exists());
        assert!(created.join("neurocode.json").exists());
        assert!(created.join("neurocode.lock").exists());
        assert!(created.join("README.md").exists());
        assert!(created.join("src").exists());

        let manifest = fs::read_to_string(created.join("neurocode.json")).unwrap();
        assert!(manifest.contains("\"nombre\": \"demo_app\""));
        assert!(manifest.contains("\"pruebas\""));

        let _ = fs::remove_file(created.join("main.neuro"));
        let _ = fs::remove_file(created.join("neurocode.json"));
        let _ = fs::remove_file(created.join("neurocode.lock"));
        let _ = fs::remove_file(created.join("README.md"));
        let _ = fs::remove_dir(created.join("src"));
        let _ = fs::remove_dir(&created);
        let _ = fs::remove_dir(&base);
    }

    #[test]
    fn nuevo_proyecto_rechaza_nombres_inseguros() {
        assert!(sanitize_project_name("").is_err());
        assert!(sanitize_project_name("../x").is_err());
        assert!(sanitize_project_name("mi app").is_err());
        assert_eq!(sanitize_project_name("mi_app-1").unwrap(), "mi_app-1");
    }

    #[test]
    fn revisar_advierte_ia_fuera_de_try() {
        let warnings = revisar_codigo("respuesta = esperar ia(\"hola\")").unwrap();
        assert!(warnings.iter().any(|w| w.contains("ia()")));
    }

    #[test]
    fn revisar_acepta_ia_con_fallback() {
        let code = r#"
respuesta = "fallback"
intentar {
    respuesta = esperar ia("hola")
} capturar e {
    respuesta = "fallback"
}
"#;
        let warnings = revisar_codigo(code).unwrap();
        assert!(warnings.is_empty(), "advertencias inesperadas: {:?}", warnings);
    }

    #[test]
    fn manifiesto_agrega_lista_y_quita_paquetes() {
        let path = std::env::temp_dir().join(format!(
            "aquila_manifest_test_{}_{}.json",
            std::process::id(),
            "paquetes"
        ));
        let lock_path = cli::paquetes::lock_path_for_manifest(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&lock_path);

        let spec = cli::paquetes::add_package_to_manifest(&path, "python:selenium").unwrap();
        assert_eq!(spec.origin, "python");
        assert_eq!(spec.package, "selenium");
        assert!(lock_path.exists());

        let spec = cli::paquetes::add_package_to_manifest(&path, "web").unwrap();
        assert_eq!(spec.origin, "neurocode");
        assert_eq!(spec.package, "web");

        let packages = cli::paquetes::package_names_from_manifest(&path).unwrap();
        assert!(packages.contains(&("selenium".to_string(), "python".to_string())));
        assert!(packages.contains(&("web".to_string(), "neurocode".to_string())));

        assert_eq!(cli::paquetes::remove_package_from_manifest(&path, "selenium").unwrap(), true);
        assert_eq!(cli::paquetes::remove_package_from_manifest(&path, "selenium").unwrap(), false);

        let packages = cli::paquetes::package_names_from_manifest(&path).unwrap();
        assert!(!packages.iter().any(|(name, _)| name == "selenium"));
        assert!(packages.iter().any(|(name, _)| name == "web"));

        let lock_content = fs::read_to_string(&lock_path).unwrap();
        let lock_json = serde_json::from_str::<serde_json::Value>(&lock_content).unwrap();
        assert_eq!(lock_json.get("formato").and_then(|v| v.as_i64()), Some(2));
        assert!(lock_json.get("dependencias").and_then(|v| v.get("web")).is_some());
        assert!(lock_json.get("dependencias").and_then(|v| v.get("selenium")).is_none());

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&lock_path);
    }
}
