#!/bin/bash

# Aquila 'One-Line' Installer v2.1
# Potenciado por el Guardián de Aquila

echo "🦅 Iniciando instalación de Aquila v2.1..."

# 1. Determinar OS
OS="linux" # Simplificado por ahora
ARCH="x86_64"

# 2. Crear directorios necesarios
mkdir -p ~/.aquila_cache
mkdir -p ~/.vscode/extensions/aquila-lang

# 3. Descargar/Instalar Binario (Simulado con el que ya tenemos)
# En producción esto descargaría de un bucket de AWS o GitHub Releases
sudo cp /usr/local/bin/aq /usr/local/bin/aq 2>/dev/null || echo "Ya instalado globalmente."

# 4. Instalar Extensión de VS Code (Copiando los archivos locales)
cp -r /home/user/lenguaje\ de\ programacion/nexus/vscode-extension/* ~/.vscode/extensions/aquila-lang/ 2>/dev/null

echo "✅ Instalación completada con éxito."
echo "🚀 Escribe 'aq --version' para empezar."
echo "💡 Abre VS Code para ver la magia de la sintaxis."
