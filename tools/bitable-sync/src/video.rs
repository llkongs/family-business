use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::feishu::auth::FeishuAuth;

/// Metadata about a processed video, stored alongside HLS output for cache invalidation
#[derive(Debug, Serialize, Deserialize)]
struct VideoMeta {
    file_token: String,
    size: u64,
    source_name: String,
}

/// Info extracted from a Feishu Bitable attachment field
#[derive(Debug, Clone)]
pub struct AttachmentInfo {
    pub file_token: String,
    pub name: String,
    pub size: u64,
}

/// Extract full attachment info from a bitable record field
pub fn extract_attachment_info(
    fields: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
) -> Option<AttachmentInfo> {
    let val = fields.get(key)?;
    let arr = val.as_array()?;
    let first = arr.first()?;

    Some(AttachmentInfo {
        file_token: first.get("file_token")?.as_str()?.to_string(),
        name: first.get("name")?.as_str()?.to_string(),
        size: first.get("size")?.as_u64()?,
    })
}

/// Normalize media type: accept both Chinese and English values
/// "视频" / "video" -> "video"
/// "图片" / "image" / anything else -> "image"
fn normalize_media_type(raw: &str) -> &'static str {
    match raw {
        "video" | "视频" => "video",
        _ => "image",
    }
}

/// Raw media item parsed from bitable, before video processing
#[derive(Debug, Clone)]
pub struct RawMediaItem {
    pub media_type: String, // normalized: "video" or "image"
    pub title: Option<String>,
    pub duration: Option<i64>,
    pub sort_order: i32,
    /// External URL (from "外部链接" field)
    pub external_url: Option<String>,
    /// Attachment info (from "文件" field)
    pub attachment: Option<AttachmentInfo>,
}

/// Parse a bitable record into a RawMediaItem
pub fn parse_raw_media_item(
    fields: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<RawMediaItem> {
    use crate::models::bitable_records::*;

    let external_url = extract_url(fields, "外部链接");
    let attachment = extract_attachment_info(fields, "文件");

    if external_url.is_none() && attachment.is_none() {
        anyhow::bail!("Media missing both '文件' attachment and '外部链接'");
    }

    let raw_type = extract_select(fields, "媒体类型").unwrap_or_else(|| "image".to_string());

    Ok(RawMediaItem {
        media_type: normalize_media_type(&raw_type).to_string(),
        title: extract_text(fields, "标题"),
        duration: extract_number(fields, "时长(ms)").map(|n| n as i64),
        sort_order: extract_number(fields, "排序").unwrap_or(0.0) as i32,
        external_url,
        attachment,
    })
}

// ============================================================
// Feishu Drive API: resolve file_token -> real download URL
// ============================================================

#[derive(Debug, Deserialize)]
struct DriveApiResponse {
    code: i32,
    msg: String,
    data: Option<TmpDownloadData>,
}

#[derive(Debug, Deserialize)]
struct TmpDownloadData {
    tmp_download_urls: Vec<TmpDownloadUrl>,
}

#[derive(Debug, Deserialize)]
struct TmpDownloadUrl {
    #[allow(dead_code)]
    file_token: String,
    tmp_download_url: String,
}

/// Resolve a file_token to a temporary download URL via Feishu Drive API.
/// The returned URL is a direct download link valid for ~30 minutes.
async fn resolve_download_url(auth: &FeishuAuth, file_token: &str) -> Result<String> {
    let token = auth.get_token().await?;
    let url = format!(
        "https://open.feishu.cn/open-apis/drive/v1/medias/batch_get_tmp_download_url?file_tokens={}",
        file_token
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("Failed to call batch_get_tmp_download_url")?
        .json::<DriveApiResponse>()
        .await
        .context("Failed to parse download URL response")?;

    if resp.code != 0 {
        anyhow::bail!(
            "Failed to get download URL for {}: {} - {}",
            file_token,
            resp.code,
            resp.msg
        );
    }

    let data = resp.data.context("No data in download URL response")?;
    let dl = data
        .tmp_download_urls
        .into_iter()
        .next()
        .context("Empty tmp_download_urls array")?;

    tracing::debug!("Resolved {} -> download URL", file_token);
    Ok(dl.tmp_download_url)
}

// ============================================================
// Slug / filesystem helpers
// ============================================================

/// Make a filesystem/URL-safe slug from a string
fn slugify(s: &str) -> String {
    let slug: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c.to_ascii_lowercase()
            } else if c == ' ' {
                '-'
            } else if c > '\x7f' {
                c // Keep CJK chars
            } else {
                '-'
            }
        })
        .collect();
    // Collapse multiple dashes
    let mut result = String::new();
    let mut prev_dash = false;
    for c in slug.chars() {
        if c == '-' {
            if !prev_dash {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    result.trim_matches('-').to_string()
}

// ============================================================
// Download + ffmpeg HLS conversion
// ============================================================

/// Download a file from a direct URL to a local path
async fn download_file(url: &str, dest: &Path) -> Result<u64> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 min for large videos
        .build()?;

    tracing::info!("Downloading to {} ...", dest.display());
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download from URL"))?;

    if !resp.status().is_success() {
        anyhow::bail!("Download failed with status {}", resp.status());
    }

    let bytes = resp
        .bytes()
        .await
        .context("Failed to read response body")?;

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let size = bytes.len() as u64;
    tokio::fs::write(dest, &bytes).await?;
    tracing::info!("Downloaded {} bytes ({:.1} MB)", size, size as f64 / 1_048_576.0);
    Ok(size)
}

/// Convert a video file to HLS segments using ffmpeg.
/// Returns the relative path (from public/) to the m3u8 playlist.
fn convert_to_hls(input: &Path, output_dir: &Path, slug: &str) -> Result<String> {
    std::fs::create_dir_all(output_dir)?;

    let playlist = output_dir.join("index.m3u8");
    let segment_pattern = output_dir.join(format!("{}_%03d.ts", slug));

    tracing::info!(
        "ffmpeg HLS: {} -> {}",
        input.display(),
        output_dir.display()
    );

    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().unwrap(),
            "-codec",
            "copy",
            "-start_number",
            "0",
            "-hls_time",
            "10",
            "-hls_list_size",
            "0",
            "-hls_segment_filename",
            segment_pattern.to_str().unwrap(),
            "-y",
            playlist.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run ffmpeg - is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed (exit {}): {}", output.status, stderr);
    }

    // Count generated segments
    let segment_count = std::fs::read_dir(output_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "ts")
                .unwrap_or(false)
        })
        .count();
    tracing::info!("HLS complete: {} segments", segment_count);

    Ok(format!("videos/{}/index.m3u8", slug))
}

// ============================================================
// Cache: skip re-processing unchanged videos
// ============================================================

fn is_cached(meta_path: &Path, file_token: &str, size: u64) -> bool {
    if let Ok(content) = std::fs::read_to_string(meta_path) {
        if let Ok(meta) = serde_json::from_str::<VideoMeta>(&content) {
            return meta.file_token == file_token && meta.size == size;
        }
    }
    false
}

fn write_meta(meta_path: &Path, file_token: &str, size: u64, source_name: &str) -> Result<()> {
    let meta = VideoMeta {
        file_token: file_token.to_string(),
        size,
        source_name: source_name.to_string(),
    };
    std::fs::write(meta_path, serde_json::to_string_pretty(&meta)?)?;
    Ok(())
}

// ============================================================
// Process pipeline: resolve URL -> download -> ffmpeg -> cache
// ============================================================

/// Process a single video: resolve download URL, download, convert to HLS.
/// Returns the relative URL path to the m3u8 playlist.
async fn process_one_video(
    auth: &FeishuAuth,
    attachment: &AttachmentInfo,
    slug: &str,
    videos_dir: &Path,
) -> Result<String> {
    let output_dir = videos_dir.join(slug);
    let meta_path = output_dir.join(".meta.json");

    // Check cache - skip if unchanged
    if is_cached(&meta_path, &attachment.file_token, attachment.size) {
        tracing::info!(
            "Video '{}' unchanged ({}), skipping",
            slug,
            &attachment.file_token
        );
        return Ok(format!("videos/{}/index.m3u8", slug));
    }

    // Step 1: Resolve file_token -> real download URL via Drive API
    tracing::info!(
        "Resolving download URL for '{}' (token={}, {:.1} MB)...",
        slug,
        &attachment.file_token,
        attachment.size as f64 / 1_048_576.0
    );
    let download_url = resolve_download_url(auth, &attachment.file_token).await?;

    // Step 2: Download to temp file
    let tmp_dir = std::env::temp_dir().join("bitable-sync-videos");
    let tmp_file = tmp_dir.join(&attachment.name);
    download_file(&download_url, &tmp_file).await?;

    // Step 3: Clean old HLS output if exists
    if output_dir.exists() {
        for entry in std::fs::read_dir(&output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "ts" || ext == "m3u8" {
                    std::fs::remove_file(&path)?;
                }
            }
        }
    }

    // Step 4: Convert to HLS
    let relative_url = convert_to_hls(&tmp_file, &output_dir, slug)?;

    // Step 5: Write cache metadata
    write_meta(
        &meta_path,
        &attachment.file_token,
        attachment.size,
        &attachment.name,
    )?;

    // Clean up temp file
    let _ = tokio::fs::remove_file(&tmp_file).await;

    Ok(relative_url)
}

// ============================================================
// Public API: process all media items
// ============================================================

/// Process all video media items: download attachments from Feishu, convert to HLS.
/// Returns a list of final MediaItems with correct URLs.
pub async fn process_media_items(
    auth: &FeishuAuth,
    raw_items: Vec<RawMediaItem>,
    public_dir: &Path,
) -> Result<Vec<crate::models::mock_data::MediaItem>> {
    let videos_dir = public_dir.join("videos");
    let mut results = Vec::new();

    for raw in &raw_items {
        let media_item = if raw.media_type == "video" {
            if let Some(ref att) = raw.attachment {
                // Video with attachment -> resolve URL -> download -> HLS
                let slug = slugify(
                    raw.title
                        .as_deref()
                        .unwrap_or(&att.name.replace('.', "-")),
                );
                match process_one_video(auth, att, &slug, &videos_dir).await {
                    Ok(hls_url) => crate::models::mock_data::MediaItem {
                        media_type: "video".to_string(),
                        url: hls_url,
                        title: raw.title.clone(),
                        duration: raw.duration,
                        sort_order: raw.sort_order,
                    },
                    Err(e) => {
                        tracing::error!(
                            "Failed to process video '{}': {}",
                            raw.title.as_deref().unwrap_or("untitled"),
                            e
                        );
                        continue;
                    }
                }
            } else if let Some(ref url) = raw.external_url {
                // Video with external URL (already hosted elsewhere)
                crate::models::mock_data::MediaItem {
                    media_type: "video".to_string(),
                    url: url.clone(),
                    title: raw.title.clone(),
                    duration: raw.duration,
                    sort_order: raw.sort_order,
                }
            } else {
                tracing::warn!(
                    "Video '{}' has no attachment or URL, skipping",
                    raw.title.as_deref().unwrap_or("untitled")
                );
                continue;
            }
        } else {
            // Image: use external URL (attachment images need separate handling)
            let url = raw
                .external_url
                .clone()
                .unwrap_or_default();

            crate::models::mock_data::MediaItem {
                media_type: raw.media_type.clone(),
                url,
                title: raw.title.clone(),
                duration: raw.duration,
                sort_order: raw.sort_order,
            }
        };

        results.push(media_item);
    }

    // Sort by sort_order
    results.sort_by_key(|m| m.sort_order);

    let video_count = results.iter().filter(|m| m.media_type == "video").count();
    let image_count = results.len() - video_count;
    tracing::info!(
        "Processed media: {} videos (HLS), {} images",
        video_count,
        image_count
    );

    Ok(results)
}

/// Collect all video-related file paths under public/videos/ for git staging
pub fn collect_video_files(public_dir: &Path) -> Vec<PathBuf> {
    let videos_dir = public_dir.join("videos");
    if !videos_dir.exists() {
        return vec![];
    }

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&videos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if let Some(ext) = sub_path.extension() {
                            if ext == "ts" || ext == "m3u8" {
                                files.push(sub_path);
                            }
                        }
                    }
                }
            }
        }
    }

    files
}
