# Plan de potencia de NeuroCode

Este documento es la ruta viva para hacer crecer NeuroCode sin romper lo que ya funciona.

Nota de compatibilidad: NeuroCode nace desde Aquila, por eso `aquila`, `.aq`, `aquila.json` y `aquila.lock` siguen siendo compatibles mientras se agrega la identidad nueva.

## Regla principal

Cada fase nueva debe pasar:

```bash
cargo test
cargo run -- test
cargo run -- test tests/compat
```

## Base protegida

Compatibilidad que no se debe romper:

- `usar` sigue funcionando.
- `depredactor` sigue funcionando.
- Modulos `.aq` relativos siguen funcionando.
- Python FFI sigue funcionando.
- `json_parsear` y `json_texto` siguen funcionando.
- `intentar/capturar` sigue atrapando errores.
- `ia()` arranca con Ollama local sin configuracion inicial.
- Si hay API externa y falla, NeuroCode vuelve a Ollama.

## Fase 1: Depredactor universal

Objetivo: centralizar la resolucion de librerias y preparar fuentes nuevas.

Estado actual:

- Existe un resolvedor interno inicial para decidir entre:
  - modulo remoto
  - archivo NeuroCode `.aq` o `.neuro`
  - modulo Python
- Se aceptan prefijos explicitos:
  - `python:modulo`
  - `py:modulo`
  - `neuro:modulo`
  - `neurocode:modulo`
  - `nc:modulo`
  - `aquila:"ruta.aq"`
  - `aq:"ruta.aq"`
  - `remoto:"https://..."`
  - `web:"https://..."`
  - `url:"https://..."`
- `sistema:` y `sys:` se reconocen, pero se bloquean por seguridad hasta tener una politica clara.
- El comportamiento publico se mantiene igual.

Siguiente expansion:

```aquila
depredactor python:selenium como web
depredactor neuro:calendario como cal
depredactor sistema:ffmpeg como video
```

## Fase 2: Sistema de paquetes

Objetivo:

```bash
neuro instalar selenium
neuro paquetes
neuro quitar selenium
```

Archivos previstos:

- `aquila.json`
- `aquila.lock`

Primera version:

- Python via `pip`.
- Modulos NeuroCode locales.
- Registro local simple de alias.

Estado actual:

- `neuro instalar paquete` registra dependencias en `aquila.json`.
- `neuro instalar python:paquete` registra e intenta instalar con `pip`.
- `neuro instalar --solo-registrar python:paquete` registra sin ejecutar `pip`.
- `neuro instalar neuro:web`, `neuro instalar aquila:web` o `neuro instalar web` registra modulo estandar NeuroCode sin usar red.
- `neuro paquetes` lista dependencias registradas.
- `neuro quitar paquete` elimina una dependencia del manifiesto.
- `aquila.lock` se sincroniza al instalar o quitar dependencias.
- Si `depredactor` falla al importar una libreria Python, revisa `aquila.json` y sugiere el comando correcto.
- Existe un registro local inicial de modulos estandar en `lib/aquila/`.
- `depredactor neuro:json`, `depredactor neuro:web`, `depredactor neuro:db`, `depredactor neuro:archivos`, `depredactor neuro:tiempo`, `depredactor neuro:pruebas` y `depredactor neuro:ia` cargan modulos reales.
- `aquila:` y `aq:` quedan como alias compatibles.
- Los paquetes NeuroCode registrados en `aquila.json` pueden resolverse como modulos estandar si existen en `lib/aquila/`.
- La stdlib incluye envoltorios iniciales para archivos, tiempo, JSON, web, DB, pruebas e IA.

Siguiente expansion:

- Ampliar `web`, `db` e `ia` con APIs mas ergonomicas.

## Fase 3: Tipos opcionales

Sintaxis deseada:

```aquila
edad: Entero = 20
nombre: Texto = "Ana"
funcion sumar(a: Entero, b: Entero) -> Entero {
    retornar a + b
}
```

Regla:

- Codigo sin tipos debe seguir funcionando.
- Tipos se validan primero en runtime.

Estado actual:

- Variables tipadas:

```aquila
edad: Entero = 20
nombre: Texto = "Ana"
activo: Booleano = verdadero
```

- Reasignar una variable tipada valida el tipo.
- Parametros tipados en funciones:

```aquila
funcion sumar(a: Entero, b: Entero) {
    retornar a + b
}
```

- Tipo de retorno opcional:

```aquila
funcion etiqueta(id: Entero) -> Texto {
    retornar "id-" + texto(id)
}
```

- Tipos soportados inicialmente:
  - `Entero`
  - `Decimal` / `Numero`
  - `Texto`
  - `Booleano`
  - `Lista`
  - `Diccionario`
  - `Nulo`
  - `Cualquiera`

Siguiente expansion:

- Tipos opcionales en funciones asincronas y lambdas ya usan la misma base; falta ampliar mensajes y tooling.
- Validacion estatica temprana en `neuro revisar`.

## Fase 4: Herramientas de proyecto

Comandos deseados:

```bash
neuro nuevo mi_app
neuro ayuda
neuro revisar
neuro formatear
neuro reparar app.aq
```

Estado actual:

- `neuro ayuda`
- `neuro ayuda nuevo`
- `neuro ayuda instalar`
- `neuro ayuda depredactor`
- `neuro ayuda tipos`
- `neuro nuevo mi_app`
- `neuro revisar [archivo|carpeta]`

`neuro nuevo` crea:

- `main.aq`
- `aquila.json`
- `aquila.lock`
- `README.md`
- `src/`

Siguiente expansion:

- `neuro formatear`
- `neuro reparar app.aq`

## Fase 5: IA nativa avanzada

Objetivo:

- Explicar errores.
- Reparar con validacion.
- Generar pruebas.
- Optimizar funciones.
- Mostrar diff antes de cambios grandes.

## Fase 6: Framework estandar

Modulos esperados:

- `web`
- `db`
- `archivos`
- `navegador`
- `ia`
- `json`
- `tiempo`
- `pruebas`

## Fase 7: Concurrencia simple

Sintaxis posible:

```aquila
resultado = esperar todo([
    descargar(url1),
    descargar(url2)
])
```

Luego:

```aquila
paralelo {
    tarea descargar()
    tarea procesar()
}
```

## Fase 8: VM y compilacion fuerte

Camino seguro:

1. Mantener interprete actual.
2. Crear bytecode interno.
3. Compilar AST a bytecode.
4. Ejecutar VM.
5. Optimizar.
6. Solo despues compilar a binario avanzado, WASM o backend nativo.

## Politica de no ruptura

- Antes de tocar parser o lexer, agregar prueba.
- Cada bug corregido se convierte en prueba.
- Sintaxis vieja se conserva.
- Cambios grandes se hacen detras de comandos o funciones nuevas.
- El Guardian no sobrescribe codigo si la reparacion no parsea o no valida.
