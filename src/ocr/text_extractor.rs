use crate::ocr::image_processor::ProcessedImage;
use anyhow::Result;
use std::collections::HashMap;

/// Text extractor for OCR functionality
pub struct TextExtractor {
    /// OCR engine configuration
    pub language: String,
    /// Confidence threshold for text recognition
    pub confidence_threshold: f32,
    /// Whether to preserve line breaks
    pub preserve_layout: bool,
}

impl Default for TextExtractor {
    fn default() -> Self {
        Self {
            language: "eng".to_string(), // English by default
            confidence_threshold: 0.6,
            preserve_layout: true,
        }
    }
}

impl TextExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new text extractor with custom configuration
    pub fn with_config(language: String, confidence_threshold: f32, preserve_layout: bool) -> Self {
        Self {
            language,
            confidence_threshold,
            preserve_layout,
        }
    }

    /// Extract text from processed image
    pub fn extract_text(&self, processed_image: &ProcessedImage) -> Result<TextExtractionResult> {
        // In a real implementation, this would:
        // 1. Initialize leptess with the specified language
        // 2. Set confidence threshold
        // 3. Process the image data
        // 4. Extract text with confidence scores
        // 5. Optionally preserve layout information

        log::info!(
            "Extracting text from image with language: {}",
            self.language
        );

        // Mock implementation - simulate realistic OCR output
        let mock_text = self.generate_mock_text(&processed_image.original_path);
        let confidence = processed_image.confidence * 0.9; // Slightly reduce confidence

        Ok(TextExtractionResult {
            text: mock_text.clone(),
            confidence: confidence,
            language_detected: self.language.clone(),
            word_confidences: self.generate_mock_word_confidences(&mock_text),
            line_count: mock_text.lines().count(),
            processing_time_ms: 150, // Mock processing time
            layout_preserved: self.preserve_layout,
        })
    }

    /// Extract text from raw image data
    pub fn extract_text_from_data(
        &self,
        image_data: &[u8],
        format: &str,
    ) -> Result<TextExtractionResult> {
        log::info!(
            "Extracting text from {} image data ({} bytes)",
            format,
            image_data.len()
        );

        // Mock implementation
        let mock_text =
            "Sample receipt text\nProduct: Cola - ¥3.50\nProduct: Chips - ¥2.00\nTotal: ¥5.50"
                .to_string();

        Ok(TextExtractionResult {
            text: mock_text.clone(),
            confidence: 0.82,
            language_detected: self.language.clone(),
            word_confidences: self.generate_mock_word_confidences(&mock_text),
            line_count: mock_text.lines().count(),
            processing_time_ms: 200,
            layout_preserved: self.preserve_layout,
        })
    }

    /// Set the OCR language
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    /// Set confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Check if the specified language is supported
    pub fn is_language_supported(&self, language: &str) -> bool {
        // In a real implementation, this would check available language data
        matches!(language, "eng" | "jpn" | "chi_sim" | "chi_tra" | "kor")
    }

    /// Get list of supported languages
    pub fn get_supported_languages(&self) -> Vec<String> {
        vec![
            "eng".to_string(),     // English
            "jpn".to_string(),     // Japanese
            "chi_sim".to_string(), // Chinese Simplified
            "chi_tra".to_string(), // Chinese Traditional
            "kor".to_string(),     // Korean
        ]
    }

    /// Generate mock text based on file path (for testing)
    fn generate_mock_text(&self, file_path: &str) -> String {
        if file_path.contains("receipt") || file_path.contains("bill") {
            // Simulate receipt text
            "FamilyMart\n東京駅店\n\nコカコーラ 330ml    ¥150\nポテトチップス      ¥120\nおにぎり ツナマヨ   ¥110\n\n小計            ¥380\n消費税           ¥38\n合計            ¥418\n\n現金            ¥500\nお釣り           ¥82\n\n2024/09/08 14:30\nありがとうございました".to_string()
        } else {
            // Simulate general text
            "Sample OCR text extracted from image\nLine 2 of extracted content\nPrice: ¥100"
                .to_string()
        }
    }

    /// Generate mock word confidences for testing
    fn generate_mock_word_confidences(&self, text: &str) -> HashMap<String, f32> {
        let mut confidences = HashMap::new();

        for word in text.split_whitespace() {
            let confidence = if word.chars().any(|c| c.is_ascii_digit()) {
                0.95 // Numbers typically have higher confidence
            } else if word.len() > 6 {
                0.85 // Longer words may have lower confidence
            } else {
                0.90 // Default confidence for regular words
            };

            confidences.insert(word.to_string(), confidence);
        }

        confidences
    }
}

/// Result of text extraction operation
#[derive(Debug, Clone)]
pub struct TextExtractionResult {
    pub text: String,
    pub confidence: f32,
    pub language_detected: String,
    pub word_confidences: HashMap<String, f32>,
    pub line_count: usize,
    pub processing_time_ms: u64,
    pub layout_preserved: bool,
}

impl TextExtractionResult {
    /// Check if the extraction result meets the confidence threshold
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }

    /// Get the average word confidence
    pub fn average_word_confidence(&self) -> f32 {
        if self.word_confidences.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.word_confidences.values().sum();
        sum / self.word_confidences.len() as f32
    }

    /// Get words with low confidence (below threshold)
    pub fn get_low_confidence_words(&self, threshold: f32) -> Vec<String> {
        self.word_confidences
            .iter()
            .filter(|entry| *entry.1 < threshold)
            .map(|(word, _)| word.clone())
            .collect()
    }

    /// Get text statistics
    pub fn get_statistics(&self) -> TextStatistics {
        let word_count = self.text.split_whitespace().count();
        let character_count = self.text.len();
        let non_whitespace_count = self.text.chars().filter(|c| !c.is_whitespace()).count();

        TextStatistics {
            character_count,
            word_count,
            line_count: self.line_count,
            non_whitespace_count,
            average_word_length: if word_count > 0 {
                non_whitespace_count as f32 / word_count as f32
            } else {
                0.0
            },
        }
    }
}

/// Statistics about extracted text
#[derive(Debug, Clone)]
pub struct TextStatistics {
    pub character_count: usize,
    pub word_count: usize,
    pub line_count: usize,
    pub non_whitespace_count: usize,
    pub average_word_length: f32,
}
