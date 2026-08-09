import os
import sys
import json
import time
import struct
import traceback

for k in ["ALL_PROXY", "all_proxy", "HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"]:
    os.environ.pop(k, None)

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

import subtitle_engine

def handle_extract_audio(params: dict) -> dict:
    video_path = params.get("video_path", "")
    out_wav_path = params.get("out_wav_path", "")
    
    if not os.path.exists(video_path):
        return {"success": False, "error": f"物理视频文件不存在: {video_path}"}
        
    ok = subtitle_engine.extract_audio_from_video(video_path, out_wav_path)
    if ok:
        return {"success": True, "wav_path": out_wav_path}
    else:
        return {"success": False, "error": "FFmpeg 从视频提取音频 WAV 失败"}

def handle_render_and_mux(params: dict) -> dict:
    t0 = time.time()
    video_path = params.get("video_path", "")
    output_dir = params.get("output_dir", "")
    segments = params.get("segments", [])
    display_mode = params.get("display_mode", "bilingual")
    show_speaker = params.get("show_speaker", False)
    font_size_multiplier = float(params.get("font_size_multiplier", 1.8))
    output_mode = params.get("output_mode", "soft_mkv")

    if not os.path.exists(video_path):
        return {"success": False, "error": f"物理视频文件不存在: {video_path}"}

    os.makedirs(output_dir, exist_ok=True)
    base_name = os.path.splitext(os.path.basename(video_path))[0]

    # 1. 导出 SRT 字幕
    srt_content = subtitle_engine.generate_srt(segments, display_mode, show_speaker)
    srt_path = os.path.join(output_dir, f"{base_name}_subtitle.srt")
    with open(srt_path, "w", encoding="utf-8") as f:
        f.write(srt_content)

    # 2. 导出 ASS 字幕
    ass_content = subtitle_engine.generate_ass(
        segments,
        display_mode=display_mode,
        show_speaker=show_speaker,
        font_size_multiplier=font_size_multiplier
    )
    ass_path = os.path.join(output_dir, f"{base_name}_subtitle.ass")
    with open(ass_path, "w", encoding="utf-8") as f:
        f.write(ass_content)

    # 3. 🛡️ 物理新增：导出结构化 JSON 产物
    json_path = os.path.join(output_dir, f"{base_name}_bilingual.json")
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump({
            "video_path": video_path,
            "total_segments": len(segments),
            "segments": segments
        }, f, ensure_ascii=False, indent=2)

    # 4. 封装/压制视频
    output_video_path = ""
    if output_mode == "hard_mp4_nvenc":
        out_mp4 = os.path.join(output_dir, f"{base_name}_zidian_burned.mp4")
        ok = subtitle_engine.burn_hard_subtitles_nvenc(video_path, ass_path, out_mp4)
        if ok:
            output_video_path = out_mp4
        else:
            return {"success": False, "error": "NVENC 显卡硬字幕压制失败"}
    else:
        out_mkv = os.path.join(output_dir, f"{base_name}_zidian_muxed.mkv")
        ok = subtitle_engine.mux_soft_subtitles(video_path, ass_path, out_mkv)
        if ok:
            output_video_path = out_mkv
        else:
            return {"success": False, "error": "0.5s 无损软挂载 MKV 失败"}

    elapsed_ms = round((time.time() - t0) * 1000, 2)
    return {
        "success": True,
        "srt_path": srt_path,
        "ass_path": ass_path,
        "json_path": json_path,
        "output_video_path": output_video_path,
        "elapsed_ms": elapsed_ms
    }

def main():
    init_ipc()
    send_message("system.ready", {"status": "紫电 AI 视频双语字幕 Worker 已点火就绪"})

    while True:
        try:
            msg = recv_message()
            if msg is None:
                break
            method = msg.get("method")
            params = msg.get("params", {})

            if method == "extract_audio":
                res = handle_extract_audio(params)
                send_message("extract_audio_result", res)

            elif method == "render_and_mux":
                res = handle_render_and_mux(params)
                send_message("render_and_mux_result", res)

            elif method == "exit":
                break
            else:
                send_message("error", {"message": f"未知指令: {method}"})

        except Exception as e:
            send_message("error", {"message": f"{str(e)}\n{traceback.format_exc()}"})

if __name__ == "__main__":
    main()
