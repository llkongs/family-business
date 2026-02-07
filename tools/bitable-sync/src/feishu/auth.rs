use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis";

#[derive(Debug, Serialize)]
struct TokenRequest {
    app_id: String,
    app_secret: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    code: i32,
    msg: String,
    tenant_access_token: Option<String>,
    expire: Option<i64>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct FeishuAuth {
    app_id: String,
    app_secret: String,
    client: reqwest::Client,
    cached: Arc<Mutex<Option<CachedToken>>>,
}

impl FeishuAuth {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            client: reqwest::Client::new(),
            cached: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_token(&self) -> Result<String> {
        // Check cache first
        {
            let cached = self.cached.lock().await;
            if let Some(ref t) = *cached {
                // Use cached token if it has at least 5 minutes left
                if t.expires_at > chrono::Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(t.token.clone());
                }
            }
        }

        // Fetch new token
        let token = self.fetch_token().await?;

        Ok(token)
    }

    async fn fetch_token(&self) -> Result<String> {
        let url = format!(
            "{}/auth/v3/tenant_access_token/internal",
            FEISHU_BASE_URL
        );

        tracing::info!("Fetching new tenant_access_token");

        let resp = self
            .client
            .post(&url)
            .json(&TokenRequest {
                app_id: self.app_id.clone(),
                app_secret: self.app_secret.clone(),
            })
            .send()
            .await
            .context("Failed to send token request")?
            .json::<TokenResponse>()
            .await
            .context("Failed to parse token response")?;

        if resp.code != 0 {
            anyhow::bail!("Failed to get token: {} - {}", resp.code, resp.msg);
        }

        let token = resp
            .tenant_access_token
            .context("No token in response")?;
        let expire = resp.expire.unwrap_or(7200);

        // Cache the token
        let cached = CachedToken {
            token: token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(expire),
        };
        *self.cached.lock().await = Some(cached);

        tracing::info!("Got new token, expires in {}s", expire);
        Ok(token)
    }
}
