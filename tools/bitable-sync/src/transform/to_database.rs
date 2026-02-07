use anyhow::Result;
use std::collections::HashMap;

use crate::models::bitable_records::RawProduct;
use crate::models::product::{Brand, Category, Product, ProductDatabase};

/// Build the complete ProductDatabase from raw bitable data.
///
/// The `brand_map` maps brand names (from linked field display text) to Brand structs.
pub fn build_product_database(
    raw_products: &[RawProduct],
    brands: &[Brand],
    categories: &[Category],
) -> Result<ProductDatabase> {
    // Build lookup maps
    let brand_map: HashMap<&str, &Brand> = brands
        .iter()
        .flat_map(|b| {
            // Allow lookup by both id and name
            vec![
                (b.id.as_str(), b),
                (b.name.as_str(), b),
            ]
        })
        .collect();

    let category_map: HashMap<&str, &Category> = categories
        .iter()
        .flat_map(|c| {
            vec![
                (c.id.as_str(), c),
                (c.name.as_str(), c),
            ]
        })
        .collect();

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let products: Vec<Product> = raw_products
        .iter()
        .filter(|p| p.status == "active")
        .map(|raw| {
            let brand = raw
                .brand_id_link
                .as_deref()
                .and_then(|name| brand_map.get(name))
                .map(|b| (*b).clone())
                .unwrap_or_else(|| Brand {
                    id: "unknown".to_string(),
                    name: raw.brand_id_link.clone().unwrap_or_else(|| "未知品牌".to_string()),
                    logo: None,
                    story: None,
                    founded_year: None,
                    origin: None,
                });

            let category = raw
                .category_id_link
                .as_deref()
                .and_then(|name| category_map.get(name))
                .map(|c| (*c).clone())
                .unwrap_or_else(|| Category {
                    id: "unknown".to_string(),
                    name: raw.category_id_link.clone().unwrap_or_else(|| "未分类".to_string()),
                    parent_id: None,
                    level: 2,
                    icon: None,
                });

            // Estimate weight from specification
            let weight = parse_weight(&raw.specification);

            Product {
                id: raw.id.clone(),
                sku: raw.sku.clone(),
                barcode: String::new(), // Not tracked in bitable
                name: raw.name.clone(),
                brand,
                category,
                specification: raw.specification.clone(),
                unit: raw.unit.clone(),
                pack_size: 1,
                weight,
                retail_price: raw.retail_price,
                cost_price: raw.cost_price,
                member_price: raw.member_price,
                promotion_price: raw.promotion_price,
                stock: raw.stock,
                safety_stock: (raw.stock as f64 * 0.25) as i32, // Default 25% safety stock
                warehouse_location: None,
                origin: "浙江绍兴".to_string(),
                shelf_life: 36,
                storage_condition: "阴凉干燥处保存".to_string(),
                alcohol_content: raw.alcohol_content,
                vintage: raw.vintage,
                brewing_process: raw.brewing_process.clone(),
                flavor_profile: raw.flavor_profile.clone(),
                serving_suggestion: None,
                main_image: raw.main_image.clone(),
                detail_images: None,
                video: None,
                short_description: raw.short_description.clone(),
                long_description: raw.long_description.clone(),
                supplier: None,
                status: raw.status.clone(),
                is_hot: raw.is_hot,
                is_new: raw.is_new,
                is_promotion: raw.is_promotion,
                created_at: now.clone(),
                updated_at: now.clone(),
            }
        })
        .collect();

    Ok(ProductDatabase {
        version: "1.0.0".to_string(),
        last_updated: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        brands: brands.to_vec(),
        categories: categories.to_vec(),
        suppliers: Vec::new(), // Suppliers not tracked in bitable for now
        products,
    })
}

/// Parse weight in ml from specification string like "500ml" or "2.5L"
fn parse_weight(spec: &str) -> i32 {
    let lower = spec.to_lowercase();
    if lower.ends_with("ml") {
        lower
            .trim_end_matches("ml")
            .trim()
            .parse::<f64>()
            .unwrap_or(500.0) as i32
    } else if lower.ends_with('l') {
        let liters = lower
            .trim_end_matches('l')
            .trim()
            .parse::<f64>()
            .unwrap_or(0.5);
        (liters * 1000.0) as i32
    } else {
        500 // Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_weight() {
        assert_eq!(parse_weight("500ml"), 500);
        assert_eq!(parse_weight("2.5L"), 2500);
        assert_eq!(parse_weight("1.5L"), 1500);
        assert_eq!(parse_weight("750ml"), 750);
    }
}
