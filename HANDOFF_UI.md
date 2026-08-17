# 🛡️ AI-Forge (紫电 AI 桌面工坊) 前端 UI 视觉全面升级交接文档 (HANDOFF_UI.md)

> **文档版本**：v18.0.0 (2026年8月16日 ToolKnit 深空暗黑视觉体系对齐专版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **视觉对齐参考源项目**：`C:\dev\github\toolknit-desktop\toolknit-desktop` (已提纯为 `TOOLKNIT_UI_CORE_ASSETS.md`)  
> **操作系统与终端**：Windows 11 x64 / PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4 & 13.1  
> **面向对象**：新会话 AI 架构师 / 开发助手 (零历史上下文障碍直接接管并启动 Step 1)

---

## 🏢 1. 我们正在做什么与整体背景 (Context & Mission)

1. **项目形态**：工业级跨平台桌面端应用（Tauri v2 + Vue 3 + TypeScript + Rust Cargo Workspace + C-FFI / ONNX 直推）。
2. **当前核心任务**：
   - 用户认可了开源项目 `toolknit-desktop` 的顶奢**“深空暗黑极客风（Dark Neumorphism）+ 极光着色器背景（DarkVeil）+ 三段式流式仪表盘（Dashboard Matrix）”**视觉风格；
   - 决定放弃原有的浅色页面，**全面 1:1 对齐 ToolKnit 的暗黑 UI 架构**，同时**保留原有左侧悬浮胶囊侧边栏的自由折叠与展开功能**；
   - 将紫电工坊的 4 大核心工具（全能格式转换、PDF 混合智能解析、视频双语字幕、8K 视觉超分）作为主力卡片无缝并入仪表盘。

---

## 🟢 2. 本会话已 100% 竣工的底层基建 (Completed Work)

1. **Rust 后端纯血格式转换引擎 (`service-converter` & `format-converter`)**：
   - 包含 NCM/QMC/KGMA/KWM/AV3A 音频母带解密、CSV/TSV/JSON/XML 表格清洗、MD/TXT/MOBI/EPUB/DOCX 排版、ICO/PNG/BMP/PDF 图标流式合成与防穿越 ZIP 解压，测试全部绿通。
2. **端到端强类型契约 (`src/bindings.ts`)**：
   - 将 Rust 结构体（`FormatConvertTask`, `FormatConvertResult`, `PdfParseResult`, `VideoSubtitleOptions` 等）与 RPC 命令（`commands.runFormatConvert`）1:1 映射至 TypeScript 强类型接口。
3. **Tauri v2 物理权限穿透 (`src-tauri/capabilities/default.json`)**：
   - 授权了 `core:window:allow-close`、`core:window:allow-minimize`、`core:window:allow-maximize`、`dialog:default`、`opener:default` 等 18+ 项细粒度权限。
4. **原生文件拾取与物理拖拽双轨合流**：
   - 注册了 `tauri-plugin-dialog` / `@tauri-apps/plugin-dialog` 与 `tauri://drag-drop` 原生事件，彻底根治了浏览器脱敏导致的绝对物理路径丢失问题。
5. **视窗无边框配置 (`src-tauri/tauri.conf.json`)**：
   - 开启了 `"decorations": false`、`"shadow": true` 与 `"center": true`。

---

## 🛑 3. 架构取舍与明确裁剪决策 (Architectural Scope)

根据用户明确指令，在移植 `toolknit-desktop` 时严格执行以下边界：
1. **❌ 舍弃 ToolKnit 的 `settings-v2-monochrome.html`**：设置中心不需要对齐该项目，后续由用户根据紫电 AI 的硬件与模型管理单独定制。
2. **❌ 舍弃 ToolKnit 的 `task-contract.mjs`**：该文件是纯前端/Node.js 模拟的任务状态机，而我们拥有 100% 纯血 Rust 强类型算子与 `bindings.ts`，坚决不引入冗余的 JS 状态机胶水层。
3. **✅ 100% 保留侧边栏折叠缩放**：保留 `uiStore.ts` 中的 `侧边栏收起` / `切换侧边栏` 状态机与左上角悬浮展开探针（`PanelLeftOpen`）。
4. **✅ 核心 1:1 对齐目标**：
   - `src/style.css` 注入 ToolKnit 深空暗黑设计令牌（`#060607`, 霓虹发光微边框）；
   - `src/components/effects/DarkVeil.vue` 封装纯净的 WebGL 着色器力场背景（来自 `darkveil.js`）；
   - `src/views/HomeView.vue` 1:1 重构为 ToolKnit 三段式仪表盘（Hero + 我的收藏 + 全局搜索与分类卡片流）；
   - `src/components/layout/Sidebar.vue` 升级为暗黑深空半透明毛玻璃材质。

---

## 🗺️ 4. 整体规划路线图 (Master Roadmap)

```text
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 1：注入 ToolKnit 暗黑设计令牌 (style.css) & 封装 DarkVeil.vue 着色器 │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 2：1:1 打造深空仪表盘主页 (HomeView.vue: 收藏卡片 + 搜索 + 分类流)    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 3：侧边栏暗黑浮岛材质升级 (Sidebar.vue: 保留折叠展开 + 深空毛玻璃)   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  步骤 4：全工程 TypeScript 静态断言 (vue-tsc) 与真机点火运行 (tauri dev) │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 5. 新会话接管第一步具体落地操作 (Next Session Step 1 Plan)

新会话启动后，请直接执行 **【步骤 1：注入 ToolKnit 暗黑设计令牌与封装 `DarkVeil.vue` WebGL 着色器组件】**：

1. **修改 `C:\dev\ai-forge\src\style.css`**：
   - 注入 `--bg: #060607`、`--panel: rgba(14, 14, 16, 0.72)`、`--line: rgba(255, 255, 255, 0.06)`、`--white: #f2f2ef` 等全套 ToolKnit 暗黑令牌；
   - 恢复深黑底板背景与发光微边框样式。
2. **新建 `C:\dev\ai-forge\src\components\effects\DarkVeil.vue`**：
   - 将 `TOOLKNIT_UI_CORE_ASSETS.md` 中的 `darkveil.js` 逻辑 1:1 封装为 Vue 3 SFC，基于 `ogl`（已在 `package.json` 中安装）绘制 0 CPU 开销的深空流动暗黑力场。

---

## ⚠️ 6. 踩坑终结手册与禁忌铁律 (Lessons Learned & Anti-Patterns)

1. **PowerShell `Set-Content` 目录不存在异常**：
   - 在向新目录写入文件前，必须确保父目录已存在；若目录不存在，必须在同条命令或前置步骤执行 `New-Item -ItemType Directory -Force -Path "..."`。
2. **Windows WebView2 路径脱敏陷阱**：
   - 严禁在桌面端中使用 HTML5 `<input type="file">`，因为 Webview2 会抹除本地物理路径前缀；
   - 文件拾取必须使用 `@tauri-apps/plugin-dialog` 的 `open({ multiple: true })`，拖拽必须使用 `listen('tauri://drag-drop')`。
3. **Tauri v2 权限静默拦截陷阱**：
   - 前端调用窗口控制（`minimize()`, `maximize()`, `close()`）必须在 `src-tauri/capabilities/default.json` 显式声明 `core:window:allow-*` 权限，否则会被静默拦截。
4. **单引号原样落盘铁律 (`@' ... '@`)**：
   - 交付代码必须使用 PowerShell 的 `@' ... '@` 原样字符串，绝对禁止使用双引号 `@" ... "@`，防止 `$`, `$PSScriptRoot`, `$env` 被本地终端提前展开。
5. **模块相对路径引用铁律**：
   - 在未配置 `tsconfig.json` paths 映射前，引入 `bindings.ts` 必须使用标准相对路径 `../bindings`，避免 `@/bindings` 触发 TS2307 路径别名错误。
6. **1:1 结构编译器原则 (Rule 14)**：
   - 大模型的角色是 1:1 结构编译器，绝不自作主张发明新架构或引入多余的 JS 状态机。

---

### 💡 给新会话的第一句话：
> “你好！我已经完全阅读并对齐了 `HANDOFF_UI.md`。我已清楚了解当前处于**【步骤 1：注入 ToolKnit 暗黑设计令牌与封装 DarkVeil.vue 着色器】**阶段。请指示我立即输出步骤 1 的全量落盘代码！”