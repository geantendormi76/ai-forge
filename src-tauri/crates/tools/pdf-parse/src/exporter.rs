use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::info;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

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

        let zip_file = File::create(output_zip_path)
            .map_err(|e| format!("创建 ZIP 容器文件失败: {e}"))?;

        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // 1. 写入 Markdown 主文件
        let md_filename = md_path.file_name().and_then(|n| n.to_str()).unwrap_or("document.md");
        zip.start_file(md_filename, options)
            .map_err(|e| format!("写入 ZIP 内部 md 文件头失败: {e}"))?;

        let mut md_content = Vec::new();
        let mut md_f = File::open(md_path).map_err(|e| format!("读取 md 文件失败: {e}"))?;
        md_f.read_to_end(&mut md_content).map_err(|e| format!("读取 md 内容失败: {e}"))?;
        zip.write_all(&md_content).map_err(|e| format!("写入 ZIP md 内容失败: {e}"))?;

        // 2. 写入 images 目录中的图片文件
        if images_dir.exists() && images_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(images_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            let zip_rel_path = format!("images/{}", file_name);
                            zip.start_file(&zip_rel_path, options)
                                .map_err(|e| format!("写入 ZIP 图片文件头失败 ({file_name}): {e}"))?;

                            let mut img_content = Vec::new();
                            let mut img_f = File::open(&path).map_err(|e| format!("读取图片文件失败: {e}"))?;
                            img_f.read_to_end(&mut img_content).map_err(|e| format!("读取图片内容失败: {e}"))?;
                            zip.write_all(&img_content).map_err(|e| format!("写入 ZIP 图片内容失败: {e}"))?;
                        }
                    }
                }
            }
        }

        zip.finish().map_err(|e| format!("完成 ZIP 容器打包失败: {e}"))?;

        let zip_size_kb = std::fs::metadata(output_zip_path)
            .map(|m| m.len() as f64 / 1024.0)
            .unwrap_or(0.0);

        info!(
            "📦 [纯血 Native ZIP 打包成功] 产物位置: {:?}, 大小: {:.2} KB",
            output_zip_path, zip_size_kb
        );

        Ok(output_zip_path.to_path_buf())
    }
}
