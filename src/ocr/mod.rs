pub mod image_processor;
pub mod models;
pub mod receipt_parser;
pub mod text_extractor;

pub use image_processor::ImageProcessor;
pub use models::{OcrConfig, ReceiptItem};
pub use receipt_parser::ReceiptParser;
pub use text_extractor::TextExtractor;
// Re-export OcrResult from models with a different name to avoid conflict
pub use models::OcrResult as OcrData;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Image processing failed: {0}")]
    ImageProcessing(String),
    #[error("Text extraction failed: {0}")]
    TextExtraction(String),
    #[error("Receipt parsing failed: {0}")]
    ReceiptParsing(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub type OcrResult<T> = Result<T, OcrError>;
