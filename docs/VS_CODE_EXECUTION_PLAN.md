# VS Code 执行方案（v2 — 2026-02-07 更新）

## 1. 文档目的

指导执行者在 VS Code 中完成以下目标：

1. 飞书数据拉取后，前端展示仅依赖 GitHub Pages 静态资源。
2. 页面在门店 55 寸 4K 竖屏长期运行稳定（视频、二维码、图片、CTA）。
3. 同步、构建、部署均可验证、可回滚、可交接。

本方案以 `docs/ARCHITECTURE.md` 为总规范。

---

## 2. 当前完成状态

### 2.1 已完成（T001-T020 + T023/T024 + T036）

| 阶段 | 任务 | 状态 |
|------|------|------|
| P0 止血修复 | T001-T010（URL 规范化、附件本地化、播放稳定性、安全声明） | **DONE** |
| P1 前端增强 | T011-T018（localStorage、CTA FAB、异常恢复、定时刷新、XSS 防护、4K 适配） | **DONE** |
| P1 同步增强 | T019（repo_root 强校验）、T020（日志标准化） | **DONE** |
| P1 CI | T023（数据完整性检查）、T024（构建产物校验） | **DONE** |
| 新增功能 | T036（飞书标语表 → 前端滚动 ticker） | **DONE** |
| 新增功能 | 静音按钮（autoplay 回退时显示） | **DONE** |
| 新增功能 | 品牌 AI 图片轮播（11 张） | **DONE** |

### 2.2 未完成待办

| ID | 优先级 | 模块 | 任务 | 说明 |
|----|--------|------|------|------|
| T021 | P1 | 运维 | launchd 定时同步 | Mac mini 重启后自动定时运行 bitable-sync |
| T022 | P1 | 运维 | 同步失败告警脚本 | 连续失败 N 次发通知 |
| T025 | P1 | 数据 | 字段字典与命名规范 | 飞书字段变更有映射文档 |
| T026 | P2 | 架构 | 媒体存储分层评估 | Git vs 对象存储的容量规划 |
| T027 | P2 | 架构 | 页面配置化 | 布局比例、CTA 位置可调 |
| T028 | P2 | 前端 | 图片轮播淡入淡出 | 可配置切换动画 |
| T029 | P2 | 前端 | 产品页超时自动返回 | 无操作超时回广告页 |
| T030 | P2 | 数据 | 商品字段校验 | 异常数据在同步时识别 |
| T031 | P2 | 文档 | 故障案例模板 | 故障复盘可直接使用 |
| T032 | P2 | 运维 | 仓库体积巡检 | 超阈值提示清理 |
| T033 | P2 | 安全 | Cloudflare Access 评估 | 成本/收益分析 |
| T034 | P2 | 质量 | bitable-sync 单元测试 | 核心转换覆盖率 |
| T035 | P2 | 质量 | 前端 e2e 脚本 | 视频/图片/页面跳转自动验收 |

---

## 3. 核心架构：数据流与静态资源保证

```
飞书多维表格（6 张表）
    │  品牌表、展示分类表、商品表、轮播媒体表、店铺信息表、标语表
    ▼
bitable-sync (Rust CLI)     ← 在 Mac mini 上定时运行
    │  1. 读取飞书 API
    │  2. 下载附件 → public/images/ 和 public/videos/
    │  3. 生成 mockData.ts + productDatabase.json
    │  4. git commit + push
    ▼
GitHub Actions (CI)
    │  1. npm ci
    │  2. validate-data.sh  ← 拦截飞书临时 URL
    │  3. npm run build (tsc + vite)
    │  4. 校验构建产物无飞书 URL
    │  5. deploy to GitHub Pages
    ▼
GitHub Pages (静态托管)
    │  所有 HTML/JS/CSS/图片/视频 = 静态文件
    │  前端浏览器不调用任何飞书 API
    ▼
门店 55 寸 4K 竖屏 (Chrome Kiosk)
```

**关键保证：前端网页不可能也不需要读飞书 API。** 所有数据在 bitable-sync 运行时就已经下载为本地静态文件，通过 Git 推送到 GitHub Pages。

---

## 4. 环境准备

### 4.1 依赖

```bash
node -v      # >= 20
npm -v
cargo -V     # Rust
ffmpeg -version  # 视频转 HLS
```

### 4.2 安全配置

- `tools/bitable-sync/.env.txt` 存在且 **不入库**（已 gitignore）
- 包含：`FEISHU_APP_ID`、`FEISHU_APP_SECRET`、`BITABLE_APP_TOKEN`
- 6 个 TABLE_ID：PRODUCTS、BRANDS、DISPLAY_CATEGORIES、MEDIA、STORE_INFO、SLOGANS

### 4.3 构建验证

```bash
# 前端
npm ci && npm run build

# 同步工具（外置硬盘需要 CARGO_TARGET_DIR）
cd tools/bitable-sync
CARGO_TARGET_DIR=/tmp/bitable-sync-target cargo check

# 数据校验
bash tools/validate-data.sh
```

---

## 5. 日常操作流程

### 5.1 修改飞书数据后同步

```bash
cd tools/bitable-sync
cargo run -- sync --no-push   # 先不推送，本地验证
# 检查 src/data/mockData.ts 和 public/images/
cargo run -- sync              # 正式同步 + git commit + push
```

### 5.2 修改前端代码

```bash
npm run dev          # 开发服务器
# 修改代码...
npm run build        # 构建验证
bash tools/validate-data.sh  # 数据校验
git add ... && git commit && git push
```

### 5.3 添加新飞书表

参考 T036（标语表）实现模式：
1. `setup.rs` 添加表 schema + 创建函数
2. `config.rs` 添加 TABLE_ID
3. `bitable_records.rs` 添加解析函数
4. `mock_data.rs` 添加数据结构
5. `sync.rs` 添加读取和传递
6. `to_mock_data.rs` 添加排序和组装
7. `ts_writer.rs` 添加 TypeScript 输出
8. 前端 `mockData.ts` 和页面组件对接

---

## 6. 后续执行计划

### Phase E：运维自动化（T021/T022）

**目标：** Mac mini 无人值守自动同步

1. **T021 — launchd 定时任务**
   - 创建 `~/Library/LaunchAgents/com.leonkong.bitable-sync.plist`
   - 每 2-4 小时运行一次 `bitable-sync sync`
   - 日志输出到 `~/Library/Logs/bitable-sync.log`

2. **T022 — 失败告警**
   - 监控 sync 退出码
   - 连续 3 次失败发送通知（macOS 通知 / 飞书 webhook）

### Phase F：数据规范化（T025）

**目标：** 飞书字段与 Rust/TypeScript 之间有明确映射文档

### Phase G：P2 优化（T026-T035）

按需挑选，非紧急。

---

## 7. CI 门禁说明

当前 `.github/workflows/deploy.yml` 包含两道校验：

### 7.1 Pre-build: `validate-data.sh`

- 检查 `mockData.ts` 不含飞书临时 URL（**error** = 阻断部署）
- 检查 `productDatabase.json` 不含飞书临时 URL（**warning** = 允许但提示）
- 检查 mockData.ts 结构完整（storeInfo/mediaPlaylist/categories/products/slogans）
- 检查静态资源文件存在（QR 码、HLS 视频）

### 7.2 Post-build: 构建产物扫描

- 扫描 `dist/` 内所有文件，出现 `open.feishu.cn` 立即失败

---

## 8. 验收清单

- [x] `mockData.ts` 不包含飞书临时链接
- [x] `public/images` 有二维码、商品图、媒体图
- [x] `public/videos` 有 HLS 资源且可播放
- [x] 前端构建通过
- [x] sync check/dry-run/no-push 通过
- [x] 页面长时间运行可恢复（onerror + 4h 刷新）
- [x] CI 数据校验阻断飞书 URL 泄露
- [x] XSS 防护（esc + safeUrlValue）
- [x] 标语从飞书表动态读取
- [ ] launchd 定时任务配置（T021）
- [ ] 同步失败告警（T022）
- [ ] productDatabase.json 品牌 logo 本地化（现为 warning）

---

## 9. 回滚方案

1. 保留每阶段 commit。
2. 若线上异常：`git revert HEAD && git push`，触发重新部署。
3. 回滚后最小验证：页面可打开、视频播放、二维码可扫、CTA 可点击。

---

## 10. 文档关联

| 文档 | 用途 |
|------|------|
| `docs/ARCHITECTURE.md` | 总体架构与 TODO 列表（35 + 1 项） |
| `docs/WORK_LOG.md` | 完整执行记录（所有 commit 说明） |
| `docs/VS_CODE_EXECUTION_PLAN.md` | 本文档 — 执行方案与当前状态 |
| `tools/validate-data.sh` | CI 数据校验脚本 |
| `tools/bitable-sync/.env.txt` | 飞书 API 配置（不入库） |
