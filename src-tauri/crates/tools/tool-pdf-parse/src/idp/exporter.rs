use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::info;

pub struct ZipContainerExporter;

impl ZipContainerExporter {
    pub async fn create_zip_package(
        md_path: &Path,
        images_dir: &Path,
        output_zip_path: &Path,
    ) -> Result<PathBuf, String> {
        if !md_path.exists() {
            return Err(format!("Markdown 物理文件不存在: {:?}", md_path));
        }

        let py_script = r#"
import sys, os, zipfile

zip_path = sys.argv[1]
md_path = sys.argv[2]
images_dir = sys.argv[3]

with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
    if os.path.exists(md_path):
        zf.write(md_path, os.path.basename(md_path))
    if os.path.exists(images_dir):
        for root, _, files in os.walk(images_dir):
            for file in files:
                full_path = os.path.join(root, file)
                rel_path = os.path.relpath(full_path, os.path.dirname(images_dir))
                zf.write(full_path, rel_path)
"#;

        let output = Command::new("python3")
            .arg("-c")
            .arg(py_script)
            .arg(output_zip_path)
            .arg(md_path)
            .arg(images_dir)
            .output()
            .await
            .map_err(|e| format!("启动内置 zipfile 压缩管道失败: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ZIP 容器打包失败: {stderr}"));
        }

        if !output_zip_path.exists() {
            return Err("ZIP 打包产物未能在磁盘上生成".into());
        }

        let zip_size_kb = std::fs::metadata(output_zip_path)
            .map(|m| m.len() as f64 / 1024.0)
            .unwrap_or(0.0);

        info!(
            "📦 [ZIP 容器打包成功] 产物位置: {:?}, 大小: {:.2} KB",
            output_zip_path, zip_size_kb
        );

        Ok(output_zip_path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zip_container_exporter() {
        let md_path = PathBuf::from("/home/zhz/ai-toolkit/test/pdf/1_hybrid_output.md");
        let images_dir = PathBuf::from("/home/zhz/ai-toolkit/test/pdf/images");
        let zip_path = PathBuf::from("/home/zhz/ai-toolkit/test/pdf/1_hybrid_output.zip");

        if md_path.exists() {
            let res_path = ZipContainerExporter::create_zip_package(&md_path, &images_dir, &zip_path)
                .await
                .expect("ZIP 容器打包失败！");

            assert!(res_path.exists(), "生成的 .zip 文件必须物理存在！");
            let size_kb = std::fs::metadata(&res_path).unwrap().len() as f64 / 1024.0;
            println!("\n🎉 [ZIP 容器打包测试成功]");
            println!("  📦 生成容器文件: {:?}", res_path);
            println!("  📊 压缩包容量: {:.2} KB", size_kb);
            assert!(size_kb > 0.0);
        } else {
            println!("⚠️ [跳过测试] 物理 Markdown 文件不存在: {:?}", md_path);
        }
    }
}
