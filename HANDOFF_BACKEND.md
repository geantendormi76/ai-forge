# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v6.0.0 (2026年8月11日 Windows Native 工业级 SSOT 终极交接版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / Windows PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文接管)

---

## 🏢 1. 项目概况与架构宣言 (Project Overview & Manifesto)

1. **物理形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU 动态库直推）。
2. **算力主权与零 Python 演进**：
   - 100% 端侧本地隐私推理 (Local Inference)，算力完全由用户本地 CPU/GPU 承担，数据绝不上云。
   - **2026 SOTA 重磅升级**：废除传统重型 Python (`.venv`) 沙箱与 Stdio/HTTP IPC 跨进程通信，全面切换为 **“Rust 主程序 + C-FFI 直连 `llama.dll` / `ggml-cuda.dll` + GGUF 单文件模型”** 的纯血 Native 架构，达到 **0 MB Python 内存开销、0 网络端口、0 微秒 IPC 延迟**。
3. **通信与显存锁铁律**：
   - 主程序与 C++ 推理引擎之间采用内存原位 FFI 直连；上层受 `shared-contracts` (`VramTokenGuard` 显存锁，RTX 3060 预留 11,000MB 池) 保护。
4. **代码库物理分布**：
   - 主项目物理路径：`C:\dev\ai-forge`
   - MVP 隔离验证路径：`C:\dev\mvp_fx_dll`
   - Rust C-FFI 绑定库：`C:\dev\github\llama-cpp-rs` (包含 `llama-cpp-2` 与 `llama-cpp-sys-2`)
   - C++ 源码母库：`C:\dev\github\llama-cpp-jan` (已全量物理拷贝至 `llama-cpp-sys-2\llama.cpp` 中，实现 100% 独立自包含)

---

## 🗺️ 2. 整体路线图与当前进度 (Roadmap & Status)

### 🟢 阶段一：物理底座与 IPC 通信基建 (100% 完成 ✅)
- `[x] Step 1.1` Tauri v2 + Vue3 + Cargo Workspace 初始化完成。
- `[x] Step 1.2` `core-ipc` crate 完成，实现 `spawn_uv_worker` 绑定沙箱。

### 🟢 阶段二：单工具闭环与模型资产管理 (100% 完成 ✅)
- `[x] Step 2.1` `tool-pdf-parse` 双轨解析引擎完成，打靶测试满分。
- `[x] Step 2.2` `core-models-download` 模型资产下载器完成，支持 SHA256 秒级校验。
- `[x] Step 2.3` `shared-contracts` 端侧显存守卫（`VramTokenGuard`）并网绿通。

### 🟢 阶段三：SaaS 云端中台与鉴权并网 (100% 完成 ✅)
- `[x] Step 3.1` 硬件指纹采集器 (`core-security`) 完成（HMAC-SHA256 生成 64 位指纹）。
- `[x] Step 3.2` Cloudflare D1 + Workers Serverless 鉴权中台完成。
- `[x] Step 3.3` Ed25519 离线授权验签引擎（`LicenseVerifier`）完成。

### 🟢 阶段四：多工具矩阵拓展与纯血 C-FFI Native 重构 (进行中 🚀)
- `[x] Step 4.1` **`service-translation` 纯血 C-FFI GPU Native 重构** ➔ **100% 完成并网 ✅**
  - **成绩**：完全剔除 Python 沙箱与 `worker.py`，单句推演延迟从 500ms 降至 **71ms**，23 句全量英文 ASR JSON 翻译仅耗时 **2.38 秒**！
  - **瘦身**：彻底清除了旧的 `service-translation\.venv` (~1.8 GB) 与 `scripts\` 文件夹，实现零死代码沉淀。
  - **产物**：输出在 `C:\dev\ai-forge\test\outs\service-translation\english_to_chinese_pure.json`。
- `[x] Step 4.2` **`tool-video-subtitle` 紫电双语字幕工坊并网** ➔ **100% 打靶通过 ✅**
  - **成绩**：3 分 11 秒（191 秒）基准视频端到端耗时 91.39 秒（**2.09 倍超实时**），自动无缝享用了新的纯血 GPU 神经翻译底座。
- `[x] Step 4.3` **全局 Cargo Workspace 校验** ➔ **100% 绿通（1 秒闪电完成）✅**
- `[ ] Step 4.4 (下个会话即刻目标)` **`service-asr` (MOSS 0.9B / Whisper GGUF 语音识别) 纯血 C-FFI 重构**：
  - 彻底淘汰 `service-asr` 中的 Python `.venv` (~2.6GB) 与 `scripts/`。
  - 使用 `llama-cpp-2` 内部的 `mtmd.rs` (`MtmdBitmap::from_audio_data`) 在 C++ 显存原位处理 16kHz PCM 音频特征提取与 GGUF ASR 识别。

---

## ⚠️ 3. 八大绝对不能踩的“物理地雷” (Fatal Pitfalls Checklist)

1. **绝对禁止在 PowerShell 中给 `python -c` 传递嵌有双引号的复杂代码**：
   在 PowerShell 命令行单行传递嵌有引号的代码会导致 PowerShell 锁死在续行等待状态（`? |`）。修改代码必须严格遵循【轨一】`Set-Content -Path "..." -Encoding UTF8 -Value @' ... '@` 单引号原样字符串协议落盘脚本后执行。
2. **绝对禁止在没有设置 CMAKE_GENERATOR 时运行编译**：
   CMake 默认会使用慢速且易报错的 `Visual Studio 17 2022` (MSBuild)。**必须在编译命令前显式加上环境变量与 Ninja 路径**：
   `$vsNinja = Get-ChildItem "C:\Program Files*\Microsoft Visual Studio\2022\*\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe" | Select-Object -First 1; $env:PATH = "$(Split-Path $vsNinja.FullName);$env:PATH"; $env:CMAKE_GENERATOR = "Ninja"`
3. **绝对禁止在换生成器时不清理旧缓存**：
   CMake 会将旧生成器名称锁死在 `target\release\build\...\out\build\CMakeCache.txt` 中。更换生成器或报错后，必须先运行 `Remove-Item -Recurse -Force "target"` 清除旧缓存。
4. **绝对禁止忽视 C-API 参数签名变更**：
   - `llama_sampler_init_dry` 接收 **7 个参数**（移除了旧版的 `n_ctx_train` 参数）。
   - `llama_sampler_init_penalties` 接收 **5 个参数**（首位参数为 `n_vocab: i32`）。
5. **绝对禁止不挂载 Jinja Chat Template 和 Penalties 采样器**：
   对 GGUF 神经翻译模型，必须调用 `model.chat_template(None)` 与 `model.apply_chat_template()` 格式化 Prompt，并在采样链首位挂载 `LlamaSampler::penalties(model.n_vocab(), 64, 1.15, 0.0, 0.0)`，否则模型会无限自循环（复读机幻觉）。
6. **绝对禁止在借用迭代器内部借用数组 `.len()` (E0502)**：
   在 `for (idx, seg) in asr_data.segments.iter_mut()` 循环体内不能调用 `asr_data.segments.len()`。必须在进入循环前提前提取 `let total_len = asr_data.segments.len();`。
7. **绝对禁止在 Windows 上使用软链接 (Symlink)**：
   软链接在 Windows Git 与 CI/CD 构建中极易失效。`llama-cpp-jan` 的 C++ 源码必须通过物理拷贝 (`Copy-Item -Recurse -Force`) 实体解压至 `C:\dev\github\llama-cpp-rs\llama-cpp-sys-2\llama.cpp` 中。
8. **绝对在 PowerShell 命令行开头带 `#` 注释**：
   PowerShell 会误解析带 `#` 的多行命令，说明文字写在代码块外部。

---

## 🚀 4. 新会话接管第一步落地计划 (Next Steps)

新会话启动后，请按以下步骤无缝开启 `service-asr` 的纯血 Native C-FFI 重构：

### 第一步：检查 `service-asr` 当前依赖与音频模型路径
在 PowerShell 中运行以下命令，检查 `service-asr` 目录与 ASR GGUF 模型文件：

```powershell
Get-ChildItem "C:\dev\ai-forge\models\tool-ASR" -ErrorAction SilentlyContinue
Get-Content "C:\dev\ai-forge\src-tauri\crates\services\service-asr\Cargo.toml"
```

### 第二步：配置 `service-asr/Cargo.toml` 挂载 `llama-cpp-2`
使用 `Set-Content` 协议将 `service-asr/Cargo.toml` 挂载 `llama-cpp-2` 本地 C-FFI 库与 `cuda` 特性。

### 第三步：重构 `service-asr/src/lib.rs`
使用 `llama-cpp-2` 的 `MtmdContext` 和 `MtmdBitmap::from_audio_data()` 接收 16kHz PCM 音频采样数组，原位 GPU 识别台词，彻底剔除 `service-asr` 中的 Python `.venv` 沙箱与 `scripts/`！
