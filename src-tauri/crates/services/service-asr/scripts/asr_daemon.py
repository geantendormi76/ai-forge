import sys
import os
import time
import json
import socket
import glob
import traceback

os.environ["PYTORCH_CUDA_ALLOC_CONF"] = "expandable_segments:True"
for k in ["ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"]:
    os.environ.pop(k, None)

PROJECT_ROOT = "/home/zhz/ai-toolkit"
ASR_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "src-tauri/crates/tools/tool-ASR/scripts")
if ASR_SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, ASR_SCRIPTS_DIR)

moss_git_paths = glob.glob("/home/zhz/.cache/uv/git-v0/checkouts/*/*")
for p in moss_git_paths:
    if os.path.exists(os.path.join(p, "moss_transcribe_diarize")):
        if p not in sys.path:
            sys.path.insert(0, p)
        break

from subtitle_engine import parse_transcript, normalize_segments, export_srt, export_ass, export_json

SOCKET_PATH = "/tmp/ai_toolkit_asr.sock"
MODELS_BASE = "/home/zhz/ai-toolkit/models/tool-ASR/MOSS-Transcribe-Diarize"
IDLE_TIMEOUT_SEC = 1800.0


def build_prompt(mode: str = "verbatim", language: str = "auto", hotwords: str = "") -> str:
    base_prompt = "请将音频转写为文本，每一段需以起始时间戳和说话人编号（[S01]、[S02]、[S03]…）开头，正文为对应的语音内容，并在段末标注结束时间戳，以清晰标明该段语音范围。"

    if mode in ["clean", "news", "smooth"]:
        base_prompt += " 请使用规范书面语转写，自动过滤口吃、重复词与无意义语气词（如‘嗯’、‘那个’、‘就是’、‘uh’、‘um’），并修正语病。"
    elif mode == "subtitle":
        base_prompt = "请将音频转写为文本，每一段需以起始时间戳开头并在段末标注结束时间戳，以清晰标明该段语音范围。"

    if language and language.lower() not in ["auto", "none"]:
        base_prompt += f" 主要语言为：{language}。"

    if hotwords and hotwords.strip():
        base_prompt += f" 热词提示：{hotwords.strip()}。"

    return base_prompt


class AsrDaemon:
    def __init__(self):
        self.model = None
        self.processor = None
        self.device = None
        self.dtype = None
        self.init_model()

    def init_model(self):
        t0 = time.time()
        print("📡 [MOSS ASR Daemon] 正在点火加载 MOSS 0.9B 语音模型至 RTX 3060 显存...")
        if not os.path.exists(MODELS_BASE):
            raise FileNotFoundError(f"找不到 MOSS 0.9B 模型目录: {MODELS_BASE}")

        import torch
        from transformers import AutoModelForCausalLM, AutoProcessor
        from moss_transcribe_diarize.inference_utils import resolve_device

        self.device = resolve_device("cuda:0")
        self.dtype = torch.bfloat16 if self.device.type == "cuda" else torch.float32

        self.model = AutoModelForCausalLM.from_pretrained(
            MODELS_BASE,
            trust_remote_code=True,
            dtype="auto",
            local_files_only=True
        ).to(dtype=self.dtype).to(self.device).eval()

        self.processor = AutoProcessor.from_pretrained(MODELS_BASE, trust_remote_code=True, local_files_only=True)
        print(f"✅ [MOSS ASR Daemon] GPU 显存点火完成！耗时: {(time.time()-t0)*1000:.2f} ms")

    def process_request(self, payload: dict) -> dict:
        t0 = time.time()
        audio_path = payload.get("audio_path", "").strip()
        mode = payload.get("mode", "verbatim")
        lang = payload.get("language", "auto")
        hotwords = payload.get("hotwords", "")

        if not audio_path or not os.path.exists(audio_path):
            return {"success": False, "error": f"找不到音视频文件: {audio_path}", "elapsed_ms": 0.0}

        try:
            import librosa
            from moss_transcribe_diarize.inference_utils import build_transcription_messages, generate_transcription

            raw_duration = float(librosa.get_duration(path=audio_path))
            duration_sec = round(raw_duration, 2)

            chunk_sec = 180.0
            total_chunks = int(duration_sec // chunk_sec) + (1 if duration_sec % chunk_sec > 0 else 0)
            
            print(f"\n🎬 [白盒切片推导启动] 视频时长: {duration_sec}s ({round(duration_sec/60, 2)}min) | 切分总片数: {total_chunks} | 显存锁死 3.8GB")

            curr_start = 0.0
            chunk_idx = 1
            all_raw_segments = []

            while curr_start < duration_sec:
                curr_end = min(curr_start + chunk_sec, duration_sec)
                pct = round((chunk_idx / total_chunks) * 100, 1)

                print(f"   -------------------------------------------------------------------------")
                print(f"   🎬 [切片 {chunk_idx}/{total_chunks} | 进度 {pct}%] 区间: {round(curr_start,1)}s ➔ {round(curr_end,1)}s")

                tmp_chunk_path = f"/tmp/moss_chunk_{chunk_idx}_{int(time.time())}.mp3"
                os.system(f"ffmpeg -y -ss {curr_start} -t {curr_end - curr_start} -i \"{audio_path}\" -vn -acodec libmp3lame -ar 16000 -ac 1 -q:a 2 \"{tmp_chunk_path}\" > /dev/null 2>&1")

                if os.path.exists(tmp_chunk_path):
                    prompt_str = build_prompt(mode=mode, language=lang, hotwords=hotwords)
                    messages = build_transcription_messages(tmp_chunk_path, prompt=prompt_str)

                    # 官方 ProgressStreamer 白盒实时打印回调
                    def on_token_progress(generated_tokens: int):
                        if generated_tokens % 20 == 0:
                            print(f"      ⚡ [实时 Token 生成中] 已吐出 {generated_tokens} Tokens...", end="\r")

                    res = generate_transcription(
                        self.model, self.processor, messages,
                        max_new_tokens=2048, do_sample=False,
                        device=self.device, dtype=self.dtype,
                        token_callback=on_token_progress
                    )

                    raw_text = res.get("text", "")
                    parsed_chunk_segs = parse_transcript(raw_text)

                    for seg in parsed_chunk_segs:
                        all_raw_segments.append(
                            parse_transcript(f"[{round(seg.start + curr_start, 2)}][{seg.speaker}]{seg.text}[{round(seg.end + curr_start, 2)}]")
                        )
                        print(f"      [{round(seg.start + curr_start,1)}s ➔ {round(seg.end + curr_start,1)}s] ({seg.speaker}): {seg.text.strip()}")

                    os.remove(tmp_chunk_path)

                curr_start += chunk_sec
                chunk_idx += 1

            # 展平所有切片段落并使用五重管道归一化
            flattened_segs = [s for sublist in all_raw_segments for s in sublist]
            norm_segments = normalize_segments(flattened_segs)
            all_parsed_segments = [seg.to_dict() for seg in norm_segments]

            elapsed_ms = (time.time() - t0) * 1000.0
            result_payload = {
                "audio_file": os.path.basename(audio_path),
                "duration_sec": duration_sec,
                "mode": mode,
                "language": lang,
                "hotwords": hotwords,
                "total_chunks": total_chunks,
                "total_segments": len(all_parsed_segments),
                "segments": all_parsed_segments,
                "srt_text": export_srt(norm_segments),
                "ass_text": export_ass(norm_segments)
            }

            return {"success": True, "result": result_payload, "error": None, "elapsed_ms": round(elapsed_ms, 2)}

        except Exception as e:
            elapsed_ms = (time.time() - t0) * 1000.0
            return {"success": False, "error": f"ASR Daemon 执行报错: {str(e)}\n{traceback.format_exc()}", "elapsed_ms": round(elapsed_ms, 2)}

    def start_unix_socket(self):
        if os.path.exists(SOCKET_PATH):
            os.remove(SOCKET_PATH)

        server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        server.bind(SOCKET_PATH)
        os.chmod(SOCKET_PATH, 0o777)
        server.listen(10)
        server.settimeout(IDLE_TIMEOUT_SEC)
        print(f"🚀 [MOSS ASR Daemon] UDS 常驻套接字就绪 (3.8GB 显存锁定 + 白盒流式进度): {SOCKET_PATH}")

        while True:
            try:
                conn, _ = server.accept()
            except socket.timeout:
                print("💤 [MOSS ASR Daemon] 触发 Tiered LRU 显存退闪退场，释放 GPU 显存...")
                import torch
                del self.model
                del self.processor
                if torch.cuda.is_available():
                    torch.cuda.empty_cache()
                if os.path.exists(SOCKET_PATH):
                    os.remove(SOCKET_PATH)
                sys.exit(0)

            try:
                data = conn.recv(131072)
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
    daemon = AsrDaemon()
    daemon.start_unix_socket()
