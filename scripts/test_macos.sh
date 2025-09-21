#!/bin/bash

# Cross-platform testing script for macOS
echo "Starting macOS cross-platform testing..."

# Test native macOS build
echo "Building for macOS target..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ macOS build successful"
else
    echo "❌ macOS build failed"
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test

if [ $? -eq 0 ]; then
    echo "✅ macOS tests passed"
else
    echo "❌ macOS tests failed"
    exit 1
fi

# Test iOS target (if available)
if rustup target list --installed | grep -q "aarch64-apple-ios"; then
    echo "Testing iOS target..."
    cargo build --target aarch64-apple-ios --release
    
    if [ $? -eq 0 ]; then
        echo "✅ iOS build successful"
    else
        echo "❌ iOS build failed"
        exit 1
    fi
fi

echo "macOS cross-platform testing completed successfully!"