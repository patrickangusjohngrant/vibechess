#!/bin/bash
set -e
cd "$(dirname "$0")/.."
wasm-pack build --target web --out-dir web/pkg
echo "Build complete! Serve the web/ directory with any static file server."
echo "For example: cd web && python3 -m http.server 8080"
