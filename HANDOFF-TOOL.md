# 🛡️ AI-Forge 纯血 Native PDF 解析工程交接文档 (HANDOFF-TOOL.md)

> **版本**：v19.0.0 (2026年8月18日 统一多模态中台 & 前后端并网竣工版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **操作系统与编译链**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 项目定位与核心使命 (Project Overview)

1. **项目定位**：
   - 跨平台工业级 AI 桌面端（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 直推）。
   - **算力主权**：彻底废除重型 Python (`.venv`) 沙箱与本地网络/IPC 端口，100% 切换为 Rust 原生驱动（`ort` ONNX Runtime 直推 + `service-pdfium` C-FFI 绑定）。
2. **核心工具模块 (`pdf-parse`)**：
   - **双黄金样本驱动**：`test/parse/1.pdf` (PP-DocLayout 论文, 4页多模态) 与 `test/parse/2.pdf` (LinearRAG 论文, 4页数字矢量)，以及 `test/parse/3.pdf` (221MB 扫描件)。
   - **当前核心使命**：前后端已全面并网，主脉络与中台完全竣工。进入**微观排版细节打磨阶段**，在 `benchmark_sota.py` 白盒评测中使 `1.pdf` 与 `2.pdf` **同时稳定达到 98+ 分**。

---

## 📐 2. 统一重构后的架构全景表 (Workspace Architecture)

```text
C:\dev\ai-forge\
├── src-tauri\crates\
│   ├── core-onnx-infer\           【ONNX 硬件会话底座】 (CUDA/DirectML/CPU 会话管理)
│   ├── services\                  【5 大底层算法底座 (100% 官方物理对齐，只读防腐区 🛡️)】
│   │   ├── service-pdfium\        : 300DPI 渲染 + 物理尺寸 + 字符级 BBox (Y 镜像映射)
│   │   ├── service-layout\        : PP-DocLayoutV3 官方 25 类别映射 + [0.0, 1.0] RGB 归一化
│   │   ├── service-formula\       : PP-FormulaNet-S (384x384 白底留边 + FastTokenizer)
│   │   ├── service-table\         : SLANet_plus (488x488 BGR + 8 坐标通道 + HTML 重构)
│   │   ├── service-ocr\           : PP-OCRv6 (Det 0.2/0.45/1.4/3000 门限 + Rec 48px 微批次)
│   │   └── service-doc-parse\     : AST 语法树 + RXYC++ 几何排序 (通用几何防腐)
│   │
│   └── tools\pdf-parse\           【纯血桌面端 PDF 解析适配器 (业务中台)】
│       ├── probe.rs               : 2026 SOTA 三级漏斗通用分类器 (0.18s 精准判别矢量/扫描)
│       ├── pipeline.rs            : 🛡️ 统一高保真多模态排版流水线中台 (1:1 承载 98.41% 算法资产)
│       ├── fast_track.rs          : ⚡ 极简门面 (绑定 TextExtractMode::VectorOnly，0 OCR 耗时)
│       ├── deep_track.rs          : 🧠 极简门面 (绑定 TextExtractMode::OpticalOcr，视觉 OCR 兜底)
│       ├── hybrid.rs              : 智能按页调度器 (全矢量/全扫描/混合按页缝合)
│       ├── exporter.rs            : 原生流式打包器
│       └── service.rs             : 桌面端 Dedicated 本地直出服务契约 (0 Zip 垃圾)
│
└── src\                           【现代化暗黑高定前端 (Vue 3 + Tailwind)】
    ├── bindings\tools\pdf-parse.ts: 强类型契约 (PdfParseResult { markdown, output_md_path, ... })
    ├── views\PdfParseView.vue     : 高定暗黑工作台 (5:7 栅格 + 6 大防逆向指南 + 4 大能力开关)
    ├── views\HomeView.vue         : 宽幅流体自适应主页 (满铺红底 PDF 经典卡片，18字无遮挡)
    └── assets\tools\pdf-parse\    : cover.svg (800x500 16:10 满铺 3D 拟真折角矢量封面)
```

---

## 🟢 3. 本会话已完成的重大突破 (Accomplished Breakthroughs)

1. **三级漏斗通用分类器落地 (`probe.rs`)**：
   - 彻底废除旧版容易误判的 `img_count == 0` 与 `vec_count <= 30`；
   - 依据 2026 SOTA 工业级标准（有效字符数 $\ge 50$ + 可读率 $\ge 60\%$）重构三级漏斗判定；
   - **实弹打靶 0.18 秒全部精准命中**：`1.pdf` $\to$ 矢量轨、`2.pdf` $\to$ 矢量轨、`3.pdf` $\to$ 扫描轨。
2. **高内聚统一排版流水线中台解耦 (`pipeline.rs`)**：
   - 严格遵循 Rule 14 零回归原则，将此前在图二中跑出 **98.41% 高分的成熟多模态拼装资产** 1:1 沉淀到 `pipeline.rs`；
   - 正交支持 `TextExtractMode::VectorOnly` 与 `TextExtractMode::OpticalOcr`；
   - `fast_track.rs` 与 `deep_track.rs` 彻底轻量化为门面（不到 30 行），实现一次调优、双轨同步受益。
3. **前后端全链路并网与本地物理直出交付**：
   - 彻底废除 Zip 打包，输出为标准 `document.md` 与 `images/` 目录；
   - 更新前端 TypeScript 强类型契约 `src/bindings/tools/pdf-parse.ts`；
   - 全面换装暗黑高定 `PdfParseView.vue`，砍掉多余的“路由模式选择器”，聚焦 4 大核心能力；
   - 解决桌面端 1280×800 窗口化卡片折行问题（流体自适应栅格）；
   - 落地 16:10 满铺纯矢量红底经典折角封面 `cover.svg`；
   - 修复文件选择对话框过滤器（专属绑定 `*.pdf`）。
4. **编译与自动化测试全绿通**：
   - `cargo test -p pdf-parse` 测试全通（`probe`、`fast_track`、`hybrid` 全量测试通过）；
   - `pnpm build`（`vue-tsc --noEmit && vite build`）打包 0 错误 0 警告；
   - `pnpm tauri dev` 真机运行通畅。

---

## 🛑 4. 当前卡点与微观排版细节真相 (Current Bottleneck)

### 现状与微观排版差异点
虽然全流程端到端已经 100% 跑通，全量 4 页的公式（`$$`）、表格（`<table>`）、插图（`images/*.png`）与正文均已成功提取，但在部分微观排版细节上与真值（Ground Truth）还存在微小间隙：
1. **作者机构信息换行缝合**：多作者/机构邮箱在矢量抽取时存在多余断行；
2. **双栏跨栏图表局部的阅读序穿透**：例如部分伴随说明文字与图表标题的相对先后次序微调；
3. **标题层级嗅探规则微调**：大写编号标题（如 `1 INTRODUCTION`、`2 PRELIMINARY STUDY`）的 `## ` 标记覆盖率。

---

## 🚀 5. 新会话下一步实施清单 (Action Plan for New Session)

新会话请按照 **TDD + 白盒对比** 顺序单步推进：

1. **第一步（运行白盒评测脚本锁定差异）**：
   - 在 `C:\dev\ai-forge` 运行 `uv run --python 3.11 -i https://pypi.tuna.tsinghua.edu.cn/simple --with python test/parse/benchmark_sota.py`；
   - 打印 `1.pdf` 与 `2.pdf` 的当前详细分值（文本相似度、标题匹配率、公式召回率、表格准确率、阅读序得分）。
2. **第二步（外科手术式打磨 `pipeline.rs` 中的微观组装算子）**：
   - 针对第一步报告中的扣分项，在 `pipeline.rs` 的 `extract_vector_text_in_bbox` 与 `stitch_and_assemble` 中微调（**绝对不修改 `service-*` 底座代码**！）；
   - 完善作者信息块缝合、KaTeX 细空格规范化与章节标题正则。
3. **第三步（双黄金样本 98+ 分验收）**：
   - 再次运行 `benchmark_sota.py`，确认 `1.pdf` 与 `2.pdf` 均稳定达到 **98+ 分**！

---

## ⚠️ 6. 避坑终结手册与禁忌铁律 (Anti-Patterns / Lessons Learned)

1. **绝对禁止退化为纯文本提取器（Rule 14 铁律）**：矢量轨必须走完整的 DocLayout + FormulaNet + SLANet + MD5 切片 + AST 流程，区别仅在于文本取 PDFium 矢量真值。
2. **绝对禁止修改 `service-*` 底座做样本特化**：底座算法保持通用防腐，微观组装在 `pipeline.rs` 的适配器层处理。
3. **绝对遵循 Windows PowerShell Set-Content UTF-8 落盘协议**：使用 `@' ... '@` 单引号字符串，禁止命令行开头带有 `#` 注释。
4. **分类器统一使用“三级漏斗通用判定算法”**：禁止提及已废弃的旧 5 维探针概念。
5. **文案商业化防逆向**：前端对外文案只讲产品能力，绝对禁止泄露底层模型名称（如 `PP-FormulaNet`、`SLANet`）与内部工程 Trick（如 `纯白底留边`、`RXYC++`）。
