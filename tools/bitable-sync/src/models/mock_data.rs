use serde::{Deserialize, Serialize};

/// Store information (matches mockData.ts StoreInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreInfo {
    pub name: String,
    pub phone: String,
    pub qr_code_url: String,
}

/// Media item for carousel (matches mockData.ts MediaItem)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub media_type: String, // "image" or "video"
    pub url: String,
    pub title: Option<String>,
    pub duration: Option<i64>,
    pub sort_order: i32,
}

/// Display category (matches mockData.ts Category)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayCategory {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub sort_order: i32,
}

/// Simplified product for mock data (matches mockData.ts Product)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockProduct {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub image: String,
    pub category_id: String,
}

/// All mock data combined
#[derive(Debug, Clone)]
pub struct MockData {
    pub store_info: StoreInfo,
    pub media_playlist: Vec<MediaItem>,
    pub categories: Vec<DisplayCategory>,
    pub products: Vec<MockProduct>,
}
