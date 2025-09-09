#[derive(Debug, Clone)]
pub struct ScanResult {
    pub barcode: String,
    pub barcode_type: BarcodeType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum BarcodeType {
    Ean13,
    Ean8,
    Code128,
    QrCode,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            fps: 30,
        }
    }
}
