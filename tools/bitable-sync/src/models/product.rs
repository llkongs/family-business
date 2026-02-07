use serde::{Deserialize, Serialize};

/// Matches the TypeScript `Brand` interface in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub founded_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// Matches the TypeScript `Category` interface in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub level: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Matches the TypeScript `Supplier` interface in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplier {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Matches the TypeScript `Product` interface in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub sku: String,
    pub barcode: String,
    pub name: String,
    pub brand: Brand,
    pub category: Category,
    pub specification: String,
    pub unit: String,
    pub pack_size: i32,
    pub weight: i32,
    pub retail_price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promotion_price: Option<f64>,
    pub stock: i32,
    pub safety_stock: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warehouse_location: Option<String>,
    pub origin: String,
    pub shelf_life: i32,
    pub storage_condition: String,
    pub alcohol_content: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vintage: Option<i32>,
    pub brewing_process: String,
    pub flavor_profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serving_suggestion: Option<String>,
    pub main_image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<String>,
    pub short_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier: Option<Supplier>,
    pub status: String,
    pub is_hot: bool,
    pub is_new: bool,
    pub is_promotion: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Matches the TypeScript `ProductDatabase` interface in types.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDatabase {
    pub version: String,
    pub last_updated: String,
    pub brands: Vec<Brand>,
    pub categories: Vec<Category>,
    pub suppliers: Vec<Supplier>,
    pub products: Vec<Product>,
}
