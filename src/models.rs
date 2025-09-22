use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// use sqlx::FromRow; // Disabled for now
use uuid::Uuid;

/// User model for authentication and user management
#[derive(Debug, Clone, Serialize, Deserialize, /* FromRow, */ PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_login: Option<DateTime<Utc>>,
    pub reputation_score: i32,
}

impl User {
    /// Create a new user with generated ID and current timestamp
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            created_at: Utc::now(),
            last_login: None,
            reputation_score: 0,
        }
    }

    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
}

/// User review model for store and product ratings
#[derive(Debug, Clone, Serialize, Deserialize /* , FromRow */)]
pub struct UserReview {
    pub id: String,
    pub user_id: String,
    pub store_id: Option<String>,
    pub product_id: Option<String>,
    pub rating: i32, // 1-5 stars
    pub comment: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl UserReview {
    /// Create a new review with generated ID and current timestamp
    pub fn new(
        user_id: String,
        store_id: Option<String>,
        product_id: Option<String>,
        rating: i32,
        comment: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            store_id,
            product_id,
            rating,
            comment,
            created_at: Utc::now(),
        }
    }
}

/// Price alert model for price monitoring
#[derive(Debug, Clone, Serialize, Deserialize /* , FromRow */)]
pub struct PriceAlert {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub target_price: f64,
    pub is_active: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl PriceAlert {
    /// Create a new price alert with generated ID and current timestamp
    pub fn new(user_id: String, product_id: String, target_price: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            product_id,
            target_price,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    /// Deactivate the price alert
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Check if the current price triggers this alert
    pub fn should_trigger(&self, current_price: f64) -> bool {
        self.is_active && current_price <= self.target_price
    }
}

/// OCR result model for receipt scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub id: String,
    pub image_path: String,
    pub extracted_text: String,
    pub parsed_items: Vec<ReceiptItem>,
    pub confidence_score: f32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl OcrResult {
    /// Create a new OCR result with generated ID and current timestamp
    pub fn new(
        image_path: String,
        extracted_text: String,
        parsed_items: Vec<ReceiptItem>,
        confidence_score: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            image_path,
            extracted_text,
            parsed_items,
            confidence_score,
            created_at: Utc::now(),
        }
    }
}

/// Receipt item model for individual items extracted from receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub price: f64,
    pub quantity: i32,
    pub category: Option<String>,
}

impl ReceiptItem {
    /// Create a new receipt item
    pub fn new(name: String, price: f64, quantity: i32, category: Option<String>) -> Self {
        Self {
            name,
            price,
            quantity,
            category,
        }
    }

    /// Calculate total price for this item
    pub fn total_price(&self) -> f64 {
        self.price * self.quantity as f64
    }
}

/// 商品结构体，包含商品的基本信息和价格记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq /* , FromRow */)]
pub struct Product {
    pub id: String,              // 商品ID
    pub name: String,            // 商品名称
    pub category: String,        // 商品类别
    pub description: String,     // 商品描述
    pub barcode: Option<String>, // 商品条码
    pub images: Vec<String>,     // 商品图片列表
    // #[sqlx(skip)] // This field is handled separately in database operations
    pub prices: Vec<PriceRecord>, // 商品价格记录
    pub tags: Vec<String>,        // 商品标签
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>, // 创建时间
}

impl Product {
    /// Create a new product with generated ID and current timestamp
    pub fn new(
        name: String,
        category: String,
        description: String,
        barcode: Option<String>,
        images: Vec<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            category,
            description,
            barcode,
            images,
            prices: Vec::new(),
            tags,
            created_at: Utc::now(),
        }
    }

    /// 获取当前最低价格的价格记录
    pub fn current_lowest_price(&self) -> Option<&PriceRecord> {
        self.prices
            .iter()
            .filter(|p| {
                p.verification_status == "verified"
                    && p.timestamp.date_naive() == Utc::now().date_naive() // 过滤出当天的价格记录
            })
            .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap()) // 找出最低价格
    }

    /// Get all verified price records for this product
    pub fn verified_prices(&self) -> Vec<&PriceRecord> {
        self.prices
            .iter()
            .filter(|p| p.verification_status == "verified")
            .collect()
    }

    /// Get average price for this product from verified records
    pub fn average_price(&self) -> Option<f64> {
        let verified_prices = self.verified_prices();
        if verified_prices.is_empty() {
            None
        } else {
            let sum: f64 = verified_prices.iter().map(|p| p.price).sum();
            Some(sum / verified_prices.len() as f64)
        }
    }
}

/// 价格记录结构体，包含价格信息和时间戳
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq /* , FromRow */)]
pub struct PriceRecord {
    pub id: Option<String>,         // 价格记录ID
    pub product_id: Option<String>, // 商品ID
    pub store_id: String,           // 门店ID
    pub user_id: Option<String>,    // 提交用户ID
    pub price: f64,                 // 商品价格
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>, // 价格记录的时间戳
    pub is_on_sale: bool,           // 是否在促销
    pub receipt_image: Option<String>, // 小票图片路径
    pub verification_status: String, // 验证状态：pending, verified, rejected
}

impl PriceRecord {
    /// Create a new price record with generated ID and current timestamp
    pub fn new(
        product_id: Option<String>,
        store_id: String,
        user_id: Option<String>,
        price: f64,
        is_on_sale: bool,
        receipt_image: Option<String>,
    ) -> Self {
        Self {
            id: Some(Uuid::new_v4().to_string()),
            product_id,
            store_id,
            user_id,
            price,
            timestamp: Utc::now(),
            is_on_sale,
            receipt_image,
            verification_status: "pending".to_string(),
        }
    }

    /// Mark the price record as verified
    pub fn verify(&mut self) {
        self.verification_status = "verified".to_string();
    }

    /// Mark the price record as rejected
    pub fn reject(&mut self) {
        self.verification_status = "rejected".to_string();
    }
}

/// 门店结构体，包含门店的基本信息
#[derive(Debug, Clone, Serialize, Deserialize /* , FromRow */, PartialEq)]
pub struct Store {
    pub id: String,            // 门店ID
    pub name: String,          // 门店名称
    pub address: String,       // 门店地址
    pub latitude: f64,         // 门店纬度
    pub longitude: f64,        // 门店经度
    pub rating: f64,           // 门店评分
    pub opening_hours: String, // 营业时间
    pub phone: String,         // 联系电话
    pub tags: Vec<String>,     // 门店标签
    pub symbol: char,          // 门店符号
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>, // 创建时间
}

impl Store {
    /// Create a new store with generated ID and current timestamp
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        address: String,
        latitude: f64,
        longitude: f64,
        opening_hours: String,
        phone: String,
        tags: Vec<String>,
        symbol: char,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            address,
            latitude,
            longitude,
            rating: 0.0,
            opening_hours,
            phone,
            tags,
            symbol,
            created_at: Utc::now(),
        }
    }

    /// 计算门店到指定位置的距离
    pub fn distance_to(&self, lat: f64, lon: f64) -> f64 {
        const EARTH_RADIUS: f64 = 6371.0; // 地球半径，单位：公里

        let lat1 = self.latitude.to_radians(); // 门店纬度转换为弧度
        let lat2 = lat.to_radians(); // 目标纬度转换为弧度
        let delta_lat = (lat - self.latitude).to_radians(); // 纬度差转换为弧度
        let delta_lon = (lon - self.longitude).to_radians(); // 经度差转换为弧度

        // 使用Haversine公式计算距离
        let a = (delta_lat / 2.0).sin() * (delta_lat / 2.0).sin()
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin() * (delta_lon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c // 返回计算出的距离
    }
}
