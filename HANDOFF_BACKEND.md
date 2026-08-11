# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v7.0.0 (2026年8月12日 Windows Native 工业级 SSOT 终极交接版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文接管)

---

## 🏢 1. 项目概况与架构宣言 (Project Overview & Manifesto)

1. **物理形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU 动态库直推）。
2. **算力主权与零 Python 演进**：
   - 100% 端侧本地隐私推理 (Local Inference)，算力完全由用户本地 CPU/GPU 承担，数据绝不上云。
   - **2026 SOTA 重磅升级**：废除传统重型 Python (`.venv`) 沙箱与 Stdio/HTTP IPC 跨进程通信，全面切换为 **“Rust 主程序 + C-FFI 直连 `transcribe.dll` / `llama.dll` / `onnxruntime.dll` + GGUF/ONNX 模型”** 的纯血 Native 架构，达到 **0 MB Python 内存开销、0 网络端口、0 微秒 IPC 延迟**。
3. **底座单一职责与防腐铁律**：
   - `crates/services/*` 为纯粹共享底座，仅输出原子级强类型数据结构（如 `RawSegment` / `translations`），杜绝任何上层 UI/字幕格式化领域污染。
   - 上层 `crates/tools/*` 调起底座服务做具体业务编排。
4. **代码库物理分布**：
   - 主项目物理路径：`C:\dev\ai-forge`
   - C++ 语音引擎源码：`C:\dev\github\transcribe.cpp`
   - C++ 翻译引擎源码：`C:\dev\github\llama-cpp-rs`

---

## 🗺️ 2. 整体路线图与当前进度 (Roadmap & Status)

### 🟢 阶段一：物理底座与 IPC 通信基建 (100% 完成 ✅)
- `[x] Step 1.1` Tauri v2 + Vue3 + Cargo Workspace 初始化完成。
- `[x] Step 1.2` `core-ipc` crate 完成。

### 🟢 阶段二：单工具闭环与模型资产管理 (100% 完成 ✅)
- `[x] Step 2.1` `core-models-download` 模型资产下载器完成，支持 SHA256 秒级校验。
- `[x] Step 2.2` `shared-contracts` 端侧显存守卫（`VramTokenGuard`）并网绿通。

### 🟢 阶段三：SaaS 云端中台与鉴权并网 (100% 完成 ✅)
- `[x] Step 3.1` 硬件指纹采集器 (`core-security`) 完成（HMAC-SHA256 生成 64 位指纹）。
- `[x] Step 3.2` Cloudflare D1 + Workers Serverless 鉴权中台完成。
- `[x] Step 3.3` Ed25519 离线授权验签引擎（`LicenseVerifier`）完成。

### 🟢 阶段四：纯血 C-FFI Native 重构 (100% 完成 ✅)
- `[x] Step 4.1` **`service-translation` 纯血 C-FFI GPU Native 重构** ➔ **100% 完成 ✅**
  - **成绩**：直连 `llama.dll`，单句推演低至 71ms，零 Python 依赖。
- `[x] Step 4.2` **`service-asr` 纯血 C-FFI 多模态 GPU Native 重构** ➔ **100% 完成 ✅**
  - **成绩**：直连 `transcribe.cpp`，挂载 `MOSS-Transcribe-Diarize-Q5_K_M.gguf` (700 MB)，1 分钟音频识别与角色分离仅用 11.04 秒（**5.35 倍超实时**）！
  - **瘦身**：彻底删除旧 `.venv`（**释放 2.6 GB 空间**）、`scripts/` 与 `pyproject.toml`，0 MB Python 依赖。
- `[x] Step 4.3` **`tool-video-subtitle` 上层工具双底座集成打靶** ➔ **100% 绿通 ✅**
  - **成绩**：48 句台词听写+翻译+ASS渲染+MKV封装仅用 17.55 秒。
- `[x] Step 4.4` **全局 Workspace 编译检查 (`cargo check --workspace`)** ➔ **100% 绿通 ✅**

### 🚀 阶段五：下个会话即刻目标——`tool-pdf-parse` 解耦重构与 C-FFI/ONNX 转化
- `[ ] Step 5.1` **拆分底座服务 (`service-ocr` / `service-doc-parse`) 与上层工具 (`tool-pdf-parse`)**。
- `[ ] Step 5.2` **版面分析与 OCR 模型的纯血 ONNX Native 重构**：
  - 针对 `C:\dev\ai-forge\models\tool-pdf-parse` 下的 5 大模型（`PP-DocLayoutV3`, `PP-FormulaNet-S`, `PP-OCRv6_medium_det`, `PP-OCRv6_medium_rec`, `SLANet_plus`）。
  - 使用 2026 静态图 SOTA 范式：转换为 ONNX 格式，通过 Rust `ort` 库 (ONNX Runtime C-FFI) 在 C++ 显存原位推理，彻底清空 `tool-pdf-parse` 下最后一个 `.venv` 沙箱！

---

## ⚠️ 3. 八大绝对不能踩的“物理地雷” (Fatal Pitfalls Checklist)

1. **绝对禁止在 PowerShell 中给 `python -c` 传递嵌有双引号的复杂代码**：
   会导致 PowerShell 锁死在续行等待状态（`? |`）。必须使用【轨一】`Set-Content -Path "..." -Encoding UTF8 -Value @' ... '@` 单引号原样字符串协议。
2. **绝对禁止在没有设置 CMAKE_GENERATOR 时运行 CMake 编译**：
   必须在编译命令前显式加上环境变量与 Ninja 路径：
   `$vsNinja = Get-ChildItem "C:\Program Files*\Microsoft Visual Studio\2022\*\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe" | Select-Object -First 1; $env:PATH = "$(Split-Path $vsNinja.FullName);$env:PATH"; $env:CMAKE_GENERATOR = "Ninja"`
3. **绝对禁止遗漏 MSVC CUDA 运行库链接 (`build.rs`)**：
   Windows MSVC 链接包含 CUDA 的静态库时，`build.rs` 必须输出 `cargo:rustc-link-search=native=CUDA_PATH/lib/x64` 并链接 `cudart`、`cublas`、`cuda`，否则抛出 `error LNK2019: cudaFuncSetAttribute`。
4. **绝对禁止底座服务（`crates/services/*`）包含领域污染参数**：
   底座服务（如 `service-asr`）只对齐官方模型标准参数，只输出原子级 `RawSegment`；SRT/ASS 格式化与业务逻辑由上层 `tool-*` 承担。
5. **绝对禁止在命令行开头带 `#` 注释**：
   PowerShell 会误解析带 `#` 的多行命令，说明文字写在代码块外部。
6. **编译命令必须显式指定 Cargo 清单路径**：
   在编译命令中必须显示指定 `--manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml`。
7. **`target/debug` 与 `target/release` 双目录隔离**：
   首次带 `--release` 标志编译 C++/CUDA 代码会在 `target/release` 建立新地基（耗时约 4 分钟），后续二次运行耗时变为 `< 0.5s` 增量加载。
8. **模型物理路径分布**：
   - ASR GGUF 模型：`C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-Q5_K_M.gguf` (700 MB)
   - 翻译 GGUF 模型：`C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf` (1.1 GB)
   - PDF/OCR 模型目录：`C:\dev\ai-forge\models\tool-pdf-parse`

---

## 🚀 4. 新会话接管第一步落地计划 (Next Steps)

新会话启动后，请按以下步骤无缝开启 `tool-pdf-parse` 的解耦重构：

### 第一步：检查 `tool-pdf-parse` 源码与模型文件结构
在 PowerShell 中运行以下命令，检查 `tool-pdf-parse` 目录：

```powershell
Get-ChildItem "C:\dev\ai-forge\src-tauri\crates\tools\tool-pdf-parse"
Get-ChildItem "C:\dev\ai-forge\models\tool-pdf-parse"
```

### 第二步：设计 `service-ocr` / `service-doc-parse` 共享底座
创建 `crates/services/service-ocr`，挂载 Rust `ort` (ONNX Runtime) C-FFI 绑定，准备接入 ONNX 格式版面分析与 OCR 模型。

### 第三步：清理 Python 沙箱并测试打靶
将 `tool-pdf-parse` 中的 Python 脚本替换为 `service-ocr` 底座调用，删除 `tool-pdf-parse/.venv`，实现全软件 100% 纯血 C-FFI Native 化！
