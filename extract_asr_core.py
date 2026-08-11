import os
from pathlib import Path

src_root = Path(r"C:\Users\52484\StudioProjects\asr-Android")
out_md = Path(r"C:\dev\ai-forge\asr_android_core_code.md")

targets = [
    r"models\tool-ASR\asr_daemon.py",
    r"models\tool-ASR\live_asr_bridge.py",
    r"models\convert_moss_to_gguf.py",
    r"rust-core\src\lib.rs",
    r"rust-core\src\asr\mod.rs",
]

lines = ["# 🛡️ asr-Android 核心源码全量提取", ""]

for rel in targets:
    p = src_root / rel
    rel_str = str(rel).replace("\\", "/")
    if p.exists():
        content = p.read_text(encoding="utf-8", errors="ignore")
        ext = p.suffix.lstrip(".")
        lines.append(f"## File: {rel_str}")
        lines.append(f"```{ext}")
        lines.append(content)
        lines.append("```")
        lines.append("")
    else:
        lines.append(f"## File: {rel_str} (NOT FOUND)")
        lines.append("")

out_md.write_text("\n".join(lines), encoding="utf-8")
print(f"✅ 核心源码提取完成！产物已落盘至: {out_md}")
