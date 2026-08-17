import os
import re

source_root = r"C:\dev\github\toolknit-desktop\toolknit-desktop"
output_md = r"C:\dev\ai-forge\toolknit-core-extracted.md"

target_files = [
    "ui-prototypes/home-v2-monochrome.html",
    "ui-prototypes/settings-v2-monochrome.html",
    "src/audio-extract-core.js",
    "src/video-convert-core.js",
    "src/audio-convert-core.js",
    "src/i18n.js"
]

extracted_blocks = []
extracted_blocks.append("# 🛡️ ToolKnit Desktop 核心 UI 原型与交互逻辑提取库\n\n")
extracted_blocks.append("> 本文件由紫电 AI 架构提取脚本自动生成，用于 1:1 外科手术式移植其暗黑单色极简美学、卡片状态机、使用步骤与 FAQ 手风琴组件。\n\n")

for rel_path in target_files:
    full_path = os.path.join(source_root, rel_path.replace("/", os.sep))
    if os.path.exists(full_path):
        with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()
        ext = rel_path.split(".")[-1]
        extracted_blocks.append(f"## 📄 文件: `{rel_path}`\n\n```{ext}\n{content}\n```\n\n---\n\n")
        print(f"✅ 成功提取: {rel_path} ({len(content)} 字符)")
    else:
        print(f"⚠️ 文件未找到: {rel_path}")

styles_path = os.path.join(source_root, "src", "styles.css")
if os.path.exists(styles_path):
    with open(styles_path, "r", encoding="utf-8", errors="ignore") as f:
        styles_content = f.read()
    extracted_blocks.append(f"## 🎨 核心样式摘录: `src/styles.css` (前 500 行核心变量与卡片组件规范)\n\n```css\n{styles_content[:35000]}\n```\n\n---\n\n")
    print(f"✅ 成功提取核心样式规范: src/styles.css")

with open(output_md, "w", encoding="utf-8") as f:
    f.writelines(extracted_blocks)

print(f"\n🎉 全部核心源码已成功聚合写入: {output_md}")
