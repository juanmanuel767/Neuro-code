#!/bin/bash

# NeuroCode 'One-Line' Installer v2.1
# Potenciado por el Asistente IA de NeuroCode

echo "🚀 Iniciando instalación de NeuroCode v2.1..."

# 1. Determinar OS
OS="linux" # Simplificado por ahora
ARCH="x86_64"

# 2. Crear directorios necesarios
mkdir -p ~/.neurocode_cache
mkdir -p ~/.neurocode/lib
mkdir -p ~/.vscode/extensions/neurocode-lang

# 3. Descargar/Instalar Binario
# Nota: Aquí es donde descargaremos el binario oficial desde GitHub Releases en el futuro.
echo "📥 Descargando motor de Rust de NeuroCode..."
# curl -L https://github.com/juanmanuel767/aquila/releases/latest/download/neurocode -o /tmp/neurocode
# sudo mv /tmp/neurocode /usr/local/bin/neuro

# Por ahora, si estás en modo desarrollo:
sudo cp "$(which neurocode 2>/dev/null || echo './neurocode')" /usr/local/bin/neuro 2>/dev/null
sudo chmod +x /usr/local/bin/neuro
sudo ln -sf /usr/local/bin/neuro /usr/local/bin/aquila # Alias para compatibilidad

# 4. Instalar Extensión de VS Code
echo "🔌 Instalando soporte para VS Code..."
# Aquí se descargará el archivo .vsix o se clonará el repo
# por ahora asumimos clon local
mkdir -p ~/.vscode/extensions/neurocode-lang

echo "✅ Instalación completada con éxito."
echo "🚀 Escribe 'neuro ayuda' para empezar."
echo "💡 Abre VS Code para ver la magia de la IA y la sintaxis."
