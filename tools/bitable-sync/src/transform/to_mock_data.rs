use crate::models::bitable_records::RawProduct;
use crate::models::mock_data::{DisplayCategory, MediaItem, MockData, MockProduct, StoreInfo};

/// Build MockData from raw bitable records.
///
/// Products are expanded: each product appears once per display_category_id.
/// Products within each category are sorted by sort_order.
pub fn build_mock_data(
    raw_products: &[RawProduct],
    display_categories: &[DisplayCategory],
    media_items: &[MediaItem],
    store_info: &StoreInfo,
) -> MockData {
    let mut mock_products: Vec<MockProduct> = Vec::new();

    // Sort raw products by sort_order
    let mut sorted_products: Vec<&RawProduct> = raw_products
        .iter()
        .filter(|p| p.status == "active")
        .collect();
    sorted_products.sort_by_key(|p| p.sort_order);

    // Expand products into display categories
    for raw in &sorted_products {
        if raw.display_category_ids.is_empty() {
            // If no display categories specified, skip (product won't appear in mock data)
            continue;
        }

        for cat_id in &raw.display_category_ids {
            // Create a unique ID for products that appear in multiple categories
            let product_id = if raw.display_category_ids.len() > 1 {
                // Check if this is not the "primary" category (first one)
                if cat_id != &raw.display_category_ids[0] {
                    format!("{}-{}", raw.id, cat_id)
                } else {
                    raw.id.clone()
                }
            } else {
                raw.id.clone()
            };

            mock_products.push(MockProduct {
                id: product_id,
                name: raw.name.clone(),
                description: raw.short_description.clone(),
                price: raw.retail_price,
                image: raw.main_image.clone(),
                category_id: cat_id.clone(),
            });
        }
    }

    // Sort media items
    let mut sorted_media = media_items.to_vec();
    sorted_media.sort_by_key(|m| m.sort_order);

    // Sort categories
    let mut sorted_categories = display_categories.to_vec();
    sorted_categories.sort_by_key(|c| c.sort_order);

    MockData {
        store_info: store_info.clone(),
        media_playlist: sorted_media,
        categories: sorted_categories,
        products: mock_products,
    }
}
