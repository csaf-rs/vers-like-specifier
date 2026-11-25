#!/usr/bin/env bash
# Builds the WASM package and serves the demo at http://localhost:${PORT:-8080}
set -euo pipefail

cd "$(dirname "$0")/.."

wasm-pack build --target web --out-dir demo/pkg --no-typescript -- --features wasm

cd demo
echo "Serving at http://localhost:${PORT:-8080} (Ctrl+C to stop)"
exec python3 -m http.server "${PORT:-8080}"
