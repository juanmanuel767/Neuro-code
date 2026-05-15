# Referencia de la Librería Estándar de NeuroCode

La librería estándar de NeuroCode está estructurada para proveer funcionalidad potente sin depender de bibliotecas externas adicionales al iniciar un proyecto.

## 🧮 `neuro:matematicas`
Módulo para operaciones elementales.
*   `abs(n)`: Retorna el valor absoluto de un número.
*   `max(a, b)`: Retorna el mayor de dos números.
*   `min(a, b)`: Retorna el menor de dos números.
*   `potencia(base, exp)`: Eleva una base a un exponente utilizando aritmética entera o dinámica.

## 📝 `neuro:texto`
Utilidades para manipular cadenas de texto respaldadas por la velocidad nativa y FFI.
*   `mayusculas(t)`: Convierte el texto `t` a mayúsculas.
*   `minusculas(t)`: Convierte el texto `t` a minúsculas.
*   `dividir(t, sep)`: Devuelve una lista de cadenas, separadas por el delimitador `sep`.

## 🌐 `neuro:http`
Herramienta nativa para realizar peticiones web de forma segura.
*   `obtener(url)`: Realiza una solicitud HTTP GET a la `url` especificada.
*   `enviar(url, datos)`: Realiza una solicitud HTTP POST eviando un diccionario o JSON `datos` a la `url`.

## 📦 `neuro:colecciones`
Funciones utilitarias para operar estructuras de datos complejas.
*   `contiene(lista, elemento)`: Verifica si el `elemento` está contenido dentro de la `lista`.
*   `invertir(lista)`: Invierte el orden de los elementos en una `lista` devolviendo una nueva lista.

## 📊 `neuro:json`
*   `parsear(texto_json)`: Convierte un texto JSON en un diccionario nativo.
*   `texto(diccionario)`: Convierte un diccionario en un String JSON (no implementado en la plantilla predeterminada del FFI Python de momento pero reservado).

## 🗄️ `neuro:db`
Acceso embebido a la base de datos SQL del proyecto.
*   `abrir(ruta)`: Devuelve un contexto de base de datos `.db`.

## 🔐 `neuro:ia`
Acceso integrado al motor de Inteligencia Artificial que potencia a NeuroCode.
*   `preguntar(prompt)`: (Asíncrono) Envía un prompt a la IA y espera la respuesta textual.

## ⏱️ `neuro:tiempo`
*   `ahora_unix()`: Retorna el timestamp unix actual.
*   `esperar_segundos(segs)`: Pausa la ejecución por la cantidad de segundos indicada.

---
**Nota:** Para utilizar cualquier módulo, simplemente invoca `usar "neuro:modulo" como alias`.
