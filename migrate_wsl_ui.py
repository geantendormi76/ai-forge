import os
import shutil
import sys
from pathlib import Path

def migrate_ui():
    wsl_src = Path(r"\\wsl.localhost\Ubuntu-24.04\home\zhz\ai-toolkit\src")
    if not wsl_src.exists():
        wsl_src = Path(r"\\wsl$\Ubuntu-24.04\home\zhz\ai-toolkit\src")
        if not wsl_src.exists():
            print("❌ 无法访问 WSL 目录，请确保 WSL 正在运行。")
            return

    target_src = Path(r"C:\dev\ai-forge\src")
    backup_dir = Path(r"C:\dev\ai-forge\src_backup_before_migration")

    print(f"🚀 开始执行 1:1 核心 UI 代码迁移...")
    print(f"源路径: {wsl_src}")
    print(f"目标路径: {target_src}")

    # 1. 自动备份当前桌面端 src
    if target_src.exists() and not backup_dir.exists():
        print(f"📦 正在备份当前 src 到 {backup_dir} ...")
        shutil.copytree(target_src, backup_dir)
        print("✅ 备份完成！")

    # 2. 需要 1:1 迁移的组件清单
    components_to_migrate = [
        "AITerminal.vue",
        "ASCIIText.vue",
        "Aurora.vue",
        "BorderGlow.vue",
        "CTA.vue",
        "CategorySelector.vue",
        "ComponentMarquee.vue",
        "CountUp.vue",
        "DecryptedText.vue",
        "DemoPlaceholder.vue",
        "DotField.vue",
        "ElectricBorder.vue",
        "FeatureIcon.vue",
        "Footer.vue",
        "GradientText.vue",
        "HeroBand.vue",
        "HeroCodeWindow.vue",
        "LiveDemo.vue",
        "QuickStart.vue",
        "SpotlightCard.vue",
        "StarCard.vue",
        "VariantTabs.vue",
    ]

    target_comp_dir = target_src / "components"
    target_comp_dir.mkdir(parents=True, exist_ok=True)

    migrated_count = 0
    for comp in components_to_migrate:
        src_file = wsl_src / "components" / comp
        if src_file.exists():
            dest_file = target_comp_dir / comp
            shutil.copy2(src_file, dest_file)
            migrated_count += 1
            print(f"  ✨ [组件已迁移] {comp}")
        else:
            print(f"  ⚠️ [未找到组件] {comp}")

    # 3. 迁移 HomeView.vue 到 views/
    target_views_dir = target_src / "views"
    target_views_dir.mkdir(parents=True, exist_ok=True)
    home_src = wsl_src / "views" / "HomeView.vue"
    if home_src.exists():
        dest_home = target_views_dir / "HomeView.vue"
        shutil.copy2(home_src, dest_home)
        print("  🎉 [核心主页已迁移] views/HomeView.vue")
        migrated_count += 1

    print("=" * 60)
    print(f"🏆 迁移完成！成功同步 {migrated_count} 个核心 UI 资产文件。")

if __name__ == "__main__":
    migrate_ui()
