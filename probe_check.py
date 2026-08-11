import os
import glob
import subprocess

target_dir = r"C:\dev\ai-forge\src-tauri\target"
caches = glob.glob(os.path.join(target_dir, "**", "CMakeCache.txt"), recursive=True)

print("=== 🔍 ai-forge 当前 CMakeCache 生成器锁定检测 ===")
for c in caches:
    if os.path.exists(c):
        print(f"📄 Cache: {c}")
        try:
            with open(c, 'r', encoding='utf-8', errors='ignore') as f:
                for line in f:
                    if 'CMAKE_GENERATOR:' in line:
                        print("  ", line.strip())
        except Exception as e:
            print("   Error:", e)

print("\n=== 🔍 正在运行的 C++ 编译进程 ===")
try:
    cmd = 'tasklist /FI "IMAGENAME eq cmake.exe" /FI "IMAGENAME eq ninja.exe" /FI "IMAGENAME eq MSBuild.exe"'
    out = subprocess.check_output(cmd, shell=True).decode('gbk', errors='ignore')
    print(out)
except Exception as e:
    print("Tasklist error:", e)
