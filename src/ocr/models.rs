// OCR models - reexport from main models module
pub use crate::models::{OcrResult, ReceiptItem};

#[derive(Debug, Clone)]
pub struct OcrConfig {
    pub language: String,
    pub confidence_threshold: f32,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            confidence_threshold: 0.5,
        }
    }
}
