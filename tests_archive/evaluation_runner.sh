#!/bin/bash
# Runner para la Suite de Evaluación de Aquila

AQUILA_BIN="./target/debug/aquila"

echo "================================================="
echo "   REPORTE DE EVALUACIÓN AUTOMÁTICA DE AQUILA"
echo "================================================="
echo ""

# 1. Prueba de Correctitud
echo "▶ Ejecutando Suite de Correctitud..."
$AQUILA_BIN tests/eval_correctitud.aq
if [ $? -eq 0 ]; then
    echo "✔ Correctitud: OK"
else
    echo "❌ Correctitud: FALLÓ (revisar salida arriba)"
fi
echo ""

# 2. Prueba de Rendimiento (Benchmark)
echo "▶ Ejecutando Benhcmark en Aquila..."
time $AQUILA_BIN tests/eval_rendimiento.aq
echo ""

echo "▶ Ejecutando Benchmark Equivalente en Python..."
time python3 tests/benchmark_python.py
echo ""

echo "================================================="
echo "                 FIN DE LA EVALUACIÓN"
echo "================================================="
