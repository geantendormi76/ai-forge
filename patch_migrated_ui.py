import os
import re
from pathlib import Path

def patch_components():
    src_dir = Path(r"C:\dev\ai-forge\src")
    components_dir = src_dir / "components"
    views_dir = src_dir / "views"

    vue_files = list(components_dir.glob("*.vue")) + list(views_dir.glob("*.vue"))
    print(f"🔍 正在白盒审计并修复 {len(vue_files)} 个 Vue 组件...")

    patched_count = 0

    for file_path in vue_files:
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()

            new_content = content

            # 1. 替换 @lucide/vue 为 lucide-vue-next
            if "@lucide/vue" in new_content:
                new_content = new_content.replace("@lucide/vue", "lucide-vue-next")

            # 2. 修复 @/components 别名路径为相对路径 (提升 Vite 构建稳定性)
            if "@/components/" in new_content:
                if "views" in str(file_path):
                    new_content = new_content.replace("@/components/", "../components/")
                else:
                    new_content = new_content.replace("@/components/", "./")

            if new_content != content:
                with open(file_path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                patched_count += 1
                print(f"  ✨ [依赖已对齐] {file_path.name}")

        except Exception as e:
            print(f"  ❌ 修复失败: {file_path.name} -> {e}")

    print("=" * 60)
    print(f"🎉 依赖对齐完成！共修正 {patched_count} 个组件的导入契约。")

if __name__ == "__main__":
    patch_components()
