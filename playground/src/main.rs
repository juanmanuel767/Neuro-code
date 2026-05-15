use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use neurocode::interpreter::{Interpreter, Environment};

#[derive(Deserialize)]
struct EjecutarRequest {
    codigo: String,
}

#[derive(Serialize)]
struct EjecutarResponse {
    ok: bool,
    resultado: Option<String>,
    error: Option<String>,
}

const INDEX_HTML: &str = include_str!("../index.html");

fn ejecutar_codigo(codigo: String) -> EjecutarResponse {
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    
    // Configurar intérprete de entorno seguro/aislado
    let mut interpreter = Interpreter::new();
    interpreter.output_buffer = Some(Arc::clone(&output_buffer));
    
    let (tokens, positions) = neurocode::lexer::tokenize_with_positions(&codigo);
    match neurocode::parser::parse_with_positions(tokens, positions) {
        Ok(ast) => {
            let (tx, rx) = std::sync::mpsc::channel();
            // Ejecutar en hilo separado para timeout
            std::thread::spawn(move || {
                let res = interpreter.interpret(ast);
                let _ = tx.send(res);
            });
            
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Ok(_)) => {
                    let out_lines = output_buffer.lock().unwrap();
                    let joined = out_lines.join("\n");
                    EjecutarResponse {
                        ok: true,
                        resultado: Some(if joined.is_empty() { "(sin salida)".to_string() } else { joined }),
                        error: None,
                    }
                },
                Ok(Err(e)) => EjecutarResponse {
                    ok: false,
                    resultado: None,
                    error: Some(format!("Error de ejecución: {}", e)),
                },
                Err(_) => EjecutarResponse {
                    ok: false,
                    resultado: None,
                    error: Some("Error: Timeout - la ejecución tardó más de 5 segundos".to_string()),
                }
            }
        },
        Err(e) => EjecutarResponse {
            ok: false,
            resultado: None,
            error: Some(format!("Error de sintaxis: {}", e)),
        }
    }
}

fn main() {
    let port = 8080;
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
    println!("🚀 NeuroCode Playground en http://localhost:{}", port);
    
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0u8; 65536]; // 64KB buffer
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 { continue; }
                let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                
                let mut lines = request.lines();
                let first_line = lines.next().unwrap_or("");
                let mut parts = first_line.split_whitespace();
                let method = parts.next().unwrap_or("GET");
                let path = parts.next().unwrap_or("/");
                
                if method == "OPTIONS" {
                    let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                    continue;
                }
                
                if method == "GET" && path == "/" {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                        INDEX_HTML.len(), INDEX_HTML
                    );
                    let _ = stream.write_all(response.as_bytes());
                } else if method == "POST" && path == "/ejecutar" {
                    // Buscar content-length
                    let mut content_length = 0;
                    for line in request.lines() {
                        if line.is_empty() { break; }
                        if line.to_lowercase().starts_with("content-length:") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() == 2 {
                                content_length = parts[1].trim().parse().unwrap_or(0);
                            }
                        }
                    }
                    
                    let body_start = request.find("\r\n\r\n").unwrap_or(request.len());
                    let mut body = if body_start + 4 < request.len() {
                        request[body_start + 4..].to_string()
                    } else {
                        String::new()
                    };
                    
                    // Leer más body si no llegó completo en la primer lectura
                    while body.len() < content_length {
                        let mut extra_buf = [0u8; 4096];
                        if let Ok(n) = stream.read(&mut extra_buf) {
                            if n == 0 { break; }
                            body.push_str(&String::from_utf8_lossy(&extra_buf[..n]));
                        } else {
                            break;
                        }
                    }
                    
                    let req: EjecutarRequest = match serde_json::from_str(&body) {
                        Ok(r) => r,
                        Err(_) => EjecutarRequest { codigo: "".to_string() }
                    };
                    
                    let result = ejecutar_codigo(req.codigo);
                    let res_json = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
                    
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                        res_json.len(), res_json
                    );
                    let _ = stream.write_all(response.as_bytes());
                } else {
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
    }
}
