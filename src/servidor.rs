use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler_params: Vec<String>,
    pub handler_body: Vec<crate::ast::Statement>,
}

#[derive(Debug)]
pub struct NeuroServer {
    pub port: u16,
    pub routes: Arc<Mutex<Vec<Route>>>,
    pub static_routes: Arc<Mutex<HashMap<String, String>>>,
}

impl NeuroServer {
    pub fn new(port: u16) -> Self {
        NeuroServer {
            port,
            routes: Arc::new(Mutex::new(Vec::new())),
            static_routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_static(&self, path: String, file_path: String) {
        self.static_routes.lock().unwrap().insert(path, file_path);
    }

    pub fn add_route(&self, method: String, path: String, params: Vec<String>, body: Vec<crate::ast::Statement>) {
        self.routes.lock().unwrap().push(Route {
            method,
            path,
            handler_params: params,
            handler_body: body,
        });
    }

    pub fn start(&self, interpreter: &mut crate::interpreter::Interpreter, env: &Arc<Mutex<crate::interpreter::Environment>>) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("No se pudo iniciar el servidor en puerto {}: {}", self.port, e))?;
        
        println!("🌐 Servidor NeuroCode escuchando en http://localhost:{}", self.port);
        
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0u8; 4096];
                    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                    
                    let (method, full_path, body) = parse_request(&request);
                    let (path, query_params) = split_path_and_params(&full_path);
                    
                    let mut matched = false;
                    
                    // Log de request
                    println!("  → {} {}", method, path);
                    
                    // Prioridad 1: Rutas Estáticas
                    let static_routes = self.static_routes.lock().unwrap().clone();
                    if let Some(file_path) = static_routes.get(&path) {
                        matched = true;
                        if let Ok(content) = std::fs::read(file_path) {
                            let content_type = guess_content_type(file_path);
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=3600\r\n\r\n",
                                content_type, content.len()
                            );
                            let _ = stream.write_all(response.as_bytes());
                            let _ = stream.write_all(&content);
                        } else {
                            let err = format!("Error al leer archivo estático: {}", file_path);
                            let response = format!(
                                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: {}\r\n\r\n{}",
                                err.len(), err
                            );
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }

                    // Prioridad 2: Rutas Dinámicas
                    if !matched {
                        let routes = self.routes.lock().unwrap().clone();
                        for route in &routes {
                            if route.method == method && route.path == path {
                                matched = true;
                                
                                // Build request dictionary
                                let mut req_map = HashMap::new();
                                req_map.insert("metodo".to_string(), crate::interpreter::RuntimeValue::Text(method.clone()));
                                req_map.insert("ruta".to_string(), crate::interpreter::RuntimeValue::Text(path.clone()));
                                req_map.insert("cuerpo_texto".to_string(), crate::interpreter::RuntimeValue::Text(body.clone()));
                                
                                // Query Parameters
                                let mut params_map = HashMap::new();
                                for (k, v) in query_params.clone() {
                                    params_map.insert(k, crate::interpreter::RuntimeValue::Text(v));
                                }
                                req_map.insert("parametros".to_string(), crate::interpreter::RuntimeValue::Dictionary(
                                    Arc::new(Mutex::new(params_map))
                                ));
                                
                                // Try parse body as JSON
                                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body) {
                                    let nexus_val = json_to_nexus(&json_val);
                                    req_map.insert("cuerpo".to_string(), nexus_val);
                                }
                                
                                let req_val = crate::interpreter::RuntimeValue::Dictionary(
                                    Arc::new(Mutex::new(req_map))
                                );
                                
                                let call_env = Arc::new(Mutex::new(
                                    crate::interpreter::Environment::new_with_parent(Arc::clone(env))
                                ));
                                
                                if let Some(param) = route.handler_params.first() {
                                    call_env.lock().unwrap().define(param.clone(), req_val);
                                }
                                
                                match interpreter.execute_block_pub(route.handler_body.clone(), &call_env) {
                                    Ok(Some(ret_val)) => {
                                        let response_body = match &ret_val {
                                            crate::interpreter::RuntimeValue::Dictionary(_) |
                                            crate::interpreter::RuntimeValue::List(_) => {
                                                nexus_to_json_string(&ret_val)
                                            },
                                            _ => format!("{}", ret_val),
                                        };
                                        
                                        let content_type = if response_body.trim().starts_with("<!DOCTYPE") || response_body.trim().starts_with("<html") {
                                            "text/html; charset=utf-8"
                                        } else if response_body.trim().starts_with('{') || response_body.trim().starts_with('[') {
                                            "application/json"
                                        } else {
                                            "text/plain; charset=utf-8"
                                        };
                                        
                                        let response = format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                                            content_type, response_body.len(), response_body
                                        );
                                        let _ = stream.write_all(response.as_bytes());
                                    },
                                    Ok(None) => {
                                        let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
                                        let _ = stream.write_all(response.as_bytes());
                                    },
                                    Err(e) => {
                                        let err_body = format!("{{\"error\": \"{}\"}}", e);
                                        let response = format!(
                                            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                                            err_body.len(), err_body
                                        );
                                        let _ = stream.write_all(response.as_bytes());
                                    }
                                }
                                break;
                            }
                        }
                    }
                    
                    if !matched {
                        let not_found = format!("{{\"error\": \"Ruta no encontrada: {} {}\"}}", method, path);
                        let response = format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            not_found.len(), not_found
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                },
                Err(e) => {
                    eprintln!("Error al aceptar conexión: {}", e);
                }
            }
        }
        Ok(())
    }
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Route {
            method: self.method.clone(),
            path: self.path.clone(),
            handler_params: self.handler_params.clone(),
            handler_body: self.handler_body.clone(),
        }
    }
}

fn parse_request(raw: &str) -> (String, String, String) {
    let mut lines = raw.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    let method = parts.get(0).unwrap_or(&"GET").to_string();
    let path = parts.get(1).unwrap_or(&"/").to_string();
    
    // Extract body (after empty line)
    let body = if let Some(pos) = raw.find("\r\n\r\n") {
        raw[pos + 4..].to_string()
    } else {
        String::new()
    };
    
    (method, path, body)
}

fn split_path_and_params(full_path: &str) -> (String, HashMap<String, String>) {
    let mut parts = full_path.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let mut params = HashMap::new();
    
    if let Some(query) = parts.next() {
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            let key = kv.next().unwrap_or("").to_string();
            let val = kv.next().unwrap_or("").replace('+', " "); // Decodificación básica
            if !key.is_empty() {
                params.insert(key, val);
            }
        }
    }
    
    (path, params)
}

pub fn json_to_nexus(val: &serde_json::Value) -> crate::interpreter::RuntimeValue {
    match val {
        serde_json::Value::String(s) => crate::interpreter::RuntimeValue::Text(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                crate::interpreter::RuntimeValue::Int(i)
            } else {
                crate::interpreter::RuntimeValue::Number(n.as_f64().unwrap_or(0.0))
            }
        },
        serde_json::Value::Bool(b) => crate::interpreter::RuntimeValue::Boolean(*b),
        serde_json::Value::Null => crate::interpreter::RuntimeValue::Null,
        serde_json::Value::Array(arr) => {
            let items: Vec<crate::interpreter::RuntimeValue> = arr.iter().map(json_to_nexus).collect();
            crate::interpreter::RuntimeValue::List(Arc::new(Mutex::new(items)))
        },
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_nexus(v));
            }
            crate::interpreter::RuntimeValue::Dictionary(Arc::new(Mutex::new(map)))
        }
    }
}

pub fn nexus_to_json_string(val: &crate::interpreter::RuntimeValue) -> String {
    match val {
        crate::interpreter::RuntimeValue::Text(s) => {
            let escaped = s.replace('\\', "\\\\")
                           .replace('"', "\\\"")
                           .replace('\n', "\\n")
                           .replace('\r', "\\r")
                           .replace('\t', "\\t");
            format!("\"{}\"", escaped)
        },
        crate::interpreter::RuntimeValue::Int(i) => format!("{}", i),
        crate::interpreter::RuntimeValue::Number(n) => format!("{}", n),
        crate::interpreter::RuntimeValue::Boolean(b) => format!("{}", b),
        crate::interpreter::RuntimeValue::Null => "null".to_string(),
        crate::interpreter::RuntimeValue::List(l) => {
            let items: Vec<String> = l.lock().unwrap().iter().map(nexus_to_json_string).collect();
            format!("[{}]", items.join(","))
        },
        crate::interpreter::RuntimeValue::Dictionary(d) => {
            let pairs: Vec<String> = d.lock().unwrap().iter()
                .map(|(k, v)| format!("\"{}\":{}", k, nexus_to_json_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(","))
        },
        _ => "null".to_string()
    }
}

fn guess_content_type(file_path: &str) -> &'static str {
    let lower = file_path.to_lowercase();
    if lower.ends_with(".html") || lower.ends_with(".htm") { "text/html; charset=utf-8" }
    else if lower.ends_with(".css") { "text/css; charset=utf-8" }
    else if lower.ends_with(".js") { "application/javascript; charset=utf-8" }
    else if lower.ends_with(".json") { "application/json; charset=utf-8" }
    else if lower.ends_with(".png") { "image/png" }
    else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "image/jpeg" }
    else if lower.ends_with(".gif") { "image/gif" }
    else if lower.ends_with(".svg") { "image/svg+xml" }
    else if lower.ends_with(".ico") { "image/x-icon" }
    else if lower.ends_with(".woff2") { "font/woff2" }
    else if lower.ends_with(".woff") { "font/woff" }
    else if lower.ends_with(".ttf") { "font/ttf" }
    else if lower.ends_with(".xml") { "application/xml" }
    else if lower.ends_with(".txt") { "text/plain; charset=utf-8" }
    else if lower.ends_with(".pdf") { "application/pdf" }
    else { "application/octet-stream" }
}
