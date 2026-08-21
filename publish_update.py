import os
import sys
import json
import subprocess
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

print("=== 🚀 紫电 AI 2026 SOTA 一键自动化版本发布与数字签名中台 ===\n")

root_dir = Path(r"C:\dev\ai-forge")
tauri_conf_path = root_dir / "src-tauri" / "tauri.conf.json"
user_home = Path(os.environ.get("USERPROFILE", r"C:\Users\52484"))
key_path = user_home / ".tauri" / "zidian-ai.key"

if not key_path.exists():
    print(f"🚨 未找到签名私钥文件: {key_path}")
    sys.exit(1)

# 1. 读取当前 tauri.conf.json 版本号
try:
    conf_data = json.loads(tauri_conf_path.read_text(encoding="utf-8"))
    current_version = conf_data.get("version", "0.1.0")
except Exception as e:
    print(f"🚨 读取 tauri.conf.json 失败: {e}")
    sys.exit(1)

version_tag = f"v{current_version}"
print(f"📦 正在准备发布版本: {version_tag}")

# 2. 设置签名私钥环境变量并执行生产构建
env = os.environ.copy()
env["TAURI_SIGNING_PRIVATE_KEY_PATH"] = str(key_path)
env["TAURI_SIGNING_PRIVATE_KEY_PASSWORD"] = ""

print("🔨 正在执行 pnpm tauri build 自动化签名编译...")
res = subprocess.run(["pnpm", "tauri", "build"], cwd=str(root_dir), env=env, shell=True)
if res.returncode != 0:
    print("🚨 构建失败，已中止发布流程！")
    sys.exit(1)

# 3. 寻找生成的安装包、更新包与签名文件
nsis_dir = root_dir / "src-tauri" / "target" / "release" / "bundle" / "nsis"
exe_files = list(nsis_dir.glob("*.exe"))
zip_files = list(nsis_dir.glob("*.nsis.zip"))
sig_files = list(nsis_dir.glob("*.nsis.zip.sig")) + list(nsis_dir.glob("*.sig"))

if not exe_files:
    print("🚨 未找到生成的 setup.exe 安装包")
    sys.exit(1)

exe_file = exe_files[0]
zip_file = zip_files[0] if zip_files else None
sig_file = sig_files[0] if sig_files else None

print(f"  ✅ 发现完整安装包: {exe_file.name} ({exe_file.stat().st_size / (1024*1024):.2f} MB)")
if zip_file:
    print(f"  ✅ 发现热更新差分包: {zip_file.name} ({zip_file.stat().st_size / (1024*1024):.2f} MB)")

signature_content = ""
if sig_file and sig_file.exists():
    signature_content = sig_file.read_text(encoding="utf-8").strip()
    print(f"  🔏 已提取 Ed25519 数字签名 ({len(signature_content)} 字符)")

# 4. 生成规范的 latest.json 清单
now_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
release_notes = f"✨ 紫电 AI {version_tag} 工业级 SOTA 版本发布：\n• 全系统协同性能与显存守卫深度优化\n• 4K/8K 图像超分、视频双语字幕与 PDF 解析稳定性增强\n• 零回归体验提升与已知问题修复"

latest_json_data = {
    "version": current_version,
    "notes": release_notes,
    "pub_date": now_utc,
    "platforms": {
        "windows-x86_64": {
            "signature": signature_content,
            "url": "https://assets.geantendormi.top/updates/zidian-ai_x64.nsis.zip"
        }
    }
}

latest_json_path = root_dir / "latest.json"
latest_json_path.write_text(json.dumps(latest_json_data, ensure_ascii=False, indent=2), encoding="utf-8")
print(f"  📄 已生成本地清单文件: {latest_json_path}")

# 5. 通过 WSL rclone 推送至 Cloudflare R2
print("\n🌐 正在同步安装包与更新清单至 Cloudflare R2...")

exe_rel_posix = str(exe_file.relative_to(Path("C:/"))).replace("\\", "/")
wsl_exe_path = f"/mnt/c/{exe_rel_posix}"
print(f"  ➔ 上传客户端安装包: {exe_file.name} -> r2:ai-toolkit-assets/downloads/zidian-ai-setup.exe")
subprocess.run(["wsl", "rclone", "copyto", wsl_exe_path, "r2:ai-toolkit-assets/downloads/zidian-ai-setup.exe", "-P"])

if zip_file:
    zip_rel_posix = str(zip_file.relative_to(Path("C:/"))).replace("\\", "/")
    wsl_zip_path = f"/mnt/c/{zip_rel_posix}"
    print(f"  ➔ 上传热更新差分包: {zip_file.name} -> r2:ai-toolkit-assets/updates/zidian-ai_x64.nsis.zip")
    subprocess.run(["wsl", "rclone", "copyto", wsl_zip_path, "r2:ai-toolkit-assets/updates/zidian-ai_x64.nsis.zip", "-P"])

json_rel_posix = str(latest_json_path.relative_to(Path("C:/"))).replace("\\", "/")
wsl_json_path = f"/mnt/c/{json_rel_posix}"
print(f"  ➔ 上传最新清单: latest.json -> r2:ai-toolkit-assets/updates/latest.json")
subprocess.run(["wsl", "rclone", "copyto", wsl_json_path, "r2:ai-toolkit-assets/updates/latest.json", "-P"])

# 6. 连通性探测
print("\n🔍 正在验证云端最新版本探针...")
check_url = "https://assets.geantendormi.top/updates/latest.json"
try:
    req = urllib.request.Request(check_url, headers={"User-Agent": "ZiDianAI-Updater/1.0"})
    with urllib.request.urlopen(req, timeout=8) as resp:
        data = json.loads(resp.read().decode("utf-8"))
        print(f"\n🎉 云端已成功激活最新版本: v{data.get('version')} (发布时间: {data.get('pub_date')})")
        print(f"🌐 清单地址: {check_url}")
        print(f"🚀 全球桌面客户端现已全面具备实时热更新与版本提醒能力！")
except Exception as e:
    print(f"⚠️ 云端验证提示: {e} (可能存在 1-2 分钟 CDN 缓存刷新延迟)")
