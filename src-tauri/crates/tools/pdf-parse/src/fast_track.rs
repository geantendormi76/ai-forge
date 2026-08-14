use crate::deep_track::DeepTrackEngine;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastTrackResult {
    pub success: bool,
    pub markdown: String,
    pub images: Vec<String>,
    pub error: Option<String>,
}

pub struct FastTrackEngine;

impl FastTrackEngine {
    pub async fn run_pages(
        pdf_path: &Path,
        output_dir: &Path,
        pages: Option<&[u32]>,
    ) -> Result<FastTrackResult, String> {
        let pages_usize: Option<Vec<usize>> = pages.map(|p| p.iter().map(|&n| n as usize).collect());
        let res = DeepTrackEngine::run_pages(pdf_path, output_dir, pages_usize.as_deref()).await?;
        Ok(FastTrackResult {
            success: res.success,
            markdown: res.markdown,
            images: res.images,
            error: res.error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_native_fasttrack_execution() {
        let pdf_path = Path::new(r"C:\dev\ai-forge\test\parse\pdf-parse-fast.pdf");
        let output_dir = Path::new(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(output_dir);

        if !pdf_path.exists() {
            println!("⚠️ [跳过测试] 测试物理 PDF 不存在: {:?}", pdf_path);
            return;
        }

        let res = FastTrackEngine::run_pages(pdf_path, output_dir, None)
            .await
            .expect("Native FastTrack 矢量轨道解析失败");

        let out_file = output_dir.join("pdf-parse-fast_out.md");
        fs::write(&out_file, &res.markdown).expect("写入物理 Markdown 失败");
    }
}
