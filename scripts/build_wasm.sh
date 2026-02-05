#!/bin/bash
set -e

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ error: wasm-pack is not installed."
    echo "👉 Please install it: cargo install wasm-pack"
    echo "ℹ️  Alternative: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

echo "🚀 Building HyperspaceDB WASM module..."
cd crates/hyperspace-wasm

# Build for web target
wasm-pack build --target web --out-dir ../../examples/wasm-demo/pkg

echo "✅ WASM Build Complete!"
echo "📂 Output: examples/wasm-demo/pkg"
