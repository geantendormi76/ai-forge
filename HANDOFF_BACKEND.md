# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v13.0.0 (2026年8月13日 阶段五 Step 5.5 数学公式识别纯血 ONNX 底座大捷版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文零障碍接管)

---

## 🏢 1. 项目概况与架构宣言 (Project Overview & Manifesto)

1. **物理形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU 动态库直推）。
2. **算力主权与零 Python 演进**：
   - 100% 端侧本地隐私推理 (Local Inference)，算力完全由用户本地 CPU/GPU 承担，数据绝不上云。
   - **2026 SOTA 双语言范式**：废除重型 Python (`.venv`) 沙箱与 Stdio/HTTP IPC 通信，全面切换为纯血 Native 架构：
     - **自回归/多模态模型 (LLM / 神经翻译 / ASR)** ➔ `llama.dll` / `transcribe.dll` C-FFI 直推。
     - **静态矩阵张量模型 (OCR / 版面分析 / 公式识别)** ➔ `ort` (ONNX Runtime) C-FFI 直推 + ONNX 单文件 + FastTokenizer。
3. **底座单一职责（SRP）与防腐铁律**：
   - 每一个底层 Crate (`crates/services/service-xxx`) 必须严格遵循单一职责原则（SRP），仅输出原子级强类型数据结构，**杜绝任何上层 UI / 字幕 / Markdown 格式化领域污染**。

---

## 📐 2. 2026 纯血 Native 解耦原子服务矩阵 (Service Matrix)

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 全新下沉！Workspace 通用 ONNX 硬件会话 Crate】
│   ├── 唯一职责: 通用 ONNX 硬件 Session 生命周期、CUDA/DirectML/CPU 三阶梯降级自愈、零拷贝内存推导
│   └── 绝密优化: 强持多输入智能投料 (image, im_shape, scale_factor)、2D/3D/2D-i64 输出自动解构
│
├── services\
│   ├── service-ocr\           【纯粹文本 OCR 底座 (100% 完成 ✅)】
│   │   └── 唯一职责: 图像 ➔ 文字行画框 + 文本识别 (`Vec<TextRegion>`)
│   │
│   ├── service-layout\        【版面物理分块底座 (100% 完成 ✅)】
│   │   └── 唯一职责: 图像 ➔ 划定版面物理区块 (`Title`, `Text`, `Table`, `Formula` ... 24 种类别)
│   │
│   ├── service-formula\       【数学公式识别底座 (100% 攻坚完成 ✅)】
│   │   ├── 物理模型: `PP-FormulaNet-S` (221.14 MB ONNX 单体模型) + `tokenizer.json`
│   │   ├── 硬件算子: `core-onnx-infer` + FastTokenizer + `OnceLock` 线程安全常驻单例
│   │   └── 物理性能: 100% 还原 LaTeX (如 $P(y|x_u)=f_{\mathrm{T}}(x_u;\theta_T)$)，常驻响应耗时 ~41 ms！
│   │
│   └── service-table\         【表格结构解析底座 (下一步攻坚目标 🎯)】
│       └── 唯一职责: 抠图表格图片 ➔ HTML 单元格 Token & 坐标网格 (`SLANet_plus` 7.4 MB ONNX)
│
└── service-doc-parse / tool-pdf-parse 【IDP 智能排版与文档解析组装服务】
    └── 组合调度: 调起上述 4 个原子底座 ➔ 产出高保真 GFM Markdown
```

---

## 🟢 3. 本会话大捷成果总结 (What Was Completed)

1. **攻克 `PP-FormulaNet-S` ONNX 导出与控制流崩溃**：
   - 探针白盒证实：`paddle2onnx 2.1.0` 在导出飞桨 3.0 PIR 控制流时，导出的 `Loop.0` / `Identity.669` 会将内部节点误标为 `{}` 0维标量而动态吐出 `{1}` 1维向量，触发 ONNXRuntime `VerifyOutputSizes` 拦截。
   - **黄金药方**：使用 **`paddle2onnx==2.0.2rc1`** 重新导出，成功生成 221.14 MB 的完美静态图 `model.onnx`，ONNXRuntime 直推零 Error。
   - 从 `config.json` 的 `PostProcess.character_dict.fast_tokenizer_file` 中抽取出 100% 官方匹配的 HuggingFace `tokenizer.json`，消灭了占位符假字典。

2. **`service-formula` 纯血算子重构与极速性能优化**：
   - 修正 `Cargo.toml` 声明为 `tokenizers = { version = "0.19", default-features = false, features = ["onig"] }`，剔除 `esaxx_fast`，压制了 C++ `esaxx-rs` `/MT` 与 `ort` `/MD` 的链接冲突。
   - 引入 `std::sync::OnceLock<Mutex<FormulaEngine>>` 单例常驻缓存，消灭了重复读盘与 Session 重复初始化开销。
   - **打靶断言**：真实图片 `1.png` 识别产出 `P(y|x_{u})=f_{\mathrm{T}}(x_{u};\theta_{T})`，字符 100% 准确对齐，温启动响应狂飙至 ~41 ms！

3. **编译期 22.4 GB 内存暴涨根治**：
   - 配置 `.cargo/config.toml` 的 `jobs = 4` 与 `strip = true`，限制了 MSVC `link.exe` 的并发链接占用，将编译期内存峰值从 22.4 GB 成功压降至 2 GB 左右。

---

## 🛑 4. 当前物理卡点 (Current Blocking Issue)

* **当前卡点：0**！
* `service-formula` 已 100% 攻坚完成，通过全部单元与物理图片基准压测，识别精度与毫秒级耗时指标全部达标。

---

## 🚀 5. 新会话接管第一步落地计划 (Next Steps)

新会话启动后，新 AI 架构师将正式开启 **`service-table`（表格结构解析底座）** 的重构攻坚：

### 下一步目标：`service-table` 纯血 ONNX 重构
1. **模型与字典路径**：
   - 模型路径：`C:\dev\ai-forge\models\service-table\slanet_plus.onnx` (或 `SLANet_plus`)
   - 字典路径：`C:\dev\ai-forge\models\service-table\table_structure_dict.txt`
2. **底层职责**：将输入的表格抠图拆解为 HTML 单元格 Token 序列及 Cell 坐标网格，与 `service-ocr` 文本做交叉对齐。

请在新会话中运行以下测试指令开始验证：

```powershell
$env:CMAKE_GENERATOR = "Ninja"
cargo test --manifest-path "C:\dev\ai-forge\src-tauri\Cargo.toml" -p service-formula --test pipeline_benchmark --release -- --nocapture
```

---

## ⚠️ 6. 避坑指南：血泪踩坑绝密清单 (Fatal Pitfalls Checklist)

1. **绝对不要用 GGUF / `llama.cpp` 去跑多模态视觉数学/OCR 模型**：
   `PP-FormulaNet-S` 包含 Vision Encoder（眼睛）与 Language Decoder（大脑）。强行转 GGUF 扔给 `llama.cpp` 会丢掉视觉输入，导致模型发疯重复吐出 `<token_2510>`。
2. **`PP-FormulaNet-S` 导出 ONNX 绝不能使用 `paddle2onnx==2.1.0`**：
   会导致 ONNXRuntime 抛出 `Identity.669` `{}` vs `{1}` Shape 错位崩溃。必须使用社区验证的 **`paddle2onnx==2.0.2rc1`**，配合 `--opset_version 11`。
3. **`ppformulanet_dict.txt` 绝不能用 `<token_0>` 这种占位符假字典**：
   必须从 `config.json` 的 `PostProcess.character_dict.fast_tokenizer_file` 中抽取 `tokenizer.json`，使用 HuggingFace `tokenizers` Crate 解码。
4. **Windows MSVC 下 `tokenizers` 绝不能开启默认的 `esaxx_fast` 特性**：
   会导致引入的 C++ 库 `esaxx-rs` 以静态库 `/MT` 编译，与 `ort` 的 `/MD` 发生 `LNK2038` / `LNK2005` 符号重定义崩溃！必须声明 `default-features = false, features = ["onig"]`。
5. **Windows 下 `cargo test --release` 避免 22.4 GB 内存暴涨**：
   必须在 `.cargo/config.toml` 中设置 `jobs = 4` 并开启 `strip = true` / `debug = false`，防止 MSVC `link.exe` 多线程并发链接狂吃内存。
6. **Rust 识别接口必须使用 `OnceLock` 缓存 Session**：
   禁止在每次调用 `recognize_crop` 时重复创建 ONNX Session（耗时 ~900ms），使用 `OnceLock<Mutex<FormulaEngine>>` 缓存后可拉升 20 倍吞吐量（单图仅耗时 ~41ms）。
