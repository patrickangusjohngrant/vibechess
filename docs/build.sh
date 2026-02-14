#!/bin/bash
set -e
cd "$(dirname "$0")/.."
wasm-pack build --target web --out-dir docs/pkg
echo "Build complete! Serve the web/ directory with any static file server."
echo "For example: cd docs && python3 -m http.server 8080"
