#!/usr/bin/env python3
import json
import os
import sys
import re

def format_srt_time(seconds: float) -> str:
    millis = int(round((seconds % 1.0) * 1000))
    total_seconds = int(seconds)
    secs = total_seconds % 60
    mins = (total_seconds // 60) % 60
    hrs = total_seconds // 3600
    return f"{hrs:02d}:{mins:02d}:{secs:02d},{millis:03d}"

def generate_srt(segments: list, filter_bgm: bool = True) -> str:
    blocks = []
    valid_idx = 1
    for seg in segments:
        orig_text = seg.get("original", seg.get("text_en", seg.get("text_ja", seg.get("text", "")))).strip()
        trans_text = seg.get("translated", "").strip()
        
        # 物理过滤算子：彻底剔除带 🎼 (BGM) 或纯音效噪音行
        if filter_bgm and ("🎼" in orig_text or "🎼" in trans_text):
            continue
            
        # 过滤过短无意义纯气音行
        clean_orig = re.sub(r'[🎼\s]', '', orig_text)
        if len(clean_orig) < 2 and not trans_text:
            continue

        start_t = format_srt_time(float(seg.get("start", 0.0)))
        end_t = format_srt_time(float(seg.get("end", 0.0)))
        spk = seg.get("speaker", "S01")
        
        spk_prefix = f"[{spk}] " if spk else ""
        
        lines = []
        if orig_text:
            lines.append(f"{spk_prefix}{orig_text}")
        if trans_text:
            lines.append(trans_text)
            
        content = "\n".join(lines)
        block = f"{valid_idx}\n{start_t} --> {end_t}\n{content}\n"
        blocks.append(block)
        valid_idx += 1
        
    return "\n".join(blocks)

def export_file(json_path: str, out_dir: str):
    if not os.path.exists(json_path):
        print(f"❌ 找不到输入 JSON: {json_path}")
        return
        
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)
        
    segments = data.get("translated_segments", data.get("segments", []))
    base_name = os.path.splitext(os.path.basename(json_path))[0]
    
    srt_content = generate_srt(segments)
    
    os.makedirs(out_dir, exist_ok=True)
    srt_path = os.path.join(out_dir, f"{base_name}.srt")
    
    with open(srt_path, "w", encoding="utf-8") as sf:
        sf.write(srt_content)
        
    print(f"✅ BGM 噪音过滤完成！纯净双语 SRT 字幕已导出:")
    print(f"   • 字幕物理路径: {srt_path}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        j_path = sys.argv[1]
        o_dir = sys.argv[2] if len(sys.argv) > 2 else os.path.dirname(j_path)
        export_file(j_path, o_dir)
