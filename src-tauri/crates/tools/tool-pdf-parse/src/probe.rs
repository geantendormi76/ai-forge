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
    }

    fn count_math_indicators(text: &str) -> usize {
        let mut count = 0;
        let math_keywords = ["\\frac", "\\sqrt", "\\sum", "\\int", "\\alpha", "\\beta", "\\theta", "\\sigma", "\\pi", "\\times", "\\matrix"];
        for kw in &math_keywords {
            count += text.matches(kw).count() * 3;
        }
        for c in text.chars() {
            if matches!(c, '∫' | '∑' | '√' | 'α' | 'β' | 'θ' | 'π' | '±' | '≤' | '≥' | '≠' | '∞' | '^' | '_') {
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
                                                raw_text_buffer.push_str(&String::from_utf8_lossy(bytes));
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
            let valid_char_count = raw_text_buffer.chars().filter(|c| !c.is_control() && !c.is_whitespace()).count();
            let readable_chars = raw_text_buffer.chars().filter(|&c| Self::is_readable_char(c)).count();
            let readable_ratio = if text_len > 0 { readable_chars as f32 / text_len as f32 } else { 0.0 };
            let math_symbol_count = Self::count_math_indicators(&raw_text_buffer);

            // 🛡️ 校准探针规则：把含有复杂无边框 Table 1 的第 3 页 (vec_count > 30) 路由给拥有 SLANet_plus 的 GPU 视觉管线
            let is_valid_text = valid_char_count >= 50;
            let is_healthy_encoding = readable_ratio >= 0.85;
            let is_low_image_impact = img_count == 0;
            let is_low_math_density = valid_char_count == 0 || (math_symbol_count as f32 / valid_char_count as f32) < 0.10;
            let is_low_vector_complexity = vec_count <= 30;

            let decision = if is_valid_text && is_healthy_encoding && is_low_image_impact && is_low_math_density && is_low_vector_complexity {
                RouteDecision::FastTrackCpu
            } else {
                RouteDecision::DeepTrackGpu
            };

            info!(
                "📄 [PDF 5维路由探针] 页码 {}: 有效字数={}, 可读率={:.2}, 图片数={}, 矢量线段={}, 公式符={} ──► 分流: {:?}",
                page_num, valid_char_count, readable_ratio, img_count, vec_count, math_symbol_count, decision
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
