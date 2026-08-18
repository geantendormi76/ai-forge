# 🛡️ AI-Forge 纯血 Native PDF 解析工程交接文档 (HANDOFF_TOOL.md)

> **版本**：v18.0.0 (2026年8月18日 纯血 Native 矢量轨 & 正交双轨架构版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **操作系统与编译链**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 项目定位与核心使命 (Project Overview)

1. **项目定位**：
   - 跨平台工业级 AI 桌面端（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 直推）。
   - **算力主权**：彻底废除重型 Python (`.venv`) 沙箱与本地网络/IPC 端口，100% 切换为 Rust 原生驱动（`ort` ONNX Runtime 直推 + `service-pdfium` C-FFI 绑定）。
2. **当前核心攻坚**：
   - **双黄金样本驱动**：`test/parse/1.pdf` 与 `test/parse/2.pdf`。
   - **分阶段目标**：通过实现纯 Rust 原生 `FastTrack`（矢量数字轨），结合已闭环的 `DeepTrack`（视觉多模态轨）与 `probe.rs`（5 维分流探针），在 `benchmark_sota.py` 白盒评测中使 `1.pdf` 与 `2.pdf` **同时稳定达到 98+ 分**。

---

## 📐 2. 工作空间全景架构表 (Workspace Architecture)

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 通用 ONNX 硬件会话底座】 (CUDA/DirectML/CPU 会话管理)
│
├── services\                  【5 大底层算法底座 (100% 官方物理对齐，只读防腐区 🛡️)】
│   ├── service-pdfium\        [已闭环] : 300DPI 渲染 + (W_pt, H_pt) + 字符级物理 BBox (Y 镜像映射)
│   ├── service-layout\        [已闭环] : PP-DocLayoutV3 官方 25 类别映射 + [0.0, 1.0] RGB 归一化
│   ├── service-formula\       [已闭环] : PP-FormulaNet-S (384x384 白底留边 + cv2 灰度 + FastTokenizer)
│   ├── service-table\         [已闭环] : SLANet_plus (488x488 BGR + 8 坐标通道 + HTML Token 解码)
│   ├── service-ocr\           [已闭环] : PP-OCRv6 (Det 0.2/0.45/1.4/3000 黄金门限 + Rec 48px 微批次)
│   └── service-doc-parse\     [已闭环] : AST 语法树 + RXYC++ 几何排序 (禁止为个别样本加脏逻辑)
│
└── tools\
    └── pdf-parse\             【纯血桌面端 PDF 解析适配器 (业务中台)】
        ├── probe.rs           : 5 维物理路由探针 (FastTrackCpu / DeepTrackGpu 分流)
        ├── fast_track.rs      : ⚡ 纯血原生矢量流解析引擎 【当前第一优先级重构目标 🎯】
        ├── deep_track.rs      : 🧠 深度视觉自愈轨 (已闭环，单图/大图裁切 + 32位 MD5 + 包含去重)
        ├── hybrid.rs          : 智能按页降维混合调度器 (串联 probe + fast_track + deep_track)
        ├── exporter.rs        : 纯 Rust 原生 zip 流式打包器
        └── service.rs         : 桌面端 Dedicated 本地交付契约 (Result<PdfParseResult, String>)
```

---

## 🟢 3. 已经完成的重大突破 (Accomplished Breakthroughs)

1. **5 大底层算法底座 100% 官方物理对齐闭环**：
   - **`service-pdfium`**：白盒解决笛卡尔坐标与位图像素坐标原点冲突，建立 $Y_{\text{top}} = H_{\text{pt}} - Y_{\text{pdfium}}$ 映射，字符提取命中率从 0% 跃升至 100%。
   - **`service-layout`**：依据官方 `inference.yml` 纠正 25 类别枚举（0:abstract, 6:doc_title, 17:paragraph_title, 21:table, 22:text 等），修正输入预处理为 RGB $[0.0, 1.0]$。
   - **`service-formula`**：1:1 直译 UniMERNet 预处理算子（384x384 纯白底留边、cv2 灰度转换、16 倍数 1.0 补齐），单图 `1.png` 识别准确率达 100%（627ms）。
   - **`service-table`**：SLANet_plus 488x488 BGR 预处理与结构字典闭环，打靶置信度 0.9999。
   - **`service-ocr`**：校准 DBPostProcess 4 大门限（`0.2 / 0.45 / 1.4 / 3000`），测试图像 56 行 100% 全中。
   - **`service-doc-parse`**：AST 修复 `abstract` 属性，剥离误加的 `## ` 二级标题。

2. **`deep_track.rs` 深度视觉自愈与样本 1 打靶**：
   - 32 位 MD5 图片资产哈希对齐；
   - 空间 NMS 文本框包含去重（$IoA > 0.70$ 抑制）；
   - 剔除 `0x02` 控制字符彻底消灭乱码方块；
   - **`1.pdf` 经 SOTA 评测实测取得 98.41 / 100.0 高分**。

---

## 🛑 4. 当前认知突破与卡点真相 (Blockers & Root Cause Analysis)

### 1. 样本 2 (`2.pdf`) 的物理属性定性
- **物理真相**：`2.pdf`（LinearRAG 学术论文）是 **100% 原生数字矢量 PDF**（由 LaTeX/pdflatex 直接编译生成），内部包含完整的 Unicode 字符流、字体大小（13pt/9.5pt）、粗体标记以及内嵌图片。
- **之前评分 85.24 的真因**：
  - 之前的 `fast_track.rs` 是个空壳（直接调用了 `deep_track.rs` 视觉深度轨）；
  - 将原生数字 PDF 强行当成“扫描件图片”送入 800x800 视觉模型，神经网络把 Figure 2 左半边的柱状图（a）误判成了文本，导致数字（`71.85%`, `30%`...）被当作普通文字输出，且把 Figure 1 右侧的伴随正文造成了排序切断。

### 2. 架构防腐铁律（坚决拒绝“打地鼠”）
- **绝对禁止修改 `service-doc-parse/src/xy_cut.rs` 来迁就单一样本**！
- 基础设施必须保持通用几何数学逻辑的纯粹性。对于原生数字矢量 PDF，正统解法是由 **`FastTrack`（纯血原生矢量轨）** 直接从 `service-pdfium` 提取文本块与内嵌图片，从根源上实现 100% 字符保真与天然阅读序，且单页耗时从 3 秒降至 30 毫秒！

---

## 🚀 5. 下一步具体落地计划 (Next Steps for New Session)

新会话请按以下顺序**单步推进**（每步完成后由用户验证）：

1. **第一步（在 `fast_track.rs` 中实现纯血原生矢量流解析器）**：
   - 利用 `service-pdfium` 提取页面的文本块（Text Blocks）与字符流；
   - 依据字号与粗体规则提取层级（字号 $\ge 13.0 \to$ `# DocTitle`，字号 $\ge 9.5$ 且粗体 $\to$ `## SectionHeader`，普通字号 $\to$ 正文段落与列表）；
   - 保留自然阅读顺序，过滤 `arXiv:` 与独立页码；
   - 内嵌图片直接从 PDF XObject / 区域无损导出为 `images/<md5>.png`。
2. **第二步（使用 `benchmark_sota.py` 单独测试 FastTrack）**：
   - 运行 `FastTrack` 生成 `1_fast_out.md` 与 `2_fast_out.md`；
   - 验证 `2.pdf` 纯文本保真度跃升至 98%+，阅读序达到 98%+。
3. **第三步（完善 `hybrid.rs` 智能调度中台）**：
   - 依据 `probe.rs` 的 5 维探针判定：
     - 原生数字矢量页 $\to$ 分流给 `FastTrackEngine`（毫秒级）；
     - 复杂图表/扫描件页 $\to$ 分流给 `DeepTrackEngine`（GPU 视觉轨）。
4. **第四步（双样本 98+ 验收与扫描件拓展）**：
   - 运行 `benchmark_sota.py` 确认双样本均在 98 分以上；
   - 引入扫描件样本 `tool-pdf-parse-deep.pdf` 进行全量 OCR 视觉闭环。

---

## ⚠️ 6. 踩坑终结手册与禁忌铁律 (Anti-Patterns / Lessons Learned)

1. **绝对禁止修改 `service-*` 底座为个别样本加 trick**：底座算法保持通用防腐，上层业务通过 FastTrack / DeepTrack 双轨分流解决。
2. **绝对禁止假设 PDF 坐标系为左上角**：PDFium C-FFI 字符坐标原点在左下角，必须使用 $Y_{\text{top}} = H_{\text{pt}} - Y_{\text{pdfium}}$ 镜像转换！
3. **绝对禁止自造模型类别映射**：PP-DocLayoutV3 必须 1:1 严格对齐官方 `inference.yml` 中的 25 类别（6: `doc_title`, 17: `paragraph_title`, 21: `table`, 22: `text` 等）！
4. **绝对禁止给 PP-DocLayoutV3 减去 ImageNet 均值**：官方预处理是 RGB 直接除以 255.0 归一化到 $[0.0, 1.0]$。
5. **绝对禁止在公式预处理中使用黑底留边**：UniMERNet 要求留边必须是**纯白底 (255, 255, 255)**，Tensor 补齐值必须是 **常量 1.0**！
6. **绝对禁止放任 PDF 连字 `0x02` 流入 Markdown**：提取字符时必须过滤 `c.is_control()` 剔除 `0x02` 乱码方块！
7. **绝对禁止在命令行开头带有 `#` 注释**：PowerShell 会直接跳过执行。
8. **绝对遵循 Windows PowerShell 分场景双轨协议**：
   - 【轨一：代码落盘】：使用 `Set-Content -Path "..." -Encoding UTF8 -Value @' ... '@` 单引号原样字符串；
   - 【轨二：即时验证】：使用 `uv run --python 3.11 -i https://pypi.tuna.tsinghua.edu.cn/simple --with <pkg> python -c "..."` 零残留求值。
