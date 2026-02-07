# 工作日志

## 2026-02-07 会话记录

### 一、会话概览

| 项目 | 内容 |
|------|------|
| 执行时间 | 2026-02-07 全天 |
| 执行者 | Claude Opus 4.6 |
| 线上地址 | https://llkongs.github.io/family-business/ |
| 起始 commit | `9b7b9f7` docs: 添加项目架构文档 |
| 结束 commit | `b71ff78` feat: add slogans table support |
| 共产出 commit | 11 个 |
| GitHub Actions | 11 次构建，全部成功 |

---

### 二、完整提交时间线

#### commit `4fc94c8` — feat: 三段式布局重构 + HLS视频播放 + bitable-sync工具

按用户提供的计划，将 AdDisplay 页面从纯图片轮播改为三段式布局：

- 安装 `hls.js` 依赖
- 重写 `src/pages/AdDisplay.ts`：Header + Video + Image Carousel + Bottom Bar
- 重写 `src/style.css`：删除旧 Ken Burns / carousel arrows 等样式，新增视频区和三段布局样式
- HLS 播放：Safari 原生 + hls.js 双路径

**环境问题处理：**

1. 本机无 Node.js → `brew install node`（安装 v25.6.0）
2. 外置硬盘 node_modules 文件系统损坏（"Resource busy"）→ 复制项目到 `/tmp/fb-build/` 构建
3. Git push 无认证 → `brew install gh` + `gh auth login --web` 浏览器授权
4. Git 无 author → 仓库级 `git config user.name / user.email`

---

#### commit `1967ac6` — fix: 修正竖屏布局 - 恢复图片轮播和产品入口按钮

用户指出严重问题：屏幕方向错误（不是横屏 1920x1080，而是竖屏 9:16 4K 2160x3840）、视频被静音、图片轮播消失。

修正内容：移除 `muted` 属性、视频区高度改为 `56.25vw`、恢复图片轮播区和底部按钮栏。

---

#### commit `c53efd6` — fix: T001-T010 止血修复 — 播放稳定性 + URL规范化 + 附件落地

按 ARCHITECTURE.md 14 节执行 Phase 0 全部 10 项任务：

| ID | 任务 | 实现方式 |
|---|---|---|
| T001 | 修复 BASE_URL + 绝对URL 拼接 | 新增 `ts_url()` 函数 |
| T002 | URL 规范化函数 + 单元测试 | 6 个 test case |
| T003 | 二维码附件下载到本地 | file_token → `public/images/qrcode.jpg` |
| T004 | 商品主图附件下载 | file_token → `public/images/products/{id}.jpg` |
| T005 | 媒体图片附件下载 | `public/images/media/{slug}.{ext}` |
| T006 | play() 增加 catch 回退 | `tryPlay()` 方法：失败 → muted 重试 |
| T007 | HLS 错误监听恢复 | NETWORK → startLoad、MEDIA → recoverMediaError |
| T008 | 视频 watchdog | 每 5s 检测，3 次卡帧 → skipToNextVideo |
| T009 | 修复删除文件未提交 | `git add -A` 目录级暂存 |
| T010 | 声明密码非安全鉴权 | 注释 + index.html 中文标题 |

---

#### commit `ea81a69` — fix: 修复 QR 码 URL 指向本地 images/qrcode.jpg

修复 mockData.ts 中 qrCodeUrl 仍然拼接飞书 API 的问题，改为本地相对路径。

---

#### commit `6830338` — feat: T011-T020 P1 稳定性增强

按 ARCHITECTURE.md 执行 Phase 1 全部 10 项任务：

| ID | 任务 | 实现方式 |
|---|---|---|
| T011 | sessionStorage → localStorage | `main.ts` 持久化认证状态 |
| T012 | 底部按钮栏 → CTA FAB | 图片区右下角悬浮按钮 |
| T013 | 全局异常兜底 | `window.onerror` + `onunhandledrejection` → 5s 重载 |
| T014 | 定时自动刷新 | 4 小时 `setTimeout → location.reload()` |
| T015 | XSS 防护 | `esc()` + `safeUrlValue()` 应用到 AdDisplay + ProductMenu |
| T016 | 页面 title | 已在 T010 完成 |
| T017 | 4K 大屏适配 | 媒体查询 `@media (min-height: 1800px)` 隔离 4K 尺寸 |
| T018 | CTA 现代化样式 | 去掉 pulse，改为 hover 轻效果 |
| T019 | repo_root 强校验 | `config.validate()` 检查 .git/package.json/src/data/public |
| T020 | 同步日志标准化 | `Instant::now()` + 结构化 summary 字段 |

---

#### commit `f7b7fbc` — fix: CSS 响应式修复

**用户反馈："页面很傻逼，如此明显的错误怎么能看不到呢"**

问题：4K 尺寸（20vh header、100px 字体、500px QR min-height）被设为基础样式，在非 4K 屏幕上完全炸裂。

修复：所有基础样式恢复为合理默认值（8vh header、rem 字体），4K 特定值隔离到 `@media (min-height: 1800px) and (orientation: portrait)` 媒体查询中。

---

#### commit `2aba354` — feat: 静音按钮 + 去掉视频标题 + 品牌图片轮播

用户要求三项改动：

1. **静音按钮**：autoplay 回退到 muted 时显示"点击开启声音"按钮
2. **去掉视频标题**：移除视频下方的标题条（"古越龙山品牌宣传片"没有意义）
3. **品牌图片**：添加 11 张 AI 生成的品牌图片到 mediaPlaylist

改动文件：`mockData.ts`（+11 张图片）、`AdDisplay.ts`（unmute 按钮、移除 video-title）、`style.css`（unmute-btn 样式）

**Git 问题**：外置硬盘 `.git/HEAD.lock` 卡死（"Resource busy"），从此改用 `/tmp/fb-git` 克隆仓库做 commit/push。

---

#### commit `16c5d2e` — fix: 恢复产品菜单入口按钮

**用户反馈："按钮呢按钮呢按钮呢"**

问题：CTA 按钮被条件逻辑隐藏了——`(categories.length > 0 && products.length > 0) ? callback : null`，而数组为空导致 `onEnterMenu = null`，`showCta = false`。

修复：移除条件判断，始终传递回调函数。

---

#### commit `642c079` — fix: 恢复商品分类和产品数据

**用户反馈："商品菜单被删干净了"**

问题：`categories` 和 `products` 数组在 commit `4fc94c8`（bitable-sync 覆写）后变为空数组。

修复：从 commit `a4482c4` 恢复 6 个分类（热销推荐、花雕酒、加饭酒、女儿红、礼盒装、坛装酒）和 10 个产品数据。

---

#### commit `437e65e` — feat: 添加滚动标语栏

用户要求在 header 和视频之间添加滚动标语。

实现：CSS `@keyframes ticker-scroll` 动画驱动 `translateX(-50%)` 无限循环，5 条标语文案（内容重复一份实现无缝滚动），金色文字在深色背景上。

---

#### commit `b71ff78` — feat: add slogans table support for dynamic ticker content

用户要求标语内容从飞书多维表格管理，不再硬编码。

**完整改动链路：**

1. **飞书**：通过 API 创建"标语表 Slogans"（`tbl7wZu8zLY6tHp3`），字段：标语内容（文本）、排序（数字）、启用（复选框），预填 5 条样例
2. **Rust bitable-sync**（8 个文件）：
   - `models/mock_data.rs`：新增 `Slogan` struct
   - `models/bitable_records.rs`：新增 `parse_slogan()` 函数
   - `config.rs`：新增 `TABLE_ID_SLOGANS`
   - `setup.rs`：标语表 schema + `create_slogans_table()` 函数
   - `main.rs`：新增 `AddSlogansTable` CLI 子命令
   - `sync.rs`：读取标语表 + 传递
   - `transform/to_mock_data.rs`：排序 + 组装
   - `output/ts_writer.rs`：生成 `Slogan` interface + `slogans` export
3. **前端**（2 个文件）：
   - `mockData.ts`：新增 `Slogan` 接口 + `slogans` 数组（临时硬编码，sync 后自动覆盖）
   - `AdDisplay.ts`：`import { slogans }` → 动态渲染 ticker，空数组时隐藏

---

### 三、当前线上状态（commit b71ff78）

**已上线功能：**

- 三段式布局：Header → 滚动标语 → 16:9 视频 → 图片轮播
- HLS 视频播放（3 个古越龙山视频循环）+ 自动恢复 + 卡帧检测
- 自动播放 → 失败回退静音播放 + unmute 按钮
- 11 张品牌 AI 图片轮播 + CTA FAB 按钮
- 产品菜单（6 分类 10 产品）
- XSS 防护 + localStorage 持久化 + 4 小时自动刷新 + 异常兜底
- 滚动标语（从 mockData 动态读取，飞书可管理）
- CI 数据校验（飞书 URL 拦截 + 构建产物扫描）

**已知限制：**

| 问题 | 影响 | 对应 TODO |
|------|------|----------|
| productDatabase.json 品牌 logo 仍为飞书 URL | 前端不引用此文件，无运行时影响 | CI warning |
| 浏览器有声自动播放受限 | 首次加载回退为静音 + 显示 unmute 按钮 | Chrome `--autoplay-policy` 参数 |
| launchd 定时同步未配置 | 需手动运行 bitable-sync | T021 |

---

### 四、验证记录

```
# TypeScript 类型检查
npx tsc --noEmit → 无错误

# 前端生产构建
npm run build → ✓ 8 modules transformed

# Rust cargo check
CARGO_TARGET_DIR=/tmp/bitable-sync-target cargo check → ✓ Finished

# Rust 单元测试
cargo test → 7 passed; 0 failed

# 数据校验
bash tools/validate-data.sh → PASSED (1 warning: productDatabase.json logo)

# GitHub Actions
11 次构建全部成功
```

---

### 五、改动文件完整清单

| 文件 | 说明 |
|------|------|
| `docs/ARCHITECTURE.md` | v2.1 架构文档（用户重写 + TODO 状态更新） |
| `docs/VS_CODE_EXECUTION_PLAN.md` | v2 执行方案（全面更新） |
| `docs/WORK_LOG.md` | 本文档 |
| `index.html` | 标题和描述改为中文 |
| `src/main.ts` | localStorage 持久化 + 异常兜底 + 4h 刷新 |
| `src/pages/AdDisplay.ts` | 三段式布局 + HLS + watchdog + XSS + ticker + unmute |
| `src/pages/ProductMenu.ts` | XSS 防护 |
| `src/style.css` | 响应式重构 + 4K 媒体查询 + CTA FAB + ticker + unmute |
| `src/data/mockData.ts` | 品牌图片 + 产品数据恢复 + slogans |
| `.github/workflows/deploy.yml` | 新增数据校验 + 构建产物扫描 |
| `tools/validate-data.sh` | 新增 CI 数据校验脚本 |
| `tools/bitable-sync/src/config.rs` | TABLE_ID_SLOGANS + repo_root 强校验 |
| `tools/bitable-sync/src/main.rs` | AddSlogansTable 子命令 |
| `tools/bitable-sync/src/setup.rs` | 标语表 schema + 创建函数 |
| `tools/bitable-sync/src/sync.rs` | 标语同步 + QR 下载 + 产品图下载 + 日志标准化 |
| `tools/bitable-sync/src/models/mock_data.rs` | Slogan struct |
| `tools/bitable-sync/src/models/bitable_records.rs` | parse_slogan + file_token 提取 |
| `tools/bitable-sync/src/transform/to_mock_data.rs` | 标语排序 + 组装 |
| `tools/bitable-sync/src/output/ts_writer.rs` | Slogan 接口 + slogans export + ts_url |
| `tools/bitable-sync/src/git.rs` | git add -A 目录级暂存 |
| `tools/bitable-sync/src/video.rs` | download_image_attachment + 图片媒体下载 |
