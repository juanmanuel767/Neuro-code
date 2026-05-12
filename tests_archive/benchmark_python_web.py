import json
from http.server import BaseHTTPRequestHandler, HTTPServer
import urllib.parse

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Python Health App</title>
    <style>
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: #f0f2f5; padding: 40px; }
        .container { background: white; padding: 30px; border-radius: 12px; max-width: 400px; box-shadow: 0 4px 15px rgba(0,0,0,0.1); margin: auto; }
        h2 { color: #333; font-size: 18px; text-transform: uppercase; border-bottom: 2px solid #ff007f; padding-bottom: 10px;}
        input { padding: 10px; width: 100%; border: 1px solid #ccc; border-radius: 6px; margin-bottom: 15px; box-sizing: border-box; }
        button { background: #ff007f; color: white; border: none; padding: 12px; width: 100%; border-radius: 6px; font-weight: bold; cursor: pointer; }
        button:hover { background: #ff3399; }
        .result { margin-top: 20px; padding: 15px; background: #e0f7fa; border-left: 4px solid #00bcd4; border-radius: 4px; }
        .result h3 { margin: 5px 0; font-size: 16px; color: #00796b; }
        .nota { font-size: 11px; color: #888; margin-top: 25px; text-align: center; }
    </style>
</head>
<body>
    <div class="container">
        <h2>App de Salud en Python (HTTP Core)</h2>
        <form method="POST">
            <label>Peso (kg):</label>
            <input type="number" step="any" name="peso" value="{peso}">
            
            <label>Altura (m):</label>
            <input type="number" step="any" name="altura" value="{altura}">
            
            <button type="submit">Calcular vía Servidor V2</button>
        </form>
        
        <div class="result">
            <h3>RESULTADO BMI: {imc}</h3>
            <h3>ESTADO: {estado}</h3>
        </div>
        
        <div class="nota">
            Para lograr esta simple calculadora en la web, requirió escribir <strong>casi 100 líneas</strong> de código en Core Python + HTML estático. En cada cálculo se realiza un recargo síncrono.
        </div>
    </div>
</body>
</html>
"""

def calcular_imc(peso, altura):
    return peso / (altura * altura)

def estado_salud(imc):
    if imc < 18.5: return "Bajo Peso (1.0)"
    elif imc > 25.0: return "Sobrepeso (1.0)"
    else: return "Saludable (0.0)"

class RequestHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass # Silenciar logs

    def send_html(self, peso, altura, imc, estado):
        self.send_response(200)
        self.send_header('Content-type', 'text/html; charset=utf-8')
        self.end_headers()
        rendered = HTML.format(peso=peso, altura=altura, imc=round(imc, 2), estado=estado)
        self.wfile.write(rendered.encode('utf-8'))

    def do_GET(self):
        peso = 70.0
        altura = 1.75
        imc = calcular_imc(peso, altura)
        estado = estado_salud(imc)
        self.send_html(peso, altura, imc, estado)
        
    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length).decode('utf-8')
        parsed = urllib.parse.parse_qs(post_data)
        
        try:
            peso = float(parsed.get('peso', ['70.0'])[0])
            altura = float(parsed.get('altura', ['1.75'])[0])
        except ValueError:
            peso = 70.0
            altura = 1.75
            
        imc = calcular_imc(peso, altura)
        estado = estado_salud(imc)
        
        self.send_html(peso, altura, imc, estado)

if __name__ == "__main__":
    print("🚀 Python Core Backend escuchando en http://127.0.0.1:5000")
    server = HTTPServer(('127.0.0.1', 5000), RequestHandler)
    server.serve_forever()
