use crate::pipeline::{PipelineResult, TextExtractMode, UnifiedPipeline};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepTrackResult {
    pub success: bool,
    pub markdown: String,
    pub images: Vec<String>,
    pub tables: Vec<String>,
    pub elapsed_ms: f64,
    pub error: Option<String>,
}

impl From<PipelineResult> for DeepTrackResult {
    fn from(res: PipelineResult) -> Self {
        Self {
            success: res.success,
            markdown: res.markdown,
            images: res.images,
            tables: res.tables,
            elapsed_ms: res.elapsed_ms,
            error: res.error,
        }
    }
}

pub struct DeepTrackEngine;

impl DeepTrackEngine {
    pub async fn run_pages(
        pdf_path: &Path,
        output_dir: &Path,
        pages: Option<&[usize]>,
    ) -> Result<DeepTrackResult, String> {
        let res = UnifiedPipeline::run_pages(
            pdf_path,
            output_dir,
            pages,
            TextExtractMode::OpticalOcr,
        )
        .await?;

        Ok(res.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_deep_track_scanned_pipeline() {
        let pdf_1 = PathBuf::from(r"C:\dev\ai-forge\test\parse\1_scanned.pdf");
        let pdf_2 = PathBuf::from(r"C:\dev\ai-forge\test\parse\2_scanned.pdf");
        let out_dir = PathBuf::from(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(&out_dir);

        println!("\n⚡ ===== [DeepTrack 纯位图扫描件视觉 OCR 实弹打靶启动] =====");

        if pdf_1.exists() {
            let res_1 = DeepTrackEngine::run_pages(&pdf_1, &out_dir, None)
                .await
                .expect("解析 1_scanned.pdf 失败");
            assert!(res_1.success);
            let out_file_1 = out_dir.join("1_scanned_out.md");
            fs::write(&out_file_1, &res_1.markdown).expect("写入 1_scanned_out.md 失败");
            println!(
                "✅ 样本 1_scanned.pdf 全量 4 页解析成功: 耗时={:.2} ms, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_1.elapsed_ms,
                res_1.markdown.len(),
                res_1.images.len(),
                res_1.tables.len(),
                out_file_1
            );
        }

        if pdf_2.exists() {
            let res_2 = DeepTrackEngine::run_pages(&pdf_2, &out_dir, None)
                .await
                .expect("解析 2_scanned.pdf 失败");
            assert!(res_2.success);
            let out_file_2 = out_dir.join("2_scanned_out.md");
            fs::write(&out_file_2, &res_2.markdown).expect("写入 2_scanned_out.md 失败");
            println!(
                "✅ 样本 2_scanned.pdf 全量 4 页解析成功: 耗时={:.2} ms, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_2.elapsed_ms,
                res_2.markdown.len(),
                res_2.images.len(),
                res_2.tables.len(),
                out_file_2
            );
        }

        println!("🎉 ===== [DeepTrack 扫描件视觉 OCR 实弹打靶全量完成！] =====\n");
    }
}
