# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v10.0.0 (2026年8月12日 Windows Native 工业级解耦服务矩阵交接版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文零障碍接管)

---

## 🏢 1. 项目概况与架构宣言 (Project Overview & Manifesto)

1. **物理形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU 动态库直推）。
2. **算力主权与零 Python 演进**：
   - 100% 端侧本地隐私推理 (Local Inference)，算力完全由用户本地 CPU/GPU 承担，数据绝不上云。
   - **2026 SOTA 双语言范式**：废除重型 Python (`.venv`) 沙箱与 Stdio/HTTP IPC 通信，全面切换为纯血 Native 架构：
     - **自回归/多模态模型 (LLM / 神经翻译 / ASR)** ➔ `llama.dll` / `transcribe.dll` C-FFI 直推 + GGUF 单文件。
     - **静态矩阵张量模型 (OCR / 版面分析 / 公式 / 表格)** ➔ `ort` (ONNX Runtime) C-FFI 直推 + ONNX 单文件。
3. **底座单一职责（SRP）与防腐铁律**：
   - 拒绝单体泥潭！每一个底层 Crate (`crates/services/service-xxx`) 必须严格遵循单一职责原则（SRP），仅输出原子级强类型数据结构，**杜绝任何上层 UI / 字幕 / Markdown 格式化领域污染**。

---

## 📐 2. 2026 纯血 Native 解耦原子服务矩阵 (Service Matrix)

我们将臃肿的单体 OCR 解析彻底解耦拆分为 **5 个原子级核心底座 + 1 个排版组装服务**：

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 全新下沉！Workspace 通用 ONNX 硬件会话 Crate】
│   ├── 唯一职责: 通用 ONNX 硬件 Session 生命周期、CUDA/DirectML/CPU 三阶梯降级自愈、零拷贝内存推导
│   └── 绝密优化: 强制 ConvAlgorithmSearch::Default (消灭 cuDNN 每帧穷举卡顿)，DirectML 串行互斥保护
│
├── services\
│   ├── service-ocr\           【纯粹文本 OCR 底座 (100% 完成 ✅)】
│   │   ├── 绑定模型: `PP-OCRv6_small` (`pp-ocrv6_small_det.onnx` + `pp-ocrv6_small_rec.onnx` + `ppocrv6_dict.txt`)
│   │   └── 唯一职责: 图像 ➔ 文字行画框 + 文本识别 (`Vec<TextRegion>`)，0.5秒极速直出
│   │
│   ├── service-layout\        【版面物理分块底座 (即将开始 🎯)】
│   │   ├── 绑定模型: `PP-DocLayoutV3` (`model.onnx` 124MB)
│   │   └── 唯一职责: 图像 ➔ 划定版面物理区块 (`Title`, `Text`, `Table`, `Formula`)
│   │
│   ├── service-formula\       【数学公式识别底座】
│   │   ├── 绑定模型: `PP-FormulaNet-S` (`model.onnx` 294MB)
│   │   └── 唯一职责: 抠图公式图片 ➔ 标准 LaTeX 字符串
│   │
│   └── service-table\         【表格结构解析底座】
│       ├── 绑定模型: `SLANet_plus` (`model.onnx` 7.4MB)
│       └── 唯一职责: 抠图表格图片 ➔ HTML 单元格 Token & 坐标网格
│
└── service-doc-parse / tool-pdf-parse 【IDP 智能排版与文档解析组装服务】
    ├── 内置算法: IDP 核心（`xy_cut` 几何阅读序、`stitch` 段落缝合、`ast` 语法树）
    └── 组合调度: 调起上述 4 个原子底座 ➔ 产出高保真 GFM Markdown
```

---

## 🗺️ 3. 整体路线图与最新进度 (Roadmap & Status)

### 🟢 阶段一~四：底座、鉴权、C-FFI GPU 直推 (100% 完成 ✅)
- `[x]` Tauri v2 + Vue3 + Cargo Workspace 初始化。
- `[x]` `core-models-download` 模型资产断点续传与 SHA256 校验。
- `[x]` `shared-contracts` 端侧显存守卫（`VramTokenGuard`）并网。
- `[x]` `service-translation` 纯血 C-FFI GPU Native 重构（直连 `llama.dll`，单句 71ms）。
- `[x]` `service-asr` 纯血 C-FFI 多模态 GPU Native 重构（直连 `transcribe.cpp`，挂载 MOSS 0.9B `Q5_K_M.gguf` 700MB，1 分钟音频识别 11.04 秒，5.35x 超实时）。

### 🟢 阶段五：`tool-pdf-parse` 解耦拆分与纯血 Native 转换
- `[x] Step 5.1` **5 大视觉模型全量 ONNX 转换打靶与契约固化**（100% 完成 ✅）。
- `[x] Step 5.2` **从 `C:\dev\github\oar-ocr` 提取前后处理 Rust 算子**（100% 完成 ✅）。
- `[x] Step 5.3` **落地下沉 `core-onnx-infer` 与纯文本底座 `service-ocr`**（100% 完成 ✅）：
  - 下沉建立 `crates/core-onnx-infer` Crate，提供全 Workspace 共享的 ONNX 硬件 Session 管理与三阶梯自愈。
  - 完成 `service-ocr`，集成 `PP-OCRv6_small` 为默认底座（物理路径 `C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\`）。
  - 实现动态宽度分桶（Width Bucketing）与微批次（Micro-Batching），消灭 `LightSVTR` 自注意力 $O(W^2)$ 计算量暴胀。
  - 在黄金基准图 `service-ocr.png` 上 100% 物理打靶通过（86 行文本识别率 > 99.5%，0.5 秒极速即出，产物保存于 `test/outs/service-ocr/`）。
  - `cargo check --workspace` **100% 绿通，0 Errors, 0 Warnings**！

- `[ ] Step 5.4` **落地 `service-layout`（版面物理分块底座）** (即将开始 🎯)。

---

## 🛑 4. 当前物理状态 (Current Physical State)

* **全工作空间状态**：`cargo check --manifest-path "C:\dev\ai-forge\src-tauri\Cargo.toml" --workspace` **100% 绿通**！
* **`service-ocr` 单元测试**：运行 `cargo test --manifest-path "C:\dev\ai-forge\src-tauri\Cargo.toml" -p service-ocr --release -- --nocapture` 100% 通过！
* **默认模型物理资产**：
  - 检测模型：`C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_det.onnx` (9.6 MB)
  - 识别模型：`C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_rec.onnx` (20.6 MB)
  - 字符字典：`C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\ppocrv6_dict.txt` (92 KB)
* **打靶产物目录**：`C:\dev\ai-forge\test\outs\service-ocr\`

---

## 🚀 5. 新会话接管第一步落地指令 (Next Steps)

新会话启动后，请新 AI 架构师执行以下 PowerShell 脚本，启动阶段五 Step 5.4，创建 **`crates/services/service-layout`** 原子底座：

```powershell
$env:CMAKE_GENERATOR = "Ninja"

New-Item -ItemType Directory -Force -Path "C:\dev\ai-forge\src-tauri\crates\services\service-layout\src"

Set-Content -Path "C:\dev\ai-forge\src-tauri\crates\services\service-layout\Cargo.toml" -Encoding UTF8 -Value @'
[package]
name = "service-layout"
version = "0.1.0"
edition = "2024"

[lib]
name = "service_layout"
path = "src/lib.rs"

[dependencies]
shared-contracts = { path = "../../shared-contracts" }
core-onnx-infer = { path = "../../core-onnx-infer" }
ndarray = "0.15"
image = "0.25"
rayon = "1.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2"
tracing = "0.1"
'@

Set-Content -Path "C:\dev\ai-forge\src-tauri\crates\services\service-layout\src\lib.rs" -Encoding UTF8 -Value @'
pub struct LayoutService;
'@

cargo check --manifest-path "C:\dev\ai-forge\src-tauri\Cargo.toml" -p service-layout
```

---

## ⚠️ 6. 避坑指南：血泪踩坑绝密清单 (Pitfalls Checklist)

1. **绝对拒绝单体泥潭与领域污染**：
   `service-ocr` 只能做文字画框与文本识别（`Vec<TextRegion>`）；`service-layout` 只能做版面划块。**严禁在底层服务中写任何 Markdown 格式化、PDF 图像提取或表格 HTML 拼装代码**！
2. **`ort-2.0.0-rc.13` 2026 最新官方 API 规范**：
   - **私有方法**：`session.inputs()` 与 `input.name()` 是方法，需加小括号 `()`，不能当作字段访问。
   - **可变借用**：`session.run()` 需要 `&mut self`。
   - **解构提取**：`output_val.try_extract_tensor::<f32>()` 的返回值类型为 `Result<(&Shape, &[f32]), Error>`，直接使用 `let (shape, data) = ...` 解构。
   - **跨 Crate 类型兼容**：向 `Tensor::from_array` 传递 `(shape_vec, data_vec)` 标准库元组（`Vec<usize>, Vec<T>`），彻底隔绝第三方 `ndarray` 版本不一致导致的 `OwnedTensorArrayData` trait mismatch 错误。
3. **`cuDNN` 算法搜索卡顿黑科技 (`ConvAlgorithmSearch::Default`)**：
   在 `core-onnx-infer` 的 CUDA EP 配置中，必须将算法搜索设为 `ConvAlgorithmSearch::Default`。若使用默认的 `Exhaustive`，每当遇到不同宽度的文本行都会触发一次几百毫秒的测速，导致耗时飙升 3 倍！
4. **DirectML 互斥铁律**：
   DirectML EP 必须配置 `parallel_execution = Some(false)`，若开启并行模式，会导致 ONNX Runtime Session 创建时驱动直接崩溃。
5. ** LightSVTR $O(W^2)$ 自注意力防爆（宽度动态分桶）**：
   多行识别时，**绝对不能把所有短图片暴力 Padding 到最长句子的宽度**！必须按实际物理宽度排序，按 6 行一组进行微批次（Micro-Batching）推导，避免自注意力计算量暴胀 256 倍。
6. **PP-OCRv6 字符字典神圣契约**：
   PP-OCRv6 的识别模型必须配合 92KB 的 **`ppocrv6_dict.txt`** 专用字典。若误用旧版 `ppocr_keys_v1.txt`，会导致生成的字符串全盘乱码！
7. **PowerShell Set-Content 落盘协议**：
   凡涉及文件修改，必须使用 `@' ... '@` 单引号原样字符串与 `Set-Content`，严禁在命令行开头带 `#` 注释，严禁向 `python -c` 传递嵌套双引号的长代码。
8. **Windows MSVC + CUDA CMake 编译约束**：
   编译 `llama-cpp-sys-2` 等 C++ 库时，必须在 PowerShell 进程中指定 `$env:CMAKE_GENERATOR = "Ninja"`，防止 MSBuild 提示 `No CUDA toolset found`。
