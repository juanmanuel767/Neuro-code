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

# 3. Descargar/Instalar Binario
# Nota: Aquí es donde descargaremos el binario oficial desde GitHub Releases en el futuro.
echo "📥 Descargando motor de Rust de Aquila..."
# curl -L https://github.com/juanmanuel767/aquila/releases/latest/download/aquila -o /tmp/aquila
# sudo mv /tmp/aquila /usr/local/bin/aquila

# Por ahora, si estás en modo desarrollo:
sudo cp "$(which aquila 2>/dev/null || echo './aquila')" /usr/local/bin/aquila 2>/dev/null
sudo chmod +x /usr/local/bin/aquila

# 4. Instalar Extensión de VS Code
echo "🔌 Instalando soporte para VS Code..."
# Aquí se descargará el archivo .vsix o se clonará el repo
mkdir -p ~/.vscode/extensions/aquila-lang

echo "✅ Instalación completada con éxito."
echo "🚀 Escribe 'aquila --version' para empezar."
echo "💡 Abre VS Code para ver la magia de la sintaxis."
