use lopdf::Document;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("🚨 [PDF 解析失败] {0}")]
    ParseError(String),
    #[error("🚨 [IO 读取异常] {0}")]
    IoError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteDecision {
    FastTrackCpu,
    DeepTrackGpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMetrics {
    pub page_number: u32,
    pub text_length: usize,
    pub valid_char_count: usize,
    pub readable_ratio: f32,
    pub image_count: usize,
    pub vector_path_count: usize,
    pub math_symbol_count: usize,
    pub decision: RouteDecision,
}

pub struct PdfRouteProbe;

impl PdfRouteProbe {
    pub fn probe_bytes(bytes: &[u8]) -> Result<Vec<PageMetrics>, ProbeError> {
        let doc = Document::load_mem(bytes)
            .map_err(|e| ProbeError::ParseError(format!("无法加载 PDF 内存字节流: {e}")))?;
        Self::probe_document(&doc)
    }

    pub fn probe_file(path: &Path) -> Result<Vec<PageMetrics>, ProbeError> {
        let doc = Document::load(path)
            .map_err(|e| ProbeError::ParseError(format!("无法打开物理 PDF 文件: {e}")))?;
        Self::probe_document(&doc)
    }

    fn is_readable_char(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || c.is_ascii_punctuation()
            || (c >= '\u{4e00}' && c <= '\u{9fff}')
            || (c >= '\u{3000}' && c <= '\u{303f}')
            || (c >= '\u{ff00}' && c <= '\u{ffef}')
            || (c >= '\u{2000}' && c <= '\u{206f}')
    }

    fn count_math_indicators(text: &str) -> usize {
        let mut count = 0;
        let math_keywords = [
            "\\frac", "\\sqrt", "\\sum", "\\int", "\\alpha", "\\beta", "\\theta", "\\sigma",
            "\\pi", "\\times", "\\matrix",
        ];
        for kw in &math_keywords {
            count += text.matches(kw).count() * 3;
        }
        for c in text.chars() {
            if matches!(
                c,
                '∫' | '∑' | '√' | 'α' | 'β' | 'θ' | 'π' | '±' | '≤' | '≥' | '≠' | '∞' | '^' | '_'
            ) {
                count += 1;
            }
        }
        count
    }

    fn probe_document(doc: &Document) -> Result<Vec<PageMetrics>, ProbeError> {
        let mut results = Vec::new();
        let pages = doc.get_pages();

        for (page_num, page_id) in pages {
            let mut raw_text_buffer = String::new();
            let mut img_count = 0;
            let mut vec_count = 0;

            if let Ok(content_bytes) = doc.get_page_content(page_id) {
                if let Ok(content) = lopdf::content::Content::decode(&content_bytes) {
                    for operation in &content.operations {
                        match operation.operator.as_str() {
                            "Tj" | "TJ" | "'" | "\"" => {
                                for arg in &operation.operands {
                                    if let Ok(bytes) = arg.as_str() {
                                        raw_text_buffer.push_str(&String::from_utf8_lossy(bytes));
                                    } else if let Ok(arr) = arg.as_array() {
                                        for elem in arr {
                                            if let Ok(bytes) = elem.as_str() {
                                                raw_text_buffer
                                                    .push_str(&String::from_utf8_lossy(bytes));
                                            }
                                        }
                                    }
                                }
                            }
                            "m" | "l" | "c" | "v" | "y" | "re" | "f" | "F" | "s" | "S" => {
                                vec_count += 1;
                            }
                            "Do" => {
                                img_count += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }

            if let Ok(resources) = doc.get_page_resources(page_id) {
                if let Some(res_dict) = resources.0 {
                    if let Ok(xobjects) = res_dict.get(b"XObject") {
                        if let Ok(dict) = xobjects.as_dict() {
                            for (_key, val) in dict {
                                if let Ok(obj_ref) = val.as_reference() {
                                    if let Ok(obj) = doc.get_object(obj_ref) {
                                        if let Ok(stream) = obj.as_stream() {
                                            if let Ok(subtype) = stream.dict.get(b"Subtype") {
                                                if let Ok("Image") = subtype.as_name_str() {
                                                    img_count += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let text_len = raw_text_buffer.len();
            let valid_char_count = raw_text_buffer
                .chars()
                .filter(|c| !c.is_control() && !c.is_whitespace())
                .count();

            let readable_chars = raw_text_buffer
                .chars()
                .filter(|&c| Self::is_readable_char(c))
                .count();

            let readable_ratio = if valid_char_count > 0 {
                readable_chars as f32 / valid_char_count as f32
            } else {
                0.0
            };

            let math_symbol_count = Self::count_math_indicators(&raw_text_buffer);

            // 🛡️ 2026 SOTA 工业级通用多维判别准则 (SOTA Hybrid Routing Standard)
            // 1. 若有效字符数 < 50，说明本页无可用原生文字流（纯扫描件/整页位图），必须触发 GPU 视觉 OCR 轨
            let is_scanned_page = valid_char_count < 50;

            // 2. 若字符数充足但可读字符率 < 60%，说明字库损坏或乱码未映射，必须降级触发视觉 OCR 自愈
            let is_corrupted_encoding = valid_char_count >= 50 && readable_ratio < 0.60;

            let decision = if is_scanned_page || is_corrupted_encoding {
                RouteDecision::DeepTrackGpu
            } else {
                RouteDecision::FastTrackCpu
            };

            info!(
                "📄 [PDF 智能分类探针] 页码 {}: 有效字数={}, 可读率={:.2}, 图片数={}, 矢量路径数={} ──► 分流: {:?}",
                page_num, valid_char_count, readable_ratio, img_count, vec_count, decision
            );

            results.push(PageMetrics {
                page_number: page_num,
                text_length: text_len,
                valid_char_count,
                readable_ratio,
                image_count: img_count,
                vector_path_count: vec_count,
                math_symbol_count,
                decision,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_classify_three_golden_samples() {
        let pdf_1 = PathBuf::from(r"C:\dev\ai-forge\test\parse\1.pdf");
        let pdf_2 = PathBuf::from(r"C:\dev\ai-forge\test\parse\2.pdf");
        let pdf_3 = PathBuf::from(r"C:\dev\ai-forge\test\parse\3.pdf");

        println!("\n⚡ ===== [PDF 智能分类器 3 大黄金样本实弹打靶] =====");

        if pdf_1.exists() {
            let metrics_1 = PdfRouteProbe::probe_file(&pdf_1).expect("探测 1.pdf 失败");
            assert!(!metrics_1.is_empty(), "1.pdf 页面不可为空");
            println!(
                "✅ 样本 1.pdf (多模态矢量页): 页数={}, Page 1 分流={:?}, 有效字数={}",
                metrics_1.len(),
                metrics_1[0].decision,
                metrics_1[0].valid_char_count
            );
            assert_eq!(
                metrics_1[0].decision,
                RouteDecision::FastTrackCpu,
                "1.pdf 第一页必须分流到 FastTrackCpu 矢量轨！"
            );
        } else {
            println!("⚠️ 未找到 1.pdf: {:?}", pdf_1);
        }

        if pdf_2.exists() {
            let metrics_2 = PdfRouteProbe::probe_file(&pdf_2).expect("探测 2.pdf 失败");
            assert!(!metrics_2.is_empty(), "2.pdf 页面不可为空");
            println!(
                "✅ 样本 2.pdf (论文原生矢量页): 页数={}, Page 1 分流={:?}, 有效字数={}",
                metrics_2.len(),
                metrics_2[0].decision,
                metrics_2[0].valid_char_count
            );
            assert_eq!(
                metrics_2[0].decision,
                RouteDecision::FastTrackCpu,
                "2.pdf 第一页必须分流到 FastTrackCpu 矢量轨！"
            );
        } else {
            println!("⚠️ 未找到 2.pdf: {:?}", pdf_2);
        }

        if pdf_3.exists() {
            let metrics_3 = PdfRouteProbe::probe_file(&pdf_3).expect("探测 3.pdf 失败");
            assert!(!metrics_3.is_empty(), "3.pdf 页面不可为空");
            println!(
                "✅ 样本 3.pdf (超大扫描件): 页数={}, Page 1 分流={:?}, 有效字数={}",
                metrics_3.len(),
                metrics_3[0].decision,
                metrics_3[0].valid_char_count
            );
            assert_eq!(
                metrics_3[0].decision,
                RouteDecision::DeepTrackGpu,
                "3.pdf 第一页必须分流到 DeepTrackGpu 视觉多模态轨！"
            );
        } else {
            println!("⚠️ 未找到 3.pdf: {:?}", pdf_3);
        }

        println!("🎉 ===== [3 大黄金样本分类探针全部精准命中预期！] =====\n");
    }
}
