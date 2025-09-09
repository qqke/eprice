pub mod barcode_decoder;
pub mod camera_manager;
pub mod models;
pub mod product_matcher;
pub mod ui;

pub use barcode_decoder::BarcodeDecoder;
pub use camera_manager::{CameraInfo, CameraManager};
pub use models::{BarcodeType, CameraConfig, ScanResult};
pub use product_matcher::{ProductMatch, ProductMatchType, ProductMatcher};
pub use ui::ScannerUI;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Camera access failed: {0}")]
    CameraAccess(String),
    #[error("Barcode detection failed: {0}")]
    BarcodeDetection(String),
    #[error("Product matching failed: {0}")]
    ProductMatching(String),
    #[error("Unsupported barcode type: {0}")]
    UnsupportedBarcodeType(String),
    #[error("No camera available")]
    NoCameraAvailable,
}

pub type ScannerResult<T> = Result<T, ScannerError>;

/// Main scanner service that integrates camera, barcode decoding, and product matching
pub struct ScannerService {
    camera_manager: CameraManager,
    barcode_decoder: BarcodeDecoder,
    product_matcher: ProductMatcher,
}

impl ScannerService {
    pub fn new() -> Self {
        Self {
            camera_manager: CameraManager::new(),
            barcode_decoder: BarcodeDecoder::new(),
            product_matcher: ProductMatcher::new(),
        }
    }

    /// Start the camera
    pub fn start_camera(&self) -> ScannerResult<()> {
        self.camera_manager.start_camera()
    }

    /// Stop the camera
    pub fn stop_camera(&self) -> ScannerResult<()> {
        self.camera_manager.stop_camera()
    }

    /// Scan for barcode and find matching product
    pub fn scan_and_match(&self) -> ScannerResult<Option<crate::models::Product>> {
        // Capture frame from camera
        let frame = self
            .camera_manager
            .capture_frame()
            .map_err(|e| ScannerError::CameraAccess(e.to_string()))?;

        // Decode barcode from frame
        let scan_result = self
            .barcode_decoder
            .decode(&frame)
            .map_err(|e| ScannerError::BarcodeDetection(e.to_string()))?;

        // Find matching product
        let product = self
            .product_matcher
            .find_product_by_scan(&scan_result)
            .map_err(|e| ScannerError::ProductMatching(e.to_string()))?;

        Ok(product)
    }

    /// Get camera status
    pub fn is_camera_running(&self) -> bool {
        self.camera_manager.is_running()
    }

    /// Get available cameras
    pub fn list_cameras(&self) -> Vec<CameraInfo> {
        CameraManager::list_cameras()
    }

    /// Access to individual components
    pub fn camera(&self) -> &CameraManager {
        &self.camera_manager
    }

    pub fn decoder(&self) -> &BarcodeDecoder {
        &self.barcode_decoder
    }

    pub fn matcher(&self) -> &ProductMatcher {
        &self.product_matcher
    }
}

impl Default for ScannerService {
    fn default() -> Self {
        Self::new()
    }
}
