#!/bin/bash

# 🔵 Función para matar ambos procesos
function cleanup() {
    echo "🛑 Parando procesos..."
    if [[ -n "$FRONTEND_PID" ]]; then
        kill "$FRONTEND_PID" 2>/dev/null
        echo "🔻 Frontend parado."
    fi
    if [[ -n "$BACKEND_PID" ]]; then
        kill "$BACKEND_PID" 2>/dev/null
        echo "🔻 Backend parado."
    fi
    exit 0
}

# 🔵 Capturar Ctrl+C (SIGINT) para matar los procesos
trap cleanup SIGINT

echo "🟡 Lanzando frontend (React + Vite)..."
cd /home/inesvj/TFG/microlab/interfaz || exit 1
npm run dev &
FRONTEND_PID=$!
echo "🛠️  Frontend corriendo (PID=$FRONTEND_PID)"

echo "⏳ Esperando 5 segundos para que frontend levante..."
sleep 5

echo "🟢 Lanzando backend (QTestApp)..."
cd /home/inesvj/TFG/microlab/qtestapp || exit 1
cargo run &
BACKEND_PID=$!
echo "🛠️  Backend corriendo (PID=$BACKEND_PID)"

echo "✅ Todo lanzado. ¡Pulsa CTRL+C para parar ambos servidores!"
echo "Ready for debugging"

wait