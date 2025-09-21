# Cross-platform testing script for Windows
Write-Host "Starting Windows cross-platform testing..."

# Test native Windows build
Write-Host "Building for Windows target..."
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Windows build successful" -ForegroundColor Green
} else {
    Write-Host "❌ Windows build failed" -ForegroundColor Red
    exit 1
}

# Run tests
Write-Host "Running tests..."
cargo test

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Windows tests passed" -ForegroundColor Green
} else {
    Write-Host "❌ Windows tests failed" -ForegroundColor Red
    exit 1
}

# Test with different feature flags
Write-Host "Testing with camera features disabled..."
cargo test --no-default-features --features="database,alerts,trends"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Feature-limited tests passed" -ForegroundColor Green
} else {
    Write-Host "❌ Feature-limited tests failed" -ForegroundColor Red
    exit 1
}

Write-Host "Windows cross-platform testing completed successfully!" -ForegroundColor Green