# 🛡️ AI-Forge 纯血 Native PDF 解析工程交接文档 (HANDOFF-TOOL.md)

> **版本**：v21.0.0 (2026年8月20日 黄金双轨SOTA测试满分与多模态神经自愈版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **操作系统与编译链**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 13.1 & 12.4 / `uv`  
> **面向对象**：新会话 AI 系统架构师 / 核心开发助手 (零历史上下文障碍无损接管)

---

## 🏢 1. 项目定位、商业模式与核心战略 (Project Overview)

1. **项目定位**：
   * 跨平台工业级 AI 桌面端（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + C-FFI 原生 GPU/CPU 直推）。
   * **算力主权**：彻底废除重型 Python (`.venv`) 沙箱与本地网络/IPC 端口，100% 切换为 Rust 原生驱动（`ort` ONNX Runtime 直推 + `service-pdfium` C-FFI 绑定）。
   * **核心使命**：为企业级 RAG（知识库检索增强生成）与 GraphRAG 提供工业级、高保真、零语义腰斩、数学公式与复杂表格 100% 结构闭合的 Markdown 入库预处理中台。
2. **商业模型与 Token 经济学**：
   * **内测阶段**：完全免费供种子用户与内部使用；
   * **正式运营**：按页面消耗 Token 计费（对齐市面标准约 0.02 元/页）；
   * **个人免费护城河**：个人用户每日赠送 **100 页 PDF 智能解析 + 10 部 2 小时电影字幕转写**（依托端侧本地 RTX 3060/CPU 运算，**云端边际算力成本 = 0 元**，形成降维打击）；
   * **战略终局**：以极速、高颜值的桌面端工坊作为“超级展示橱窗（Showcase）”，承接政企与高校的高客单价私有化 RAG 知识库定制与部署大单。

---

## 📐 2. 统一架构全景与底座分工 (Workspace Architecture)

```text
C:\dev\ai-forge\
├── bin\cuda12\                    【CUDA 13.1 / 12.4 满血 DLL 仓库】 (cublas, cudart, cudnn9, onnxruntime)
├── models\                        【纯血 ONNX 模型资产库】 (PP-DocLayoutV3, PP-FormulaNet-S, SLANet_plus, PP-OCRv6)
├── src-tauri\crates\
│   ├── core-onnx-infer\           【ONNX 硬件会话底座】 (默认激活 CUDA 特性, 自动注入 bin/cuda12 动态库目录)
│   ├── shared-contracts\          【端侧显存守卫】 (VramTokenGuard, 11000MB 令牌并发锁)
│   ├── core-security\             【算力收费站】 (HMAC 设备指纹 + Ed25519 离线许可 + Gatekeeper)
│   │
│   ├── services\                  【5 大底层算法底座 (100% 官方物理对齐，只读防腐区 🛡️)】
│   │   ├── service-pdfium\        : 300DPI 渲染 + 物理尺寸 + 字符级 BBox 提取 + 多路径 dll 自愈绑定
│   │   ├── service-layout\        : PP-DocLayoutV3 官方 25 类别映射 + [0.0, 1.0] 归一化
│   │   ├── service-formula\       : PP-FormulaNet-S (384x384 白底留边 + FastTokenizer + Token 1900 cases 换行)
│   │   ├── service-table\         : SLANet_plus (488x488 BGR + 8 坐标通道 + HTML 单元格重构 + 局部 OCR 填充)
│   │   ├── service-ocr\           : PP-OCRv6 (Det 0.2/0.45/1.4/3000 + Rec 48px 微批次)
│   │   └── service-doc-parse\     : AST 语法树 + RXYC++ 几何排序 (通用几何防腐)
│   │
│   └── tools\pdf-parse\           【纯血桌面端 PDF 解析适配器 (业务中台)】
│       ├── probe.rs               : 2026 SOTA 三级漏斗通用分类器 (0.18s 精准判别矢量/扫描)
│       ├── pipeline.rs            : 🛡️ 统一高保真多模态排版流水线中台 (1:1 承载 98.41% 算法资产)
│       ├── fast_track.rs          : ⚡ 极简门面 (绑定 TextExtractMode::VectorOnly)
│       ├── deep_track.rs          : 🧠 极简门面 (绑定 TextExtractMode::OpticalOcr)
│       ├── hybrid.rs              : 智能按页调度器 (全矢量/全扫描/混合按页缝合)
│       └── service.rs             : 桌面端 Dedicated 本地直出服务契约 (0 Zip 垃圾)
│
└── test\parse\                    【2026 SOTA 黄金评测系统 (Dual-Engine Ground Truth Depot)】
    ├── ground_truth\              : 专家精标黄金真值库 (1_golden.md, 2_golden.md)
    ├── 1.pdf / 2.pdf              : 原生数字矢量 PDF 样本
    ├── 1_scanned.pdf / 2_scanned  : 300DPI 纯位图扫描件 PDF 样本 (96MB/篇)
    └── benchmark_sota.py          : 照妖镜评测脚本 (集成 OmniDocBench TEDS/NED + olmOCR 16 项事实穿透断言)
```

---

## 🟢 3. 本会话已攻克的重大里程碑 (Accomplished Breakthroughs)

1. **数学公式与语法自愈 100% 满分突破**：
   * **公式 (3)**：消灭了宏命令反斜杠脱落故障（如 `operatornamearg r a **mmx`），精准归一化为标准的 `\boldsymbol{\tau}_c^* = \mathop{\operatorname{arg}\operatorname*{max}}_{\tau\in[0,1]} F_c(\tau; P(y|x_l))`；
   * **公式 (4)**：破译了 FastTokenizer 中 `Token 1900` 解码为 `\ ` 导致分段函数挤在一行的难题，精准升格为 `\\`，使 `\begin{cases} ... \end{cases}` 完美渲染为标准上下两行；
   * **公式 (1)**：$\mathcal{L}_{\mathrm{Distill}}$ 双范数与矩阵乘法 100% 高保真转写；
   * **单测全绿**：`service-formula` 4 张真实公式图（`1.png` ~ `4.png`）测试全部 100% 通过（耗时 260~360ms）。
2. **2026 SOTA 黄金评测系统（`benchmark_sota.py`）16/16 满贯全绿**：
   * `1.pdf` [⚡ 矢量轨]：**98.39 分**（Order Score: 100.00 分，4/4 🟢 PASS）
   * `1_scanned.pdf` [🧠 扫描轨]：**97.36 分**（Order Score: 100.00 分，4/4 🟢 PASS）
   * `2.pdf` [⚡ 矢量轨]：**94.40 分**（Order Score: 100.00 分，4/4 🟢 PASS）
   * `2_scanned.pdf` [🧠 扫描轨]：**91.44 分**（Order Score: 100.00 分，4/4 🟢 PASS）
3. **参考文献排版与假大标题拦截**：
   * 部署了 **标题保镖机制（`is_false_heading`）**：拦截以小写字母（如 `## tic...`）、`[数字]` 开头或长篇文本被误打 `## ` 的事故；
   * 实现了 **纯血 Rust 零依赖参考文献自动分行（`format_citations_to_lines`）**：使黏连的 `[6]...[15]` 全部整齐分行。
4. **真实扫描件端到端验证成功**：
   * 将 9 页学术论文 `444.pdf` 渲染为 300DPI 纯位图 `444_scanned.pdf`，经桌面端真机转换，公式 (1)、(3)、(4) 和表格在产物 `444_scanned_hybrid_output.md` 中 100% 完整生成。

---

## 🛑 4. 当前精确卡点与物理根因分析 (Current Bottlenecks & Root Causes)

### 现场真相：
在桌面端前端转换原始数字 PDF `1346.pdf`（或 `444.pdf`）时，耗时较慢（约 53 秒），且产物中的公式变成了 `$$$$` 空框，正文中部分数学符号显示为黄色问号 `⍰`。

### 深度白盒根因逆向：
1. **探针分流过度简化（契约断层 1）**：
   * `probe.rs` 中的分流逻辑仅判断了 `valid_char_count < 50` 和 `readable_ratio < 0.60`，**漏掉了 `math_symbol_count > 0`**；
   * 导致包含复杂数学符号的 9 页数字论文被草率分流给了 CPU 矢量轨（`FastTrackCpu`）；
   * 而 arXiv 论文采用了非标 TeX 嵌入字库（缺失 `ToUnicode` 映射），CPU 矢量流直接读出来的特殊符号变成未定义字符（`\u{fffd}`），在 VS Code 中显示为 `⍰` 问号框！
2. **全局 `split("$$")` 引发的“相位反转大塌陷”（契约断层 2）**：
   * 在 `clean_katex_markdown` 中，代码使用全局 `res.split("$$")` 并按奇偶索引假设公式与正文；
   * 在多页长文档中，一旦某个正文区域包含单 `$` 或公式为空，文档的奇偶索引瞬间发生**全局相位反转**，导致正文被当成公式清洗，公式被当成正文剔除，最终全部坍塌为 **`$$$$`** 空白框！
3. **CUDA DLL 自动并网已就绪，需在编译期默认激活**：
   * `core-onnx-infer/Cargo.toml` 中已将默认特性修改为 `default = ["cuda"]`，并在 `config.rs` 中增加了 `ensure_cuda_dll_registered()` 自动注入 `bin/cuda12` 到进程 `PATH`。

---

## 🚀 5. 新会话下一步单步实施清单 (Action Plan for New Session)

新会话接管后，请严格按照以下步骤单步推进：

### 第一步：修复 `probe.rs` 分流决策与公式智能路由
* 在 `probe.rs` 中，恢复对 `math_symbol_count` 和非标 TeX 字符的敏感检测：
  * 若页面包含公式（`math_symbol_count >= 1`）或检测到大量 `\u{fffd}` 乱码，强制分流给 **`DeepTrackGpu`**（由神经视觉模型直接看图推导）；
* 编译单测：`cargo test --manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml -p pdf-parse`

### 第二步：废除全局 `res.split("$$")`，回归 AST 节点局部闭合
* 在 `pipeline.rs` 的 `clean_katex_markdown` 中，彻底移除全局 `res.split("$$")` 切割逻辑，将 `balance_latex_braces` 严格限定在每个公式节点自身内部（`BlockContent::Formula`）执行，彻底消灭“相位反转”导致的 `$$$$` 灾难。

### 第三步：全量真机重启与 9 页极速压测
* 执行 `pnpm tauri dev`；
* 拖入 `C:\Users\52484\Pictures\444.pdf` 进行全量真机解析；
* 验证 9 页学术论文在 GPU 满血加速下于 **8~12 秒内完成**，且公式 (1)、(2)、(3)、(4) 与正文符号 100% 完整无乱码！

---

## ⚠️ 6. 避坑终结手册与禁忌铁律 (Anti-Patterns / Lessons Learned)

1. **绝对不要在全局 Markdown 字符串上执行 `split("$$")`**：
   长文档中的正文 `$100` 或单 `$` 会导致奇偶索引颠倒，摧毁整篇文档的所有公式！公式规整必须在 AST 节点内部完成。
2. **绝对不要将公式裁切外扩设为小于 25px/10px**：
   过小的裁切边距会导致 $\arg\max$ 和 $\tau_c^*$ 首尾字母被截肢，诱发 BPE 乱码。必须严格锁定 `pad_x = 25.0, pad_y = 10.0`。
3. **绝对不要对 `\operatorname` 使用暴力字母连接正则**：
   禁止使用 `LETTER_TO_NONLETTER_PATTERN` 等粗暴正则剥离反斜杠 `\`。
4. **绝对不要忽略数字 PDF 中的非标 TeX 字体乱码**：
   只要检测到学术公式或非标字符，必须由多模态视觉神经轨（`DeepTrackGpu`）接管。
5. **Windows Native 铁律**：
   * 所有 Rust 编译命令必须显式指定 `--manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml`；
   * 代码落盘必须使用 PowerShell `@' ... '@` 单引号原样字符串与 `Set-Content` 协议；
   * 命令行绝不能开头带有 `#` 注释；
   * 临时断言必须使用 `uv run --python 3.11 python -c "..."` 行内求值。
