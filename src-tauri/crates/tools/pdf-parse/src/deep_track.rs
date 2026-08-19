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
