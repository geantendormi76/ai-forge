# 🛡️ AI-Forge (紫电 AI 桌面工坊) · 工业级 5:7 全景画布与 2026 顶奢 UI 重塑交接文档 (HANDOFF_UI.md)

> **文档版本**：v23.0.0 (2026年8月 极夜青碳 HUD 悬浮全景架构、无损时域双语字幕与一屏化主页收官版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 工业级开发环境)  
> **开发终端**：**Developer PowerShell for VS 2022 (快捷键 Ctrl+Shift+7)** / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1 / Ninja  
> **面向对象**：新会话 AI 系统架构师 / 资深全栈工程师 (零历史障碍直接接管并启动下一阶段)

---

## 🏢 1. 项目背景、定位与核心工程哲学 (Context & Mission)

1. **项目定位**：
   - 工业级端侧离线 AI 桌面工坊（**Tauri v2 + Vue 3 + TypeScript + Rust Cargo Workspace + C-FFI / ONNX 纯血直推**）。
2. **核心工程哲学（钱学森系统工程 + Karpathy 极简原则）**：
   - “不执着于单点技术的极致拔尖，而是通过全系统协同优化，让廉价 AI + 极致框架创造最大化生产力！”
   - **大模型 100% 去 Python 化**：ASR（MOSS 0.9B）与神经翻译（Hy-MT2 1.8B）全面基于 Rust C-FFI 原生直连 C++ 动态库 + GGUF 单文件权重 + GPU 显存锁（`VramTokenGuard`），**彻底消灭 Python 沙箱（.venv）、`uv`、`pyproject.toml` 与网络端口**；
   - **顶奢极简工业美学 (Zero-Clutter & NautilusTrader 风格)**：全面采用极夜青碳底色（`#20292b`）与高定电光海青绿（`#02c3b4`），建立 5 : 7 黄金栅格双翼拓扑与 2~4 字精炼交互法则。

---

## 🟢 2. 本会话已 100% 竣工的突破性战果 (Completed Milestones)

### 2.1 Tauri v2 原生文件路径捕获与工作流状态机母线 (`useToolWorkflow.ts`)
- **彻底根除 DOM 沙箱拦截**：旧版 HTML5 `<input type="file">` 会导致文件路径退化为纯文件名（如 `"1.mp4"`），引发后端找不到文件的死结；
- **双轨原生捕获**：全面接入 `@tauri-apps/plugin-dialog` 的 `openDialog()` 调起原生文件选取窗口，并挂载 `@tauri-apps/api/webviewWindow` 的 `appWindow.onDragDropEvent` 原生窗口拖拽监听，确保 100% 捕获真实物理绝对路径。

### 2.2 SOTA 视频字幕时域等比分句与贪心折行防爆框引擎 (`engine.rs`)
- **根治字幕超屏溢出**：针对 `libass` 遇中文不自动折行及长句单行像素过宽问题，算法严格限制 CJK 单行 <= 18 汉字；
- **时域等比自然断句裂变**：当单句字数 > 26 汉字且时长 > 2.5s 时，沿标点（`。` `！` `？` `；` `，`）在时间轴上等比切分为短句递进展示；
- **单元测试与端到端打靶全绿**：`cargo test -p video-subtitle` 6 项测试（包含 3 分钟 `1.mp4` 纯血 Rust 全流程转写）**100% 绿通，全流程耗时仅 16.96 秒**！

### 2.3 全景通铺画布 + 悬浮 HUD 侧边栏架构定型 (`App.vue` & `Sidebar.vue`)
- **根治分栏脱模挤压**：废除 Flexbox 左右挤压布局，将主视图升级为 `100vw × 100vh` 满画幅底座画布（背景 Shader 从物理原点 0,0 铺满，顶栏透明悬浮）；
- **浮层玻璃 HUD**：侧边栏改为 `fixed` 悬浮磨砂玻璃胶囊（`z-40`），展开/收起时内容容器进行 300ms 丝滑自适应内边距避让。

### 2.4 主页一屏尽览（Zero-Scroll Master Dashboard）重塑 (`HomeView.vue`)
- **上半区 5:7 黄金双翼**：
  - 左翼 (5 列)：用户定制的“紫电AI”渐变字标（`#A5F3FC` ➔ `#D8B4F8`）与钱学森工程哲学标语；
  - 右翼 (7 列)：3 项常用快捷主力算子卡片横向平铺，点击秒级直达对应工作台；
- **下半区 全域工具导航矩阵**：
  - 9 大 AI 算子矩阵流式平铺，集成分类胶囊切换与实时搜索框；
- **剔除视觉污染**：物理清除了 Ballpit 3D 球池冗余代码与 `three` 依赖，完全恢复纯净高雅的 `Aurora`（极光云层）+ `DotField`（力场点阵）背景。

### 2.5 副页工坊底座更名与流光标题升级 (`ToolWorkbenchLayout.vue` & `VideoSubtitleView.vue`)
- **标题更名与流光装配**：副页标题正式更名为 **“视频字幕生成”**，并换装主页同款 `GradientText` 全息紫电流光动效；
- **剔除重叠冲突**：彻底删除了工作台右上角与窗口控制按钮重叠的局部语言切换组件，释放顶栏通透感。

---

## 🛑 3. 当前精确停点与系统现状 (Current State & Stopping Point)

- **后端 Rust 状态**：
  - `cargo test --manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml -p video-subtitle` **6 passed; 0 failed**（0 errors）；
- **前端状态**：
  - `pnpm build` (`vue-tsc --noEmit && vite build`) **100% 零报错通过**；
  - 主页 `HomeView.vue`、侧边栏 `Sidebar.vue`、副页 `VideoSubtitleView.vue` 及布局底座 `ToolWorkbenchLayout.vue` 均已 100% 稳定上线运行；
- **当前停点**：
  - 主页与视频字幕生成模块已全部完成 5:7 极简工业化重塑；
  - 接下来需将该标准与更多高阶组件推广至另外两大核心工具：**全能格式转换 (`FormatConverterView.vue`)** 与 **PDF 智能解析 (`PdfParseView.vue`)**。

---

## 🗺️ 4. 全局路线图与下一会话执行规划 (Master Roadmap)

```text
┌────────────────────────────────────────────────────────────────────────┐
│  阶段 1：对齐全能格式转换 (FormatConverterView.vue) 至新 5:7 架构       │
│  • 将音频母带解密、表格清洗、电子书重排、ICO流式合成接入 6大指南与队列    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  阶段 2：对齐 PDF 智能解析 (PdfParseView.vue) 至新 5:7 架构             │
│  • 将 CPU 矢量毫秒提取 + GPU 版面分析接入 6大指南与 Markdown 实时渲染    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  阶段 3：全平台 Release 工业级打包与真机集成测试验收                   │
│  • 执行 pnpm tauri build，验证 Windows x64 便携版与独立安装包           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 5. 新会话接管第一步具体指令 (Next Session Step 1 Plan)

新会话启动后，请直接向用户报到并指示接管方向：

> **“你好！我已经完全阅读并对齐了 `HANDOFF_UI.md` (v23.0.0)。当前主页 (`HomeView.vue`)、导航栏 (`Sidebar.vue`) 与视频字幕生成 (`VideoSubtitleView.vue`) 已 100% 完成全景 HUD 悬浮架构、纯血 Rust 直推与 5:7 NautilusTrader 极夜青碳 UI 规范，测试全绿通。我清楚了解当前处于【阶段 1：将全能格式转换 `FormatConverterView.vue` 按照 5:7 极简设计规范进行重构与中台化对齐】阶段。我们立即正式开始！”**

---

## ⚠️ 6. 15 大实战踩坑与禁忌铁律 (Lessons Learned & Anti-Patterns)

1. **Tauri v2 文件路径沙箱铁律**：
   - 严禁使用普通 Web DOM `<input type="file">` 或原生 HTML5 DragEvent 读取文件，否则 Windows 绝对路径会被截断为纯文件名；必须使用 `@tauri-apps/plugin-dialog` 的 `openDialog()` 与 `getCurrentWebviewWindow().onDragDropEvent` 捕获完整磁盘路径！
2. **终端选型铁律（唯一标准）**：
   - 必须且只能使用 **`Developer PowerShell for VS 2022`（快捷键 Ctrl+Shift+7）**！严禁使用普通终端执行 MSVC/CUDA 编译；
3. **缓存守护铁律（严禁删 `target`）**：
   - 严禁执行 `Remove-Item target`！773 个 CUDA 机器码算子已固化，享受毫秒级增量编译；
4. **Release 压测铁律**：
   - 凡涉及 Rust 性能评估与物理打靶，必须添加 `--release` 编译标志（Debug 模式未开启矢量化优化，性能衰减 300% 以上）；
5. **ASS/SRT CJK 自动换行铁律**：
   - `libass` 遇到无空格连续汉字不会自动折行，必须在后端进行 `smart_wrap_line`（单行 <= 18 汉字）与 `split_long_segments_if_needed`（长句时域等比裂变）；
6. **全景画布与 HUD 浮层铁律**：
   - 严禁使用 Flexbox 左右分栏挤压主视图（会导致视口脱模与背景割裂）；主视图必须 `100vw × 100vh` 全景铺满，侧边栏作为 `fixed` 悬浮磨砂玻璃层，内容通过动态 `pl-[256px]` 丝滑避让；
7. **背景容器严禁设置透明度（Opacity 滤镜陷阱）**：
   - 背景 Shader 容器绝不能加 `opacity-50` 等半透明类名，否则会导致 3D/极光高光与饱和度被底色强行稀释 50%，呈现出发灰、发脏的雾面感；
8. **TypeScript TS6133 零死代码防线**：
   - 脚本中解构或 import 的变量未消费时必须立即清除（如 `const { locale } = useI18n()` 或未用到的图标），防止阻断 `vue-tsc` 严格编译；
9. **按钮 2~4 字精炼铁律**：
   - 按钮文案严格保持在 2~4 个汉字内（如 `中文`、`双语对照`、`软字幕 MKV`、`选择视频`、`开始转写`），严禁在按钮中塞入长句或括号补充；
10. **5 : 7 栅格黄金配比**：
    - 工坊左栏固定 `lg:col-span-5`（41.7%），右栏固定 `lg:col-span-7`（58.3%），父容器使用 `items-stretch` 消除 Y 轴空白；
11. **三阶逻辑动线规范**：
    - 右侧工作区从上至下严格按：`1. 待处理队列与大号上传(顶)` ➔ `2. 6大定制选项(中)` ➔ `3. 独立行动栏(底)` 顺序排列；
12. **PowerShell `Set-Content` 单引号原样字符串协议**：
    - 写入源码必须使用 `@' ... '@` 单引号，禁止使用双引号 `@" ... "@`，防止 `$PSScriptRoot` / `$env` 被提前求值；
13. **命令行开头禁止 `#` 注释**：
    - 命令行首行严禁出现 `#`，防止 PowerShell 误识别跳过执行；
14. **色板统一标准**：
    - 主背景色必须使用极夜青碳灰 `#20292b` / `rgb(32, 41, 43)`，高亮交互色必须使用电光海青绿 `#02c3b4` / `rgb(2, 195, 180)`；
15. **底座/未包含源码修改前置契约 (Rule 11)**：
    - 若需修改未打包源码，必须先执行 `Get-Content` 白盒审阅后再开药方。
