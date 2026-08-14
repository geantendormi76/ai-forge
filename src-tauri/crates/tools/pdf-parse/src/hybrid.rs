use crate::deep_track::{clean_katex_markdown, DeepTrackEngine};
use crate::fast_track::FastTrackEngine;
use crate::probe::{PdfRouteProbe, RouteDecision};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone)]
pub struct HybridResult {
    pub markdown: String,
    pub images: Vec<String>,
    pub tables: Vec<String>,
    pub fast_pages_count: usize,
    pub deep_pages_count: usize,
    pub output_md_path: Option<PathBuf>,
}

pub struct HybridEngine;

impl HybridEngine {
    pub async fn run_pdf(
        pdf_path: &Path,
        output_dir: &Path,
    ) -> Result<HybridResult, String> {
        if !pdf_path.exists() {
            return Err(format!("输入物理文件不存在: {:?}", pdf_path));
        }

        let metrics = PdfRouteProbe::probe_file(pdf_path)
            .map_err(|e| format!("探针分析失败: {e}"))?;

        let total_pages = metrics.len();
        let mut fast_pages: Vec<u32> = Vec::new();
        let mut deep_pages: Vec<usize> = Vec::new();

        for m in &metrics {
            match m.decision {
                RouteDecision::FastTrackCpu => fast_pages.push(m.page_number),
                RouteDecision::DeepTrackGpu => deep_pages.push(m.page_number as usize),
            }
        }

        info!(
            "🧠 [Hybrid 按需降维引擎] 总页数: {}, CPU 矢量轨 ({:?}), GPU 神经网络轨 ({:?})",
            total_pages, fast_pages, deep_pages
        );

        let mut page_markdowns: std::collections::BTreeMap<u32, String> = std::collections::BTreeMap::new();
        let mut all_images = Vec::new();
        let mut all_tables = Vec::new();

        if !fast_pages.is_empty() {
            match FastTrackEngine::run_pages(pdf_path, output_dir, Some(&fast_pages)).await {
                Ok(res) => {
                    all_images.extend(res.images);
                    page_markdowns.insert(fast_pages[0], res.markdown);
                }
                Err(e) => {
                    tracing::warn!("⚠️ FastTrack 分流异常: {e}, 退回 DeepTrack");
                    deep_pages.extend(fast_pages.iter().map(|&p| p as usize));
                    fast_pages.clear();
                }
            }
        }

        if !deep_pages.is_empty() {
            match DeepTrackEngine::run_pages(pdf_path, output_dir, Some(&deep_pages)).await {
                Ok(res) => {
                    all_images.extend(res.images);
                    all_tables.extend(res.tables);
                    page_markdowns.insert(deep_pages[0] as u32, res.markdown);
                }
                Err(e) => return Err(format!("DeepTrack 分流推演失败: {e}")),
            }
        }

        let mut combined_markdown = String::new();
        for (_p_num, md_text) in page_markdowns {
            if !combined_markdown.is_empty() {
                combined_markdown.push_str("\n\n---\n\n");
            }
            combined_markdown.push_str(&md_text);
        }

        let cleaned_markdown = clean_katex_markdown(&combined_markdown);

        let stem = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let md_filename = format!("{}_hybrid_output.md", stem);
        let out_md_path = output_dir.join(&md_filename);
        if let Err(e) = tokio::fs::write(&out_md_path, &cleaned_markdown).await {
            tracing::warn!("⚠️ Markdown 文件落盘失败: {e}");
        } else {
            info!("📄 [Hybrid 产物已落盘] {:?}", out_md_path);
        }

        Ok(HybridResult {
            markdown: cleaned_markdown,
            images: all_images,
            tables: all_tables,
            fast_pages_count: fast_pages.len(),
            deep_pages_count: deep_pages.len(),
            output_md_path: Some(out_md_path),
        })
    }
}
