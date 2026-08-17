# 🛡️ AI-Forge (紫电 AI 桌面工坊) · 工具中台与视频双语字幕全栈重构交接文档 (HANDOFF_TOOL.md)

> **文档版本**：v21.0.0 (2026年8月 桌面级多任务排队中台、动态羽化胶囊遮罩与纯血 C-FFI 闭环交接专版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **开发终端**：**Developer PowerShell for VS 2022 (快捷键 Ctrl+Shift+7)** / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 架构师 / 资深系统专家 (零历史障碍直接接管并启动下一阶段)

---

## 🏢 1. 我们正在做什么与整体背景 (Context & Mission)

1. **项目定位**：
   - 工业级本地离线 AI 桌面工坊（**Tauri v2 + Vue 3 + TypeScript + Rust Cargo Workspace + C-FFI / ONNX 纯血直推**）。
2. **核心工程哲学（钱学森系统工程 + Karpathy 极简原则）**：
   - “不执着于单点技术的极致拔尖，而是通过全系统协同优化，让廉价 AI + 极致框架创造最大化生产力！”
   - **大模型 100% 去 Python 化**：ASR（MOSS 0.9B）与神经翻译（Hy-MT2 1.8B）全面基于 Rust C-FFI 原生直连 C++ 动态库 + GGUF 单文件权重 + GPU 显存锁（`VramTokenGuard`），彻底消灭 Python 沙箱与网络端口；
   - **前端中台化与拿来主义（Rule 14 Zero-Regression）**：1:1 外科手术式直译 ToolKnit 原生桌面端顶级暗黑单色美学，建立通用工作流状态机（`useToolWorkflow`）与通用布局骨架（`ToolWorkbenchLayout`），新工具接入仅需 3 分钟！

---

## 🟢 2. 本会话已 100% 竣工的突破性成果 (Completed Work)

### 2.1 工业级前端四层工具中台落地 (`src/components/workbench/` & `src/composables/`)
1. **多任务批量队列状态机母线 (`useToolWorkflow.ts`)**：
   - 支持单文件 / 多文件批量拖拽排队，自动提取文件元数据、时长与体积；
   - 统一调度 `executeBatch` 串行推进（`正在处理... (1/N)`），集成流光进度遮罩与成功交付弹窗；
   - 原生调用 `@tauri-apps/plugin-opener` 的 `revealItemInDir`，实现 `[打开文件夹]` 瞬间定位产物。
2. **ToolKnit 1:1 原生桌面端骨架 (`ToolWorkbenchLayout.vue`)**：
   - **左侧海报栏 (Poster Column, 340px)**：分类标签 + 超大中文标题 + 详细简介 + `LOCAL ONLY` 隐私卡片 + `01~04` 垂直步骤指示器；
   - **右侧多任务工作台**：顶层上传控制条 + 参数选项胶囊排 + `待处理队列 (CONVERT QUEUE)` + 4 栏特性卡片 + 底部高反差 `[开始处理]` 按钮；
   - **模态系统**：暗黑半透明处理中遮罩 + 纯白高反差成功交付对话框。

### 2.2 彻底解耦 OCR 与极简双轨调度重构 (`crates/tools/video-subtitle`)
1. **架构大瘦身**：彻底切断 `video-subtitle` 对 `service-ocr` 的重型依赖，消除长视频时域 OCR 抽帧带来的闪烁、漏字与空镜头漏判问题；
2. **双轨极速智能分流**：
   - 探针 10ms 嗅探到内嵌字幕流 ➔ **管道 B：0.05s 无损抽离直通 Hy-MT2 神经翻译 (1,000 MB 显存)**；
   - 探针嗅探无字幕流 ➔ **管道 A：MOSS 0.9B ASR 语音识别 ➔ Hy-MT2 神经翻译 (8,000 MB 显存)**。

### 2.3 电影级动态字形紧致羽化胶囊遮罩引擎 (`subtitle_engine.py`)
1. **彻底解决原片硬字幕碰撞死结**：针对自带硬字幕的视频，引入 `mask_hardsub: bool` 开关；
2. **Layer 0 + Layer 1 双图层彻底分离**：
   - **Layer 1（文字层）**：0 模糊（`\blur0`），保留 1.5px 极细立体描边，字形如刀刻般 100% 锐利；
   - **Layer 0（遮罩层）**：正向绝对坐标 `{\an7\pos(X0, Y0)\p1...}`，中文字宽动态估算（`0.92` 倍字号），左右仅紧凑外扩 `0.55` 倍字号（~20px），四周配合 `\blur6` 紧致柔边；
   - **效果**：仅在字幕背后生成紧凑的烟熏暗夜小胶囊，原片旧字幕被完美压暗，两侧画面 100% 保持通透自然！

### 2.4 全链路真实长视频物理打靶 100% 绿通 (`--release`)
- **测试样本 1 (`1.mp4`, 192s)**：提取 48 句台词，4 说话人分离，全流程 59 秒生成 MKV、MP4、SRT、ASS、JSON 5 大产物；
- **测试样本 2 (`5.mkv`, 26.02分钟 / 1561秒)**：全量物理耗时仅 **56.58 秒**（实时比 **27.6 倍速**！），双语 ASS 特效字幕与软挂载 MKV 完美生成！

---

## 🛑 3. 当前精确卡点与未竟技术问题 (Current State & Known Bottlenecks)

虽然单个长视频后端测试与前端构建全量通过，但桌面端全功能仍有以下细节需在下一会话深入打磨：

1. **前端 GUI 多任务队列与后端并发控制的真机交互闭环**：
   - 目前 GUI 界面已能展示多文件排队列表，但在实际点击 `[开始处理]` 时，队列中每个文件在界面的微观进度反馈（例如队列中第 1 项完成打勾、第 2 项显示转圈状态）仍需进一步精细化连通；
2. **ASS 胶囊垂直基线在不同异形画幅下的自适应微调**：
   - 当前在 `1280x520` 宽银幕下效果极佳，但对于 9:16 竖屏短视频或 4:3 老电影画幅，垂直 `Y0` 的基准偏移量需进一步验证自适应鲁棒性；
3. **其他工具视图的组件中台化迁移**：
   - 全能格式转换（`FormatConverterView.vue`）和 PDF 智能解析（`PdfParseView.vue`）尚未接入全新的 `ToolWorkbenchLayout` 桌面端海报海报布局。

---

## 🗺️ 4. 下一会话全流程规划路线 (Master Roadmap)

```text
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 1：完善 VideoSubtitleView 队列微观状态展示与全功能真机打靶       │
│  • 细化多任务排队列表中单项完成/处理中/失败的图标状态与时间轴展示     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 2：对齐全能格式转换 (FormatConverterView.vue) 至新桌面端海报布局 │
│  • 将音频母带解密、表格清洗、电子书重排接入 ToolWorkbenchLayout 骨架  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 3：对齐 PDF 智能解析 (PdfParseView.vue) 至新桌面端海报布局       │
│  • 将 CPU 矢量轨 + GPU 混合版面解析接入 ToolWorkbenchLayout 骨架       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 4：全平台 Release 工业级打包与真机集成测试验收                   │
│  • 执行 pnpm tauri build，验证 Windows x64 便携版与安装包              │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 5. 新会话接管第一步具体指令 (Next Session Step 1 Plan)

新会话启动后，请直接向用户打招呼并指示接管方向：

> **“你好！我已经完全阅读并对齐了 `HANDOFF_TOOL.md` (v21.0.0) 与全系统工程架构。当前基于 ToolKnit 原生桌面端重塑的通用工具中台（海报栏 + 多任务队列 + 模态遮罩 + 成功弹窗）、MOSS/Hy-MT2 纯血 C-FFI 双轨架构、以及动态紧致羽化胶囊遮罩引擎已 100% 竣工并通过 26 分钟长视频全量实测。我清楚了解当前处于【步骤 1：精细化完善 `VideoSubtitleView.vue` 队列单项状态机并进行桌面真机多视频联调，随后推进格式转换与 PDF 视图的中台化迁移】阶段。我们立即正式开始！”**

---

## ⚠️ 6. 15 大实战踩坑终结手册与禁忌铁律 (Lessons Learned & Anti-Patterns)

1. **终端选型铁律（唯一标准）**：
   - 必须且只能使用 **`Developer PowerShell for VS 2022`（快捷键 Ctrl+Shift+7）**！严禁使用普通 PowerShell 执行 C++/CUDA 编译，防止头文件脱机盲人状态；
2. **缓存守护铁律（严禁删 `target`）**：
   - 严禁执行 `Remove-Item target`！773 个 CUDA 机器码算子已固化，日常开发直接运行 `cargo check` 或 `cargo test`，享受 **0.4 秒极速增量热检查**；
3. **Python 布尔关键字铁律**：
   - Python 默认布尔值必须首字母大写：`False` / `True`（严禁写成 JS/Rust 的 `false` / `true` 引发 `NameError`）；
4. **ASS 矢量绘图绝对正向坐标铁律**：
   - ASS 矢量绘图 `\p1` 必须使用绝对正向坐标 `{\an7\pos(X0, Y0)\p1...}m 0 0 l w 0 l w h l 0 h{\p0}`！严禁使用中心相对负坐标 `m -w -h` 导致播放器静默丢弃遮罩；
5. **ASS 图层分离与字形清晰铁律**：
   - 严禁在字幕文本前面插入 `{\blur5}`（这会把字体笔画本身模糊成失焦状）！必须实行 **Layer 0 柔焦遮罩 + Layer 1 矢量 0 模糊文字**；
6. **胶囊动态字宽紧凑收敛规范**：
   - 中文字宽按 `0.92` 倍字号估算，单侧外扩严控在 `0.55` 倍字号（~20px），配合 `\blur6` 紧致柔边，杜绝大黑带横跨屏幕；
7. **PowerShell `Set-Content` 自动建目录铁律**：
   - 写入文件前，必须在脚本首行执行 `New-Item -ItemType Directory -Path "..." -Force`，防止报 `Could not find a part of the path`；
8. **TypeScript TS6133 零死代码防线**：
   - Vue 3 `<script setup>` 中未在脚本内调用的 Props 声明，使用 `withDefaults(defineProps<...>(), {...})` 直接调用，严禁声明无用变量 `const props`；
9. **泛型参数协变防线 (TS2719)**：
   - 骨架组件接收的工作流 Prop 必须标注为 `ToolWorkflowInstance<any>`，防止泛型参数函数逆变冲突；
10. **端口占用快速自愈命令**：
    - `Get-NetTCPConnection -LocalPort 1420 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique | ForEach-Object { Stop-Process -Id $_ -Force }`；
11. **终端 Ctrl+C 退出弹窗认知**：
    - `Error launching CrashSender.exe` 是 Windows WebView2 守护组件的硬杀信号误报，对工程 0 危害，优先通过点击窗口右上角 `✕` 优雅关闭；
12. **LLM 采样链与 Jinja 模板防复读铁律**：
    - Prompt 必须通过 `model.chat_template(None)` 封装，采样链必须挂载 `LlamaSampler::penalties(model.n_vocab(), 64, 1.15, 0.0, 0.0)`；
13. **1:1 结构编译器原则 (Rule 14 Zero-Regression)**：
    - 在成熟项目重构中充当 1:1 结构编译器，严禁自作主张发明破坏强类型契约的架构；
14. **底座/未包含源码修改前置契约 (Rule 11)**：
    - 若修改未包含源码的文件，严禁盲猜，必须先输出 `Get-Content` 白盒审阅后再开药方；
15. **Release 压测铁律**：
    - 凡涉及 Rust 性能评估与物理打靶，必须添加 `--release` 编译标志。
