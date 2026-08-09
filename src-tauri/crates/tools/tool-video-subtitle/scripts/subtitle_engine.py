import os
import sys
import json
import re
import subprocess

COLOR_ZIDIAN_PRIMARY = "&H00F8B4D8"      # 紫电主色 #D8B4F8 (上行译文)
COLOR_QINGSHUANG_SECONDARY = "&H00FCF3A5" # 青霜副色 #A5F3FC (下行原文)

SPEAKER_COLORS = [
    "&H00F8B4D8", "&H00FCF3A5", "&H00FFB56B", "&H008FF286",
    "&H0000D7FF", "&H00DB8EFF", "&H00FFE75B", "&H00D8D8D8",
]

def resolve_ffmpeg_bin() -> str:
    candidates = [
        r"C:\dev\ai-forge\src-tauri\bin\ffmpeg-x86_64-pc-windows-msvc.exe",
        r"C:\dev\ai-forge\src-tauri\bin\ffmpeg.exe",
        r"C:\dev\ai-toolkit\src-tauri\bin\ffmpeg-x86_64-pc-windows-msvc.exe",
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return "ffmpeg"

def clean_acoustic_noise(text: str) -> str:
    if not text:
        return ""
    cleaned = re.sub(r'\[(laughter|music|applause|singing|cough|snicker|sigh|groan)\]', '', text, flags=re.IGNORECASE)
    cleaned = re.sub(r'\((laughter|music|applause|singing|cough|snicker|sigh|groan)\)', '', cleaned, flags=re.IGNORECASE)
    return cleaned.strip()

def escape_ass_text(text: str) -> str:
    if not text:
        return ""
    return text.replace("{", "(").replace("}", ")").replace("\n", "\\N")

def smart_wrap_line(text: str, max_chars: int = 22) -> str:
    """SOTA 智能语义切分算法：超过阈值时在标点或空格处优雅插入 \\N 换行"""
    if not text or len(text) <= max_chars:
        return text

    # 1. 优先在标点符号处切分
    puncts = ['，', '；', '：', '。', '？', '！', ',', ';', ':', '?', '!']
    mid = len(text) // 2
    best_split = -1
    min_diff = len(text)

    for idx, ch in enumerate(text):
        if ch in puncts and 8 <= idx <= len(text) - 8:
            diff = abs(idx - mid)
            if diff < min_diff:
                min_diff = diff
                best_split = idx + 1

    if best_split != -1:
        return text[:best_split].strip() + "\\N" + text[best_split:].strip()

    # 2. 英文按中间空格切分
    if ' ' in text:
        words = text.split(' ')
        half_len = len(text) / 2
        curr_len = 0
        split_idx = 0
        for i, w in enumerate(words):
            curr_len += len(w) + 1
            if curr_len >= half_len:
                split_idx = i + 1
                break
        if split_idx > 0 and split_idx < len(words):
            line1 = " ".join(words[:split_idx])
            line2 = " ".join(words[split_idx:])
            return f"{line1}\\N{line2}"

    # 3. 中文无标点按中间位置切分
    return f"{text[:mid]}\\N{text[mid:]}"

def format_srt_time(seconds: float) -> str:
    ms = max(0, round(float(seconds) * 1000))
    hrs, rem = divmod(ms, 3600000)
    mins, rem = divmod(rem, 60000)
    secs, millis = divmod(rem, 1000)
    return f"{hrs:02d}:{mins:02d}:{secs:02d},{millis:03d}"

def format_ass_time(seconds: float) -> str:
    cs = max(0, round(float(seconds) * 100))
    hrs, rem = divmod(cs, 360000)
    mins, rem = divmod(rem, 6000)
    secs, centis = divmod(rem, 100)
    return f"{hrs:d}:{mins:02d}:{secs:02d}.{centis:02d}"

def generate_srt(segments: list, display_mode: str = "bilingual", show_speaker: bool = False) -> str:
    blocks = []
    valid_idx = 1
    for seg in segments:
        src = clean_acoustic_noise(seg.get("source_text", ""))
        tgt = clean_acoustic_noise(seg.get("target_text", ""))
        spk = seg.get("speaker", "S01") if show_speaker else ""

        if not src and not tgt:
            continue

        spk_prefix = f"[{spk}] " if spk else ""
        lines = []

        if display_mode == "bilingual":
            if tgt:
                lines.append(f"{spk_prefix}{smart_wrap_line(tgt, 24)}")
            if src:
                lines.append(smart_wrap_line(src, 42))
        elif display_mode == "target_only":
            if tgt:
                lines.append(f"{spk_prefix}{smart_wrap_line(tgt, 24)}")
            elif src:
                lines.append(f"{spk_prefix}{smart_wrap_line(src, 42)}")
        elif display_mode == "source_only":
            if src:
                lines.append(f"{spk_prefix}{smart_wrap_line(src, 42)}")

        if not lines:
            continue

        t_start = format_srt_time(float(seg.get("start_sec", 0.0)))
        t_end = format_srt_time(float(seg.get("end_sec", 0.0)))

        block_content = "\n".join(lines)
        blocks.append(f"{valid_idx}\n{t_start} --> {t_end}\n{block_content}\n")
        valid_idx += 1

    return "\n".join(blocks)

def generate_ass(
    segments: list,
    display_mode: str = "bilingual",
    show_speaker: bool = False,
    font_size_multiplier: float = 1.8,
    video_width: int = 1920,
    video_height: int = 1080,
) -> str:
    # 响应式基础字号计算
    base_font_size = max(24, round(video_height * 0.0437 * font_size_multiplier))
    sec_font_size = max(18, round(video_height * 0.0283 * font_size_multiplier))

    # 左右安全边距锚定：防止任何文字贴边或超出屏幕（设为视频宽度的 5%，1080p 下为 96px）
    margin_l = max(30, round(video_width * 0.05))
    margin_r = margin_l
    margin_v = max(20, round(video_height * 0.03))

    speakers = sorted({seg.get("speaker", "S01") for seg in segments})

    # 🛡️ 智能防线一：MarginL/R 限制宽度 + 开启 WrapStyle: 0 (ASS 引擎智能折行)
    style_lines = [
        f"Style: PrimaryStyle,Arial,{base_font_size},{COLOR_ZIDIAN_PRIMARY},&H000000FF,&H00000000,&H64000000,-1,0,0,0,100,100,0,0,1,2,1,2,{margin_l},{margin_r},{margin_v},1",
        f"Style: SecondaryStyle,Arial,{sec_font_size},{COLOR_QINGSHUANG_SECONDARY},&H000000FF,&H00000000,&H64000000,0,0,0,0,100,100,0,0,1,2,1,2,{margin_l},{margin_r},{margin_v},1",
    ]

    for idx, spk in enumerate(speakers):
        color = SPEAKER_COLORS[idx % len(SPEAKER_COLORS)]
        spk_style_name = f"Speaker_{spk}"
        style_lines.append(
            f"Style: {spk_style_name},Arial,{base_font_size},{color},&H000000FF,&H00000000,&H64000000,-1,0,0,0,100,100,0,0,1,2,1,2,{margin_l},{margin_r},{margin_v},1"
        )

    dialogue_lines = []
    for seg in segments:
        raw_src = clean_acoustic_noise(seg.get("source_text", ""))
        raw_tgt = clean_acoustic_noise(seg.get("target_text", ""))
        spk = seg.get("speaker", "S01")

        if not raw_src and not raw_tgt:
            continue

        # 🛡️ 智能防线二：字符数感知语义优雅换行
        wrapped_src = smart_wrap_line(raw_src, max_chars=40)
        wrapped_tgt = smart_wrap_line(raw_tgt, max_chars=22)

        src = escape_ass_text(wrapped_src)
        tgt = escape_ass_text(wrapped_tgt)

        spk_prefix = f"[{spk}] " if show_speaker and spk else ""
        t_start = format_ass_time(float(seg.get("start_sec", 0.0)))
        t_end = format_ass_time(float(seg.get("end_sec", 0.0)))

        style_name = f"Speaker_{spk}" if show_speaker and spk in speakers else "PrimaryStyle"

        if display_mode == "bilingual":
            if tgt and src:
                text_content = f"{spk_prefix}{tgt}\\N{{\\rSecondaryStyle}}{src}"
            elif tgt:
                text_content = f"{spk_prefix}{tgt}"
            else:
                text_content = f"{spk_prefix}{src}"
        elif display_mode == "target_only":
            text_content = f"{spk_prefix}{tgt}" if tgt else f"{spk_prefix}{src}"
        elif display_mode == "source_only":
            text_content = f"{spk_prefix}{src}"
        else:
            text_content = f"{spk_prefix}{tgt}"

        dialogue_lines.append(
            f"Dialogue: 0,{t_start},{t_end},{style_name},,0,0,0,,{text_content}"
        )

    header = [
        "[Script Info]",
        "Title: 紫电 AI 视频双语字幕工坊",
        "ScriptType: v4.00+",
        "WrapStyle: 0",  # 🛡️ 核心：设置为 0，开启 ASS 智能折行引擎！
        "ScaledBorderAndShadow: yes",
        f"PlayResX: {video_width}",
        f"PlayResY: {video_height}",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, "
        "Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, "
        "Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
        *style_lines,
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
        *dialogue_lines,
        "",
    ]
    return "\n".join(header)

def extract_audio_from_video(video_path: str, out_wav_path: str) -> bool:
    ffmpeg_bin = resolve_ffmpeg_bin()
    cmd = [
        ffmpeg_bin, "-y",
        "-i", video_path,
        "-vn", "-acodec", "pcm_s16le",
        "-ar", "16000", "-ac", "1",
        out_wav_path
    ]
    res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return res.returncode == 0 and os.path.exists(out_wav_path)

def mux_soft_subtitles(video_path: str, ass_path: str, out_mkv_path: str) -> bool:
    ffmpeg_bin = resolve_ffmpeg_bin()
    cmd = [
        ffmpeg_bin, "-y",
        "-i", video_path,
        "-i", ass_path,
        "-c:v", "copy",
        "-c:a", "copy",
        "-c:s", "copy",
        "-metadata:s:0", "language=zho",
        "-metadata:s:0", "title=紫电 AI 双语字幕",
        "-disposition:s:0", "default",
        out_mkv_path
    ]
    res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return res.returncode == 0 and os.path.exists(out_mkv_path)

def burn_hard_subtitles_nvenc(video_path: str, ass_path: str, out_mp4_path: str) -> bool:
    ffmpeg_bin = resolve_ffmpeg_bin()
    escaped_ass = ass_path.replace("\\", "/").replace(":", "\\:")
    vf_param = f"subtitles='{escaped_ass}'"
    
    cmd = [
        ffmpeg_bin, "-y",
        "-hwaccel", "cuda",
        "-i", video_path,
        "-vf", vf_param,
        "-c:v", "h264_nvenc",
        "-preset", "p4",
        "-cq", "23",
        "-c:a", "copy",
        out_mp4_path
    ]
    res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if res.returncode == 0 and os.path.exists(out_mp4_path):
        return True
    
    cmd_cpu = [
        ffmpeg_bin, "-y",
        "-i", video_path,
        "-vf", vf_param,
        "-c:v", "libx264",
        "-crf", "23",
        "-preset", "fast",
        "-c:a", "copy",
        out_mp4_path
    ]
    res_cpu = subprocess.run(cmd_cpu, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return res_cpu.returncode == 0 and os.path.exists(out_mp4_path)
