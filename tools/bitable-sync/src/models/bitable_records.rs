use anyhow::{Context, Result};
use std::collections::HashMap;

/// Helper to extract a text field from bitable record fields.
/// Bitable text fields come as either a plain string or an array of objects with "text" key.
pub fn extract_text(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    let val = fields.get(key)?;

    // Plain string
    if let Some(s) = val.as_str() {
        return Some(s.to_string());
    }

    // Array of text segments: [{"text": "value", "type": "text"}]
    if let Some(arr) = val.as_array() {
        let parts: Vec<String> = arr
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .map(|s| s.to_string())
            .collect();
        if !parts.is_empty() {
            return Some(parts.join(""));
        }
    }

    // Number serialized as string
    if let Some(n) = val.as_f64() {
        return Some(n.to_string());
    }

    None
}

/// Extract a number field
pub fn extract_number(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<f64> {
    let val = fields.get(key)?;

    if let Some(n) = val.as_f64() {
        return Some(n);
    }

    // Sometimes numbers come as strings
    if let Some(s) = val.as_str() {
        return s.parse::<f64>().ok();
    }

    None
}

/// Extract a boolean/checkbox field
pub fn extract_bool(fields: &HashMap<String, serde_json::Value>, key: &str) -> bool {
    fields
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Extract a single-select field (returns the option text)
pub fn extract_select(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    let val = fields.get(key)?;

    // Single select can be a string directly or an object with "text" key
    if let Some(s) = val.as_str() {
        return Some(s.to_string());
    }

    // Or it might be an object like {"text": "value", "id": "..."}
    if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }

    None
}

/// Extract a linked record field (returns the display text of the first link)
pub fn extract_link_text(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    let val = fields.get(key)?;

    // Link fields come as an array of linked record objects
    if let Some(arr) = val.as_array() {
        if let Some(first) = arr.first() {
            // The linked record might have a "text" field or be a string
            if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                return Some(text.to_string());
            }
            if let Some(s) = first.as_str() {
                return Some(s.to_string());
            }
        }
    }

    None
}

/// Extract a linked record ID (the record_id of the first linked record)
#[allow(dead_code)]
pub fn extract_link_record_id(
    fields: &HashMap<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    let val = fields.get(key)?;

    if let Some(arr) = val.as_array() {
        if let Some(first) = arr.first() {
            if let Some(id) = first.get("record_id").and_then(|t| t.as_str()) {
                return Some(id.to_string());
            }
            if let Some(s) = first.as_str() {
                return Some(s.to_string());
            }
        }
    }

    None
}

/// Extract a phone field (type 13 returns plain string)
pub fn extract_phone(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    extract_text(fields, key)
}

/// Extract a URL/hyperlink field (type 15)
/// URL fields can come as {"text": "url", "link": "url"} or just a string
pub fn extract_url(fields: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    let val = fields.get(key)?;

    if let Some(s) = val.as_str() {
        return Some(s.to_string());
    }

    // URL field as object
    if let Some(link) = val.get("link").and_then(|l| l.as_str()) {
        return Some(link.to_string());
    }
    if let Some(text) = val.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }

    None
}

/// Extract attachment field (type 17) - returns the first file's URL
/// Attachment fields come as an array of file objects with "url", "name", "tmp_url" etc.
pub fn extract_attachment_url(
    fields: &HashMap<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    let val = fields.get(key)?;

    if let Some(arr) = val.as_array() {
        if let Some(first) = arr.first() {
            // Try tmp_url first (temporary download URL), then url
            if let Some(url) = first.get("tmp_url").and_then(|u| u.as_str()) {
                return Some(url.to_string());
            }
            if let Some(url) = first.get("url").and_then(|u| u.as_str()) {
                return Some(url.to_string());
            }
        }
    }

    None
}

// ============================================================
// Field name mapping: Chinese field names in bitable
// ============================================================

/// Parse a bitable record into a Brand
pub fn parse_brand(
    fields: &HashMap<String, serde_json::Value>,
) -> Result<super::product::Brand> {
    Ok(super::product::Brand {
        id: extract_text(fields, "品牌ID").context("Brand missing '品牌ID'")?,
        name: extract_text(fields, "品牌名称").context("Brand missing '品牌名称'")?,
        logo: extract_attachment_url(fields, "品牌Logo"),
        story: extract_text(fields, "品牌故事"),
        founded_year: extract_number(fields, "创立年份").map(|n| n as i32),
        origin: extract_text(fields, "产地"),
    })
}

/// Parse a bitable record into a DisplayCategory
pub fn parse_display_category(
    fields: &HashMap<String, serde_json::Value>,
) -> Result<super::mock_data::DisplayCategory> {
    Ok(super::mock_data::DisplayCategory {
        id: extract_text(fields, "分类ID").context("Category missing '分类ID'")?,
        name: extract_text(fields, "分类名称").context("Category missing '分类名称'")?,
        icon: extract_text(fields, "图标"),
        sort_order: extract_number(fields, "排序").unwrap_or(0.0) as i32,
    })
}

/// Parse a bitable record into a MediaItem (legacy, use video::parse_raw_media_item instead)
#[allow(dead_code)]
pub fn parse_media_item(
    fields: &HashMap<String, serde_json::Value>,
) -> Result<super::mock_data::MediaItem> {
    // Prefer attachment file URL, fallback to external URL field
    let url = extract_attachment_url(fields, "文件")
        .or_else(|| extract_url(fields, "外部链接"))
        .context("Media missing both '文件' and '外部链接'")?;

    Ok(super::mock_data::MediaItem {
        media_type: extract_select(fields, "媒体类型").unwrap_or_else(|| "image".to_string()),
        url,
        title: extract_text(fields, "标题"),
        duration: extract_number(fields, "时长(ms)").map(|n| n as i64),
        sort_order: extract_number(fields, "排序").unwrap_or(0.0) as i32,
    })
}

/// Parse a bitable record into StoreInfo
pub fn parse_store_info(
    fields: &HashMap<String, serde_json::Value>,
) -> Result<super::mock_data::StoreInfo> {
    Ok(super::mock_data::StoreInfo {
        name: extract_text(fields, "店铺名称").context("StoreInfo missing '店铺名称'")?,
        phone: extract_phone(fields, "联系电话").context("StoreInfo missing '联系电话'")?,
        qr_code_url: extract_attachment_url(fields, "二维码").unwrap_or_default(),
    })
}

/// Intermediate product record before brand/category resolution
pub struct RawProduct {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub brand_id_link: Option<String>,
    pub category_id_link: Option<String>,
    pub specification: String,
    pub unit: String,
    pub retail_price: f64,
    pub cost_price: Option<f64>,
    pub member_price: Option<f64>,
    pub promotion_price: Option<f64>,
    pub stock: i32,
    pub alcohol_content: f64,
    pub vintage: Option<i32>,
    pub brewing_process: String,
    pub flavor_profile: String,
    pub main_image: String,
    pub short_description: String,
    pub long_description: Option<String>,
    pub status: String,
    pub is_hot: bool,
    pub is_new: bool,
    pub is_promotion: bool,
    pub display_category_ids: Vec<String>,
    pub sort_order: i32,
}

/// Parse a bitable record into a RawProduct
pub fn parse_raw_product(
    fields: &HashMap<String, serde_json::Value>,
) -> Result<RawProduct> {
    let display_cats = extract_text(fields, "展示分类")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // For main_image: try attachment URL first, fallback to empty
    let main_image = extract_attachment_url(fields, "商品主图").unwrap_or_default();

    Ok(RawProduct {
        id: extract_text(fields, "商品ID").context("Product missing '商品ID'")?,
        sku: extract_text(fields, "商品编码").unwrap_or_default(),
        name: extract_text(fields, "商品名称").context("Product missing '商品名称'")?,
        brand_id_link: extract_link_text(fields, "品牌"),
        category_id_link: extract_link_text(fields, "分类"),
        specification: extract_text(fields, "规格").unwrap_or_default(),
        unit: extract_select(fields, "单位").unwrap_or_else(|| "瓶".to_string()),
        retail_price: extract_number(fields, "零售价").unwrap_or(0.0),
        cost_price: extract_number(fields, "成本价"),
        member_price: extract_number(fields, "会员价"),
        promotion_price: extract_number(fields, "促销价"),
        stock: extract_number(fields, "库存").unwrap_or(0.0) as i32,
        alcohol_content: extract_number(fields, "酒精度%").unwrap_or(0.0),
        vintage: extract_number(fields, "年份").map(|n| n as i32),
        brewing_process: extract_text(fields, "酿造工艺").unwrap_or_default(),
        flavor_profile: extract_text(fields, "风味描述").unwrap_or_default(),
        main_image,
        short_description: extract_text(fields, "简短描述").unwrap_or_default(),
        long_description: extract_text(fields, "详细描述"),
        status: extract_select(fields, "状态").unwrap_or_else(|| "active".to_string()),
        is_hot: extract_bool(fields, "热销"),
        is_new: extract_bool(fields, "新品"),
        is_promotion: extract_bool(fields, "促销中"),
        display_category_ids: display_cats,
        sort_order: extract_number(fields, "排序").unwrap_or(0.0) as i32,
    })
}
