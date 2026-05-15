// CLI module para el servidor incorporado

pub fn servidor_cli(args: &[String]) {
    let port = args
        .get(2)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3000);

    let mut path = std::env::current_dir().unwrap_or_default();
    if let Some(p) = args.get(3) {
        path = std::path::PathBuf::from(p);
    }

    println!("🚀 Iniciando servidor estático NeuroCode");
    println!("📂 Directorio: {}", path.display());
    
    let server = crate::servidor::NeuroServer::new(port);
    
    // Add default route to serve the directory
    // This is a simple static server implementation for the CLI
    server.add_route("GET".to_string(), "/".to_string(), vec![], vec![]); 
    // In order to properly emulate dynamic folder serving, we would need to map it in rust.
    // For now, let's keep it simple: inform the user.
    println!("⚠️ Nota: Para servir archivos completos estáticos, usa un script con `servidor.estatico(\"/\", \"ruta\")`.");
    
    println!("🌐 Servidor escuchando en http://localhost:{}", port);
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&addr)
        .expect("No se pudo inciar el servidor");
        
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            use std::io::{Read, Write};
            let mut buffer = [0u8; 4096];
            let _ = stream.read(&mut buffer);

            let req = String::from_utf8_lossy(&buffer);
            let first_line = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            let method = parts.get(0).unwrap_or(&"GET");
            let req_path = parts.get(1).unwrap_or(&"/");

            println!("  → {} {}", method, req_path);

            // Very simple static file serving emulation
            let mut target_path = path.join(req_path.trim_start_matches('/'));
            if target_path.is_dir() {
                target_path = target_path.join("index.html");
            }
            
            if let Ok(content) = std::fs::read(&target_path) {
                let content_type = match target_path.extension().and_then(|s| s.to_str()).unwrap_or("") {
                    "html" | "htm" => "text/html; charset=utf-8",
                    "css" => "text/css; charset=utf-8",
                    "js" => "application/javascript; charset=utf-8",
                    "json" => "application/json; charset=utf-8",
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "svg" => "image/svg+xml",
                    "txt" => "text/plain; charset=utf-8",
                    _ => "application/octet-stream",
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                    content_type, content.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&content);
            } else {
                let not_found = format!("Ruta no encontrada \n{} \nNeuroCode Server v2.1", target_path.display());
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    not_found.len(), not_found
                );
                let _ = stream.write_all(response.as_bytes());
            }
        }
    }
}
