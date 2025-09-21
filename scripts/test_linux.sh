#!/bin/bash

# Cross-platform testing script for Linux
echo "Starting Linux cross-platform testing..."

# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Test native Linux build
echo "Building for Linux target..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Linux build successful"
else
    echo "❌ Linux build failed"
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test

if [ $? -eq 0 ]; then
    echo "✅ Linux tests passed"
else
    echo "❌ Linux tests failed"
    exit 1
fi

# Test headless mode (for server environments)
echo "Testing headless mode..."
DISPLAY= cargo test

if [ $? -eq 0 ]; then
    echo "✅ Headless tests passed"
else
    echo "❌ Headless tests failed"
    exit 1
fi

echo "Linux cross-platform testing completed successfully!"