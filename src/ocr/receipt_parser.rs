use crate::models::{Product, ReceiptItem};
use crate::ocr::text_extractor::TextExtractionResult;
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

/// Receipt parser for extracting structured data from OCR text
pub struct ReceiptParser {
    /// Store-specific parsing patterns
    pub store_patterns: HashMap<String, StorePattern>,
    /// Currency symbol patterns
    pub currency_patterns: Vec<String>,
    /// Date parsing patterns
    pub date_patterns: Vec<String>,
}

impl Default for ReceiptParser {
    fn default() -> Self {
        let mut parser = Self {
            store_patterns: HashMap::new(),
            currency_patterns: vec![
                "¥".to_string(), // Japanese Yen
                "$".to_string(), // US Dollar
                "€".to_string(), // Euro
                "£".to_string(), // British Pound
            ],
            date_patterns: vec![
                r"\d{4}/\d{1,2}/\d{1,2}".to_string(),
                r"\d{1,2}/\d{1,2}/\d{4}".to_string(),
                r"\d{4}-\d{1,2}-\d{1,2}".to_string(),
                r"\d{1,2}-\d{1,2}-\d{4}".to_string(),
            ],
        };

        // Initialize common store patterns
        parser.init_common_patterns();
        parser
    }
}

impl ReceiptParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse receipt text and extract structured data
    pub fn parse_receipt(
        &self,
        extraction_result: &TextExtractionResult,
    ) -> Result<ReceiptParseResult> {
        let text = &extraction_result.text;
        log::info!("Parsing receipt text: {} characters", text.len());

        // Extract store information
        let store_info = self.extract_store_info(text)?;

        // Extract items
        let items = self.extract_items(text)?;

        // Extract totals
        let totals = self.extract_totals(text)?;

        // Extract date and time
        let datetime = self.extract_datetime(text)?;

        Ok(ReceiptParseResult {
            store_info,
            items: items.clone(),
            totals: totals.clone(),
            datetime,
            raw_text: text.clone(),
            confidence: extraction_result.confidence,
            parsing_confidence: self.calculate_parsing_confidence(&items, &totals),
        })
    }

    /// Extract store information from receipt text
    fn extract_store_info(&self, text: &str) -> Result<StoreInfo> {
        let lines: Vec<&str> = text.lines().collect();

        // Try to identify store from known patterns
        for (store_name, pattern) in &self.store_patterns {
            if let Ok(regex) = Regex::new(&pattern.name_pattern) {
                if regex.is_match(text) {
                    return Ok(StoreInfo {
                        name: store_name.clone(),
                        branch: self.extract_branch_info(text, pattern)?,
                        address: self.extract_address_info(text)?,
                        phone: self.extract_phone_info(text)?,
                    });
                }
            }
        }

        // Fallback: use first non-empty line as store name
        let store_name = lines
            .first()
            .filter(|line| !line.trim().is_empty())
            .unwrap_or(&"Unknown Store")
            .trim()
            .to_string();

        Ok(StoreInfo {
            name: store_name,
            branch: None,
            address: None,
            phone: None,
        })
    }

    /// Extract items from receipt text
    fn extract_items(&self, text: &str) -> Result<Vec<ReceiptItem>> {
        let mut items = Vec::new();

        // Pattern for item lines: [name] [price]
        let item_pattern = Regex::new(r"(.+?)\s+[¥$€£]?([0-9,]+\.?[0-9]*)$")?;

        for line in text.lines() {
            let line = line.trim();

            // Skip header lines, empty lines, and total lines
            if self.is_header_line(line) || line.is_empty() || self.is_total_line(line) {
                continue;
            }

            if let Some(captures) = item_pattern.captures(line) {
                if let (Some(name_match), Some(price_match)) = (captures.get(1), captures.get(2)) {
                    let name = name_match.as_str().trim().to_string();
                    let price_str = price_match.as_str().replace(',', "");

                    if let Ok(price) = price_str.parse::<f64>() {
                        let item = ReceiptItem {
                            name: name.clone(),
                            price,
                            quantity: 1, // Default quantity
                            category: self.classify_item_category(&name),
                        };
                        items.push(item);
                    }
                }
            }
        }

        log::info!("Extracted {} items from receipt", items.len());
        Ok(items)
    }

    /// Extract total amounts from receipt text
    fn extract_totals(&self, text: &str) -> Result<ReceiptTotals> {
        let mut subtotal = None;
        let mut tax = None;
        let mut total = None;

        // Patterns for different total types
        let subtotal_pattern =
            Regex::new(r"(?i)(小計|小计|subtotal|sub total)\s*[¥$€£]?([0-9,]+\.?[0-9]*)")?;
        let tax_pattern =
            Regex::new(r"(?i)(税|消費税|消费税|tax|vat)\s*[¥$€£]?([0-9,]+\.?[0-9]*)")?;
        let total_pattern =
            Regex::new(r"(?i)(合計|合计|total|grand total|final)\s*[¥$€£]?([0-9,]+\.?[0-9]*)")?;

        for line in text.lines() {
            if let Some(captures) = subtotal_pattern.captures(line) {
                if let Some(amount) = captures.get(2) {
                    subtotal = amount.as_str().replace(',', "").parse().ok();
                }
            }

            if let Some(captures) = tax_pattern.captures(line) {
                if let Some(amount) = captures.get(2) {
                    tax = amount.as_str().replace(',', "").parse().ok();
                }
            }

            if let Some(captures) = total_pattern.captures(line) {
                if let Some(amount) = captures.get(2) {
                    total = amount.as_str().replace(',', "").parse().ok();
                }
            }
        }

        Ok(ReceiptTotals {
            subtotal,
            tax,
            total,
            discount: None, // Could be extracted similarly
        })
    }

    /// Extract date and time from receipt text
    fn extract_datetime(&self, text: &str) -> Result<Option<DateTime<Utc>>> {
        for pattern in &self.date_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(date_match) = regex.find(text) {
                    // Try to parse the date (simplified)
                    // In a real implementation, this would handle various date formats
                    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(
                        &format!("{} 00:00:00", date_match.as_str()),
                        "%Y/%m/%d %H:%M:%S",
                    ) {
                        return Ok(Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc)));
                    }
                }
            }
        }

        // Fallback to current time if no date found
        Ok(Some(Utc::now()))
    }

    /// Match receipt items with known products
    pub fn match_products(
        &self,
        items: &[ReceiptItem],
        products: &[Product],
    ) -> Result<Vec<ProductMatch>> {
        let mut matches = Vec::new();

        for item in items {
            let best_match = self.find_best_product_match(item, products);
            matches.push(best_match);
        }

        Ok(matches)
    }

    /// Find the best matching product for a receipt item
    fn find_best_product_match(&self, item: &ReceiptItem, products: &[Product]) -> ProductMatch {
        let mut best_score = 0.0;
        let mut best_product = None;

        for product in products {
            let score = self.calculate_match_score(item, product);
            if score > best_score {
                best_score = score;
                best_product = Some(product.clone());
            }
        }

        ProductMatch {
            receipt_item: item.clone(),
            matched_product: best_product,
            confidence: best_score,
            match_type: if best_score > 0.8 {
                MatchType::Exact
            } else if best_score > 0.5 {
                MatchType::Partial
            } else {
                MatchType::None
            },
        }
    }

    /// Calculate similarity score between receipt item and product
    fn calculate_match_score(&self, item: &ReceiptItem, product: &Product) -> f32 {
        // Simple similarity calculation based on name matching
        let item_name = item.name.to_lowercase();
        let product_name = product.name.to_lowercase();

        // Exact match
        if item_name == product_name {
            return 1.0;
        }

        // Substring match
        if item_name.contains(&product_name) || product_name.contains(&item_name) {
            return 0.8;
        }

        // Word-based similarity
        let item_words: Vec<&str> = item_name.split_whitespace().collect();
        let product_words: Vec<&str> = product_name.split_whitespace().collect();

        let common_words = item_words
            .iter()
            .filter(|word| product_words.contains(word))
            .count();

        let total_words = (item_words.len() + product_words.len()) as f32;
        let similarity = (common_words as f32 * 2.0) / total_words;

        similarity.clamp(0.0, 1.0)
    }

    // Helper methods

    fn is_header_line(&self, line: &str) -> bool {
        let line_lower = line.to_lowercase();
        line_lower.contains("store")
            || line_lower.contains("shop")
            || line_lower.contains("mart")
            || line_lower.contains("receipt")
    }

    fn is_total_line(&self, line: &str) -> bool {
        let line_lower = line.to_lowercase();
        line_lower.contains("total")
            || line_lower.contains("合計")
            || line_lower.contains("小計")
            || line_lower.contains("税")
    }

    fn classify_item_category(&self, name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();

        if name_lower.contains("cola")
            || name_lower.contains("コーラ")
            || name_lower.contains("可乐")
        {
            Some("飲料".to_string())
        } else if name_lower.contains("chip") || name_lower.contains("ポテト") {
            Some("スナック".to_string())
        } else if name_lower.contains("水") || name_lower.contains("water") {
            Some("飲料".to_string())
        } else {
            Some("その他".to_string())
        }
    }

    fn extract_branch_info(&self, _text: &str, _pattern: &StorePattern) -> Result<Option<String>> {
        // Mock implementation
        Ok(Some("Main Branch".to_string()))
    }

    fn extract_address_info(&self, _text: &str) -> Result<Option<String>> {
        // Mock implementation
        Ok(None)
    }

    fn extract_phone_info(&self, _text: &str) -> Result<Option<String>> {
        // Mock implementation
        Ok(None)
    }

    fn calculate_parsing_confidence(&self, items: &[ReceiptItem], totals: &ReceiptTotals) -> f32 {
        let mut confidence: f32 = 0.8; // Base confidence

        // Increase confidence if we found items
        if !items.is_empty() {
            confidence += 0.1;
        }

        // Increase confidence if we found totals
        if totals.total.is_some() {
            confidence += 0.1;
        }

        confidence.clamp(0.0, 1.0)
    }

    fn init_common_patterns(&mut self) {
        // FamilyMart pattern
        self.store_patterns.insert(
            "FamilyMart".to_string(),
            StorePattern {
                name_pattern: r"(?i)(family\s*mart|ファミリーマート|ファミマ)".to_string(),
                item_pattern: r"(.+?)\s+¥([0-9,]+)".to_string(),
            },
        );

        // 7-Eleven pattern
        self.store_patterns.insert(
            "7-Eleven".to_string(),
            StorePattern {
                name_pattern: r"(?i)(seven\s*eleven|セブンイレブン|セブン)".to_string(),
                item_pattern: r"(.+?)\s+¥([0-9,]+)".to_string(),
            },
        );
    }
}

// Supporting structures

#[derive(Debug, Clone)]
pub struct StorePattern {
    pub name_pattern: String,
    pub item_pattern: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptParseResult {
    pub store_info: StoreInfo,
    pub items: Vec<ReceiptItem>,
    pub totals: ReceiptTotals,
    pub datetime: Option<DateTime<Utc>>,
    pub raw_text: String,
    pub confidence: f32,
    pub parsing_confidence: f32,
}

#[derive(Debug, Clone)]
pub struct StoreInfo {
    pub name: String,
    pub branch: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReceiptTotals {
    pub subtotal: Option<f64>,
    pub tax: Option<f64>,
    pub total: Option<f64>,
    pub discount: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ProductMatch {
    pub receipt_item: ReceiptItem,
    pub matched_product: Option<Product>,
    pub confidence: f32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Partial,
    None,
}
