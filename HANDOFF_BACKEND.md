# 🛡️ AI-Forge (AI 桌面工坊) 终极工程交接文档 (HANDOFF_BACKEND.md)

> **文档版本**：v5.0.0 (2026年8月9日 Windows Native 工业级 SSOT 终极交接版)  
> **项目物理绝对路径**：`C:\dev\ai-forge` (Windows Native 开发环境)  
> **操作系统与终端**：Windows 11 x64 / Windows PowerShell 7+ / MSVC (`x86_64-pc-windows-msvc`) / CUDA 12.4/13.1  
> **面向对象**：新会话 AI 架构师 / 开发助手 (无历史上下文接管)

---

## 🏢 1. 项目概况与架构宣言 (Project Overview & Manifesto)

1. **物理形态**：跨平台 AI 桌面端应用（Tauri v2 + Vue3 + TypeScript + Rust Cargo Workspace + Windows Native Python `uv` 沙箱）。
2. **算力主权**：100% 本地隐私推理 (Local Inference)，算力完全由用户本地 CPU/GPU 承担，数据绝不上云，规避隐私风险。
3. **通信铁律（0 网络端口定理）**：
   - Rust 主程序与 Python 沙箱之间**绝对禁止使用网络端口（TCP/HTTP/UDS）**，彻底消除 Windows Defender 防火墙警告。
   - 强制采用 **`Stdio + 4 字节大端序长度头 JSON-RPC`** 专线，配合 Python 入口处的 **OS 级 FD 劫持 (`os.dup2(2, 1)`)**，彻底封死 CUDA/C++ 日志对 JSON 通信流的污染。
4. **三层解耦与零领域污染**：
   - **底座基建层 (`core-*`)**：`core-ipc`、`core-security`（硬件指纹 + Gatekeeper 拦截 + Ed25519 离线验签）、`core-models-download`、`shared-contracts`（`VramTokenGuard` 显存锁，RTX 3060 预留 11,000MB 池）。
   - **AI 共享服务底座层 (`services/*`)**：`service-asr` (MOSS 0.9B 语音识别) 与 `service-translation` (混元 Hy-MT2 1.8B GGUF 神经翻译)。只输出纯粹 AI 数据（纯 `RawSegment`、纯文本数组），严禁夹带字幕格式/颜色代码。
   - **用户产品工具层 (`tools/*`)**：组合服务与 UI 渲染（如 `tool-video-subtitle` 双语字幕工坊）。
5. **Windows 原生开发定理**：全面采用 Native Windows 宿主（PowerShell + MSVC 2022/2026 + Windows 原生 `uv` + Rust `x86_64-pc-windows-msvc`）进行全生命周期开发与构建，实现本地调试与最终 Windows `.exe` 安装包 1:1 的绝对对齐。

---

## 🗺️ 2. 整体路线图与当前进度 (Roadmap & Status)

### 🟢 阶段一：物理底座与 IPC 通信基建 (100% 完成 ✅)
- `[x] Step 1.1` Tauri v2 + Vue3 + Cargo Workspace 初始化完成。
- `[x] Step 1.2` `core-ipc` crate 完成，实现 `spawn_uv_worker` 绑定沙箱。
- `[x] Step 1.3` Python 侧 `hijack_stdout()` OS 级描述符劫持（`os.dup2(2, 1)`）完成。

### 🟢 阶段二：单工具闭环与模型资产管理 (100% 完成 ✅)
- `[x] Step 2.1` `tool-pdf-parse` 双轨解析引擎完成，打靶测试满分。
- `[x] Step 2.2` `core-models-download` 模型资产下载器完成，支持 SHA256 秒级校验。
- `[x] Step 2.3` `shared-contracts` 端侧显存守卫（`VramTokenGuard`）并网绿通。

### 🟢 阶段三：SaaS 云端中台与鉴权并网 (100% 完成 ✅)
- `[x] Step 3.1` 硬件指纹采集器 (`core-security`) 完成（HMAC-SHA256 生成 64 位指纹）。
- `[x] Step 3.2` Cloudflare D1 + Workers Serverless 鉴权中台完成。
- `[x] Step 3.3` Ed25519 离线授权验签引擎（`LicenseVerifier`）完成。

### 🟡 阶段四：多工具矩阵拓展与 Windows Native 架构迁移 (进行中 🚀)
- `[x] Step 4.1` **`service-asr` 共享底座** ➔ **100% 完成 ✅**（输出纯净 `RawSegment`）。
- `[🟡] Step 4.2` **`service-translation` 神经翻译共享底座** ➔ **突破核心瓶颈，处于点火收尾阶段 🟡**
  - **环境迁移**：项目已从 WSL2 全量同步落盘至 Windows Native 宿主物理路径 `C:\dev\ai-forge`。
  - **轮子替换**：已成功下载并安装官方专为 Windows 打包的 CUDA 12.4 预编译 GPU 轮子 `llama_cpp_python-0.3.34-py3-none-win_amd64.whl`。
  - **依赖补全**：已成功向 `.venv` 沙箱注入 PyPI 官方 `nvidia-cuda-runtime-cu12`、`nvidia-cublas-cu12` 与 `nvidia-cuda-nvrtc-cu12`，在 `.venv\Lib\site-packages\` 下落盘了 **902 MB 的纯血 `ggml-cuda.dll`** 以及 `cudart64_12.dll`、`cublas64_12.dll`、`nvrtc64_120_0.dll`。
  - **DLL 白盒探针断言**：通过探针验证，`llama.dll` 的 Windows 依赖挂载已实现 **100% 成功（Handle: `7ffde02e0000`）**。
- `[ ] Step 4.3` **`tool-video-subtitle` “紫电青霜” 视频双语字幕工坊构建**（下一个待开发任务）。
- `[ ] Step 4.4` **全量主程序 Windows Native 并网与 Vue3 RPA 纯白 UI 渲染**。

---

## ✅ 3. 本会话已攻克的重大物理物理瓶颈与白盒证据

1. **破除 WSL2 ➔ Windows 交付断层**：
   摒弃了 WSL2 中复杂的 `.so` 动态库与 Linux 虚拟路径，全面切换至 Windows 宿主机（PowerShell + `C:\dev\ai-forge`）。
2. **解决 `llama-cpp-python` CPU 慢速退化瓶颈**：
   白盒证实 PyPI 默认发的是纯 CPU 轮子（内部仅有 `libggml-cpu.so`）。我们通过从 GitHub Release 下载官方 `v0.3.34-cu124` 的 `win_amd64.whl` 包，将 **902 MB 的 C++ CUDA 算子动态库 `ggml-cuda.dll`** 成功解压到了沙箱中。
3. **攻克 Windows DLL 缺失与 Python 垃圾回收死锁（最关键破局点！）**：
   - **死锁真相**：Python 3.8+ Windows 下 `os.add_dll_directory(path)` 会返回一个 `AddedDLLDirectory` 句柄对象。如果未用全局列表（如 `_dll_handles.append(...)`）强行持有，Python 垃圾回收器（GC）会在循环结束的瞬间**注销已注册的 DLL 目录**，导致 `import llama_cpp` 报 `FileNotFoundError`！
   - **破局方案**：通过全局句柄列表 `_dll_handles` 强行持有一切 `.dll` 目录，配合 `nvidia-cuda-runtime-cu12` 提供的 `cudart64_12.dll`，使 `llama.dll` 加载成功率达到 **100%**！

---

## 📍 4. 当前精确物理断言位置

1. **项目根目录**：`C:\dev\ai-forge`
2. **配置文件状态**：`src-tauri\crates\services\service-translation\pyproject.toml` 已配置针对 Windows CUDA 12.4 的 `explicit = true` 源。
3. **沙箱环境状态**：`src-tauri\crates\services\service-translation\.venv` 已安装好 Windows 版 Python 3.12、`llama-cpp-python==0.3.34`（GPU 版）、`nvidia-cuda-runtime-cu12`、`nvidia-cublas-cu12` 与 `nvidia-cuda-nvrtc-cu12`。
4. **探针验证结果**：`ctypes.CDLL(".../llama_cpp/lib/llama.dll")` 返回 `handle 7ffde02e0000`（100% 挂载成功）。

---

## 🚀 5. 下一会话第一步落地计划

新会话启动后，请按以下步骤执行：

### 第一步：更新 `worker.py` 注入显式句柄保持逻辑
在 Windows PowerShell 终端中，使用 `Set-Content` 协议将注入了 `_dll_handles` 长效句柄持有逻辑的 `worker.py` 落盘：

```powershell
Set-Content -Path "C:\dev\ai-forge\src-tauri\crates\services\service-translation\scripts\worker.py" -Encoding UTF8 -Value @'
import os
import sys
import json
import time
import struct
import sqlite3
import hashlib
import traceback

# ==========================================
# 🛡️ 工业级防线：SOCKS 代理隔离与脱网离线主权
# ==========================================
for k in ["ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"]:
    os.environ.pop(k, None)

# ==========================================
# 🛡️ Windows Native DLL 防线：CUDA 动态库显式注册与长效句柄持有
# ==========================================
_dll_handles = []
if sys.platform == "win32":
    import site
    try:
        for sp in site.getsitepackages():
            for root, dirs, files in os.walk(sp):
                if any(f.lower().endswith('.dll') for f in files):
                    try:
                        _dll_handles.append(os.add_dll_directory(root))
                    except Exception:
                        pass
    except Exception:
        pass

# ==========================================
# 🛡️ 工业级防线：OS 级别文件描述符劫持 (FD Hijacking)
# ==========================================
def hijack_stdout():
    try:
        real_stdout_fd = os.dup(1)
        os.dup2(2, 1)
        ipc_out = os.fdopen(real_stdout_fd, 'wb')
        return ipc_out
    except Exception as e:
        print("FD 劫持失败: " + str(e), file=sys.stderr)
        sys.exit(1)

IPC_OUT = None
IPC_IN = None

def init_ipc():
    global IPC_OUT, IPC_IN
    IPC_OUT = hijack_stdout()
    IPC_IN = sys.stdin.buffer

def send_message(method: str, params: dict):
    msg = {"method": method, "params": params}
    data = json.dumps(msg, ensure_ascii=False).encode('utf-8')
    IPC_OUT.write(struct.pack('>I', len(data)))
    IPC_OUT.write(data)
    IPC_OUT.flush()

def recv_message():
    header = IPC_IN.read(4)
    if not header or len(header) < 4:
        return None
    msg_len = struct.unpack('>I', header)[0]
    data = IPC_IN.read(msg_len)
    if len(data) < msg_len:
        return None
    return json.loads(data.decode('utf-8'))

def resolve_model_path():
    candidates = [
        r"C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf",
        r"C:\dev\ai-toolkit\models\tool-translation\Hy-MT2-1.8B-Q4.gguf",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return r"C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf"

def get_db_connection():
    db_path = r"C:\dev\ai-forge\data\translation\translation_store.sqlite3"
    os.makedirs(os.path.dirname(db_path), exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("""
        CREATE TABLE IF NOT EXISTS translation_cache (
            hash_key TEXT PRIMARY KEY,
            source_text TEXT,
            target_lang TEXT,
            translated_result TEXT,
            created_at INTEGER
        )
    """)
    conn.commit()
    return conn

_llm = None

def get_translation_model():
    global _llm
    if _llm is None:
        from llama_cpp import Llama
        model_path = resolve_model_path()
        if not os.path.exists(model_path):
            raise FileNotFoundError("找不到 Hy-MT2 GGUF 模型文件: " + model_path)
        _llm = Llama(model_path=model_path, n_gpu_layers=-1, n_ctx=2048, verbose=False)
    return _llm

def translate_single_text(conn, llm, text: str, target_lang: str = "Chinese") -> str:
    clean_text = text.strip()
    if not clean_text:
        return ""

    hash_key = hashlib.md5((target_lang + ":" + clean_text).encode("utf-8")).hexdigest()
    cursor = conn.cursor()
    cursor.execute("SELECT translated_result FROM translation_cache WHERE hash_key = ?", (hash_key,))
    row = cursor.fetchone()

    if row and row[0]:
        return row[0]

    prompt = (
        "Translate the following text into " + target_lang + ". "
        "Note that you should ONLY output the translated result without any additional explanation:\n\n"
        + clean_text
    )

    response = llm(
        prompt,
        max_tokens=256,
        temperature=0.1,
        stop=["\n\n", "<|im_end|>"],
        echo=False
    )

    trans_text = response["choices"][0]["text"].strip()

    cursor.execute(
        "INSERT OR REPLACE INTO translation_cache (hash_key, source_text, target_lang, translated_result, created_at) VALUES (?, ?, ?, ?, ?)",
        (hash_key, clean_text, target_lang, trans_text, int(time.time()))
    )
    conn.commit()
    return trans_text

def process_translation_job(payload: dict) -> dict:
    t0 = time.time()
    conn = get_db_connection()
    llm = get_translation_model()

    target_lang = payload.get("target_lang", "Chinese")
    texts = payload.get("texts", [])

    if not isinstance(texts, list):
        conn.close()
        return {"success": False, "error": "texts 参数必须是数组 List[str]"}

    translated_list = []
    for t in texts:
        trans_text = translate_single_text(conn, llm, str(t), target_lang)
        translated_list.append(trans_text)

    conn.close()
    return {
        "success": True,
        "translations": translated_list,
        "elapsed_ms": round((time.time() - t0) * 1000, 2)
    }

def main():
    init_ipc()
    send_message("system.ready", {"status": "Hy-MT2 1.8B Translation Worker 已点火就绪"})

    while True:
        try:
            msg = recv_message()
            if msg is None:
                break
            method = msg.get("method")
            params = msg.get("params", {})

            if method == "translate":
                res_obj = process_translation_job(params)
                if res_obj.get("success"):
                    send_message("translate_result", {"success": True, "result": res_obj})
                else:
                    send_message("error", {"message": res_obj.get("error", "翻译失败")})
            elif method == "exit":
                break
            else:
                send_message("error", {"message": "未知指令: " + str(method)})

        except Exception as e:
            send_message("error", {"message": str(e) + "\n" + traceback.format_exc()})

if __name__ == "__main__":
    main()
'@
```

### 第二步：运行极速 GPU 点火命令
在 PowerShell 中运行以下单行探针，检验 RTX 3060 显卡全量卸载推演（预期单句 $<0.1$ 秒）：

```powershell
& "C:\dev\ai-forge\src-tauri\crates\services\service-translation\.venv\Scripts\python.exe" -c "import os, sys, site, time
_dll_handles = []
if sys.platform == 'win32':
    for sp in site.getsitepackages():
        for root, dirs, files in os.walk(sp):
            if any(f.lower().endswith('.dll') for f in files):
                try: _dll_handles.append(os.add_dll_directory(root))
                except Exception: pass
import llama_cpp; from llama_cpp import Llama
print('=== 🚀 Windows Native RTX 3060 CUDA 点火断言 ===')
model_path = r'C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf'
t0 = time.time()
llm = Llama(model_path=model_path, n_gpu_layers=-1, n_ctx=2048, verbose=True)
print('1. 模型 GPU 载入耗时:', round(time.time()-t0, 2), 's')
t1 = time.time()
res = llm('Translate into Chinese: Can we get a table for two?', max_tokens=64)
print('2. 单句 GPU 推理耗时:', round(time.time()-t1, 2), 's')
print('3. 结果:', res['choices'][0]['text'].strip())"
```

### 第三步：运行 Cargo Release 单元打靶测试
显卡点火成功后，运行全量 Rust 单元打靶（验证 23 句英译中与 48 句日译中）：

```powershell
cargo test --release --manifest-path C:\dev\ai-forge\src-tauri\Cargo.toml -p service-translation -- --nocapture
```

---

## ⚠️ 6. 十大绝对不能再踩的“物理地雷” (Fatal Pitfalls Checklist)

1. **绝对禁止在 PowerShell 中使用 Bash `cat << 'EOF'`**：在 Windows PowerShell 中会导致 `ParserError` 语法抛错！改文件必须严格使用 PowerShell 专属协议：`Set-Content -Path "..." -Encoding UTF8 -Value @' ... '@`。
2. **绝对禁止让 `os.add_dll_directory()` 句柄被垃圾回收**：Python 3.8+ Windows 下 `os.add_dll_directory(path)` 返回 `AddedDLLDirectory` 句柄对象。如果不放入全局列表（如 `_dll_handles.append(...)`），GC 销毁句柄的瞬间会自动注销 DLL 目录，导致 `import llama_cpp` 报 `FileNotFoundError`！
3. **绝对禁止使用未锁定版本的 `llama-cpp-python`**：在 `pyproject.toml` 中必须写 `dependencies = ["llama-cpp-python==0.3.34"]`（或 `0.3.16`），不能写 `>=0.3.0`，防止 `uv` 去 PyPI 盲目抓取最新未对齐版本。
4. **绝对禁止在不带 `--no-sync` 时依赖 `uv run` 联网**：如果开启了代理，`uv run` 每次会尝试与 PyPI / GitHub 联网校验索引，导致终端停留在 `⠏ llama-cpp-python` 卡死。调试时直接呼叫 `.venv\Scripts\python.exe` 绕过网络！
5. **绝对禁止使用 TCP/HTTP 网络端口（如 `llama-server`）**：严格遵循定理二（0 网络端口），主进程与沙箱只走 `Stdio + 4 字节长度头 JSON-RPC`，避免 Windows Defender 弹出防火墙警告。
6. **绝对禁止在 Python f-string 大括号 `{}` 内部使用反斜杠 `\`**：Python <3.12 解析抛错。所有 Windows 路径在 Python 中必须使用原始字符串（`r"C:\..."`）或正斜杠（`"C:/..."`），防止 `\t`, `\n`, `\U` 转义抛错。
7. **绝对禁止在 PowerShell 命令行开头带 `#` 注释**：PowerShell 会将开头带 `#` 的多行命令解析异常或跳过执行。说明文字写在代码块外部。
8. **绝对禁止改动 `core-*` 底座代码**：遵循单一职责与工作空间防腐原则，`core-*` 视作只读算法资产，只通过公开 API 在适配器层并网。
9. **绝对禁止在 debug 模式下评估 Rust 性能**：Rust 评测性能必须带 `--release` 标志。
10. **修改未打包源码前必须执行【底座源码检查契约】**：若修改未在上下文打包全量源码的文件，先用 `Get-Content <文件路径>` 打印全量真实源码，审阅后再输出药方！