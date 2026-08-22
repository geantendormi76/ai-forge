import os
import sys
import json
import base64
import shutil
import subprocess
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

print("=== 🚀 紫电 AI 2026 SOTA 一键自动化版本发布与数字签名中台 ===\n")

print("🧹 [步骤 0/6] 检查并释放后台进程锁...")
subprocess.run(["taskkill", "/F", "/IM", "ai-forge.exe"], capture_output=True, shell=True)

root_dir = Path(r"C:\dev\ai-forge")
tauri_conf_path = root_dir / "src-tauri" / "tauri.conf.json"
package_json_path = root_dir / "package.json"
dist_dir = root_dir / "dist"
nsis_dir = root_dir / "src-tauri" / "target" / "release" / "bundle" / "nsis"
user_home = Path(os.environ.get("USERPROFILE", r"C:\Users\52484"))
key_path = user_home / ".tauri" / "zidian-ai.key"
pub_path = user_home / ".tauri" / "zidian-ai.key.pub"

if not key_path.exists():
    print(f"🚨 未找到签名私钥文件: {key_path}")
    sys.exit(1)

private_key_text = key_path.read_text(encoding="utf-8").strip()

def to_wsl_path(win_path: Path) -> str:
    p_str = str(win_path.resolve())
    drive = p_str[0].lower()
    rest = p_str[2:].replace("\\", "/")
    return f"/mnt/{drive}{rest}"

print("🔑 [步骤 1/6] 正在校验公钥格式并执行 SemVer 语义化版本自增...")
conf_data = json.loads(tauri_conf_path.read_text(encoding="utf-8"))

# 1. 权威公钥校验与单层 Base64 自愈
if pub_path.exists():
    raw_pub = pub_path.read_text(encoding="utf-8").strip()
    if raw_pub.startswith("ZFc1"):
        raw_pub = base64.b64decode(raw_pub).decode("utf-8")
    if not raw_pub.startswith("dW50"):
        raw_pub = base64.b64encode(raw_pub.encode("utf-8")).decode("utf-8")
    conf_data.setdefault("plugins", {}).setdefault("updater", {})["pubkey"] = raw_pub
    print(f"  ✅ 权威公钥校验通过: {raw_pub[:28]}... ({len(raw_pub)} 字符)")

# 2. 严谨的 SemVer 三段式版本号自动递增
old_version = conf_data.get("version", "0.1.14")
parts = [int(p) for p in old_version.split(".")]
if len(parts) == 3:
    parts[2] += 1
    new_version = f"{parts[0]}.{parts[1]}.{parts[2]}"
else:
    new_version = "0.1.15"

conf_data["version"] = new_version
version_tag = f"v{new_version}"
tauri_conf_path.write_text(json.dumps(conf_data, ensure_ascii=False, indent=2), encoding="utf-8")

# 同步写入 package.json
if package_json_path.exists():
    pkg_data = json.loads(package_json_path.read_text(encoding="utf-8"))
    pkg_data["version"] = new_version
    package_json_path.write_text(json.dumps(pkg_data, ensure_ascii=False, indent=2), encoding="utf-8")

print(f"  ✅ 版本号自增成功: v{old_version} ➔ {version_tag}")

print("\n🎨 [步骤 2/6] 清理旧构建缓存并编译前端...")
if dist_dir.exists():
    shutil.rmtree(dist_dir, ignore_errors=True)
    print("  🧹 已物理清理 dist 目录")
if nsis_dir.exists():
    shutil.rmtree(nsis_dir, ignore_errors=True)
    print("  🧹 已物理清理旧安装包目录 (nsis/)")

build_res = subprocess.run(["pnpm", "build"], cwd=str(root_dir), shell=True)
if build_res.returncode != 0:
    print("🚨 前端构建 (pnpm build) 失败，已中止发布流程！")
    sys.exit(1)
print("  ✅ 前端最新产物已 100% 编译落盘至 dist/")

print(f"\n🔨 [步骤 3/6] 执行 pnpm tauri build 自动化生产打包与数字签名 ({version_tag})...")
env = os.environ.copy()
env["TAURI_SIGNING_PRIVATE_KEY"] = private_key_text
env["TAURI_SIGNING_PRIVATE_KEY_PATH"] = str(key_path)
env["TAURI_SIGNING_PRIVATE_KEY_PASSWORD"] = ""
res = subprocess.run(["pnpm", "tauri", "build"], cwd=str(root_dir), env=env, shell=True)
if res.returncode != 0:
    print("🚨 Tauri 构建失败，已中止发布流程！")
    sys.exit(1)

print("\n🔏 [步骤 4/6] 提取最新安装包与 Ed25519 数字签名...")
exe_files = sorted(nsis_dir.glob("*.exe"), key=lambda f: f.stat().st_mtime, reverse=True)
if not exe_files:
    print(f"🚨 未在 {nsis_dir} 找到生成的 setup.exe 安装包")
    sys.exit(1)
exe_file = exe_files[0]

sig_files = sorted(nsis_dir.glob("*.sig"), key=lambda f: f.stat().st_mtime, reverse=True)
if not sig_files:
    print("🚨 缺失 .sig 签名文件！")
    sys.exit(1)
sig_file = sig_files[0]

signature_content = sig_file.read_text(encoding="utf-8").strip()
file_size_mb = exe_file.stat().st_size / (1024 * 1024)
print(f"  ✅ 锁定最新安装包: {exe_file.name} ({file_size_mb:.2f} MB)")
print(f"  🔏 已提取 Ed25519 签名: {sig_file.name} ({len(signature_content)} 字符)")

now_utc = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
release_notes = f"✨ 紫电 AI {version_tag} 工业级 SOTA 版本发布：\n• 全面并网 4 大 AI 工具白盒细粒度进度看板 (毫秒级阶段提示与子进度)\n• 极简轻量架构与便携数据安全隔离 (./Data)\n• 4 大 AI 生产力工具算子与 1.78GB CUDA 运行时按需自愈\n• 纯血 Rust 端侧静默热更新与切块进度事件总线闭环"

latest_json_data = {
    "version": new_version,
    "notes": release_notes,
    "pub_date": now_utc,
    "platforms": {
        "windows-x86_64": {
            "signature": signature_content,
            "url": "https://assets.geantendormi.top/downloads/zidian-ai-setup.exe"
        }
    }
}

latest_json_path = root_dir / "latest.json"
latest_json_path.write_text(json.dumps(latest_json_data, ensure_ascii=False, indent=2), encoding="utf-8")
print(f"  📄 已生成本地最新清单: {latest_json_path}")

print("\n🌐 [步骤 5/6] 同步安装包与更新清单至 Cloudflare R2...")
wsl_exe_path = to_wsl_path(exe_file)
print(f"  ➔ [1/2] 上传安装包: {exe_file.name} -> r2:ai-toolkit-assets/downloads/zidian-ai-setup.exe")
subprocess.run(["wsl", "rclone", "copyto", wsl_exe_path, "r2:ai-toolkit-assets/downloads/zidian-ai-setup.exe", "-P"], check=True)

wsl_json_path = to_wsl_path(latest_json_path)
print(f"  ➔ [2/2] 上传清单: latest.json -> r2:ai-toolkit-assets/updates/latest.json")
subprocess.run(["wsl", "rclone", "copyto", wsl_json_path, "r2:ai-toolkit-assets/updates/latest.json", "-P"], check=True)

print("\n🔍 [步骤 6/6] 正在执行 Cloudflare R2 全球 CDN 探针打靶...")

def probe_url(url: str, label: str):
    req = urllib.request.Request(url, headers={"User-Agent": "ZiDianAI-Updater/1.0"})
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            status = resp.status
            size = resp.headers.get("Content-Length", "未知")
            print(f"  ✅ [{label}] 探针 200 OK | 大小: {size} 字节 | URL: {url}")
            return True
    except Exception as e:
        print(f"  🚨 [{label}] 探针失败: {e} | URL: {url}")
        return False

p1 = probe_url("https://assets.geantendormi.top/updates/latest.json", "更新清单 latest.json")
p2 = probe_url("https://assets.geantendormi.top/downloads/zidian-ai-setup.exe", "最新完整安装包 zidian-ai-setup.exe")

if p1 and p2:
    print(f"\n🎉 恭喜！紫电 AI {version_tag} 全部资产已在全球 Cloudflare CDN 成功点火上线！")
    print("🚀 用户端热更新与便携下载已完全恢复正常，0 个 404 错误！")
else:
    print("\n⚠️ 部分云端资源探针异常，请检查 R2 桶绑定或 CDN 缓存！")
