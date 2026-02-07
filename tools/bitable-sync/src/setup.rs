use anyhow::Result;

use crate::config::Config;
use crate::feishu::auth::FeishuAuth;
use crate::feishu::bitable::{BitableClient, FieldDef};

/// Table schema definition
struct TableSchema {
    name: &'static str,
    view_name: &'static str,
    /// Fields to create with the table (no link fields here)
    fields: Vec<FieldDef>,
    /// Link fields to add after all tables are created (name, linked_table_index)
    links: Vec<(&'static str, usize)>,
}

/// Guide table records (使用说明)
fn guide_records() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"fields": {
            "表名": "品牌表 Brands",
            "用途": "管理黄酒品牌信息。每个品牌一行。",
            "必填字段": "品牌ID、品牌名称",
            "填写说明": "品牌ID格式如 brand_gyl、brand_nz。品牌Logo可直接上传图片附件。品牌故事支持多行文本。"
        }}),
        serde_json::json!({"fields": {
            "表名": "展示分类表 Display Categories",
            "用途": "定义网站首页的商品展示分类（如热销、花雕、礼盒等）。",
            "必填字段": "分类ID、分类名称",
            "填写说明": "分类ID如 hot、huadiao、jiafan、gift。图标填emoji（如🔥、🏺）。排序数字越小越靠前。"
        }}),
        serde_json::json!({"fields": {
            "表名": "商品表 Products",
            "用途": "所有商品的详细信息。每件商品一行，是最核心的数据表。",
            "必填字段": "商品ID、商品名称、零售价",
            "填写说明": "商品ID格式如 P001。品牌和分类通过关联字段选择（不用手填ID）。单位从下拉选项中选。状态: active=上架, inactive=下架, outOfStock=缺货。热销/新品/促销中打勾即可。展示分类填分类ID用逗号分隔（如 hot,huadiao）。商品主图可直接上传图片。"
        }}),
        serde_json::json!({"fields": {
            "表名": "轮播媒体表 Media",
            "用途": "管理首页轮播区的图片和视频素材。",
            "必填字段": "媒体类型 + 文件或外部链接（二选一）",
            "填写说明": "媒体类型选 image 或 video。图片/视频可直接上传到「文件」字段，或填写「外部链接」URL。所属品牌通过关联选择。排序数字越小越靠前。视频可填时长(毫秒)。"
        }}),
        serde_json::json!({"fields": {
            "表名": "店铺信息表 Store Info",
            "用途": "店铺基本信息，只需填一行。",
            "必填字段": "店铺名称、联系电话",
            "填写说明": "只需要一行数据。二维码可直接上传图片附件（微信收款码等）。"
        }}),
        serde_json::json!({"fields": {
            "表名": "⚠️ 注意事项",
            "用途": "数据会自动同步到网站，请谨慎修改。",
            "必填字段": "—",
            "填写说明": "1. 修改后同步工具会自动拉取数据并更新网站\n2. 删除商品前请先将状态改为 inactive\n3. 图片建议尺寸: 商品图 800×800，品牌Logo 400×400，轮播图 1920×1080\n4. 本表（使用说明）不会同步，仅供参考"
        }}),
    ]
}

/// Define all 5 data table schemas + 1 guide table
fn define_schemas() -> Vec<TableSchema> {
    vec![
        // [0] 品牌表 Brands
        TableSchema {
            name: "品牌表 Brands",
            view_name: "全部品牌",
            fields: vec![
                FieldDef::text("品牌ID"),
                FieldDef::text("品牌名称"),
                FieldDef::attachment("品牌Logo"),
                FieldDef::text("品牌故事"),
                FieldDef::number("创立年份", "0"),
                FieldDef::text("产地"),
            ],
            links: vec![],
        },
        // [1] 展示分类表 Display Categories
        TableSchema {
            name: "展示分类表 Display Categories",
            view_name: "全部分类",
            fields: vec![
                FieldDef::text("分类ID"),
                FieldDef::text("分类名称"),
                FieldDef::text("图标"),
                FieldDef::number("排序", "0"),
            ],
            links: vec![],
        },
        // [2] 商品表 Products
        TableSchema {
            name: "商品表 Products",
            view_name: "全部商品",
            fields: vec![
                FieldDef::text("商品ID"),
                FieldDef::text("商品编码"),
                FieldDef::text("商品名称"),
                // brand_id and category_id are added as links after creation
                FieldDef::text("规格"),
                FieldDef::single_select("单位", &["瓶", "箱", "坛", "盒"]),
                FieldDef::number("零售价", "0.00"),
                FieldDef::number("成本价", "0.00"),
                FieldDef::number("会员价", "0.00"),
                FieldDef::number("促销价", "0.00"),
                FieldDef::number("库存", "0"),
                FieldDef::number("酒精度%", "0.0"),
                FieldDef::number("年份", "0"),
                FieldDef::text("酿造工艺"),
                FieldDef::text("风味描述"),
                FieldDef::attachment("商品主图"),
                FieldDef::text("简短描述"),
                FieldDef::text("详细描述"),
                FieldDef::single_select("状态", &["active", "inactive", "outOfStock", "discontinued"]),
                FieldDef::checkbox("热销"),
                FieldDef::checkbox("新品"),
                FieldDef::checkbox("促销中"),
                FieldDef::text("展示分类"),
                FieldDef::number("排序", "0"),
            ],
            links: vec![
                ("品牌", 0),   // -> 品牌表 (index 0)
                ("分类", 1),   // -> 展示分类表 (index 1)
            ],
        },
        // [3] 轮播媒体表 Media
        TableSchema {
            name: "轮播媒体表 Media",
            view_name: "全部媒体",
            fields: vec![
                FieldDef::text("标题"),
                FieldDef::single_select("媒体类型", &["image", "video"]),
                FieldDef::attachment("文件"),
                FieldDef::url("外部链接"),
                FieldDef::number("时长(ms)", "0"),
                FieldDef::number("排序", "0"),
            ],
            links: vec![
                ("所属品牌", 0), // -> 品牌表 (index 0)
            ],
        },
        // [4] 店铺信息表 Store Info
        TableSchema {
            name: "店铺信息表 Store Info",
            view_name: "店铺信息",
            fields: vec![
                FieldDef::text("店铺名称"),
                FieldDef::phone("联系电话"),
                FieldDef::attachment("二维码"),
            ],
            links: vec![],
        },
        // [5] 使用说明 Guide (not synced, for human reference)
        TableSchema {
            name: "使用说明 Guide",
            view_name: "使用说明",
            fields: vec![
                FieldDef::text("表名"),
                FieldDef::text("用途"),
                FieldDef::text("必填字段"),
                FieldDef::text("填写说明"),
            ],
            links: vec![],
        },
    ]
}

/// Create all tables from scratch.
/// Deletes any existing tables first (except the last one which can't be deleted).
pub async fn setup_tables(config: &Config) -> Result<()> {
    let auth = FeishuAuth::new(config.feishu_app_id.clone(), config.feishu_app_secret.clone());
    let client = BitableClient::new(auth, config.bitable_app_token.clone());

    let schemas = define_schemas();

    // Step 1: Create all tables (without link fields)
    tracing::info!("Creating {} tables...", schemas.len());
    let mut table_ids: Vec<String> = Vec::new();

    for schema in &schemas {
        let table_id = client
            .create_table(schema.name, schema.view_name, &schema.fields)
            .await?;
        table_ids.push(table_id);
    }

    // Step 2: Populate guide table with instructions
    let guide_table_id = &table_ids[5]; // index 5 = 使用说明
    let records = guide_records();
    client
        .batch_create_records(guide_table_id, &records)
        .await?;
    tracing::info!("Populated 使用说明 table with {} records", records.len());

    // Step 3: Add link fields (now that all tables exist)
    for (i, schema) in schemas.iter().enumerate() {
        for (field_name, linked_index) in &schema.links {
            let linked_table_id = &table_ids[*linked_index];
            let field = FieldDef::link(field_name, linked_table_id);
            client.create_field(&table_ids[i], &field).await?;
        }
    }

    // Step 4: Delete old tables if any
    let all_tables = client.list_tables().await?;
    for table in &all_tables {
        if !table_ids.contains(&table.table_id) {
            match client.delete_table(&table.table_id).await {
                Ok(_) => tracing::info!("Deleted old table: {} ({})", table.name, table.table_id),
                Err(e) => tracing::warn!("Could not delete table {}: {}", table.table_id, e),
            }
        }
    }

    // Print summary
    println!("\nSetup complete! Table IDs for .env.txt:\n");
    let env_keys = [
        "TABLE_ID_BRANDS",
        "TABLE_ID_DISPLAY_CATEGORIES",
        "TABLE_ID_PRODUCTS",
        "TABLE_ID_MEDIA",
        "TABLE_ID_STORE_INFO",
    ];
    for (key, id) in env_keys.iter().zip(table_ids.iter()) {
        println!("{}={}", key, id);
    }

    println!("\nNote: The order is Brands, Display Categories, Products, Media, Store Info");
    Ok(())
}
