# 🛡️ AI-Forge (紫电 AI 桌面工坊) 终极工程交接文档 (HANDOFF.md)

> **文档版本**：v17.0.0 (2026年8月14日 全能格式转换底座竣工 & 前端 Vue 3 界面挂载准备版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 架构师 / 开发助手 (零历史上下文障碍接管)

---

## 🏢 1. 项目概况与双轮驱动战略 (Project Overview & Strategy)

1. **项目形态**：工业级跨平台桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI / ONNX 直推）。
2. **商业闭环与产品战略（钱学森系统最优化）**：
   - **🆓 免费高频引流层 (Zero Marginal Cost Tier)**：
     - **全能格式转换中心 (`format-converter` / `service-converter`)**：包含全平台音频母带离线解锁（NCM / QMC / KGMA / KWM / AV3A / KGG）、多维结构化数据清洗（CSV / TSV / JSON / XML）、原生图标与流式 PDF 合成（ICO / BMP / PDF）、电子书排版（EPUB / MOBI / DOCX）与防穿越 ZIP 归档。
     - **特性**：纯 CPU 内存直推，毫秒级完成，零显存占用，零服务器边际成本，作为装机必备工具源源不断积累活跃用户与品牌口碑。
   - **💎 商业变现与 AI 展厅层 (High Moat Commercial AI Tier)**：
     - **深度混合 PDF->Markdown (98% 准确率基准)**：面向大模型 RAG 知识库与微调语料提炼。
     - **多语言视频双语字幕工坊**：自回归 ASR + 神经机翻 C-FFI 直推。
     - **8K 视觉张量超分辨率**：RealESRGAN 矩阵 GPU 直推。
     - **特性**：由 `shared-contracts` 显存守卫（`VramTokenGuard`）与 `core-security` 硬件鉴权守卫（`Gatekeeper`）进行配额管控与商业授权。

---

## 📐 2. 当前 Crate 架构图景 (Service & Tool Architecture)

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 通用 ONNX 硬件会话底座】
├── core-security\             【硬件指纹采集与 Gatekeeper 商业收费站】
├── shared-contracts\          【显存锁 VramTokenGuard (RTX 3060 11000MB 防爆守卫)】
│
├── services\
│   ├── service-converter\     【🌟 全能格式转换超级底座 (100% 竣工 ✅)】
│   │   ├── audio\             : NCM / QMC2(双TEA) / KGMA / KWM / AV3A / KGG DB 离线逆向解密
│   │   ├── image\             : ICO 多尺寸容器合成 / 1~32bit BMP 解码 / 纯内存流式 PDF 1.4 对象装配
│   │   ├── text\              : RFC 4180 CSV/TSV 解析 / JSON 路径扁平化 / 递归下降 XML ↔ JSON
│   │   ├── ebook\             : 标准 EPUB 2.0 流式打包 / MOBI PalmDOC 4级 zlib 探测解压
│   │   ├── docx\              : Markdown / HTML 转 WordprocessingML DOCX 容器生成
│   │   ├── archive\           : 内存流式 ZIP 打包/解压 / Zip Slip 路径穿越安全防护
│   │   └── media\             : Windows Native FFmpeg 静默管道 (HEIC / TGA / GIF / 视频转码)
│   │
│   ├── service-pdfium\        【纯血 PDFium 底座】 (300DPI 渲染 + 字符 BBox 矢量提取)
│   ├── service-ocr\           【纯血文本 OCR 底座】 (PP-OCRv6 C-FFI 直推)
│   ├── service-layout\        【版面物理分块底座】 (PP-DocLayoutV3 C-FFI 直推)
│   ├── service-formula\       【数学公式识别底座】 (PP-FormulaNet-S C-FFI 直推)
│   ├── service-table\         【表格结构解析底座】 (SLANet_plus C-FFI 直推)
│   └── service-doc-parse\     【超级排版与 AST 文档树底座】
│
└── tools\
    ├── format-converter\      【🌟 全能格式转换用户端调度中枢 (100% 并网 ✅)】
    │   ├── service.rs         : 格式智能嗅探路由与任务交付状态机
    │   └── lib.rs             : FormatConvertTask & FormatConvertResult 契约定义
    │
    ├── pdf-parse\             【深度混合 PDF->Markdown 桌面端工具】
    └── tool-video-subtitle\   【视频双语字幕工坊】
```

---

## 🟢 3. 本会话已完成的建设 (What Was Completed)

1. **全新建立并点火 `crates/services/service-converter`**：
   - 1:1 外科手术式移植了开源项目 `flyingmouse-format` 的全部精华算法，彻底废除 Node.js 胶水层，全部转化为纯血 Rust 2024 高内聚计算中台。
   - 包含音频解密、多尺寸 ICO/位图、流式 PDF 1.4、结构化数据清洗、EPUB/DOCX/MOBI 排版、安全 ZIP 归档与 FFmpeg 管道封装。
   - 编写并全部绿通 **16 项核心单元测试（0 警告 0 报错，耗时 0.00s）**。

2. **全新建立并点火 `crates/tools/format-converter`**：
   - 实现了基于扩展名与物理魔数的智能分流调度中枢（`FormatConvertService::convert`）。
   - 确立了面向前端的统一交付契约 `FormatConvertResult`。

3. **主程序 Tauri 指令并网**：
   - 在 `src-tauri/src/lib.rs` 中注册了 `#[tauri::command] async fn run_format_convert`。
   - 通过 `window.emit("format-convert-finished", &res)` 完成前后端实时事件通信闭环。
   - 全工作空间执行 `cargo check` 0 警告 0 报错完美通过。

---

## 🛑 4. 当前卡点与任务交接点 (Current Status & Next Handover)

* **当前状态**：后端底座、工具层调度与 Tauri 指令**已 100% 竣工就绪**。
* **下一步明确目标**：**【方向 2：前端 Vue 3 界面与交互挂载】**。
  - 需要在前端 Vue 3 工程中为【全能格式转换工坊】挂载美观的用户界面（拖拽上传区、目标格式智能下拉菜单、批量转换卡片列表、进度监听）。

---

## 🚀 5. 新会话接管第一步落地计划 (Next Session Step 1 Plan)

新会话启动后，请严格遵循以下单步引导流程推进：

1. **第一步 (前端拓扑审阅 - Rule 11 前置契约)**：
   在前端修改前，先执行一条 PowerShell 命令，调阅前端 `src/` 下的视图组件与路由目录结构：
   ```powershell
   Get-ChildItem -Path "C:\dev\ai-forge\src" -Recurse -Depth 2 | Select-Object FullName
   ```
2. **第二步 (设计前端格式转换组件)**：
   在前端视图中新增或更新 `FormatConverterView.vue`（或对应格式转换组件），通过 Tauri API `invoke("run_format_convert", { task: ... })` 调起后端，并监听 `"format-convert-finished"` 事件。
3. **第三步 (路由与侧边栏挂载)**：
   将转换工坊挂载至左侧主导航栏，实现一键平滑切换。
4. **第四步 (真机交互与拖拽打靶)**：
   运行 `pnpm dev` 验证拖拽 `.ncm` / `.csv` / `.png` / `.md` 等文件的一键秒级转换体验。

---

## ⚠️ 6. 踩坑终结手册与禁忌铁律 (Lessons Learned & Anti-Patterns)

1. **Rust 所有权转移陷阱 (Ownership Move)**：在遍历判断多格式分支时，处理 `raw_bytes` 必须使用只读切片借用 `std::str::from_utf8(&raw_bytes)`，严禁使用 `String::from_utf8(raw_bytes)` 消耗所有权。
2. **DOCX / 字符串生命周期悬垂 (Temporary Lifetime Drop)**：构建 XML 属性时，`format!(...).as_str()` 会产生临时变量立即析构，必须使用 `Vec<String>` 保存所有权后再统一 `.join("")`。
3. **TEA 解密闭包借用冲突 (E0503 Borrow Conflict)**：在密码学循环解密中，禁止使用可变闭包跨迭代捕获局部数组，应采用平铺内联（Inlined）方式消除借用检查器冲突。
4. **Windows Native PowerShell `@' ... '@` 落盘铁律**：凡涉及源码修改或新建文件，必须使用单引号原样字符串与 `Set-Content`，防止 PowerShell 提前转义变量。
5. **项目物理根目录主权**：所有编译命令必须在 `C:\dev\ai-forge` 根目录下执行，并显式指定 `--manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml`。
