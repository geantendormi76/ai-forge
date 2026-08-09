import os
import sys
import json
import time
import glob
import tempfile
import traceback
import numpy as np

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

from subtitle_engine import TranscriptStreamParser


def emit_event(event_type: str, data: dict):
    payload = {
        "event": event_type,
        "data": data,
        "timestamp": round(time.time(), 3)
    }
    print("___JSON_START___")
    print(json.dumps(payload, ensure_ascii=False))
    print("___JSON_END___")
    sys.stdout.flush()


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else "mic"
    test_audio_path = sys.argv[2] if len(sys.argv) > 2 else None

    models_base = "/home/zhz/ai-toolkit/models/tool-ASR/MOSS-Transcribe-Diarize"
    if not os.path.exists(models_base):
        emit_event("error", {"message": f"🚨 找不到模型目录: {models_base}"})
        return

    try:
        import torch
        import soundfile as sf
        from transformers import AutoModelForCausalLM, AutoProcessor
        from moss_transcribe_diarize.inference_utils import (
            build_transcription_messages,
            generate_transcription,
            resolve_device,
        )

        device = resolve_device("cuda:0")
        dtype = torch.bfloat16 if device.type == "cuda" else torch.float32

        emit_event("status", {"message": "1️⃣ 正在点火加载 MOSS 0.9B VAD 实时听写模型到 GPU..."})

        model = AutoModelForCausalLM.from_pretrained(
            models_base,
            trust_remote_code=True,
            dtype="auto",
            local_files_only=True
        ).to(dtype=dtype).to(device).eval()

        processor = AutoProcessor.from_pretrained(models_base, trust_remote_code=True, local_files_only=True)

        emit_event("status", {"message": "2️⃣ 实时听写模型已上线，VAD 自然断句就绪！"})

        # 🧪 模式 A: 自动化测试模拟模式 (读取音频文件测试)
        if mode == "test" and test_audio_path and os.path.exists(test_audio_path):
            import librosa
            emit_event("status", {"message": f"🧪 [TDD 模拟流式读取]: {os.path.basename(test_audio_path)}"})

            raw_audio, sr = librosa.load(test_audio_path, sr=16000)
            chunk_samples = 3 * 16000
            total_samples = len(raw_audio)

            for start_idx in range(0, total_samples, int(1.5 * 16000)):
                end_idx = min(start_idx + chunk_samples, total_samples)
                audio_chunk = raw_audio[start_idx:end_idx]
                if len(audio_chunk) < 8000:
                    continue

                energy = float(np.max(np.abs(audio_chunk)))
                if energy < 0.01:
                    continue

                with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp_f:
                    sf.write(tmp_f.name, audio_chunk, 16000)
                    tmp_wav = tmp_f.name

                messages = build_transcription_messages(tmp_wav)
                res = generate_transcription(
                    model, processor, messages,
                    max_new_tokens=256, do_sample=False,
                    device=device, dtype=dtype
                )
                raw_text = res.get("text", "")
                segs = parse_transcript(raw_text)

                for seg in segs:
                    if seg.text.strip():
                        emit_event("live_transcript", {
                            "speaker": seg.speaker,
                            "text": seg.text.strip(),
                            "chunk_start": round(start_idx / 16000.0, 2),
                            "chunk_end": round(end_idx / 16000.0, 2),
                            "energy": round(energy, 4)
                        })

                if os.path.exists(tmp_wav):
                    os.remove(tmp_wav)

            emit_event("complete", {"message": "🎉 [TDD 模拟测试完成]"})

        # 🎙️ 模式 B: 真实麦克风 VAD 自然断句流式听写模式
        else:
            import sounddevice as sd

            sample_rate = 16000
            accumulated_audio = []
            silent_chunks_count = 0
            speaking_active = False

            def audio_callback(indata, frames, time_info, status):
                nonlocal accumulated_audio, silent_chunks_count, speaking_active
                mono_data = indata[:, 0].copy()
                peak = float(np.max(np.abs(mono_data)))

                if peak >= 0.012:
                    speaking_active = True
                    silent_chunks_count = 0
                    accumulated_audio.append(mono_data)
                elif speaking_active:
                    accumulated_audio.append(mono_data)
                    silent_chunks_count += 1

            emit_event("status", {"message": "🎙️ VAD 自然断句实时听写开始监听！"})

            with sd.InputStream(samplerate=sample_rate, channels=1, blocksize=1600, callback=audio_callback):
                while True:
                    time.sleep(0.1)
                    if speaking_active and silent_chunks_count >= 6 and len(accumulated_audio) >= 8:
                        audio_np = np.concatenate(accumulated_audio)
                        accumulated_audio = []
                        speaking_active = False
                        silent_chunks_count = 0

                        if len(audio_np) < 8000:
                            continue

                        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp_f:
                            sf.write(tmp_f.name, audio_np, sample_rate)
                            tmp_wav = tmp_f.name

                        messages = build_transcription_messages(tmp_wav)
                        res = generate_transcription(
                            model, processor, messages,
                            max_new_tokens=256, do_sample=False,
                            device=device, dtype=dtype
                        )
                        raw_text = res.get("text", "")
                        segs = parse_transcript(raw_text)

                        for seg in segs:
                            if seg.text.strip():
                                emit_event("live_transcript", {
                                    "speaker": seg.speaker,
                                    "text": seg.text.strip(),
                                    "timestamp": time.strftime("%H:%M:%S")
                                })

                        if os.path.exists(tmp_wav):
                            os.remove(tmp_wav)

    except Exception as e:
        emit_event("error", {"message": f"流式听写报错: {str(e)}\n{traceback.format_exc()}"})


if __name__ == "__main__":
    main()
