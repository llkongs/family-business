use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::auth::FeishuAuth;

const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis";
const PAGE_SIZE: i32 = 500;

// ============================================================
// Response types
// ============================================================

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    code: i32,
    msg: String,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct ListRecordsData {
    has_more: bool,
    page_token: Option<String>,
    total: Option<i32>,
    items: Option<Vec<RecordItem>>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RecordItem {
    pub record_id: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListTablesData {
    has_more: Option<bool>,
    page_token: Option<String>,
    items: Option<Vec<TableInfo>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TableInfo {
    pub table_id: String,
    pub name: String,
    pub revision: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CreateTableData {
    table_id: String,
    default_view_id: Option<String>,
    field_id_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FieldData {
    field: FieldInfo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FieldInfo {
    pub field_id: String,
    pub field_name: String,
    #[serde(rename = "type")]
    pub field_type: i32,
    pub ui_type: Option<String>,
    pub is_primary: Option<bool>,
    pub property: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListFieldsData {
    has_more: bool,
    items: Option<Vec<FieldInfo>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WikiNodeData {
    node: WikiNode,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct WikiNode {
    pub node_token: String,
    pub obj_token: String,
    pub obj_type: String,
    pub title: String,
}

// ============================================================
// Field type definitions for creating tables/fields
// ============================================================

/// Feishu Bitable field types
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum FieldType {
    /// 多行文本 (type=1)
    Text = 1,
    /// 数字 (type=2)
    Number = 2,
    /// 单选 (type=3)
    SingleSelect = 3,
    /// 多选 (type=4)
    MultiSelect = 4,
    /// 日期 (type=5)
    DateTime = 5,
    /// 复选框 (type=7)
    Checkbox = 7,
    /// 人员 (type=11)
    Person = 11,
    /// 电话号码 (type=13)
    Phone = 13,
    /// 超链接 (type=15)
    Url = 15,
    /// 附件 (type=17)
    Attachment = 17,
    /// 单向关联 (type=18)
    SingleLink = 18,
    /// 双向关联 (type=21)
    DuplexLink = 21,
}

/// Field definition for creating a table or adding a field
#[derive(Debug, Clone, Serialize)]
pub struct FieldDef {
    pub field_name: String,
    #[serde(rename = "type")]
    pub field_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<serde_json::Value>,
}

impl FieldDef {
    /// Create a text field
    pub fn text(name: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::Text as i32,
            property: None,
        }
    }

    /// Create a number field with optional formatter
    /// formatter: "0" = integer, "0.0" = 1 decimal, "0.00" = 2 decimals
    pub fn number(name: &str, formatter: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::Number as i32,
            property: Some(serde_json::json!({"formatter": formatter})),
        }
    }

    /// Create a single select field with options
    pub fn single_select(name: &str, options: &[&str]) -> Self {
        let opts: Vec<serde_json::Value> = options
            .iter()
            .map(|o| serde_json::json!({"name": o}))
            .collect();
        Self {
            field_name: name.to_string(),
            field_type: FieldType::SingleSelect as i32,
            property: Some(serde_json::json!({"options": opts})),
        }
    }

    /// Create a checkbox field
    pub fn checkbox(name: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::Checkbox as i32,
            property: None,
        }
    }

    /// Create a phone field
    pub fn phone(name: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::Phone as i32,
            property: None,
        }
    }

    /// Create an attachment field
    pub fn attachment(name: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::Attachment as i32,
            property: None,
        }
    }

    /// Create a single-direction link field (关联)
    pub fn link(name: &str, linked_table_id: &str) -> Self {
        Self {
            field_name: name.to_string(),
            field_type: FieldType::SingleLink as i32,
            property: Some(serde_json::json!({"table_id": linked_table_id})),
        }
    }
}

// ============================================================
// Client
// ============================================================

pub struct BitableClient {
    auth: FeishuAuth,
    client: reqwest::Client,
    app_token: String,
}

impl BitableClient {
    pub fn new(auth: FeishuAuth, app_token: String) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
            app_token,
        }
    }

    fn tables_url(&self) -> String {
        format!(
            "{}/bitable/v1/apps/{}/tables",
            FEISHU_BASE_URL, self.app_token
        )
    }

    // ---- Read operations ----

    /// List all tables in the bitable app
    pub async fn list_tables(&self) -> Result<Vec<TableInfo>> {
        let token = self.auth.get_token().await?;
        let resp = self
            .client
            .get(&self.tables_url())
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list tables")?
            .json::<ApiResponse<ListTablesData>>()
            .await
            .context("Failed to parse tables response")?;

        if resp.code != 0 {
            anyhow::bail!("Failed to list tables: {} - {}", resp.code, resp.msg);
        }

        Ok(resp
            .data
            .and_then(|d| d.items)
            .unwrap_or_default())
    }

    /// List all fields in a table
    #[allow(dead_code)]
    pub async fn list_fields(&self, table_id: &str) -> Result<Vec<FieldInfo>> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}/fields", self.tables_url(), table_id);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list fields")?
            .json::<ApiResponse<ListFieldsData>>()
            .await
            .context("Failed to parse fields response")?;

        if resp.code != 0 {
            anyhow::bail!("Failed to list fields: {} - {}", resp.code, resp.msg);
        }

        Ok(resp
            .data
            .and_then(|d| d.items)
            .unwrap_or_default())
    }

    /// Read all records from a table (handles pagination)
    pub async fn read_all_records(&self, table_id: &str) -> Result<Vec<RecordItem>> {
        let mut all_records = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let token = self.auth.get_token().await?;
            let mut url = format!(
                "{}/{}/records?page_size={}",
                self.tables_url(),
                table_id,
                PAGE_SIZE
            );

            if let Some(ref pt) = page_token {
                url.push_str(&format!("&page_token={}", pt));
            }

            tracing::debug!("Fetching records from {}", table_id);

            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .with_context(|| format!("Failed to read records from table {}", table_id))?
                .json::<ApiResponse<ListRecordsData>>()
                .await
                .with_context(|| format!("Failed to parse records from table {}", table_id))?;

            if resp.code != 0 {
                anyhow::bail!(
                    "Failed to read records from {}: {} - {}",
                    table_id,
                    resp.code,
                    resp.msg
                );
            }

            if let Some(data) = resp.data {
                if let Some(total) = data.total {
                    tracing::info!("Table {} has {} total records", table_id, total);
                }

                if let Some(items) = data.items {
                    all_records.extend(items);
                }

                if data.has_more {
                    page_token = data.page_token;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        tracing::info!(
            "Read {} records from table {}",
            all_records.len(),
            table_id
        );
        Ok(all_records)
    }

    // ---- Write operations: Table management ----

    /// Create a new table with fields
    /// Returns the new table_id
    pub async fn create_table(
        &self,
        name: &str,
        view_name: &str,
        fields: &[FieldDef],
    ) -> Result<String> {
        let token = self.auth.get_token().await?;
        let body = serde_json::json!({
            "table": {
                "name": name,
                "default_view_name": view_name,
                "fields": fields,
            }
        });

        let resp = self
            .client
            .post(&self.tables_url())
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to create table '{}'", name))?
            .json::<ApiResponse<CreateTableData>>()
            .await
            .with_context(|| format!("Failed to parse create table response for '{}'", name))?;

        if resp.code != 0 {
            anyhow::bail!("Failed to create table '{}': {} - {}", name, resp.code, resp.msg);
        }

        let table_id = resp
            .data
            .context("No data in create table response")?
            .table_id;

        tracing::info!("Created table '{}' -> {}", name, table_id);
        Ok(table_id)
    }

    /// Delete a table
    pub async fn delete_table(&self, table_id: &str) -> Result<()> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}", self.tables_url(), table_id);

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .with_context(|| format!("Failed to delete table {}", table_id))?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;

        if resp.code != 0 {
            anyhow::bail!("Failed to delete table {}: {} - {}", table_id, resp.code, resp.msg);
        }

        tracing::info!("Deleted table {}", table_id);
        Ok(())
    }

    // ---- Write operations: Record management ----

    /// Batch create records in a table
    #[allow(dead_code)]
    pub async fn batch_create_records(
        &self,
        table_id: &str,
        records: &[serde_json::Value],
    ) -> Result<()> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}/records/batch_create", self.tables_url(), table_id);
        let body = serde_json::json!({"records": records});

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to batch create records in {}", table_id))?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;

        if resp.code != 0 {
            anyhow::bail!(
                "Failed to batch create records in {}: {} - {}",
                table_id,
                resp.code,
                resp.msg
            );
        }

        tracing::info!("Created {} records in table {}", records.len(), table_id);
        Ok(())
    }

    // ---- Write operations: Field management ----

    /// Add a field to a table
    pub async fn create_field(
        &self,
        table_id: &str,
        field: &FieldDef,
    ) -> Result<String> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}/fields", self.tables_url(), table_id);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(field)
            .send()
            .await
            .with_context(|| format!("Failed to create field '{}'", field.field_name))?
            .json::<ApiResponse<FieldData>>()
            .await?;

        if resp.code != 0 {
            anyhow::bail!(
                "Failed to create field '{}': {} - {}",
                field.field_name,
                resp.code,
                resp.msg
            );
        }

        let field_id = resp.data.context("No data")?.field.field_id;
        tracing::info!("Created field '{}' -> {}", field.field_name, field_id);
        Ok(field_id)
    }

    /// Update a field (rename, change type, etc.)
    /// Note: type is required even if only renaming
    #[allow(dead_code)]
    pub async fn update_field(
        &self,
        table_id: &str,
        field_id: &str,
        field: &FieldDef,
    ) -> Result<()> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}/fields/{}", self.tables_url(), table_id, field_id);

        let resp = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(field)
            .send()
            .await
            .with_context(|| format!("Failed to update field {}", field_id))?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;

        if resp.code != 0 {
            anyhow::bail!(
                "Failed to update field {}: {} - {}",
                field_id,
                resp.code,
                resp.msg
            );
        }

        tracing::info!("Updated field {} -> '{}'", field_id, field.field_name);
        Ok(())
    }

    /// Delete a field from a table
    #[allow(dead_code)]
    pub async fn delete_field(&self, table_id: &str, field_id: &str) -> Result<()> {
        let token = self.auth.get_token().await?;
        let url = format!("{}/{}/fields/{}", self.tables_url(), table_id, field_id);

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .with_context(|| format!("Failed to delete field {}", field_id))?
            .json::<ApiResponse<serde_json::Value>>()
            .await?;

        if resp.code != 0 {
            anyhow::bail!("Failed to delete field {}: {} - {}", field_id, resp.code, resp.msg);
        }

        tracing::info!("Deleted field {}", field_id);
        Ok(())
    }

    // ---- Wiki integration ----

    /// Resolve a wiki node token to get the actual bitable app_token
    /// Wiki-embedded bitables have a different URL format; this extracts the obj_token
    #[allow(dead_code)]
    pub async fn resolve_wiki_node(auth: &FeishuAuth, wiki_token: &str) -> Result<WikiNode> {
        let token = auth.get_token().await?;
        let client = reqwest::Client::new();
        let url = format!(
            "{}/wiki/v2/spaces/get_node?token={}",
            FEISHU_BASE_URL, wiki_token
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to resolve wiki node")?
            .json::<ApiResponse<WikiNodeData>>()
            .await
            .context("Failed to parse wiki node response")?;

        if resp.code != 0 {
            anyhow::bail!(
                "Failed to resolve wiki node '{}': {} - {}",
                wiki_token,
                resp.code,
                resp.msg
            );
        }

        let node = resp.data.context("No data in wiki response")?.node;
        tracing::info!(
            "Wiki node '{}' -> obj_token={}, obj_type={}, title={}",
            wiki_token,
            node.obj_token,
            node.obj_type,
            node.title
        );
        Ok(node)
    }
}
