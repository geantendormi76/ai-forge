
# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v16.0.0 (2026年8月14日 纯血 Native PDF 解析重构 & Zero-Regression 迁移范式确立版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 架构师 / 开发助手 (零历史上下文障碍接管)

---

## 🏢 1. 项目概况与重构定位 (Project Overview)

1. **项目形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 动态库直推）。
2. **算力主权与零 Python 演进**：
   - 彻底废除重型 Python (`.venv`) 沙箱与 Stdio/HTTP IPC 通信，全面切换为纯血 Rust Native 架构。
   - 静态视觉图模型 (OCR / 版面分析 / 公式识别 / 表格解析) ➔ `ort` (ONNX Runtime) C-FFI 直推 + ONNX 单文件。
   - PDF 图像渲染与矢量字符提取 ➔ `service-pdfium` (绑定 `pdfium.dll` C-FFI)。
3. **最高迁移铁律：Zero-Regression (零回归) 代码迁移范式 3 大支柱**：
   - **API Grounding (接口锚定)**：必须强制锚定成熟源码中关键的数据交汇中台——`RawLayoutElement` 数据交换 Schema。
   - **Replay-based Validation (重放断言校验)**：将旧 Python 算子产出的中间 JSON (`RawLayoutElement[]`) 作为 Ground Truth。Rust 算子不看最终 Markdown 分数，而是必须**断言产出的中间 JSON 1:1 逐字段卡位对齐**。
   - **Surgical Transpilation (外科手术式直译)**：旧代码怎么写的判别、怎么切的框，新代码 1:1 直译，**严禁 AI 引入任何未在旧源码中出现的“新算法”或“新架构”**。

---

## 📐 2. 当前 Crate 架构图景 (Service & Tool Architecture)

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 通用 ONNX 硬件会话底座】
│
├── services\
│   ├── service-pdfium\        【纯血 PDFium 底座】 (300DPI 渲染 + 字符 BBox 矢量提取)
│   ├── service-ocr\           【纯血文本 OCR 底座】 (PP-OCRv6 C-FFI 直推)
│   ├── service-layout\        【版面物理分块底座】 (PP-DocLayoutV3 C-FFI 直推)
│   ├── service-formula\       【数学公式识别底座】 (PP-FormulaNet-S C-FFI 直推)
│   ├── service-table\         【表格结构解析底座】 (SLANet_plus C-FFI 直推)
│   │
│   └── service-doc-parse\     【超级排版与 AST 文档树底座 (100% 升级完成 ✅)】
│       ├── config.rs          : 几何 BoundingBox、排序标签与排序数据块
│       ├── ast.rs             : 5 层 DocumentNode AST 树与 GFM 渲染器
│       ├── xy_cut.rs          : RXYC++ 双栏中轴通道检测与阅读顺序重排
│       ├── stitching.rs       : CJK 中英文缝合、去连字符、KaTeX 转义清洗与 Caption 绑定
│       └── virtual_table.rs   : 跨页表格合并状态机与 K-Means 网格聚类
│
└── tools\
    └── pdf-parse\             【全新纯血桌面端 PDF 解析适配器 (独立全新白纸 Crate ✅)】
        ├── probe.rs           : 5 维物理路由探针 (FastTrackCpu / DeepTrackGpu 分流)
        ├── fast_track.rs      : 毫秒级极速矢量轨 (Pdfium 文本直接提取)
        ├── deep_track.rs      : 深度视觉自愈轨 (BBox 矢量剪切 + 专有模型直推 + 1:1 AST 组装)
        ├── hybrid.rs          : 按页降维混合调度器
        ├── exporter.rs        : 纯 Rust 原生 zip 流式打包器
        └── service.rs         : 桌面端 Dedicated 本地交付契约 & Tauri Window EventEmitter 进度接口
```

---

## 🟢 3. 本会话已完成的建设 (What Was Completed)

1. **升级 `service-doc-parse` 为超级底座**：
   - 1:1 无损移植了成熟源码 `tool-pdf-parse/src/idp` 中的所有黄金算法（`config.rs`、`ast.rs`、`xy_cut.rs`、`stitching.rs`、`virtual_table.rs`）。
   - 彻底完成了 RXYC++ 几何排序、CJK 中英文自然缝合、KaTeX 转义清洗（彻底消灭前端 `\_` 与 `^^` 报错）以及 AST 文档树高保真导出。

2. **在全新白纸 `crates/tools/pdf-parse` 构建桌面端工具**：
   - 移除了所有 Web API 遗毒（如硬编码的 `/api/v1/outputs/xxx.zip` 和强制 ZIP 打包）。
   - 实现 5 维探针分流 `probe.rs`、矢量轨 `fast_track.rs`、深度轨 `deep_track.rs`、混合分流 `hybrid.rs` 与纯 Rust 打包 `exporter.rs`。
   - 建立了桌面端专属交付契约 `PdfParseResult`，并在 `src-tauri/src/lib.rs` 中通过 `tauri::Emitter` 接入 `pdf-parse-progress` 零开销实时进度事件推送。

3. **建立双级裁判客观评估系统**：
   - 在 `C:\dev\ai-forge\test\parse\` 建立了以 `1_hybrid_output.md`（98% 准确率锚点）为黄金基准的 `evaluate_pdf_parse.py` 客观跑分系统。

---

## 🛑 4. 当前卡点与物理真因 (Current Blocker)

* **物理卡点现象**：
  在对 `pdf-parse-fast.pdf` 运行打靶时，当前 Rust 产物的双级裁判跑分为 **86.67 分**（标题召回率为 35.71%），尚未达到黄金基准 `1_hybrid_output.md` 的 **98.00 分**。
* **白盒根源 (Zero-Regression视角)**：
  之前 Rust 侧 `deep_track.rs` 偏离了旧 Python `deeptrack_worker.py` 的处理流程。Python 侧的核心优势是：用 `PP-DocLayoutV3` 画框后，对标题和段落框调用 `Pdfium` 逐框提取矢量字符（`extract_vector_text_in_bbox`），并附带 `left_margin_px = 30 if x0 > 400 else 150` 的右栏隔离 Padding，产出标准的 `RawLayoutElement` JSON 数组。而先前 Rust 侧未完成这一中间 JSON 的 **1:1 逐字段对齐**。

---

## 🚀 5. 新会话接管第一步落地计划 (Next Steps)

下一会话将严格遵循 **Zero-Regression (零回归) 代码迁移范式** 推进：

1. **第一步 (生成 JSON 黄金真值)**：
   运行一段行内 Python 探针，用旧 `deeptrack_worker.py` 解析 `pdf-parse-fast.pdf`，导出中间产物 `pdf-parse-fast_gt_elements.json`（`RawLayoutElement[]`）。
2. **第二步 (1:1 外科手术式直译)**：
   检查并调整 `pdf-parse/src/deep_track.rs` 中的 `extract_vector_text_in_bbox` 剪切函数与 Padding 边界逻辑，确保 Rust 提炼出的 `RawLayoutElement[]` JSON 与 Python GT JSON **1:1 卡位完全一致**。
3. **第三步 (重放断言校验)**：
   编写 Rust 测试 `test_raw_elements_json_replay`，断言 Rust 产生的 JSON 与 Python GT JSON 逐字段一致。
4. **第四步 (直接复用黄金组装)**：
   将 1:1 对齐好的 `RawLayoutElement[]` 直接灌入原封不动的 `stitch_and_assemble()` 渲染，**直接复现 98%+ 跑分**！

---

## ⚠️ 6. 踩坑终结手册与禁忌铁律 (Lessons Learned & Anti-Patterns)

1. **绝对禁止盲猜与调参造轮子**：当存在成熟落地源码时，严禁自作主张编写新的数据转换逻辑或重写 AST 渲染器，必须 1:1 直译并对齐中间 JSON！
2. **绝对禁止在 Debug 模式下评估性能**：Rust 的 ONNX 推理与图像裁切压测**必须使用 `--release` 编译标志**（Debug 模式慢 10~30 倍）。
3. **绝对禁止运行非定点测试**：运行 `cargo test` 时，必须显式指定目标包与函数名（例如：`cargo test --release --package pdf-parse test_native_fasttrack_execution -- --nocapture`），绝不盲目触发全量测试。
4. **绝对遵循 PowerShell `@' ... '@` 落盘协议**：凡涉及源码落盘，必须使用 PowerShell 单引号原样字符串与 `Set-Content`，防止 PowerShell 变量转义坏代码。
5. **绝对遵循场景双轨协议**：持久化改动用落盘，一次性数据排查与探针验证必须用 `uv run --python 3.11 python -c "..."` 零痕迹即时求值。
