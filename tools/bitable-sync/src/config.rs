use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub feishu_app_id: String,
    pub feishu_app_secret: String,
    pub bitable_app_token: String,
    pub table_id_products: String,
    pub table_id_brands: String,
    pub table_id_display_categories: String,
    pub table_id_media: String,
    pub table_id_store_info: String,
    /// Root of the family-business repo (auto-detected from binary location)
    pub repo_root: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try loading .env.txt first, then .env
        let _ = dotenvy::from_filename(".env.txt").or_else(|_| dotenvy::dotenv());

        // Auto-detect repo root: binary is in tools/bitable-sync/, repo root is ../../
        let repo_root = std::env::var("FAMILY_BUSINESS_REPO")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Relative to working directory: tools/bitable-sync -> repo root
                let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                // If cwd ends with tools/bitable-sync, go up two levels
                if cwd.ends_with("tools/bitable-sync") {
                    cwd.parent().unwrap().parent().unwrap().to_path_buf()
                } else {
                    // Fallback: try ../../ from the directory containing .env.txt
                    cwd.join("../..").canonicalize().unwrap_or(cwd)
                }
            });

        Ok(Self {
            feishu_app_id: std::env::var("FEISHU_APP_ID")
                .context("FEISHU_APP_ID not set")?,
            feishu_app_secret: std::env::var("FEISHU_APP_SECRET")
                .context("FEISHU_APP_SECRET not set")?,
            bitable_app_token: std::env::var("BITABLE_APP_TOKEN")
                .context("BITABLE_APP_TOKEN not set")?,
            table_id_products: std::env::var("TABLE_ID_PRODUCTS")
                .context("TABLE_ID_PRODUCTS not set")?,
            table_id_brands: std::env::var("TABLE_ID_BRANDS")
                .context("TABLE_ID_BRANDS not set")?,
            table_id_display_categories: std::env::var("TABLE_ID_DISPLAY_CATEGORIES")
                .context("TABLE_ID_DISPLAY_CATEGORIES not set")?,
            table_id_media: std::env::var("TABLE_ID_MEDIA")
                .context("TABLE_ID_MEDIA not set")?,
            table_id_store_info: std::env::var("TABLE_ID_STORE_INFO")
                .context("TABLE_ID_STORE_INFO not set")?,
            repo_root,
        })
    }

    /// T019: Strong validation — fail fast if repo_root is wrong
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.repo_root.exists(),
            "Repo root does not exist: {}",
            self.repo_root.display()
        );
        anyhow::ensure!(
            self.repo_root.join(".git").exists(),
            "Not a git repository (no .git): {}",
            self.repo_root.display()
        );
        anyhow::ensure!(
            self.repo_root.join("package.json").exists(),
            "package.json not found — wrong repo root? {}",
            self.repo_root.display()
        );
        anyhow::ensure!(
            self.data_dir().exists(),
            "src/data/ not found in repo: {}",
            self.repo_root.display()
        );
        anyhow::ensure!(
            self.public_dir().exists(),
            "public/ not found in repo: {}",
            self.repo_root.display()
        );
        Ok(())
    }

    pub fn data_dir(&self) -> PathBuf {
        self.repo_root.join("src/data")
    }

    pub fn public_dir(&self) -> PathBuf {
        self.repo_root.join("public")
    }
}
