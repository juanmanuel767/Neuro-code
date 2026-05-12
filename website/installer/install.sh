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

# 3. Descargar/Instalar Binario (Copiando el binario local recién compilado)
sudo cp /home/user/lenguaje\ de\ programacion/aquila/target/debug/aquila /usr/local/bin/aquila
sudo chmod +x /usr/local/bin/aquila

# 4. Instalar Extensión de VS Code (Copiando los archivos locales)
cp -r /home/user/lenguaje\ de\ programacion/aquila/vscode-extension/* ~/.vscode/extensions/aquila-lang/ 2>/dev/null

echo "✅ Instalación completada con éxito."
echo "🚀 Escribe 'aquila --version' para empezar."
echo "💡 Abre VS Code para ver la magia de la sintaxis."
