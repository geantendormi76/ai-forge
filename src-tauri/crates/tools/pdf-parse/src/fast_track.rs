use crate::pipeline::{PipelineResult, TextExtractMode, UnifiedPipeline};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastTrackResult {
    pub success: bool,
    pub markdown: String,
    pub images: Vec<String>,
    pub tables: Vec<String>,
    pub elapsed_ms: f64,
    pub error: Option<String>,
}

impl From<PipelineResult> for FastTrackResult {
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

pub struct FastTrackEngine;

impl FastTrackEngine {
    pub async fn run_pages(
        pdf_path: &Path,
        output_dir: &Path,
        pages: Option<&[usize]>,
    ) -> Result<FastTrackResult, String> {
        let res = UnifiedPipeline::run_pages(
            pdf_path,
            output_dir,
            pages,
            TextExtractMode::VectorOnly,
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
    async fn test_fast_track_all_pages_pipeline() {
        let pdf_1 = PathBuf::from(r"C:\dev\ai-forge\test\parse\1.pdf");
        let pdf_2 = PathBuf::from(r"C:\dev\ai-forge\test\parse\2.pdf");
        let out_dir = PathBuf::from(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(&out_dir);

        println!("\n⚡ ===== [FastTrack 全量多页高保真排版实弹测试] =====");

        if pdf_1.exists() {
            // 传入 None 代表全量遍历 1.pdf 全部 4 个页面
            let res_1 = FastTrackEngine::run_pages(&pdf_1, &out_dir, None)
                .await
                .expect("解析 1.pdf 全量页面失败");
            assert!(res_1.success);
            let out_file_1 = out_dir.join("1_fast_out.md");
            fs::write(&out_file_1, &res_1.markdown).expect("写入 1_fast_out.md 失败");
            println!(
                "✅ 样本 1.pdf 全量 4 页解析成功: 耗时={:.2} ms, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_1.elapsed_ms,
                res_1.markdown.len(),
                res_1.images.len(),
                res_1.tables.len(),
                out_file_1
            );
            assert!(res_1.markdown.contains("PP-DocLayout"), "1.pdf 大标题必须存在");
            assert!(res_1.markdown.contains("Abstract"), "1.pdf Abstract 必须存在");
            assert!(res_1.markdown.contains("Introduction"), "1.pdf Introduction 必须存在");
        }

        if pdf_2.exists() {
            // 传入 None 代表全量遍历 2.pdf 全部 4 个页面
            let res_2 = FastTrackEngine::run_pages(&pdf_2, &out_dir, None)
                .await
                .expect("解析 2.pdf 全量页面失败");
            assert!(res_2.success);
            let out_file_2 = out_dir.join("2_fast_out.md");
            fs::write(&out_file_2, &res_2.markdown).expect("写入 2_fast_out.md 失败");
            println!(
                "✅ 样本 2.pdf 全量 4 页解析成功: 耗时={:.2} ms, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_2.elapsed_ms,
                res_2.markdown.len(),
                res_2.images.len(),
                res_2.tables.len(),
                out_file_2
            );
            assert!(res_2.markdown.contains("LinearRAG") || res_2.markdown.contains("LINEARRAG"), "2.pdf 大标题必须存在");
        }

        println!("🎉 ===== [FastTrack 全量多页高保真实弹打靶全量通过！] =====\n");
    }
}
