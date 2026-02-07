use anyhow::{Context, Result};
use std::path::Path;

use crate::config::Config;
use crate::feishu::auth::FeishuAuth;
use crate::feishu::bitable::BitableClient;
use crate::models::bitable_records;
use crate::models::mock_data::StoreInfo;
use crate::models::product::Category;

pub struct SyncOptions {
    pub dry_run: bool,
    pub no_push: bool,
}

pub async fn run_sync(config: &Config, opts: &SyncOptions) -> Result<()> {
    // 1. Initialize auth and client
    let auth = FeishuAuth::new(config.feishu_app_id.clone(), config.feishu_app_secret.clone());
    let client = BitableClient::new(auth.clone(), config.bitable_app_token.clone());

    // 2. Read all tables concurrently
    tracing::info!("Reading all tables from bitable...");
    let (products_raw, brands_raw, categories_raw, media_raw, store_raw) = tokio::try_join!(
        client.read_all_records(&config.table_id_products),
        client.read_all_records(&config.table_id_brands),
        client.read_all_records(&config.table_id_display_categories),
        client.read_all_records(&config.table_id_media),
        client.read_all_records(&config.table_id_store_info),
    )?;

    // 3. Parse records
    tracing::info!("Parsing records...");

    let brands: Vec<_> = brands_raw
        .iter()
        .filter_map(|r| match bitable_records::parse_brand(&r.fields) {
            Ok(b) => Some(b),
            Err(e) => {
                tracing::warn!("Skipping brand record: {}", e);
                None
            }
        })
        .collect();
    tracing::info!("Parsed {} brands", brands.len());

    let display_categories: Vec<_> = categories_raw
        .iter()
        .filter_map(|r| match bitable_records::parse_display_category(&r.fields) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::warn!("Skipping category record: {}", e);
                None
            }
        })
        .collect();
    tracing::info!("Parsed {} display categories", display_categories.len());

    let raw_media_items: Vec<_> = media_raw
        .iter()
        .filter_map(|r| match crate::video::parse_raw_media_item(&r.fields) {
            Ok(m) => Some(m),
            Err(e) => {
                tracing::warn!("Skipping media record: {}", e);
                None
            }
        })
        .collect();
    tracing::info!("Parsed {} media items", raw_media_items.len());

    // 3b. Process video attachments: download from Feishu -> ffmpeg HLS -> public/videos/
    let media_items = if opts.dry_run {
        // In dry-run mode, skip video downloads and just use placeholder URLs
        raw_media_items
            .iter()
            .map(|raw| crate::models::mock_data::MediaItem {
                media_type: raw.media_type.clone(),
                url: raw
                    .external_url
                    .clone()
                    .or_else(|| raw.attachment.as_ref().map(|a| format!("[attachment:{}]", a.file_token)))
                    .unwrap_or_default(),
                title: raw.title.clone(),
                duration: raw.duration,
                sort_order: raw.sort_order,
            })
            .collect()
    } else {
        crate::video::process_media_items(&auth, raw_media_items, &config.public_dir()).await?
    };

    let store_info = store_raw
        .first()
        .map(|r| bitable_records::parse_store_info(&r.fields))
        .transpose()?
        .unwrap_or(StoreInfo {
            name: "绍兴黄酒专卖".to_string(),
            phone: "15936229925".to_string(),
            qr_code_url: "images/qrcode.jpg".to_string(),
        });
    tracing::info!("Store info: {}", store_info.name);

    let raw_products: Vec<_> = products_raw
        .iter()
        .filter_map(|r| match bitable_records::parse_raw_product(&r.fields) {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::warn!("Skipping product record: {}", e);
                None
            }
        })
        .collect();
    tracing::info!("Parsed {} products", raw_products.len());

    // 4. Build product categories for productDatabase.json
    // We use the display categories as the category source, converting to the full Category type
    let db_categories: Vec<Category> = display_categories
        .iter()
        .map(|dc| Category {
            id: dc.id.clone(),
            name: dc.name.clone(),
            parent_id: None,
            level: 1,
            icon: dc.icon.clone(),
        })
        .collect();

    // 5. Transform data
    tracing::info!("Transforming data...");
    let product_db = crate::transform::to_database::build_product_database(
        &raw_products,
        &brands,
        &db_categories,
    )?;

    let mock_data = crate::transform::to_mock_data::build_mock_data(
        &raw_products,
        &display_categories,
        &media_items,
        &store_info,
    );

    // Summary
    tracing::info!(
        "Transform complete: {} products in DB, {} products in mock data, {} categories, {} media items",
        product_db.products.len(),
        mock_data.products.len(),
        mock_data.categories.len(),
        mock_data.media_playlist.len()
    );

    if opts.dry_run {
        tracing::info!("Dry run mode - not writing files");
        // Print a preview
        let preview_json = serde_json::to_string_pretty(&product_db)?;
        tracing::info!(
            "productDatabase.json preview ({} bytes)",
            preview_json.len()
        );
        let preview_ts = crate::output::ts_writer::generate_mock_data_ts(&mock_data)?;
        tracing::info!("mockData.ts preview ({} bytes)", preview_ts.len());
        return Ok(());
    }

    // 6. Write files
    let json_path = config.data_dir().join("productDatabase.json");
    let ts_path = config.data_dir().join("mockData.ts");

    tracing::info!("Writing files...");
    crate::output::json_writer::write_product_database(&product_db, &json_path)?;
    crate::output::ts_writer::write_mock_data_ts(&mock_data, &ts_path)?;

    // 7. Validate image paths
    validate_image_paths(&config.repo_root, &product_db, &mock_data)?;

    if opts.no_push {
        tracing::info!("No-push mode - files written but not committed");
        return Ok(());
    }

    // 8. Git commit and push
    if crate::git::has_changes(&config.repo_root)? {
        tracing::info!("Committing and pushing changes...");
        let mut files_to_stage: Vec<String> = vec![
            "src/data/productDatabase.json".to_string(),
            "src/data/mockData.ts".to_string(),
        ];

        // Stage video HLS files (public/videos/*/*.ts, *.m3u8)
        let video_files = crate::video::collect_video_files(&config.public_dir());
        for vf in &video_files {
            if let Ok(rel) = vf.strip_prefix(&config.repo_root) {
                files_to_stage.push(rel.to_string_lossy().to_string());
            }
        }

        let refs: Vec<&str> = files_to_stage.iter().map(|s| s.as_str()).collect();
        crate::git::commit_and_push(&config.repo_root, &refs)?;
    } else {
        tracing::info!("No changes detected, nothing to commit");
    }

    tracing::info!("Sync completed successfully!");
    Ok(())
}

fn validate_image_paths(
    repo_path: &Path,
    db: &crate::models::product::ProductDatabase,
    mock: &crate::models::mock_data::MockData,
) -> Result<()> {
    let public_dir = repo_path.join("public");
    let mut missing = Vec::new();

    // Check product images in database
    for product in &db.products {
        if !product.main_image.is_empty() {
            let img_path = public_dir.join(&product.main_image);
            if !img_path.exists() {
                missing.push(format!(
                    "Product '{}': {}",
                    product.name, product.main_image
                ));
            }
        }
    }

    // Check media images
    for item in &mock.media_playlist {
        let img_path = public_dir.join(&item.url);
        if !img_path.exists() {
            missing.push(format!(
                "Media '{}': {}",
                item.title.as_deref().unwrap_or("untitled"),
                item.url
            ));
        }
    }

    if !missing.is_empty() {
        tracing::warn!(
            "Missing {} image files:\n  {}",
            missing.len(),
            missing.join("\n  ")
        );
    } else {
        tracing::info!("All image paths validated");
    }

    Ok(())
}

/// Check configuration and connectivity
pub async fn check_config(config: &Config) -> Result<()> {
    tracing::info!("Checking configuration...");

    // Validate paths
    config.validate().context("Config validation failed")?;
    tracing::info!("Repository path OK: {}", config.repo_root.display());

    // Test auth
    let auth = FeishuAuth::new(config.feishu_app_id.clone(), config.feishu_app_secret.clone());
    let token = auth.get_token().await.context("Auth failed")?;
    tracing::info!("Auth OK (token: {}...)", &token[..8.min(token.len())]);

    // Test table access
    let client = BitableClient::new(auth, config.bitable_app_token.clone());
    let tables = client.list_tables().await.context("Failed to list tables")?;
    tracing::info!("Found {} tables in bitable app", tables.len());
    for t in &tables {
        tracing::info!("  - {} ({})", t.name, t.table_id);
    }

    tracing::info!("All checks passed!");
    Ok(())
}

/// List all tables in the bitable app
pub async fn list_tables(config: &Config) -> Result<()> {
    let auth = FeishuAuth::new(config.feishu_app_id.clone(), config.feishu_app_secret.clone());
    let client = BitableClient::new(auth, config.bitable_app_token.clone());

    let tables = client.list_tables().await?;
    println!("Tables in bitable app ({}):", config.bitable_app_token);
    println!("{:<30} {}", "Name", "Table ID");
    println!("{}", "-".repeat(60));
    for t in &tables {
        println!("{:<30} {}", t.name, t.table_id);
    }

    Ok(())
}
