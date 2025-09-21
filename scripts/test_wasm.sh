#!/bin/bash

# Cross-platform testing script for WASM target
echo "Starting WASM cross-platform testing..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Install WASM target if not present
rustup target add wasm32-unknown-unknown

echo "Building for WASM target..."
cargo build --target wasm32-unknown-unknown --release

if [ $? -eq 0 ]; then
    echo "✅ WASM build successful"
else
    echo "❌ WASM build failed"
    exit 1
fi

# Test with wasm-pack
echo "Running wasm-pack tests..."
wasm-pack test --headless --chrome

if [ $? -eq 0 ]; then
    echo "✅ WASM tests passed"
else
    echo "❌ WASM tests failed"
    exit 1
fi

echo "WASM cross-platform testing completed successfully!"