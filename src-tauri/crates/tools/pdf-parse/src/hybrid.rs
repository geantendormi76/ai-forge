use crate::deep_track::DeepTrackEngine;
use crate::fast_track::FastTrackEngine;
use crate::pipeline::clean_katex_markdown;
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
            .map_err(|e| format!("三级漏斗分类分析失败: {e}"))?;

        let total_pages = metrics.len();
        let mut fast_pages: Vec<usize> = Vec::new();
        let mut deep_pages: Vec<usize> = Vec::new();

        for m in &metrics {
            let p_idx = (m.page_number as usize).saturating_sub(1);
            match m.decision {
                RouteDecision::FastTrackCpu => fast_pages.push(p_idx),
                RouteDecision::DeepTrackGpu => deep_pages.push(p_idx),
            }
        }

        info!(
            "🧠 [Hybrid 智能按页调度] 总页数: {}, 矢量轨 ({:?}), 扫描轨 ({:?})",
            total_pages, fast_pages, deep_pages
        );

        let mut all_images = Vec::new();
        let mut all_tables = Vec::new();
        let mut combined_markdown = String::new();

        // 场景 A：纯矢量文档 (如 1.pdf / 2.pdf，全量 1~N 页直通 FastTrack 单次批处理)
        if deep_pages.is_empty() && !fast_pages.is_empty() {
            let res = FastTrackEngine::run_pages(pdf_path, output_dir, None).await?;
            all_images.extend(res.images);
            all_tables.extend(res.tables);
            combined_markdown = res.markdown;
        }
        // 场景 B：纯扫描件文档 (如 3.pdf，全量 1~N 页直通 DeepTrack 视觉多模态)
        else if fast_pages.is_empty() && !deep_pages.is_empty() {
            let res = DeepTrackEngine::run_pages(pdf_path, output_dir, None).await?;
            all_images.extend(res.images);
            all_tables.extend(res.tables);
            combined_markdown = res.markdown;
        }
        // 场景 C：混合文档 (按页分别调度并在内存中按真实页码顺序拼接)
        else {
            let mut page_map: std::collections::BTreeMap<usize, String> = std::collections::BTreeMap::new();

            if !fast_pages.is_empty() {
                let res = FastTrackEngine::run_pages(pdf_path, output_dir, Some(&fast_pages)).await?;
                all_images.extend(res.images);
                all_tables.extend(res.tables);
                page_map.insert(fast_pages[0], res.markdown);
            }

            if !deep_pages.is_empty() {
                let res = DeepTrackEngine::run_pages(pdf_path, output_dir, Some(&deep_pages)).await?;
                all_images.extend(res.images);
                all_tables.extend(res.tables);
                page_map.insert(deep_pages[0], res.markdown);
            }

            for (_p_idx, md_text) in page_map {
                if !combined_markdown.is_empty() {
                    combined_markdown.push_str("\n\n---\n\n");
                }
                combined_markdown.push_str(&md_text);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_hybrid_engine_e2e() {
        let pdf_1 = PathBuf::from(r"C:\dev\ai-forge\test\parse\1.pdf");
        let pdf_2 = PathBuf::from(r"C:\dev\ai-forge\test\parse\2.pdf");
        let out_dir = PathBuf::from(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(&out_dir);

        println!("\n⚡ ===== [Hybrid 智能按页全量调度端到端打靶测试] =====");

        if pdf_1.exists() {
            let res_1 = HybridEngine::run_pdf(&pdf_1, &out_dir)
                .await
                .expect("Hybrid 解析 1.pdf 失败");
            println!(
                "✅ 样本 1.pdf 端到端成功: 矢量页数={}, 扫描页数={}, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_1.fast_pages_count,
                res_1.deep_pages_count,
                res_1.markdown.len(),
                res_1.images.len(),
                res_1.tables.len(),
                res_1.output_md_path
            );
            assert_eq!(res_1.fast_pages_count, 4, "1.pdf 全部 4 页必须命中矢量轨！");
            assert_eq!(res_1.deep_pages_count, 0);
        }

        if pdf_2.exists() {
            let res_2 = HybridEngine::run_pdf(&pdf_2, &out_dir)
                .await
                .expect("Hybrid 解析 2.pdf 失败");
            println!(
                "✅ 样本 2.pdf 端到端成功: 矢量页数={}, 扫描页数={}, 字符数={}, 插图数={}, 表格数={}, 产物={:?}",
                res_2.fast_pages_count,
                res_2.deep_pages_count,
                res_2.markdown.len(),
                res_2.images.len(),
                res_2.tables.len(),
                res_2.output_md_path
            );
            assert_eq!(res_2.fast_pages_count, 4, "2.pdf 全部 4 页必须命中矢量轨！");
            assert_eq!(res_2.deep_pages_count, 0);
        }

        println!("🎉 ===== [Hybrid 端到端多模态全量调度打靶全量通过！] =====\n");
    }
}
