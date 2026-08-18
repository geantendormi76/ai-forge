use crate::deep_track::clean_katex_markdown;
use serde::{Deserialize, Serialize};
use service_pdfium::PdfiumEngine;
use std::path::Path;
use tracing::info;

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
        _output_dir: &Path,
        pages: Option<&[u32]>,
    ) -> Result<FastTrackResult, String> {
        if !pdf_path.exists() {
            return Err(format!("物理文件不存在: {:?}", pdf_path));
        }

        let page_count = PdfiumEngine::get_page_count(pdf_path)
            .map_err(|e| format!("Pdfium 获取页数失败: {e}"))?;

        let target_pages: Vec<usize> = if let Some(p_list) = pages {
            p_list
                .iter()
                .map(|&p| (p as usize).saturating_sub(1))
                .filter(|&p| p < page_count)
                .collect()
        } else {
            (0..page_count).collect()
        };

        let mut page_markdowns = Vec::new();

        for &p_idx in &target_pages {
            let raw_text = PdfiumEngine::extract_page_text(pdf_path, p_idx)
                .unwrap_or_default();

            // 1. 过滤不可见控制字符 (彻底消灭 0x02 等连字乱码)
            let filtered_chars: String = raw_text
                .chars()
                .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
                .collect();

            // 2. 按行清洗与去连字符 (Dehyphenation)
            let mut page_lines = Vec::new();
            let raw_lines: Vec<&str> = filtered_chars.lines().collect();

            let mut i = 0;
            while i < raw_lines.len() {
                let mut line = raw_lines[i].trim().to_string();
                if line.is_empty() {
                    i += 1;
                    continue;
                }

                // 检查行末跨行连字符 (如 "multi-\ncolumn")
                while line.ends_with('-') && i + 1 < raw_lines.len() {
                    let next_line = raw_lines[i + 1].trim();
                    if !next_line.is_empty() && next_line.chars().next().map_or(false, |c| c.is_alphabetic()) {
                        line.pop(); // 移除连字符 '-'
                        line.push_str(next_line);
                        i += 1;
                    } else {
                        break;
                    }
                }

                // 启发式检测文档大标题与章节标题 (首页首行大写/编号)
                if p_idx == 0 && i == 0 && line.len() > 5 && !line.starts_with('#') {
                    line = format!("# {}", line);
                } else if is_likely_section_heading(&line) {
                    if !line.starts_with('#') {
                        line = format!("## {}", line);
                    }
                }

                page_lines.push(line);
                i += 1;
            }

            let page_md = page_lines.join("\n\n");
            if !page_md.trim().is_empty() {
                page_markdowns.push(page_md);
            }
        }

        let combined = page_markdowns.join("\n\n---\n\n");
        let cleaned_md = clean_katex_markdown(&combined);

        info!(
            "⚡ [Native FastTrack 极速矢量轨成功] 页数: {}, 提炼 Markdown 字符数: {}",
            target_pages.len(),
            cleaned_md.len()
        );

        Ok(FastTrackResult {
            success: true,
            markdown: cleaned_md,
            images: Vec::new(),
            error: None,
        })
    }
}

fn is_likely_section_heading(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return false;
    }

    if trimmed.eq_ignore_ascii_case("abstract")
        || trimmed.eq_ignore_ascii_case("introduction")
        || trimmed.eq_ignore_ascii_case("conclusion")
        || trimmed.eq_ignore_ascii_case("references")
    {
        return true;
    }

    let starts_with_num = trimmed.chars().next().map_or(false, |c| c.is_ascii_digit());
    if starts_with_num {
        if let Some(space_pos) = trimmed.find(' ') {
            let prefix = &trimmed[..space_pos];
            if prefix.chars().all(|c| c.is_ascii_digit() || c == '.') {
                let rest = trimmed[space_pos..].trim();
                if !rest.is_empty() && (rest.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) || rest.len() < 40) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_native_fasttrack_execution() {
        let pdf_path = Path::new(r"C:\dev\ai-forge\test\parse\1.pdf");
        let fallback_path = Path::new(r"C:\dev\ai-forge\test\fixtures\1.pdf");
        let target_pdf = if pdf_path.exists() { pdf_path } else { fallback_path };

        let output_dir = Path::new(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(output_dir);

        if !target_pdf.exists() {
            println!("⚠️ [跳过测试] 测试物理 PDF 不存在: {:?}", target_pdf);
            return;
        }

        let res = FastTrackEngine::run_pages(target_pdf, output_dir, None)
            .await
            .expect("Native FastTrack 矢量轨道解析失败");

        assert!(res.success);
        let out_file = output_dir.join("1_fast_out.md");
        fs::write(&out_file, &res.markdown).expect("写入物理 Markdown 失败");
        println!("✅ FastTrack 测试产物落盘至: {:?}", out_file);
    }
}
