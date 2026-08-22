# 🛡️ 紫电 AI (ai-forge) 工业级桌面工坊 - 架构演进与极简打包全域交接文档 (HANDOFF_DB.md)

> **文档版本**：v3.6.0 (2026年8月 SOTA 纯血 Native 极简架构)  
> **项目物理根目录**：`C:\dev\ai-forge`  
> **服务对象**：无编程基础但具战略直觉的项目负责人（熟悉 Rust + Tauri v2 + Vue 3 技术栈，开发环境为 Windows 11 x64、MSVC 2022、Ninja、`uv`、`pnpm`）。  
> **核心定位**：供新会话 AI 系统架构师在零历史上下文状态下，1:1 精确接棒后续功能迭代、全量压测与全球版本发布。

---

## 1. 我们在做什么（核心使命与架构定位）

本项目是基于 **Tauri v2 + 纯血 Rust + Vue 3 (Pinia + Tailwind)** 构建的工业级高性能端侧 AI 生产力工坊（包含四大核心业务工具与 10 大底层算法服务）：
1. **4K/8K 视觉超分（`upscale-48k` / `service-upscale`）**：RealESRGAN 8K 极速切块推演，绑定 `VramTokenGuard` 显存锁，RTX 3060 显卡满血直推；
2. **全能格式转换（`format-converter` / `service-converter`）**：音频解密、结构化表格、Word/EPUB、图片容器、ZIP 防穿越等 5 大领域纯 Rust 算法直出（0 Tokens，0 依赖）；
3. **PDF 智能多模态解析（`pdf-parse` / `service-doc-parse`）**：PDFium 矢量提取 + PP-DocLayoutV3 + PP-OCRv6 + PP-FormulaNet-S + SLANet_plus 深度排版恢复；
4. **视频双语字幕（`video-subtitle` / `service-asr` / `service-translation`）**：FFmpeg 音频抽离 + MOSS 0.9B ASR 角色分流 + Hy-MT2 1.8B 神经翻译 + NVENC 显卡硬压。

---

## 2. 整体规划路线与已完成的核心资产

我们已经严格按照钱学森系统工程思想与 TDD 规范，完成了**三大核心攻关与架构重构**：

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                              紫电 AI 2026 SOTA 全域资产架构图谱                             │
├───────────────────────────────┬───────────────────────────────┬─────────────────────────────┤
│ 模块维度                      │ 落地实现与机制                │ 验证状态 / 客观性能指标     │
├───────────────────────────────┼───────────────────────────────┼─────────────────────────────┤
│ 1. 主安装包极简瘦身           │ 移出 2GB CUDA DLL，仅打基础包  │ ✅ 190.00 MB (体积暴降 >93%) │
│ 2. 现代 currentUser 安装模式  │ %LOCALAPPDATA%\Programs\紫电AI│ ✅ 彻底消除 Windows UAC 权限 │
│ 3. GPU 运行时 R2 按需自愈     │ cuda13-runtime-v1.0.0.zip     │ ✅ 1.78 GB 完备黄金包 (25 DLLs)
│ 4. ONNX 插件自动同级投影      │ 自动拷贝 provider 至主程序同级│ ✅ 4K 超分 10.36 秒 (提速21.9x)
│ 5. 企业级绿色便携协议         │ crates/core-security/portable │ ✅ 单元测试 3/3 通过 (./Data)|
│ 6. GGUF 64KB 极速嗅探底座     │ crates/core-models-download   │ ✅ 5.2ms 破译 1.08GB 混元模型│
│ 7. 品牌全套原生图标           │ pnpm tauri icon 派生 14 套尺寸│ ✅ 1:1 镜面对齐极光闪电新战袍│
│ 8. 全工作空间联合回归压测     │ cargo test --release --workspace│ ✅ 10 大服务 20+ Crate 100% 全绿
└───────────────────────────────┴───────────────────────────────┴─────────────────────────────┘
```

---

## 3. 当前处于哪个节点（Current State & Next Step）

* **当前状态**：
  - 【阶段 1：便携协议】、【阶段 2：GGUF 64KB 嗅探】与【阶段 3：全工作空间 `--workspace` 全量 Release 回归压测】均已 **100% 满分全绿通过**；
  - `src-tauri/tauri.conf.json` 已配置为 `currentUser`，`core-onnx-infer` 已集成自动同级投影；
  - `cuda13-runtime-v1.0.0.zip`（1.78 GB，哈希 `cd9bd6...`）已完整就绪于 Cloudflare R2。
* **下一步立即执行的动作（Next Action）**：
  - 调起 `publish_update.py` 自动递增版本至 **`v0.1.12`**，执行最终生产打包、Ed25519 签名与 Cloudflare R2 全球发布！

---

## 4. 14 大避坑红线（新会话绝对不要再踩的雷区！）

1. ❌ **严禁误删 `cublasLt64_12.dll` (428 MB)**：微软预编译的 `onnxruntime_providers_cuda.dll` 在底层写死了动态寻找该 12 版动态库，缺失会导致静默回退至 CPU 软解（跑 188 秒慢速）；
2. ❌ **严禁使用 `static Once` 锁死动态库注册**：必须使用 `AtomicBool` 状态自愈，防止开机时尚未下载运行时导致后续即使下载完成也无法并网；
3. ❌ **严禁使用 `perMachine` 打包**：`C:\Program Files` 具有强 UAC 权限隔离，导致应用无法自愈投影 DLL；必须锁定 `currentUser`（安装至 `%LOCALAPPDATA%\Programs\`）；
4. ❌ **严禁缺失 `cudnn_engines_precompiled64_9.dll` (521 MB)**：cuDNN 9 的卷积算子库，缺少会在推演第 1 块切片（17%）时直接崩溃闪退；
5. ❌ **严禁将 25 个 GPU 动态库塞回 `tauri.conf.json resources`**：否则安装包会暴增至 2GB 以上，必须保持极简安装包（~190 MB）；
6. ❌ **严禁在 Python 脚本中发送裸 `User-Agent`**：Cloudflare WAF 默认将 `Python-urllib` 拦截为 403，必须显式携带 `headers={"User-Agent": "ZiDianAI-Client/1.0"}`；
7. ❌ **严禁误判 rclone 501 警告**：Cloudflare R2 不支持 S3 ACL，rclone 首轮 501 会自动在第二轮降级分块流式上传并 100% 成功；
8. ❌ **严禁在命令行开头带 `#` 注释**：PowerShell 执行带 `#` 开头的多行命令会导致整行被跳过或解析异常；
9. ❌ **严禁在 Debug 模式下评测性能**：必须执行 `--release`，Debug 下文件 I/O 与反归一化耗时会失真偏慢数倍；
10. ❌ **严禁在 Vue 3 中使用 `ref()` 包装 Tauri 原生 Class**：Vue 3 深度 Proxy 代理会破坏 `#rid` 私有字段，导致热更新报 `Cannot read private member`；
11. ❌ **严禁在 Python f-string `{}` 内部出现反斜杠 `\`**：严格遵循变量外置范式与原始字符串 `r"C:\..."`；
12. ❌ **严禁全量读入 GGUF 模型做属性检查**：必须使用 `core_models_download::gguf_meta::probe_file_header`，仅读前 64KB 即可在 0.005 秒内截停提取元数据；
13. ❌ **严禁漏掉 `onnxruntime_providers_shared.dll` (20 KB)**：Provider 插件的 DllMain 强依赖此共享库，缺失会导致 WinError 1114；
14. ❌ **修改未包含源码必须遵循 Rule 11 前置契约**：严禁盲猜打补丁，必须先输出 `Get-Content` 白盒审阅全量真值源码后再出药方！

---

## 5. 新会话接入后的第一步指令

新会话启动后，直接在项目物理根目录 `C:\dev\ai-forge` 下执行以下一键发布命令，完成最终生产包发布：

```powershell
uv run --python 3.11 C:\dev\ai-forge\publish_update.py
```
