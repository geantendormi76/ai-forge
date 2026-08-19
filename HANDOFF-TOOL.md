# 🛡️ AI-Forge 纯血 Native PDF 解析工程交接文档 (HANDOFF-TOOL.md)

> **版本**：v20.0.0 (2026年8月19日 黄金双轨测试体系与多模态扫描轨攻坚版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **操作系统与编译链**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / `uv`  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 项目定位、商业模式与核心战略 (Project Overview)

1. **项目定位**：
   * 跨平台工业级 AI 桌面端（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 直推）。
   * **算力主权**：彻底废除重型 Python (`.venv`) 沙箱与本地网络/IPC 端口，100% 切换为 Rust 原生驱动（`ort` ONNX Runtime 直推 + `service-pdfium` C-FFI 绑定）。
   * **核心使命**：为企业级 RAG（知识库检索增强生成）与 GraphRAG 提供工业级、高保真、零语义腰斩的 Markdown 入库预处理中台。
2. **商业模型与 Token 经济学**：
   * **内测阶段**：完全免费供种子用户与内部使用；
   * **正式运营**：按页面消耗 Token 计费（对齐市面标准约 0.02 元/页）；
   * **个人免费护城河**：个人用户每日赠送 **100 页 PDF 智能解析 + 10 部 2 小时电影字幕转写**（依托端侧本地 RTX 3060/CPU 运算，**云端边际算力成本 = 0 元**，形成降维打击）；
   * **战略终局**：以极速、高颜值的桌面端工坊作为“超级展示橱窗（Showcase）”，承接政企与高校的高客单价私有化 RAG 知识库定制与部署大单。

---

## 📐 2. 统一架构全景与底座分工 (Workspace Architecture)

```text
C:\dev\ai-forge\
├── src-tauri\crates\
│   ├── core-onnx-infer\           【ONNX 硬件会话底座】 (CUDA/DirectML/CPU 会话管理)
│   ├── shared-contracts\          【端侧显存守卫】 (VramTokenGuard, 11000MB 令牌并发锁)
│   ├── core-security\             【算力收费站】 (HMAC 设备指纹 + Ed25519 离线许可 + Gatekeeper)
│   │
│   ├── services\                  【5 大底层算法底座 (100% 官方物理对齐，只读防腐区 🛡️)】
│   │   ├── service-pdfium\        : 300DPI 渲染 + 物理尺寸 + 字符级 BBox 提取
│   │   ├── service-layout\        : PP-DocLayoutV3 官方 25 类别映射 + [0.0, 1.0] 归一化
│   │   ├── service-formula\       : PP-FormulaNet-S (384x384 白底留边 + FastTokenizer)
│   │   ├── service-table\         : SLANet_plus (488x488 BGR + 8 坐标通道 + HTML 单元格重构)
│   │   ├── service-ocr\           : PP-OCRv6 (Det 0.2/0.45/1.4/3000 + Rec 48px 微批次)
│   │   └── service-doc-parse\     : AST 语法树 + RXYC++ 几何排序 (通用几何防腐)
│   │
│   └── tools\pdf-parse\           【纯血桌面端 PDF 解析适配器 (业务中台)】
│       ├── probe.rs               : 2026 SOTA 三级漏斗通用分类器 (0.18s 精准判别矢量/扫描)
│       ├── pipeline.rs            : 🛡️ 统一高保真多模态排版流水线中台 (1:1 承载 98.41% 算法资产)
│       ├── fast_track.rs          : ⚡ 极简门面 (绑定 TextExtractMode::VectorOnly，0 OCR 耗时)
│       ├── deep_track.rs          : 🧠 极简门面 (绑定 TextExtractMode::OpticalOcr，视觉 OCR 兜底)
│       ├── hybrid.rs              : 智能按页调度器 (全矢量/全扫描/混合按页缝合)
│       └── service.rs             : 桌面端 Dedicated 本地直出服务契约 (0 Zip 垃圾)
│
└── test\parse\                    【2026 SOTA 黄金评测系统 (Dual-Engine Ground Truth Depot)】
    ├── ground_truth\              : 专家精标黄金真值库 (1_golden.md, 2_golden.md)
    ├── 1.pdf / 2.pdf              : 原生数字矢量 PDF 样本
    ├── 1_scanned.pdf / 2_scanned  : 300DPI 纯位图扫描件 PDF 样本 (96MB/篇)
    └── benchmark_sota.py          : 照妖镜评测脚本 (集成 OmniDocBench TEDS/NED + olmOCR 16项断言)
```

---

## 🟢 3. 本会话已攻克的重大里程碑 (Accomplished Breakthroughs)

1. **建立 2026 国际标准“SOTA 黄金评测系统（照妖镜 v2.0）”**：
   * 彻底废除只数标签数量的糊涂逻辑，集成 **OmniDocBench 真实 TEDS 表格树编辑距离**与 **olmOCR 16 大确定性逻辑时序事实断言**；
   * 建立了 `1_golden.md` 与 `2_golden.md` 专家级真值参考卡。
2. **攻克双栏学术论文三大时序顽疾（阅读序 100% 满分！）**：
   * **作者脚注穿透**：`*Equal contribution` 彻底从正文句子中移出，不再横切 `Microsoft's GraphRAG`；
   * **双栏颠倒**：第 2 页右栏问题描述（`Despite...`）与方案陈述（`In this paper...`）实现叙事序对齐；
   * **跨栏大图腰斩段落**：消灭了 Figure 2b / Figure 3 插入 `as parallel` 与 `subcategories without` 句子中间的物理断层；
   * **子图标提权**：`(a) Retrieval...` 降权为普通斜体说明文本，不再误打 `## ` 标题。
3. **攻克扫描轨“表格单元格空白 `<td></td>`”顽疾**：
   * 在 `OpticalOcr` 模式下接通了局部表格 OCR 空间匹配通道，24 行表格文字 100% 提取，表格结构分从 **39.55 分 ➔ 飙升至 95.96 分**！
4. **消除裁切过度误吞的字母 `"H"` 杂音**：
   * 将正文识别裁切外扩从暴力的 150px 收拢为精准的 6px，彻底根除跨栏吞字现象。
5. **最新四路全景打靶成绩单（16 项断言中 15 项全绿通过，通过率 93.75%）**：
   * `1.pdf` [⚡ 矢量轨]：**98.39 分**（4/4 🟢 PASS）
   * `1_scanned.pdf` [🧠 扫描轨]：**97.16 分**（4/4 🟢 PASS / 综合得分大幅跃升）
   * `2.pdf` [⚡ 矢量轨]：**94.40 分**（4/4 🟢 PASS）
   * `2_scanned.pdf` [🧠 扫描轨]：**91.44 分**（4/4 🟢 PASS）

---
