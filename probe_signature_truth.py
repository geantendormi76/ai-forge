import os
import sys
import json
import base64
import hashlib
import subprocess
from pathlib import Path
import urllib.request

print("======================================================================")
print("🔍 [紫电 AI] 密码学验签与 Cloudflare CDN 缓存一致性全链路逆向探针")
print("======================================================================\n")

root_dir = Path(r"C:\dev\ai-forge")
local_appdata = Path(os.environ.get("LOCALAPPDATA", r"C:\Users\52484\AppData\Local"))
nsis_dir = root_dir / "src-tauri" / "target" / "release" / "bundle" / "nsis"
tauri_conf_path = root_dir / "src-tauri" / "tauri.conf.json"

def get_sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def get_file_sha256(path: Path) -> str:
    if not path.exists(): return "不存在"
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()

# 1. 探针 A: 本地最新编译出来的 0.1.17 安装包真值
print("📌 【探针 1/4】本地最新编译构建产物:")
local_exe = None
exe_files = sorted(nsis_dir.glob("*.exe"), key=lambda f: f.stat().st_mtime, reverse=True)
if exe_files:
    local_exe = exe_files[0]
    local_hash = get_file_sha256(local_exe)
    print(f"  • 文件名: {local_exe.name}")
    print(f"  • 大小:   {local_exe.stat().st_size} 字节")
    print(f"  • SHA256: {local_hash}")
else:
    print("  🚨 未找到本地构建的 setup.exe")

# 2. 探针 B: Cloudflare R2 公网 CDN 缓存与文件一致性
print("\n📌 【探针 2/4】Cloudflare CDN 云端文件与缓存状态探测:")
cloud_setup_url = "https://assets.geantendormi.top/downloads/zidian-ai-setup.exe"
try:
    req = urllib.request.Request(cloud_setup_url, headers={"User-Agent": "ZiDianAI-Probe/1.0"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        headers = dict(resp.headers)
        body = resp.read()
        cloud_hash = get_sha256(body)
        cloud_size = len(body)
        print(f"  • 云端 URL:        {cloud_setup_url}")
        print(f"  • HTTP 状态:       {resp.status} OK")
        print(f"  • 云端文件大小:    {cloud_size} 字节")
        print(f"  • 云端文件 SHA256: {cloud_hash}")
        print(f"  • CDN 缓存状态:    {headers.get('cf-cache-status', '未知 / DIRECT')}")
        print(f"  • Last-Modified:   {headers.get('last-modified', '未知')}")
        print(f"  • ETag:            {headers.get('etag', '未知')}")

        if local_exe:
            if cloud_hash == local_hash:
                print("  ✅ [哈希完全一致] 云端下载的安装包 与 本地构建出的 0.1.17 安装包 100% 相同！")
            else:
                print("  🚨 [严重失配] 云端文件哈希 与 本地 0.1.17 哈希不一致！")
                print("     原因: Cloudflare CDN 正在向客户端返回旧版本缓存文件，导致签名验证必然失败！")
except Exception as e:
    print(f"  🚨 云端安装包请求失败: {e}")

# 3. 探针 C: 本地安装目录实际落地 EXE 状态
print("\n📌 【探针 3/4】当前本地实际安装的 EXE 状态:")
candidate_exes = [
    local_appdata / "Programs" / "紫电AI" / "ai-forge.exe",
    local_appdata / "紫电AI" / "ai-forge.exe",
]
for p in candidate_exes:
    if p.exists():
        p_hash = get_file_sha256(p)
        print(f"  • 发现路径: {p}")
        print(f"    大小: {p.stat().st_size} 字节 | SHA256: {p_hash}")
        try:
            cmd = f'(Get-Item "{p}").VersionInfo | Select-Object FileVersion, ProductVersion | ConvertTo-Json -Compress'
            res = subprocess.run(["powershell", "-Command", cmd], capture_output=True, text=True)
            print(f"    版本信息: {res.stdout.strip()}")
        except: pass

# 4. 探针 D: 签名与公钥的 Key ID 密码学精确拆解比对
print("\n📌 【探针 4/4】密码学 Key ID 与签名数据结构白盒比对:")
try:
    conf = json.loads(tauri_conf_path.read_text(encoding="utf-8"))
    conf_pubkey = conf.get("plugins", {}).get("updater", {}).get("pubkey", "")
    
    # 解码公钥
    raw_pub = base64.b64decode(conf_pubkey).decode("utf-8")
    pub_key_line = raw_pub.splitlines()[1]
    pub_key_bytes = base64.b64decode(pub_key_line)
    pub_key_id = pub_key_bytes[2:10].hex().upper()
    print(f"  • tauri.conf.json 公钥 Key ID: {pub_key_id}")

    # 读取本地最新签名
    sig_files = sorted(nsis_dir.glob("*.sig"), key=lambda f: f.stat().st_mtime, reverse=True)
    if sig_files:
        raw_sig = base64.b64decode(sig_files[0].read_text(encoding="utf-8")).decode("utf-8")
        sig_line = raw_sig.splitlines()[1]
        sig_bytes = base64.b64decode(sig_line)
        sig_key_id = sig_bytes[2:10].hex().upper()
        print(f"  • 本地 .sig 签名声明 Key ID:    {sig_key_id}")
        if pub_key_id == sig_key_id:
            print("  ✅ [密码学 Key ID 吻合] 公钥 Key ID 与 签名 Key ID 100% 吻合！")
        else:
            print(f"  🚨 [密码学 Key ID 冲突] 公钥 ({pub_key_id}) != 签名 ({sig_key_id})！")
except Exception as e:
    print(f"  ⚠️ 密码学解析异常: {e}")

print("\n======================================================================")
print("📊 探针证据采集完毕，请将上方输出完整发回！")
print("======================================================================")
