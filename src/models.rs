use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 商品结构体，包含商品的基本信息和价格记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,               // 商品ID
    pub name: String,             // 商品名称
    pub category: String,         // 商品类别
    pub description: String,      // 商品描述
    pub images: Vec<String>,      // 商品图片列表
    pub prices: Vec<PriceRecord>, // 商品价格记录
    pub tags: Vec<String>,        // 商品标签
}

/// 价格记录结构体，包含价格信息和时间戳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRecord {
    pub store_id: String, // 门店ID
    pub price: f64,       // 商品价格
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>, // 价格记录的时间戳
    pub is_on_sale: bool, // 是否在促销
}

impl Product {
    /// 获取当前最低价格的价格记录
    pub fn current_lowest_price(&self) -> Option<&PriceRecord> {
        self.prices
            .iter()
            .filter(|p| p.timestamp.date_naive() == Utc::now().date_naive()) // 过滤出当天的价格记录
            .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap()) // 找出最低价格
    }
}

/// 门店结构体，包含门店的基本信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
}

impl Store {
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
