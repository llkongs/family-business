# 工作日志

## 2026-02-07 会话记录

### 一、会话概览

| 项目 | 内容 |
|------|------|
| 执行时间 | 2026-02-07 全天 |
| 执行者 | Claude Opus 4.6 |
| 线上地址 | https://llkongs.github.io/family-business/ |
| 起始 commit | `9b7b9f7` docs: 添加项目架构文档 |
| 结束 commit | `c53efd6` fix: T001-T010 止血修复 |
| 共产出 commit | 3 个 |
| GitHub Actions | 3 次构建，全部成功 |

---

### 二、执行时间线

#### 阶段 1：三段式布局重构

**commit** `4fc94c8` feat: 三段式布局重构 + HLS视频播放 + bitable-sync工具

按用户提供的计划，将 AdDisplay 页面从纯图片轮播改为三段式布局：

- 安装 `hls.js` 依赖（手动写入 package.json，因环境无 npm）
- 重写 `src/pages/AdDisplay.ts`：Header + Video + Image Carousel + Bottom Bar
- 重写 `src/style.css`：删除旧 Ken Burns / carousel arrows 等样式，新增视频区和三段布局样式
- HLS 播放：Safari 原生 + hls.js 双路径

**环境问题处理：**

1. 本机无 Node.js → `brew install node`（安装 v25.6.0）
2. 外置硬盘 node_modules 文件系统损坏（"Resource busy"）→ 复制项目到 `/tmp/fb-build/` 构建
3. Git push 无认证 → `brew install gh` + `gh auth login --web` 浏览器授权
4. Git 无 author → 仓库级 `git config user.name / user.email`

**结果：** 构建成功，部署成功。

#### 阶段 2：用户反馈 + 紧急修正

**commit** `1967ac6` fix: 修正竖屏布局 - 恢复图片轮播和产品入口按钮

用户指出严重问题：

| 问题 | 说明 |
|------|------|
| 屏幕方向错误 | 不是横屏 1920x1080，而是 **竖屏 9:16 4K（2160x3840）** |
| 视频被静音 | 代码加了 `muted` 属性，用户要求有声播放 |
| 图片轮播消失 | 底部图片区和"点击查看产品"按钮被移除 |
| 架构破坏 | 改动过大，未充分阅读原有代码 |

修正内容：

- 移除 `muted` 属性
- 视频区高度改为 `56.25vw`（16:9 视频在竖屏全宽下的正确高度）
- 恢复图片轮播区（含 placeholder）、carousel dots、底部按钮栏
- 恢复 `enter-menu-btn` 的 pulse 动画

**结果：** 构建成功，部署成功。

**遗留问题：** 视频无法自动播放（浏览器 autoplay 策略阻止有声自动播放，play() 无 catch）。

#### 阶段 3：架构审查

用户要求创建"高级架构师"Agent 审查代码。产出 12 点审查意见：

- **P0**：play() 无 catch、无 HLS 错误处理、无 watchdog、无图片数据（mediaPlaylist 图片为 0）、QR URL 拼接错误
- **P1**：ended 事件不可靠、rem 字体在大屏太小、sessionStorage 认证
- **P2**：空产品页、pulse 动画干扰、XSS 风险

#### 阶段 4：架构文档重写

用户多次修正后亲自重写 `docs/ARCHITECTURE.md` v2.1（749 行），包含：

- As-Is / To-Be 架构
- 4K 竖屏精确尺寸（Header 760px / Video 1215px / Image+CTA 1865px）
- 二维码 2 米扫码规范（边长 580-620px，静区 32px）
- CTA 悬浮 FAB 规范（替代底部按钮栏）
- 35 个 TODO（T001-T035），分 P0/P1/P2 三级
- 开发/上线/回归/巡检/应急 五套 Checklist

#### 阶段 5：T001-T010 止血修复

**commit** `c53efd6` fix: T001-T010 止血修复 — 播放稳定性 + URL规范化 + 附件落地

按 ARCHITECTURE.md 14 节的 TODO 列表执行 Phase 0 全部 10 项任务。

---

### 三、T001-T010 完成明细

| ID | 优先级 | 模块 | 任务 | 实现方式 | 改动文件 | 状态 |
|---|---|---|---|---|---|---|
| T001 | P0 | 同步工具 | 修复 BASE_URL + 绝对URL 拼接 | 新增 `ts_url()` 函数：绝对 URL 输出为 `'https://...'`，相对路径输出为 `` `${BASE_URL}...` `` | `ts_writer.rs` | DONE |
| T002 | P0 | 同步工具 | URL 规范化函数 + 单元测试 | `ts_url()` 含 6 个 test case：相对路径、子目录、https、http、空字符串、含单引号 | `ts_writer.rs` | DONE |
| T003 | P0 | 同步工具 | 二维码附件下载到本地 | 提取 file_token → 下载到 `public/images/qrcode.jpg` → qr_code_url 指向本地路径；下载失败回退已有本地文件 | `bitable_records.rs` `mock_data.rs` `sync.rs` `video.rs` | DONE |
| T004 | P0 | 同步工具 | 商品主图附件下载 | 提取 main_image_file_token → 下载到 `public/images/products/{id}.jpg`；已存在则跳过 | `bitable_records.rs` `sync.rs` | DONE |
| T005 | P0 | 同步工具 | 媒体图片附件下载 | `process_media_items` 中 image 分支：有附件则下载到 `public/images/media/{slug}.{ext}`，URL 指向本地 | `video.rs` | DONE |
| T006 | P0 | 前端 | play() 增加 catch 回退 | 新增 `tryPlay()` 方法：play() 失败 → `muted=true` + retry → 二次失败 console.error | `AdDisplay.ts` | DONE |
| T007 | P0 | 前端 | HLS 错误监听恢复 | `Hls.Events.ERROR` 监听：NETWORK_ERROR → `startLoad()`，MEDIA_ERROR → `recoverMediaError()`，其它 fatal → `skipToNextVideo()` | `AdDisplay.ts` | DONE |
| T008 | P0 | 前端 | 视频 watchdog | 每 5s 检测 `currentTime`，连续 3 次未推进 → `skipToNextVideo()`；加载新视频时重置计数 | `AdDisplay.ts` | DONE |
| T009 | P0 | 同步工具 | 修复删除文件未提交 | `commit_and_push()` 对 `src/data`、`public/videos`、`public/images` 三个目录执行 `git add -A`（追踪新增+删除），其余文件仍逐个 add | `git.rs` | DONE |
| T010 | P0 | 安全/文档 | 声明密码非安全鉴权 | main.ts 密码常量处增加中文注释说明仅防误触；index.html title 改为"伟盛酒业"、description 改为中文 | `main.ts` `index.html` | DONE |

---

### 四、当前线上状态

**已上线（c53efd6）：**

- 三段式布局：Header → 16:9 视频 → 图片轮播/占位 → 底部按钮
- HLS 视频播放（3 个古越龙山视频循环）
- 自动播放 → 失败回退静音播放（不再卡死黑屏）
- HLS 错误自动恢复 + 卡帧 15s 自动跳转
- 页面标题"伟盛酒业"

**已知未修复问题（线上可见）：**

| 问题 | 原因 | 影响 | 对应后续 TODO |
|------|------|------|--------------|
| QR 码不显示 | `mockData.ts` 第 39 行仍为 `${BASE_URL}https://open.feishu.cn/...`（当前数据文件未被 sync 工具重新生成） | Header 右侧 QR 码 broken image | 需跑一次 `bitable-sync sync` 重新生成 mockData.ts |
| 图片轮播为空 | `mediaPlaylist` 中无 type=image 项（品牌图存在磁盘但未加入 playlist） | 底部显示"宣传图片即将上线"占位 | 需跑一次 sync 或手动添加 |
| 产品页为空 | `categories` 和 `products` 数组为空 | 点击"查看产品"进入空白菜单页 | 需在飞书填入数据后 sync |
| 浏览器有声自动播放受限 | 非 Kiosk 模式下浏览器阻止有声自动播放 | 首次加载回退为静音播放 | 展示屏使用 Chrome `--autoplay-policy=no-user-gesture-required` 参数启动 |

**说明：** T001-T005 和 T009 的改动在同步工具侧（Rust 代码），需要下次执行 `bitable-sync sync` 时才会重新生成 `mockData.ts`，届时 QR URL 和图片路径才会在数据文件中生效。当前线上的 `mockData.ts` 是之前 sync 产出的旧版本。

---

### 五、验证记录

```
# 前端 TypeScript 类型检查
cd /tmp/fb-build && npx tsc --noEmit
→ 无错误

# 前端生产构建
cd /tmp/fb-build && npm run build
→ ✓ 8 modules transformed, built in 540ms

# Rust cargo check
CARGO_TARGET_DIR=/tmp/bitable-sync-target cargo check
→ ✓ Finished dev profile

# Rust 单元测试
CARGO_TARGET_DIR=/tmp/bitable-sync-target cargo test
→ running 7 tests — 7 passed; 0 failed

# GitHub Actions 构建部署
→ completed success, 51s
```

---

### 六、改动文件完整清单

| 文件 | 行数变化 | 说明 |
|------|---------|------|
| `docs/ARCHITECTURE.md` | +816 -94 | 用户重写的 v2.1 架构文档 |
| `index.html` | +2 -2 | 标题和描述改为中文 |
| `src/main.ts` | +4 -1 | 密码注释声明 |
| `src/pages/AdDisplay.ts` | +84 行变更 | play() catch + HLS 错误恢复 + watchdog |
| `tools/bitable-sync/src/output/ts_writer.rs` | +71 行变更 | ts_url() + 6 个单元测试 |
| `tools/bitable-sync/src/git.rs` | +25 行变更 | git add -A 目录级暂存 |
| `tools/bitable-sync/src/models/bitable_records.rs` | +16 | extract_attachment_file_token + file_token 字段 |
| `tools/bitable-sync/src/models/mock_data.rs` | +3 | StoreInfo.qr_file_token |
| `tools/bitable-sync/src/sync.rs` | +55 行变更 | QR 下载 + 产品图下载 |
| `tools/bitable-sync/src/video.rs` | +53 行变更 | download_image_attachment + 图片媒体附件下载 |

---

### 七、后续工作建议

#### 立即可做

1. **运行一次 bitable-sync sync**：让 T001-T005 的改动生效，重新生成 mockData.ts，修复 QR URL 和填充图片数据。
2. **在飞书媒体表添加品牌图片记录**：让 mediaPlaylist 包含 type=image 项，底部轮播区才有内容。

#### Phase 1（T011-T025）优先项

- T012: 底部按钮栏 → CTA 悬浮 FAB（架构文档 4.6 节已有详细规范）
- T011: sessionStorage → localStorage（设备重启免输密码）
- T013+T014: 全局异常兜底 + 定时自动刷新（24x7 稳定性）
- T017: 4K 大屏字体和二维码尺寸调整

#### 技术债

- `mockData.ts` 中接口定义与 `src/data/types.ts` 存在重复，应统一
- `AdDisplay.ts` 和 `ProductMenu.ts` 使用 innerHTML 拼接外部数据，存在 XSS 风险（T015）
- 视频文件 239MB 在 Git 仓库中，长期会导致历史膨胀（T026）
