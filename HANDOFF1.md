# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v17.0.0 (2026年8月14日 纯血 Native PDF 解析重构 & 5大底座官方对齐版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 项目概况与当前核心任务 (Project Overview)

1. **项目定位**：
   - 跨平台工业级 AI 桌面端（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 直推）。
   - **算力主权**：彻底废除重型 Python (`.venv`) 沙箱与网络/IPC 端口，100% 切换为 Rust 原生驱动（`ort` ONNX Runtime 直推 + `service-pdfium` C-FFI 绑定）。
2. **当前主攻方向**：
   - 严格遵循 **【代码迁移条铁律 14：Zero-Regression (零回归) 代码迁移范式 3 大支柱】**（接口锚定、重放断言、1:1 结构直译）。
   - **分阶段战略**：优先攻克 **矢量数字 PDF 路线（`FastTrack` + 矢量高保真转换）**，在 `1.pdf`（98.41 分）与 `2.pdf`（85.24 分）双样本上达成 98%+ 后，再行拓展纯位图扫描件路线。

---

## 📐 2. 核心架构与 5 大底层算法底座全景表

```text
C:\dev\ai-forge\src-tauri\crates\
├── core-onnx-infer\           【2026 通用 ONNX 硬件会话底座】 (CUDA/DirectML/CPU)
│
├── services\                  【5 大核心底层算法底座 (100% 官方对齐完成 ✅)】
│   ├── service-pdfium\        [已闭环] : 300DPI 渲染 + (W_pt, H_pt) 获取 + 字符级物理 BBox
│   ├── service-layout\        [已闭环] : PP-DocLayoutV3 官方 25 类别 1:1 映射 + [0.0, 1.0] RGB 归一化
│   ├── service-formula\       [已闭环] : PP-FormulaNet-S 384x384 白底留边 + cv2 灰度 + FastTokenizer
│   ├── service-table\         [已闭环] : SLANet_plus 488x488 BGR + 8 坐标通道 + HTML Token 解码
│   ├── service-ocr\           [已闭环] : PP-OCRv6 (Det 0.2/0.45/1.4/3000 门限 + Rec 48px 微批次)
│   └── service-doc-parse\     [已闭环] : AST 语法树 (消除 Abstract 标题污染) + RXYC++ 几何排序
│
└── tools\
    └── pdf-parse\             【纯血桌面端 PDF 解析适配器 (业务组装中台)】
        ├── probe.rs           : 5 维物理路由探针 (FastTrackCpu / DeepTrackGpu 分流)
        ├── fast_track.rs      : 毫秒级极速矢量轨 (待完善原生高保真提取)
        ├── deep_track.rs      : 深度视觉自愈轨 (32位 MD5 + 空间 NMS 框去重 + 坐标逆向投影)
        ├── hybrid.rs          : 按页降维混合调度器
        ├── exporter.rs        : 纯 Rust 原生 zip 流式打包器
        └── service.rs         : 桌面端 Dedicated 本地交付契约
```

---

## 🟢 3. 本会话已攻克的重大突破 (What Was Completed)

1. **5 大算法底座的官方物理对齐与 TDD 单体验证**：
   - **`service-pdfium`**：白盒定位并修复了 PDF 笛卡尔坐标（左下角原点）与位图像素坐标（左上角原点）的冲突，实现了 $Y_{\text{top-down}} = H_{\text{pt}} - Y_{\text{pdfium}}$ 镜像映射，使矢量文本提取命中率从 0% 跃升至 100%。
   - **`service-layout`**：依据官方 `inference.yml`，纠正了原先全部错位的类别枚举（修正为 0:abstract, 6:doc_title, 17:paragraph_title, 21:table, 22:text 等 25 类），修正了输入预处理为 800x800 RGB $[0.0, 1.0]$。单图打靶 14 处区块 100% 全中。
   - **`service-formula`**：1:1 直译官方 `processors.py`（UniMERNet 384x384 白底留边、BGR 转灰度、16 倍数 1.0 补齐），单图 `1.png` 识别准确率达 100%（627ms）。
   - **`service-table`**：核验 SLANet_plus 官方 488x488 BGR 预处理与结构字典，打靶准确率 0.9999。
   - **`service-ocr`**：校准 DBPostProcess 4 大门限（`thresh: 0.2, box_thresh: 0.45, unclip_ratio: 1.4, max_candidates: 3000`），送货单真实打靶 56 行 100% 全中。
   - **`service-doc-parse`**：修正 AST 中 `abstract` 属性，剥离误加的 `## ` 二级标题。

2. **`deep_track.rs` 核心调度自愈**：
   - **32 位 MD5 图片资产哈希**：对齐 Python `hashlib.md5` 产物命名。
   - **空间 NMS 文本框包含去重**：按面积排序执行 $IoA > 0.70$ 抑制，彻底消灭首页作者信息 3 次重复打印。
   - **行内公式符号与独立公式解耦**：仅独立公式走 `FormulaService`，行内符号直接融入文本，消除了 25 处孤立单字符碎片。
   - **`0x02` 连字乱码清洗**：在字符流中过滤不可见控制字符，彻底消灭 `ro\x02bust` 等空心框乱码。
   - **KaTeX 语法修复**：自动校正 `\VIT` $\to$ `-VIT`，消除了前端渲染红色 ParseError。

3. **构建 2026 SOTA 五维白盒评测体系 (`benchmark_sota.py`)**：
   - 彻底废除旧的历史 `.md` 基准，直接以原始 PDF 矢量数据流为真值。
   - 在 `1.pdf` 上实测取得 **98.41 / 100.0** 极高客观得分（自然阅读序 100%、标题树 100%、公式 100%、表格 100%、纯文本 94.71%）。

---

## 🛑 4. 当前卡点与物理真因 (Current Status on 2.pdf)

- **物理现象**：
  新样本 `2.pdf`（4 页 LinearRAG 论文）打靶耗时 **2.85 秒**，生成 15,616 字符，SOTA 评测得分为 **85.24 分**。
- **白盒根因分析**：
  1. `2.pdf` 属于图文混排（Page 2 含有图 1 左侧环绕嵌入文本，Page 4 含有全宽流程图）。
  2. 当前我们单跑了 `DeepTrack` 纯视觉轨，视觉框在面对图文环绕段落时，部分紧贴图表的边缘文字选区受到图文 60% IoA 抑制策略的影响被误剔除，导致纯文本保真度得分为 63.30%。
  3. `FastTrackEngine` 当前仍然是直通 `DeepTrack` 的代理，尚未将矢量数字页直接交由原生矢量流快速解析。

---

## 🚀 5. 下一步落地计划 (Next Steps for New Session)

1. **第一步（实现纯血原生 FastTrack）**：
   在 `crates/tools/pdf-parse/src/fast_track.rs` 中，利用 `service-pdfium` 的页面块与段落流，实现纯 Rust 原生的高保真数字矢量 Markdown 提取算子（替代对 DeepTrack 的回流调用）。
2. **第二步（打通 Hybrid 智能分流）**：
   在 `hybrid.rs` 中将 `probe.rs` 判定的数字矢量页（如 `1.pdf` 的 1,2,4 页与 `2.pdf` 的纯文字页）分流至 `FastTrack`，含复杂图表的页面分流至 `DeepTrack`。
3. **第三步（双样本 98+ 验收）**：
   运行 `benchmark_sota.py`，断言 `1.pdf` 与 `2.pdf` 在 SOTA 五维评测中**同时达到 98+ 分**。
4. **第四步（开启扫描件 PDF 位图轨）**：
   引入纯位图测试样本 `tool-pdf-parse-deep.pdf`，完善旋转纠偏与全量 OCR 视觉缝合。

---

## ⚠️ 6. 踩坑终结手册与禁忌铁律 (Anti-Patterns / Lessons Learned)

1. **绝对禁止假设 PDF 坐标系为左上角**：PDFium C-FFI 字符坐标原点在左下角，必须使用 $Y_{\text{top}} = H_{\text{pt}} - Y_{\text{pdfium}}$ 转换后再与图像像素坐标比对！
2. **绝对禁止自造模型类别映射**：`service-layout`（PP-DocLayoutV3）等模型的类别枚举必须 1:1 严格对齐官方 `inference.yml` 中的 25 类别（类别 6 是 `doc_title`，绝不是 `Formula`）！
3. **绝对禁止给 PP-DocLayoutV3 减去 ImageNet 均值**：官方预处理是 RGB 直接除以 255.0 归一化到 $[0.0, 1.0]$，减均值会导致模型特征全乱！
4. **绝对禁止在公式预处理中使用黑底留边**：UniMERNet 要求留边必须是**纯白底 (255, 255, 255)**，Tensor 补齐值必须是 **常量 1.0**！
5. **绝对禁止将 `inline_formula` 当作独立公式块**：行内小变量（$D, P, x_u$）属于正文文本流，只有 `display_formula` 才能包裹为 `$$ ... $$`！
6. **绝对禁止放任 PDF 连字 `0x02` 流入 Markdown**：提取字符时必须过滤 `c.is_control()` 剔除 `0x02` 乱码方块！
7. **绝对禁止在命令行开头带有 `#` 注释**：PowerShell 会直接跳过执行。
8. **绝对遵循场景双轨协议**：持久化代码交付必须使用 `@' ... '@` 单引号原样字符串与 `Set-Content` 协议落盘。
