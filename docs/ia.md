# 🧠 NeuroCode - Guía de Inteligencia Artificial

## Configuración

### Ollama (Local - Recomendado)

```bash
# Instalar Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Descargar modelo
ollama pull llama3.2

# Configurar NeuroCode
neuro auth ollama
```

### OpenAI / Groq / Anthropic

```bash
neuro auth openai "sk-tu-clave-api"
neuro auth groq "gsk_tu-clave-api"
neuro auth claude "sk-ant-tu-clave-api"
```

## Funciones Nativas

### `ia(prompt)` - Consulta directa

```neuro
asincrono funcion principal() {
    respuesta = esperar ia("¿Qué es la recursión?")
    imprimir(respuesta)
}
esperar principal()
```

### `ia_generar_codigo(descripcion)` - Genera código NeuroCode

```neuro
asincrono funcion principal() {
    codigo = esperar ia_generar_codigo("función que calcule factorial recursivo")
    imprimir(codigo)
}
esperar principal()
```

## Librería Estándar: `neuro:ia`

```neuro
depredactor neuro:ia como ai

// Preguntar
respuesta = esperar ai.preguntar("¿Cuál es la capital de Francia?")

// Resumir texto largo
resumen = esperar ai.resumir("Texto largo aquí...")

// Traducir
traduccion = esperar ai.traducir("Hello world", "español")

// Clasificar
categoria = esperar ai.clasificar("Me encanta este producto", "positivo, negativo, neutro")
```

### Funciones disponibles

| Función | Parámetros | Descripción |
|:--------|:-----------|:------------|
| `preguntar(prompt)` | Texto | Consulta directa a la IA |
| `resumir(texto)` | Texto | Resume en 3 oraciones |
| `traducir(texto, idioma)` | Texto, Texto | Traduce al idioma indicado |
| `clasificar(texto, categorias)` | Texto, Texto | Clasifica el texto |

## CLI Interactivo

```bash
# Pregunta directa
neuro ia "Explica qué es un árbol binario"

# Modo chat interactivo
neuro ia
🧠 NeuroCode IA - Modo Interactivo
> ¿Qué es una lista enlazada?
  ...
> salir
```

## El Guardián

El Guardián es el sistema de auto-reparación de NeuroCode. Cuando tu código tiene un error, el Guardián:

1. Analiza el error y el código fuente
2. Genera una explicación clara
3. Propone un parche de código
4. Te pregunta si deseas aplicarlo

```bash
# El Guardián se activa automáticamente al ejecutar un script con errores
neuro mi_script.neuro
# Si hay error → El Guardián propone una reparación
```

## Variables de Entorno

| Variable | Descripción |
|:---------|:------------|
| `NEUROCODE_AI_KEY` | Clave API para el proveedor de IA |
| `OLLAMA_HOST` | URL personalizada de Ollama (por defecto: localhost:11434) |

## Prioridad de Credenciales

1. `~/.neurocode_keys` (principal)
2. `~/.aquila_keys` (fallback compatibilidad)
3. Variable `NEUROCODE_AI_KEY`
4. Variable `AQUILA_AI_KEY` (fallback)
