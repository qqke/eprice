# Cross-Platform Testing Configuration

This document outlines the cross-platform testing strategy for the eprice application.

## Supported Platforms

### Desktop Platforms
- **Windows** (x86_64-pc-windows-msvc)
- **macOS** (x86_64-apple-darwin, aarch64-apple-darwin)
- **Linux** (x86_64-unknown-linux-gnu)

### Web Platform
- **WASM** (wasm32-unknown-unknown)

### Mobile Platforms (Future)
- **iOS** (aarch64-apple-ios)
- **Android** (aarch64-linux-android)

## Testing Scripts

### Windows Testing
Run: `powershell -ExecutionPolicy Bypass -File scripts/test_windows.ps1`
- Tests native Windows build
- Verifies feature flags compatibility
- Checks Windows-specific dependencies

### Linux Testing
Run: `bash scripts/test_linux.sh`
- Tests native Linux build  
- Verifies headless operation
- Checks Linux system dependencies

### macOS Testing
Run: `bash scripts/test_macos.sh`
- Tests native macOS build
- Optional iOS target testing
- Verifies macOS-specific features

### WASM Testing
Run: `bash scripts/test_wasm.sh`
- Tests web assembly build
- Runs headless browser tests
- Verifies web compatibility

## Platform-Specific Considerations

### Windows
- Requires Visual Studio Build Tools
- Camera integration uses Windows Media Foundation
- File system paths use backslashes

### Linux
- Requires development packages (build-essential, pkg-config, libssl-dev)
- Camera integration uses V4L2
- Headless testing for server environments

### macOS
- Requires Xcode command line tools
- Camera integration uses AVFoundation
- Support for both Intel and Apple Silicon

### WASM
- Limited file system access
- No native camera access (uses web APIs)
- Reduced feature set for web compatibility

## Feature Matrix

| Feature | Windows | macOS | Linux | WASM |
|---------|---------|-------|-------|------|
| Core UI | ✅ | ✅ | ✅ | ✅ |
| Database | ✅ | ✅ | ✅ | ✅* |
| Camera | ✅ | ✅ | ✅ | ⚠️** |
| OCR | ✅ | ✅ | ✅ | ❌*** |
| File I/O | ✅ | ✅ | ✅ | ⚠️**** |

*  WASM database uses IndexedDB
** WASM camera uses web APIs with limitations
*** OCR requires native libraries not available in WASM
**** WASM file I/O limited to downloads/uploads

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: Cross-Platform Tests
on: [push, pull_request]
jobs:
  test-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: powershell -ExecutionPolicy Bypass -File scripts/test_windows.ps1
  
  test-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: bash scripts/test_linux.sh
  
  test-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: bash scripts/test_macos.sh
      
  test-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: bash scripts/test_wasm.sh
```

## Testing Checklist

- [ ] Windows native build compiles
- [ ] Windows tests pass
- [ ] macOS native build compiles
- [ ] macOS tests pass
- [ ] Linux native build compiles
- [ ] Linux tests pass
- [ ] Linux headless mode works
- [ ] WASM build compiles
- [ ] WASM tests pass in browser
- [ ] All feature combinations work
- [ ] Platform-specific features function correctly

## Known Issues

### WASM Limitations
- OCR functionality disabled (requires native libraries)
- Camera access limited to web APIs
- File system access restricted

### Mobile Platforms
- iOS and Android targets planned for future releases
- Cross-compilation setup required
- Platform-specific UI adaptations needed

## Dependencies by Platform

### Windows
- Visual Studio Build Tools or MSVC
- Windows SDK
- OpenSSL (via vcpkg or precompiled)

### Linux
- GCC or Clang
- pkg-config
- libssl-dev
- libudev-dev (for camera support)

### macOS
- Xcode Command Line Tools
- Homebrew (optional, for dependencies)

### WASM
- wasm-pack
- Node.js (for testing)
- Chrome/Firefox (for headless testing)