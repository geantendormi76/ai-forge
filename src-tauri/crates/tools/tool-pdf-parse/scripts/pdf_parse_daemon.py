import sys
import os
import time
import json
import socket
import tempfile
import traceback
import io

os.environ["PYTORCH_CUDA_ALLOC_CONF"] = "expandable_segments:True"
for k in ["ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"]:
    os.environ.pop(k, None)

SOCKET_PATH = "/tmp/ai_toolkit_pdf_parse.sock"
PROJECT_DIR = "/home/zhz/ai-toolkit/src-tauri/crates/tools/tool-pdf-parse"
SCRIPTS_DIR = os.path.join(PROJECT_DIR, "scripts")
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

class PdfParseDaemon:
    def __init__(self):
        self.doclayout_model = None
        self.init_models()

    def init_models(self):
        t0 = time.time()
        print("📡 [PDF Parse Daemon] 正在点火加载 PP-DocLayoutV3 等视觉神经网络至 RTX 3060 显存...")
        try:
            import paddle
            from paddlex import create_model

            use_gpu = paddle.is_compiled_with_cuda() and paddle.device.get_device().startswith("gpu")
            primary_device = "gpu:0" if use_gpu else "cpu"

            local_layout_dir = "/home/zhz/ai-toolkit/models/tool-pdf-parse/PP-DocLayoutV3"
            if os.path.exists(local_layout_dir):
                self.doclayout_model = create_model(model_name="PP-DocLayoutV3", model_dir=local_layout_dir, device=primary_device)
            else:
                self.doclayout_model = create_model(model_name="PP-DocLayoutV3", device=primary_device)

            print(f"✅ [PDF Parse Daemon] GPU 显存点火完成！耗时: {(time.time()-t0)*1000:.2f} ms")
        except Exception as e:
            print(f"⚠️ [PDF Parse Daemon] 点火加载模型失败: {e}")

    def process_request(self, payload: dict) -> dict:
        t0 = time.time()
        input_path = payload.get("input_path", "").strip()
        output_dir = payload.get("output_dir", "").strip()
        pages_str = payload.get("pages_str", "")

        if not input_path or not os.path.exists(input_path):
            return {"success": False, "error": f"物理文件不存在: {input_path}", "elapsed_ms": 0.0}

        try:
            import deeptrack_worker

            old_argv = sys.argv
            old_stdout = sys.stdout

            # 内存重定向捕获 Worker 的 print 输出，防止终端刷屏
            captured_output = io.StringIO()
            sys.stdout = captured_output

            sys.argv = ["deeptrack_worker.py", input_path, output_dir, pages_str]
            deeptrack_worker.main()

            sys.stdout = old_stdout
            sys.argv = old_argv

            raw_output = captured_output.getvalue()

            json_str = ""
            if "___JSON_START___" in raw_output and "___JSON_END___" in raw_output:
                start = raw_output.find("___JSON_START___") + len("___JSON_START___")
                end = raw_output.rfind("___JSON_END___")
                json_str = raw_output[start:end].strip()

            parsed_res = json.loads(json_str) if json_str else {}
            elapsed_ms = (time.time() - t0) * 1000.0

            parsed_res["elapsed_ms"] = round(elapsed_ms, 2)
            parsed_res["output_dir"] = output_dir

            return parsed_res

        except Exception as e:
            sys.stdout = old_stdout
            elapsed_ms = (time.time() - t0) * 1000.0
            return {"success": False, "error": f"PDF Parse Daemon 执行报错: {str(e)}\n{traceback.format_exc()}", "elapsed_ms": round(elapsed_ms, 2)}

    def start_unix_socket(self):
        if os.path.exists(SOCKET_PATH):
            os.remove(SOCKET_PATH)

        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        server.bind(SOCKET_PATH)
        os.chmod(SOCKET_PATH, 0o777)
        server.listen(10)
        print(f"🚀 [PDF Parse Daemon] UDS 常驻本地域套接字监听中: {SOCKET_PATH}")

        while True:
            conn, _ = server.accept()
            try:
                data = conn.recv(524288) # 扩展至 512KB 缓冲区，确保完整接收大规模版面元素 JSON
                if not data:
                    conn.close()
                    continue
                req_json = json.loads(data.decode("utf-8"))
                res_obj = self.process_request(req_json)
                conn.sendall(json.dumps(res_obj, ensure_ascii=False).encode("utf-8"))
            except Exception as e:
                err_res = {"success": False, "error": str(e), "elapsed_ms": 0.0}
                conn.sendall(json.dumps(err_res, ensure_ascii=False).encode("utf-8"))
            finally:
                conn.close()

if __name__ == "__main__":
    daemon = PdfParseDaemon()
    daemon.start_unix_socket()
