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
# 🛡️ Windows Native DLL 防线：终极双轨注入 (Dual-Track Injection)
# ==========================================
_dll_handles = []
if sys.platform == "win32":
    search_paths = [p for p in sys.path if os.path.isdir(p)]
    for sp in search_paths:
        for root, dirs, files in os.walk(sp):
            if any(f.lower().endswith(".dll") for f in files):
                # 1. Python 级注入 (喂给 ctypes)
                try:
                    _dll_handles.append(os.add_dll_directory(root))
                except Exception:
                    pass
                # 2. OS 级注入 (喂给 C++ 底层 LoadLibrary)
                current_path = os.environ.get("PATH", "")
                if root not in current_path:
                    os.environ["PATH"] = root + os.pathsep + current_path

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
        # 禁用 verbose 防止底层 C++ 日志污染 stderr
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
