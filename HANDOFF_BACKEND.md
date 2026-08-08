# 🛡️ AI-Forge (AI 桌面工坊) 全量工程交接文档 (HANDOFF.md)

> **生成时间**：2026年8月8日  
> **项目物理绝对路径**：`/home/zhz/ai-forge`  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文接管)

---

## 🏢 1. 项目定位与核心技术栈 (Project Overview)

- **定位**：跨平台 AI 桌面端平台 (Tauri v2 + Vue3)，主打 **100% 本地隐私推理 (Local Inference)** 与 **白嫖 Serverless 鉴权**。
- **UI 风格**：RPA 纯净高雅白调 (`#f8fafc` 浅灰背景、`#ffffff` 圆角卡片、柔和阴影 `shadow-sm`、高度对比排版)。
- **客户端架构**：
  - **前端**：Vue 3 + Vite + TailwindCSS v3 + TypeScript + vue-i18n (中英双语) + Pinia + Lucide 图标 + Vue Bits 动效。
  - **Rust 后端**：Tauri v2 虚拟工作空间 (Cargo Workspace)，包含 `core-ipc`、`core-security`、`core-models-download`、`shared-contracts` 和 `tool-pdf-parse`。
  - **Python 沙箱**：基于 `uv` 构建 100% 物理隔离的虚拟环境，配有 **OS 级文件描述符劫持护盾 (`os.dup2`)**。
  - **通信协议**：绝对禁止使用 TCP/UDS 端口，强制采用 **`Stdio + 4字节大端序长度头 JSON-RPC`**。
  - **离线模型存储**：0 空间损耗软链接 `/home/zhz/ai-forge/models` ➔ `/home/zhz/ai-toolkit/models`。

---

## 🗺️ 2. 整体路线图与当前进度 (Status & Milestones)

### 🟢 阶段一：物理底座与 IPC 通信基建 (100% 完成 ✅)
- `[x] Step 1.1` Tauri v2 + Vue3 + Cargo Workspace 初始化完成。
- `[x] Step 1.2` `core-ipc` crate 完成，实现 `spawn_uv_worker` 绑定沙箱。
- `[x] Step 1.3` Python 侧 `hijack_stdout()` OS 级描述符劫持（`os.dup2(2, 1)`）完成。
- `[x] Step 1.4` TDD Ping-Pong 测试 0.02s 绿通。

### 🟡 阶段二：单工具闭环与模型资产管理 (100% 完成 ✅)
- `[x] Step 2.1` `tool-pdf-parse` 双轨引擎（CPU 矢量轨 `FastTrack` + GPU 视觉轨 `DeepTrack`）搬运并网。在双级裁判评估打分平台 (`test/pdf/evaluate_pdf_strict.py`) 上获得 **AST 标题召回率 100%、KaTeX 100%、水印乱码 0 污染** 的满分表现！
- `[x] Step 2.2` `core-models-download` 模型资产下载器完成，支持 SHA256 秒级校验与多源断点续传（单元测试 100% 绿通）。
- `[x] Step 2.3` `shared-contracts` 端侧显存守卫（`VramTokenGuard`，RTX 3060 11,000MB 池）并网，实现非阻塞 `try_acquire` 显存不足自动降级 CPU 推理，带守卫 E2E 压测绿通！
- `[x] Step 2.4` Tauri Command `parse_pdf` 注册；RPA 纯净白 Vue3 UI（`PdfParseView.vue`）、`vue-i18n` 双语字典、Tailwind 样式、`pnpm build` 100% 类型编译绿通！开发服务器 `http://localhost:1420` 正常运行。

### 🟠 阶段三：SaaS 云端中台与鉴权并网 (当前准备进入 🚀)
- `[ ] Step 3.1` **硬件指纹采集器 (`core-security`)** ➔ **【下会话第一步任务】**
- `[ ] Step 3.2` Cloudflare D1 + Workers Serverless 鉴权中台 (Hono.js)
- `[ ] Step 3.3` Durable Objects (DO) 高并发原子扣减引擎
- `[ ] Step 3.4` 非对称加密离线授权引擎 (Ed25519 `.lic` 验证)
- `[ ] Step 3.5` 客户端算力拦截器 (Gatekeeper)

---

## 📍 3. 当前精确断言位置与下会话第一步计划

- **当前状态**：阶段一、阶段二已全线打通，前端界面 `http://localhost:1420` 渲染完美，Tauri Command `parse_pdf` 联调就绪！
- **下一个明确任务**：**执行 阶段三 [Step 3.1]：开发 `core-security` crate 硬件指纹采集器**。
  - 任务目标：在 `src-tauri/crates/core-security` 中编写 Rust 代码，读取主板 UUID、CPU 序列号、系统盘 GUID，经 HMAC-SHA256 生成不可逆的 `device_fingerprint` 硬件指纹。

---

## ⚠️ 4. 十一条绝对不能再踩的“物理地雷” (Fatal Pitfalls Checklist)

新会话的 AI 必须时刻牢记并严格遵守以下铁律：

1. **绝对禁止使用 TCP/UDS 网络端口**：桌面端跨语言 IPC 必须且只能使用 `core-ipc` 的 `Stdio + 4字节长度头 JSON-RPC`，避免 Windows Defender 弹窗。
2. **`uv` 调起参数铁律**：绝对不能直接执行 `uv script.py`！必须使用 `core_ipc::spawn_uv_worker` 调起命令：`uv run --project <工具沙箱路径> python3 <脚本路径>`。
3. **PEP 508 显卡包防护**：`tool-pdf-parse/pyproject.toml` 中必须保留 `override-dependencies = ["onnxruntime; sys_platform == 'never'"]`，防止纯 CPU 版覆盖 `onnxruntime-gpu`。
4. **软链接自环防线**：重新建立 `models` 软链接前，必须先执行 `rm -rf /home/zhz/ai-forge/models`，绝不能直接重复 `ln -s`，否则会引发无限递归。
5. **Vite 监听防线**：`vite.config.ts` 中必须包含 `server.watch.ignored: ["**/models/**", "**/src-tauri/**", "**/test/**"]`，防止 Node.js 触发 `ELOOP` 崩溃。
6. **Tailwind v3 指令拼写**：`style.css` 中必须使用 `@tailwind base;`（**没有 `s`**），绝不能写成 `@tailwindcss`。
7. **Tailwind Config ESM 规范**：`package.json` 设置了 `"type": "module"`，`tailwind.config.js` 插件必须使用 `import typography from '@tailwindcss/typography'`，不能用 `require()`。
8. **绝对禁止盲猜 Bbox 裁切**：遇到 OCR 乱码或布局异常，**必须先执行轨二探针 (`uv run -c "..."`) 打印真实 Bbox**，获得客观日志后方可修改代码。
9. **Tauri v2 Capability 依赖对齐**：若 `capabilities/default.json` 声明了 `"opener:default"`，`src-tauri/Cargo.toml` 必须包含 `tauri-plugin-opener = "2.0.0"`，否则 `tauri-build` 脚本会报 1 错熔断。
10. **Cargo 库命名对齐**：当 `[package] name = "ai-forge"`（带连字符 `-`）时，`Cargo.toml` 必须显式配置 `[lib] name = "ai_forge_lib"`，与 `src/main.rs` 中的 `ai_forge_lib::run()` 对齐。
11. **pnpm v11 构空白名单**：`package.json` 中配置 `"pnpm": { "onlyBuiltDependencies": ["esbuild", "vue-demi"] }`，并执行 `pnpm approve-builds --all` 避免脚本被拦截。

---

## 🛠️ 5. 新会话快速恢复命令 (Quick Sanity Commands)

新会话开始时，可依次运行以下命令校验环境健康度：

```bash
cd /home/zhz/ai-forge

# 1. 验证 Rust 虚拟工作空间全量编译
cargo check --manifest-path src-tauri/Cargo.toml

# 2. 验证端到端 PDF 解析打靶测试 (含 VRAM 显存守卫)
cargo test --release --manifest-path src-tauri/Cargo.toml -p tool-pdf-parse test_pdf_parse_e2e_with_vram_guard -- --nocapture

# 3. 验证前端类型检查与打包
pnpm build