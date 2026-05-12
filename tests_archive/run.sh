#!/bin/bash
echo "======================================"
echo "⚡ INICIANDO MOTOR NEXUS ⚡"
echo "======================================"

# Matar servidores viejos para evitar colisiones
pkill -f "nexus_runtime" 2>/dev/null

echo "[1/3] Compilando lógica asíncrona..."
cargo build

echo "[2/3] Levantando Servidor Reactivo (Puerto 8080)..."
TARGET=${1:-"../cerebro_grafos/calculadora.nx"}
cargo run -- "$TARGET" &

sleep 2
echo "[3/3] Desplegando Universal IDE en el navegador..."
xdg-open "file://$(pwd)/dashboard/index.html" 2>/dev/null || open "file://$(pwd)/dashboard/index.html" 2>/dev/null

echo "======================================"
echo "¡EXITO! El lenguaje está corriendo universalmente."
echo "Edita tu archivo .nx, guarda, y el IDE se adaptará."
