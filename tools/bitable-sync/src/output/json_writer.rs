use anyhow::{Context, Result};
use std::path::Path;

use crate::models::product::ProductDatabase;

/// Write ProductDatabase to a JSON file (productDatabase.json)
pub fn write_product_database(db: &ProductDatabase, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(db)
        .context("Failed to serialize ProductDatabase to JSON")?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(output_path, &json)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    // Validate: re-parse to ensure valid JSON
    serde_json::from_str::<serde_json::Value>(&json)
        .context("Generated JSON is not valid")?;

    tracing::info!(
        "Wrote productDatabase.json ({} bytes, {} products)",
        json.len(),
        db.products.len()
    );

    Ok(())
}
