use ureq;
use serde_json::json;

fn main() {
    let prompt = "Hola, eres un experto en Aquila. Explica por qué falta una llave en este código: si verdadero {";
    let body = json!({
        "model": "llama3.2:latest",
        "prompt": prompt,
        "stream": false
    });

    println!("Consultando Ollama...");
    match ureq::post("http://localhost:11434/api/generate")
        .timeout(std::time::Duration::from_secs(30))
        .send_json(body) 
    {
        Ok(resp) => {
            println!("Respuesta recibida.");
            if let Ok(json) = resp.into_json::<serde_json::Value>() {
                println!("JSON parseado: {:?}", json);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
