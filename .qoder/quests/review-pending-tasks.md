# Review and Organize Pending Tasks

## Overview

This document provides a comprehensive review of incomplete tasks and pending features in the eprice project, organized by priority and implementation complexity. The analysis is based on code examination, TODO comments, disabled features, and architectural gaps identified in the codebase.

## Critical Pending Tasks (High Priority)

### 1. String Encoding Issues in Alerts UI
**Status**: Critical Issue - UI Components Disabled
**Location**: `src/alerts/mod.rs`, `src/app.rs`
**Impact**: Alert UI functionality completely unavailable

**Current State**:
```rust
// pub mod ui; // TODO: Fix string encoding issues
// pub use ui::AlertUI; // TODO: Fix string encoding issues
// alert_ui: AlertUI, // Alert UI component - TODO: Fix string encoding issues
```

**Required Actions**:
- Investigate and resolve string encoding conflicts in alerts UI
- Re-enable AlertUI module and integration
- Test alert UI with Unicode text (Japanese characters)
- Implement proper UTF-8 string handling

### 2. Database Layer Implementation
**Status**: Core Infrastructure Missing
**Location**: `Cargo.toml`, `src/database/`
**Impact**: No persistent data storage

**Current State**:
```toml
# sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
# tokio = { version = "1.0", features = ["full"] }
```

**Required Actions**:
- Enable SQLx and Tokio dependencies
- Implement database connection management
- Complete migration system implementation
- Integrate repository pattern with actual SQLite operations
- Test database operations with async/await patterns

### 3. Authentication & Password Security
**Status**: Security-Critical Stub Implementation
**Location**: `src/utils/crypto.rs`, `Cargo.toml`
**Impact**: Insecure password handling

**Current State**:
```rust
/// Hash a password using bcrypt (stub implementation)
pub fn hash_password(password: &str) -> String {
    // Stub implementation - would use bcrypt in real app
    format!("hashed_{}", password)
}
```

**Required Actions**:
- Enable bcrypt dependency in Cargo.toml
- Implement proper password hashing with salt
- Replace stub authentication with secure methods
- Add password validation and complexity requirements

## Major Feature Implementation (Medium Priority)

### 4. OCR Receipt Processing System
**Status**: Stubbed Implementation
**Location**: `src/ocr/`, `Cargo.toml`
**Impact**: Key feature unavailable for automatic price updates

**Current State**:
```toml
# leptess = "0.14"
# image = { version = "0.25", features = ["jpeg", "png"] }
```

**Required Actions**:
- Enable leptess and image processing dependencies
- Implement OCR text extraction pipeline
- Build receipt parsing and item recognition
- Create product matching algorithms
- Add manual correction interface for OCR results

### 5. Camera Integration for Barcode Scanning
**Status**: Dependency Disabled
**Location**: `src/scanner/`, `Cargo.toml`
**Impact**: Barcode scanning requires manual image upload

**Current State**:
```toml
# nokhwa = "0.10"  # For camera access (barcode scanning)
```

**Required Actions**:
- Enable nokhwa camera dependency
- Implement camera access and image capture
- Integrate with barcode decoding system
- Handle platform-specific camera permissions
- Add camera preview UI component

### 6. Price Trend Visualization
**Status**: UI Placeholder Only
**Location**: `src/app.rs` (line 848)
**Impact**: Users cannot visualize price histories

**Current State**:
```rust
Tab::Trends => {
    ui.heading("价格趋势分析");
    ui.label("商品价格历史走势");
    // TODO: 添加价格趋势图表
}
```

**Required Actions**:
- Implement interactive price trend charts
- Add time period selection (7 days, 30 days, etc.)
- Create trend analysis algorithms
- Build historical data visualization components
- Add price prediction indicators

## User Interface Enhancements (Medium Priority)

### 7. Settings and Configuration System
**Status**: UI Placeholder Only
**Location**: `src/app.rs` (line 856)
**Impact**: No user customization options

**Current State**:
```rust
Tab::Settings => {
    ui.heading("设置");
    ui.label("在这里可以设置应用的配置");
    // TODO: 添加设置功能
}
```

**Required Actions**:
- Design settings data structure
- Implement user preferences system
- Add theme and language selection
- Create notification preferences
- Build data export/import functionality

### 8. Enhanced Camera UI Components
**Status**: Placeholder Implementation
**Location**: `src/scanner/ui.rs`
**Impact**: Poor user experience for scanning

**Current State**:
```rust
// Camera preview (placeholder)
ui.label("Camera preview would appear here");
```

**Required Actions**:
- Implement real-time camera preview
- Add capture controls and flash toggle
- Create image quality indicators
- Build zoom and focus controls
- Add scanning guidance overlays

## Data Management Tasks (Lower Priority)

### 9. Price Verification Status System
**Status**: Database Structure Ready, Logic Incomplete
**Location**: `src/models.rs`, `src/services/price_service.rs`
**Impact**: No community moderation of price data

**Current Implementation**:
```rust
pub verification_status: String, // 验证状态：pending, verified, rejected
```

**Required Actions**:
- Complete verification workflow implementation
- Add user voting and reporting systems
- Implement moderator tools
- Create reputation-based auto-verification
- Build verification status UI indicators

### 10. Advanced Search and Filtering
**Status**: Basic Implementation Only
**Location**: Various service files
**Impact**: Limited product discovery

**Required Actions**:
- Implement fuzzy search algorithms
- Add advanced filtering by price ranges
- Create category-based navigation
- Build tag-based search system
- Add search result ranking and relevance

## Architecture and Performance Tasks

### 11. Async/Await Pattern Implementation
**Status**: Mixed Implementation
**Location**: Service layer files
**Impact**: Potential UI blocking operations

**Required Actions**:
- Refactor service calls to use async/await
- Implement proper error handling for async operations
- Add loading states for long-running operations
- Optimize database query performance
- Test concurrent operation handling

### 12. Comprehensive Error Handling
**Status**: Basic ServiceError enum exists
**Location**: `src/services/mod.rs`
**Impact**: Poor error user experience

**Required Actions**:
- Standardize error messages across modules
- Add user-friendly error displays in UI
- Implement error logging and monitoring
- Create error recovery mechanisms
- Add validation error details

## Testing and Quality Assurance

### 13. Test Coverage Expansion
**Status**: Basic Integration Tests Only
**Location**: `tests/integration_tests.rs`
**Impact**: Low confidence in system reliability

**Required Actions**:
- Achieve 80% code coverage target
- Add unit tests for all service methods
- Create UI component tests
- Implement end-to-end testing scenarios
- Add performance benchmarking tests

### 14. Cross-Platform Compatibility
**Status**: WASM Build Configured, Testing Incomplete
**Location**: Build configuration
**Impact**: Inconsistent behavior across platforms

**Required Actions**:
- Test all features on Windows, macOS, Linux
- Verify WASM build functionality
- Optimize performance for web deployment
- Test camera access on different browsers
- Validate file system operations

## Documentation and Deployment

### 15. API Documentation
**Status**: Basic rustdoc comments
**Location**: Throughout codebase
**Impact**: Difficult for new developers to contribute

**Required Actions**:
- Complete rustdoc documentation for public APIs
- Add usage examples to service methods
- Create developer getting started guide
- Document database schema
- Build deployment documentation

### 16. Production Deployment Pipeline
**Status**: Development Setup Only
**Location**: Build scripts and configuration
**Impact**: No production release capability

**Required Actions**:
- Create CI/CD pipeline configuration
- Set up automated testing and builds
- Configure release packaging
- Implement version management
- Add monitoring and logging for production

## Priority Implementation Roadmap

### Phase 1: Critical Infrastructure (Weeks 1-2)
1. Fix string encoding issues in alerts UI
2. Enable and implement database layer
3. Implement secure password hashing
4. Add comprehensive error handling

### Phase 2: Core Features (Weeks 3-6)
1. Enable OCR receipt processing
2. Implement camera integration
3. Build price trend visualization
4. Complete verification status system

### Phase 3: User Experience (Weeks 7-9)
1. Implement settings system
2. Enhance camera UI components
3. Add advanced search capabilities
4. Improve async operation handling

### Phase 4: Quality and Deployment (Weeks 10-12)
1. Expand test coverage to 80%
2. Complete cross-platform testing
3. Finalize API documentation
4. Set up production deployment pipeline

## Risk Assessment

### High Risk Items
- **String encoding issues**: Blocks critical alert functionality
- **Database implementation**: Core data persistence missing
- **Security vulnerabilities**: Weak password handling

### Medium Risk Items
- **OCR accuracy**: Dependent on external library performance
- **Camera permissions**: Platform-specific implementation challenges
- **Performance**: UI responsiveness with large datasets

### Low Risk Items
- **Documentation gaps**: Does not affect functionality
- **Advanced features**: Can be implemented iteratively
- **Testing coverage**: Important but not blocking

## Resource Requirements

### Development Skills Needed
- Rust async/await patterns
- egui UI framework expertise
- Database design and SQLite
- OCR and image processing
- Cross-platform deployment

### External Dependencies
- OCR engine (leptess)
- Camera access (nokhwa)
- Database layer (sqlx)
- Authentication (bcrypt)
- Testing frameworks

### Timeline Estimates
- **Critical fixes**: 2-3 weeks
- **Major features**: 6-8 weeks
- **Complete implementation**: 10-12 weeks
- **Production ready**: 14-16 weeks

## Success Metrics

### Functional Completeness
- All TODO comments resolved
- Core features fully implemented
- Security vulnerabilities addressed
- Cross-platform compatibility verified

### Quality Targets
- 80% test coverage achieved
- Zero critical security issues
- Sub-100ms UI response times
- Successful deployment pipeline

### User Experience Goals
- Intuitive camera scanning workflow
- Reliable OCR accuracy (>85%)
- Fast price comparison searches
- Comprehensive alert system