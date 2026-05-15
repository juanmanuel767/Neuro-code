# 📖 Guía Oficial de Sintaxis de NeuroCode v1.0

Bienvenidos a la guía oficial de **NeuroCode** (Aquila), el lenguaje diseñado para la automatización, la Inteligencia Artificial y el desarrollo moderno en español.

---

## 1. Variables y Tipos de Datos

NeuroCode utiliza tipado dinámico pero fuerte. No necesitas declarar el tipo, pero el lenguaje lo conoce.

```aquila
// Variables simples
nombre = "Neuro"            // Texto
version = 1.0               // Numero (Decimal)
es_estable = verdadero      // Booleano
pruebas = 39                // Entero

// Colecciones
mi_lista = [1, 2, 3]        // Lista
mi_dicc = {"ia": "nativa"}  // Diccionario
nulo_val = nulo             // Nulo
```

---

## 2. Operadores

### Aritméticos
- `+`, `-`, `*`, `/`

### Lógicos y Comparación
- `y`, `o`, `no`
- `==`, `!=`, `>`, `<`, `>=`, `<=`

```aquila
x = 10
y = 20
es_mayor = x < y y x != 0   // verdadero
resultado = (1 + 2) * 3     // 9
```

---

## 3. Condicionales (`si` / `sino`)

Las condiciones no requieren paréntesis.

```aquila
puntos = 85

si puntos >= 90 {
    imprimir("Excelente")
} sino si puntos >= 70 {
    imprimir("Aprobado")
} sino {
    imprimir("Reprobado")
}
```

---

## 4. Bucles

### `mientras`
Ejecuta un bloque mientras la condición sea verdadera.

```aquila
cuenta = 5
mientras cuenta > 0 {
    imprimir("T-minus:", cuenta)
    cuenta = cuenta - 1
}
```

### `para` (en listas o rangos)
```aquila
para i en rango(5) {
    imprimir("Iteración:", i)
}

frutas = ["manzana", "pera", "uva"]
para f en frutas {
    imprimir("Me gusta la", f)
}
```

---

## 5. Funciones

Se definen con la palabra clave `funcion` y pueden retornar valores.

```aquila
funcion saludo(nombre) {
    retornar "Hola, " + nombre + "!"
}

mensaje = saludo("Juan")
imprimir(mensaje)
```

---

## 6. Manejo de Errores (`intentar` / `capturar`)

Ideal para operaciones que pueden fallar como llamadas de red o IA.

```aquila
intentar {
    resultado = 10 / 0
} capturar error {
    imprimir("¡Ocurrió un error esperado!", error)
}
```

---

## 7. Inteligencia Artificial Nativa

Usa el poder de los LLMs directamente en tu código.

```aquila
// ia() es asíncrono y requiere 'esperar' o ejecutarse en bloques asíncronos
respuesta = ia("Dime un dato curioso sobre Rust")
imprimir(respuesta)
```

---

## 8. El Depredador (Importación)

Usa `depredactor` para "cazar" librerías de Python o módulos locales.

```aquila
depredactor python:os como os
imprimir("Directorio actual:", os.getcwd())

depredactor "utils.neuro" como utils
utils.hacer_algo()
```

---

## 9. Errores Comunes y Soluciones

| Error | Causa Probable | Solución |
| :--- | :--- | :--- |
| `Variable no definida: 'x'` | Intentaste usar una variable antes de asignarla. | Asigna un valor inicial: `x = 0`. |
| `División por cero.` | Estás dividiendo entre una variable que vale 0. | Añade un `si divisor != 0 { ... }`. |
| `esperaba 1 argumentos pero recibió 2` | Llamaste a una función con más o menos parámetros de los definidos. | Revisa la definición de la función y tus argumentos. |
| `Error de Sintaxis: falta '}'` | Te olvidaste de cerrar un bloque de código. | Revisa que cada `{` tenga su correspondiente `}`. |

---

<p align="center">Construido con ❤️ por la comunidad de NeuroCode.</p>
