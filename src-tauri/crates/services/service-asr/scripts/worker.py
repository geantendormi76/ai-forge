import os
import sys
import json
import time
import uuid
import struct
import traceback
import subprocess
import torch
import librosa
from transformers import AutoModelForCausalLM, AutoProcessor

for k in ["ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"]:
    os.environ.pop(k, None)

os.environ["PYTORCH_CUDA_ALLOC_CONF"] = "expandable_segments:True"

_dll_handles = []
if sys.platform == "win32":
    search_paths = [p for p in sys.path if os.path.isdir(p)]
    for sp in search_paths:
        for root, dirs, files in os.walk(sp):
            if any(f.lower().endswith(".dll") for f in files):
                try:
                    _dll_handles.append(os.add_dll_directory(root))
                except Exception:
                    pass
                current_path = os.environ.get("PATH", "")
                if root not in current_path:
                    os.environ["PATH"] = root + os.pathsep + current_path

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

scripts_dir = os.path.dirname(os.path.abspath(__file__))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from subtitle_engine import parse_transcript

def resolve_model_dir():
    candidates = [
        r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize",
        r"C:\dev\ai-forge\models\tool-ASR\MOSS-Transcribe-Diarize",
        r"C:\Users\52484\StudioProjects\asr-Android\models\tool-ASR\MOSS-Transcribe-Diarize",
        r"C:\dev\ai-toolkit\models\tool-ASR\MOSS-Transcribe-Diarize",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize"

_model = None
_processor = None

def get_asr_model():
    global _model, _processor
    if _model is None or _processor is None:
        models_base = resolve_model_dir()
        if not os.path.exists(models_base):
            raise FileNotFoundError("找不到 MOSS 0.9B 模型目录: " + models_base)

        device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
        dtype = torch.bfloat16 if device.type == "cuda" else torch.float32

        _model = AutoModelForCausalLM.from_pretrained(
            models_base,
            trust_remote_code=True,
            dtype="auto",
            local_files_only=True
        ).to(dtype=dtype).to(device).eval()

        _processor = AutoProcessor.from_pretrained(models_base, trust_remote_code=True, local_files_only=True)

    return _model, _processor

OFFICIAL_DEFAULT_PROMPT = "请将音频转写为文本，每一段需以起始时间戳和说话人编号（[S01]、[S02]、[S03]…）开头，正文为对应的语音内容，并在段末标注结束时间戳，以清晰标明该段语音范围。"

def run_transcription(
    audio_path: str,
    language: str = "auto",
    custom_prompt: str = None,
    hotwords: str = None,
    max_new_tokens: int = 2048,
    temperature: float = 0.0
):
    t0 = time.time()
    if not os.path.exists(audio_path):
        return False, None, "找不到音频文件: " + audio_path

    try:
        model, processor = get_asr_model()
        device = torch.device("cuda:0" if torch.cuda.is_available() else "cpu")
        dtype = torch.bfloat16 if device.type == "cuda" else torch.float32

        raw_duration = float(librosa.get_duration(path=audio_path))
        duration_sec = round(raw_duration, 2)

        chunk_sec = 180.0
        curr_start = 0.0
        all_parsed_segments = []

        temp_dir = os.environ.get("TEMP", "/tmp")

        prompt = custom_prompt if custom_prompt and custom_prompt.strip() else OFFICIAL_DEFAULT_PROMPT
        if hotwords and hotwords.strip():
            prompt += f"热词提示：{hotwords.strip()}。"

        while curr_start < duration_sec:
            curr_end = min(curr_start + chunk_sec, duration_sec)
            
            unique_tag = uuid.uuid4().hex[:12]
            tmp_chunk_path = os.path.join(temp_dir, f"moss_chunk_{unique_tag}.mp3")
            
            cmd = [
                "ffmpeg", "-y",
                "-ss", str(curr_start),
                "-t", str(curr_end - curr_start),
                "-i", audio_path,
                "-vn", "-acodec", "libmp3lame",
                "-ar", "16000", "-ac", "1", "-q:a", "2",
                tmp_chunk_path
            ]
            subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

            if os.path.exists(tmp_chunk_path):
                audio_np, _ = librosa.load(tmp_chunk_path, sr=16000)

                messages = [
                    {"role": "system", "content": prompt},
                    {"role": "user", "content": [{"type": "audio", "audio_url": tmp_chunk_path}]}
                ]

                text_prompt = processor.apply_chat_template(messages, tokenize=False, add_generation_prompt=True)
                inputs = processor(text=text_prompt, audio=[audio_np], return_tensors="pt").to(device=device, dtype=dtype)

                do_sample = temperature > 0.0
                gen_kwargs = {
                    "max_new_tokens": max_new_tokens,
                    "do_sample": do_sample
                }
                if do_sample:
                    gen_kwargs["temperature"] = temperature

                with torch.no_grad():
                    outputs = model.generate(**inputs, **gen_kwargs)

                raw_text = processor.decode(outputs[0][inputs.input_ids.shape[-1]:], skip_special_tokens=True)

                for seg in parse_transcript(raw_text):
                    clean_text = seg.text.strip()
                    if clean_text:
                        all_parsed_segments.append({
                            "speaker": seg.speaker,
                            "start": round(float(seg.start) + curr_start, 2),
                            "end": round(float(seg.end) + curr_start, 2),
                            "text": clean_text
                        })
                try:
                    os.remove(tmp_chunk_path)
                except Exception:
                    pass

            curr_start += chunk_sec

        for idx, seg in enumerate(all_parsed_segments, 1):
            seg["id"] = idx

        elapsed_ms = (time.time() - t0) * 1000.0
        result_payload = {
            "audio_file": os.path.basename(audio_path),
            "duration_sec": duration_sec,
            "language": language,
            "total_segments": len(all_parsed_segments),
            "segments": all_parsed_segments
        }
        return True, result_payload, None

    except Exception as e:
        err_msg = str(e) + "\n" + traceback.format_exc()
        return False, None, err_msg

def main():
    init_ipc()
    send_message("system.ready", {"status": "MOSS 0.9B ASR Worker 已点火就绪"})

    while True:
        try:
            msg = recv_message()
            if msg is None:
                break
            method = msg.get("method")
            params = msg.get("params", {})

            if method == "transcribe":
                audio_path = params.get("audio_path", "")
                lang = params.get("language", "auto")
                prompt = params.get("prompt")
                hotwords = params.get("hotwords")
                max_new_tokens = int(params.get("max_new_tokens", 2048))
                temperature = float(params.get("temperature", 0.0))

                ok, res_payload, err = run_transcription(
                    audio_path,
                    language=lang,
                    custom_prompt=prompt,
                    hotwords=hotwords,
                    max_new_tokens=max_new_tokens,
                    temperature=temperature
                )
                if ok:
                    send_message("transcribe_result", {"success": True, "result": res_payload})
                else:
                    send_message("error", {"message": err})

            elif method == "exit":
                break
            else:
                send_message("error", {"message": "未知指令: " + str(method)})

        except Exception as e:
            send_message("error", {"message": str(e) + "\n" + traceback.format_exc()})

if __name__ == "__main__":
    main()
