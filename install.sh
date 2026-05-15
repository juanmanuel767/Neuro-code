#!/bin/bash
# ==============================================================================
# NeuroCode Installer
# ==============================================================================
# Este script instala el ecosistema de NeuroCode v2.1 de forma global en tu sistema.
# Realiza lo siguiente:
# 1. Verifica las dependencias (Rust, Cargo, Python).
# 2. Compila el motor interno de NeuroCode para el máximo rendimiento (--release).
# 3. Mueve la librería estándar a ~/.neurocode/lib
# 4. Instala el binario en /usr/local/bin/neuro

set -e

# Colores para UI
VERDE='\033[0;32m'
AZUL='\033[0;34m'
ROJO='\033[0;31m'
NC='\033[0m' # Sin color

echo -e "${AZUL}==============================================${NC}"
echo -e "${VERDE}               NEUROCODE v2.1                 ${NC}"
echo -e "${AZUL}==============================================${NC}"
echo ""

# Paso 1: Verificar Dependencias
echo -e "🔍 Verificando dependencias..."
if ! command -v cargo &> /dev/null; then
    echo -e "${ROJO}Error: Rust y Cargo no están instalados.${NC}"
    echo "NeuroCode necesita Rust para compilarse."
    echo "Instálalo usando: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
if ! command -v python3 &> /dev/null; then
    echo -e "${ROJO}Error: Python3 no está instalado.${NC}"
    echo "NeuroCode necesita Python nativo para su FFI rápido."
    exit 1
fi
echo -e "✅ Dependencias OK."

# Paso 2: Compilación de Motor (Engine)
echo -e "\n🔨 Compilando motor nativo NeuroCode (esto puede tardar unos minutos)..."
cargo build --release

# Paso 3: Instalación de Librerías Estándar
echo -e "\n📂 Configurando entorno global..."
LIB_DIR="$HOME/.neurocode/lib"
mkdir -p "$LIB_DIR"

if [ -d "lib/neurocode" ]; then
    cp -r lib/neurocode/* "$LIB_DIR/"
    echo -e "✅ Standard Library instalada en $LIB_DIR"
else
    echo -e "${ROJO}Advertencia: Directorio de stdlib 'lib/neurocode' no encontrado.${NC}"
fi

# Paso 4: Mover Binario (requiere sudo si es en /usr/local/bin)
echo -e "\n🚀 Instalando ejecutable global 'neuro' en /usr/local/bin ..."
if [ -w /usr/local/bin ]; then
    cp target/release/neurocode /usr/local/bin/neuro
else
    echo -e "Se necesitan permisos de administrador para instalar en /usr/local/bin."
    sudo cp target/release/neurocode /usr/local/bin/neuro
fi

# Hacer un alias seguro de compatibilidad (opcional, por conveniencia)
if [ -w /usr/local/bin ]; then
    # Para el alias compatibile 'aquila'
    ln -sf /usr/local/bin/neuro /usr/local/bin/aquila
else
    sudo ln -sf /usr/local/bin/neuro /usr/local/bin/aquila
fi

# Paso 5: Instrucciones Finales
echo -e "\n${VERDE}==============================================${NC}"
echo -e "🎉 NeuroCode ha sido instalado con éxito!"
echo -e "${VERDE}==============================================${NC}"
echo -e "Comienza tu viaje con:"
echo -e "  ${AZUL}neuro ayuda${NC}         (Manual de comandos)"
echo -e "  ${AZUL}neuro ia${NC}            (Asistente de Inteligencia Artificial)"
echo -e "  ${AZUL}neuro nuevo mi_app${NC}  (Crear tu primer proyecto)"
echo ""
echo -e "Tu entorno está completamente adaptado. Todo listo."
