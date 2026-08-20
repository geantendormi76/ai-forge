# 🛡️ 紫电 AI (Zidian AI) 工业级工程与商业化全量交接文档 (HANDOFF_TOOL.md)

> **文档版本**：v22.0.0 (2026年8月 4K/8K 图像超分完工、UI 视觉统一与 SOTA 商业代币中台落盘版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级多包 Workspace)  
> **操作系统与环境**：Windows 11 x64 / Developer PowerShell for VS 2022 / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja / RTX 3060 12GB  
> **技术栈**：Tauri v2 + Vue 3 + TypeScript + Rust Cargo Workspace + ONNX Runtime (纯血 Native 架构，0 Python 沙箱，0 端口开销)  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 业务目标与整体定位 (What We Are Doing)

1. **核心定位**：
   - 打造 100% 纯血 Rust Native、本地离线隐私安全、零显存泄漏的顶奢桌面 AI 算力工坊（紫电 AI / Zidian AI）；
   - 商业哲学：深度贯彻钱学森系统工程论 —— **“不执着于单点技术的极致拔尖，而是通过全系统协同优化，让廉价 AI + 极致框架创造最大化生产力”**。
2. **四大主力工坊矩阵**：
   - **4K/8K 图像超分 (`upscale-48k`)**：RealESRGAN 神经网络矩阵直推，4K 极速秒级出图 / 8K 旗舰巨幅重构；
   - **视频字幕生成 (`video-subtitle`)**：MOSS 0.9B ASR 语音识别 + 混元 Hy-MT2 1.8B 神经翻译 + 显卡硬压；
   - **PDF 智能解析 (`pdf-parse`)**：双轨多模态排版重构、公式转写与表格还原；
   - **全能格式转换 (`format-converter`)**：纯血 Rust 音频母带解密、结构数据清洗与格式流转（永久 100% 免费基座）。

---

## 🏛️ 2. 本会话已完成的坚实里程碑 (Accomplished)

### 1. ⚡ `service-upscale` 底座 12 倍性能突破 (116s ➔ 9.6s)
* **病灶破译**：
  - 排查出 ORT 1.20+ 官方将 `ConvAlgorithmSearch::Default` 映射为 `cuDNN Fallback` 慢速模式的致命 Bug；
  - 排查出 Windows 缺少 `cublas64_13.dll` / `cublasLt64_13.dll` 引发静默 CPU 回退的物理原因；
* **实施修复**：
  - 在 `core-onnx-infer` 中将 CUDA 卷积搜索重构为 `ConvAlgorithmSearch::Heuristic`（Mode A 启发式满血加速），关闭动态图 `enable_memory_pattern`；
  - 1:1 外科手术式直译旧版成熟 Commit `776957e` 的 `preprocessor.rs` 与 `postprocessor.rs`（Rayon 多核解码 + 零拷贝 Overlay）；
  - 实测 20 切块图像超分总耗时从 **116.05 秒暴降至 9.62 秒**，0 显存泄漏，0 Fallback 告警！

### 2. 🛡️ `upscale-48k` 独立工具适配层落盘
* 在 `src-tauri/crates/tools/upscale-48k/` 创建独立工具 crate，实现 SRP 单一职责与领域防腐；
* 挂载 `Gatekeeper` 算力收费站与 `VramTokenGuard` 显存守卫。

### 3. 🎨 全套 3D 拟真折角封面 (`cover.svg`) 统一
* 按照 `pdf-parse` 的 16:10 黄金画卷、右上角 3D 翻折角（带物理阴影 `feDropShadow`）与瑞士国际排版，重构了全套工具封面：
  - `pdf-parse/cover.svg`（正红 `PDF`）
  - `video-subtitle/cover.svg`（海青 `ASR`）
  - `format-converter/cover.svg`（流金 `CONV`）
  - `upscale-48k/cover.svg`（电光紫 `8K`）

### 4. 💻 前端工坊视图与交互升级
* **`Upscale48kView.vue`**：极简白盒直觉，仅保留【1. 目标放大倍率（2x/4x/8x）】与【2. 画面规格限制（4K极速/8K满血）】，隐藏底层切块参数；
* **`Sidebar.vue`**：升级为双态智能导轨（展开宽栏 236px $\leftrightarrow$ 迷你纯图标轨 64px），左下方常驻设置与账户卡片；
* **`HomeView.vue`**：左翼升级为 68px 旗舰大字标、舒展金句理念与大胶囊按钮（官网直达 `https://geantendormi.top/` + 反馈）；
* 全局构建 `pnpm build` 达成 **0 Errors 0 Warnings** 绿通！

### 5. 💰 SOTA 商业代币（Tokenomics）与 Beta/GA 平滑切换中台
* 在 `deploy/cloudflare/auth_worker.js` 落盘 v3.0 商业中台：
  - 统一代币：**1 Token = ¥0.01 RMB / $0.001 USD**；
  - 动态策略：`STAGE = "beta" (600点/天)` $\leftrightarrow$ `STAGE = "production" (300点/天)` 1 秒无损热切；
  - 扣费规则：PDF 1点/页、视频 10点/小时、超分 1点/张、格式转换 0点；
  - 优先扣除当日免费额度，不足部分扣减付费余额，老用户创世充值永久保留。

---

## 🔍 3. 当前状态与下一步执行计划 (Current Status & Next Steps)

### 当前停留在：
全栈代码编译绿通，4K/8K 图像超分已全链路挂载就绪，商业代币中台 `auth_worker.js` 已完成落盘。

### 🎯 新会话启动后的下一步计划：
1. **步骤 1：Rust 侧 Gatekeeper 代币扣减点数细化**：
   - 在 `src-tauri/crates/core-security/src/gatekeeper.rs` 中，将各工具扣费参数与 Cloudflare 商业中台精确对齐（PDF 传页数、视频传时长小时数、超分传张数）；
2. **步骤 2：前端个人账户/设置面板（HUD）与额度展示**：
   - 在点击左下角【设置】或【账户】时，弹出精致的“今日剩余可用算力点数（如 280/300 点）”进度条，以及加油包充值入口；
3. **步骤 3：支付中台 Webhook 联调与全量打包验收**：
   - 接入爱发电 / 微信扫码（国内）与 Lemon Squeezy / Stripe（出海）支付回调；
   - 运行 `cargo tauri build` 进行 Windows Native 安装包打包验收。

---

## ⚠️ 4. 绝对血泪避坑铁律 (Strict Anti-Patterns)

1. **终端唯一铁律**：
   - 编译必须唯一使用 **`Developer PowerShell for VS 2022` (`Ctrl+Shift+7`)**，严禁在普通 PowerShell 盲跑 MSVC 编译！
2. **ONNX Runtime cuDNN 卷积算法铁律（GitHub #22705）**：
   - `core-onnx-infer` 中必须使用 **`ConvAlgorithmSearch::Heuristic`**（Mode A 启发式满血加速）！
   - **绝对不能写 `ConvAlgorithmSearch::Default`**，否则会被 ORT 强行降级为 `Fallback` 慢速模式，性能暴跌 10 倍！
3. **动态图像显存模式铁律**：
   - 视觉切块尺寸动态变动时，必须设置 **`enable_memory_pattern: Some(false)`**，防止显存反复打碎重新分配（Memory Thrashing）。
4. **CUDA DLL 依赖链完整性**：
   - `C:\dev\ai-forge\bin\cuda12` 必须同时包含 `onnxruntime.dll`、`onnxruntime_providers_shared.dll`、`onnxruntime_providers_cuda.dll`、`cublas64_13.dll`、`cublasLt64_13.dll`、`cudart64_13.dll`，缺一不可！
5. **落盘协议铁律**：
   - 文件交付必须唯一使用 PowerShell `@' ... '@` 单引号原样字符串与 `Set-Content`，命令行首行绝对不能带 `#` 注释！
