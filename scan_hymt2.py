import os

repo_dir = r"C:/dev/github/Hy-MT2"
print("=== 📂 Hy-MT2 仓库完整目录清单 ===")
for root, dirs, files in os.walk(repo_dir):
    if ".git" in root:
        continue
    rel = os.path.relpath(root, repo_dir)
    print(f"[{rel}]")
    for f in files:
        print(f"  - {f}")
