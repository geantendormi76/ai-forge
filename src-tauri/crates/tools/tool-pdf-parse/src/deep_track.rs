use std::path::{Path, PathBuf};
use core_ipc::{spawn_uv_worker, IpcMessage};
use crate::idp::ast::*;
use crate::idp::config::{BoundingBox, 增强版面元素, 排序标签};
use crate::idp::xy_cut::sort_by_xycut_enhanced;
use crate::idp::stitch::{join_ocr_texts, normalize_latex_formula, is_within_caption_window, compute_knn_caption_distance};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawCellBox {
    pub bbox: Vec<f32>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLayoutElement {
    pub page_index: Option<usize>,
    pub label: String,
    pub bbox: Vec<f32>,
    pub text: Option<String>,
    pub cells: Option<Vec<RawCellBox>>,
    pub structure_tokens: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepTrackWorkerOutput {
    pub success: bool,
    pub page_width: f32,
    pub page_height: f32,
    pub elements: Vec<RawLayoutElement>,
    pub images: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepTrackResult {
    pub markdown: String,
    pub tables: Vec<String>,
}

pub struct DeepTrackEngine;

pub fn apply_class_margin(bbox: &BoundingBox, label: &str) -> BoundingBox {
    let (dx, dy) = match label.to_lowercase().as_str() {
        "formula" | "isolate_formula" | "display_formula" | "inline_formula" | "math" => (10.0, 10.0),
        "table" => (8.0, 8.0),
        _ => (4.0, 4.0),
    };
    BoundingBox::from_coords(
        (bbox.x_min() - dx).max(0.0),
        (bbox.y_min() - dy).max(0.0),
        bbox.x_max() + dx,
        bbox.y_max() + dy,
    )
}

pub fn compute_max_capacity(width: f32, height: f32) -> usize {
    let area = (width * height).max(0.0);
    ((0.0175 * area) as usize) + 64
}

fn map_label_to_sort_tag(label: &str, bbox: &BoundingBox, page_width: f32) -> 排序标签 {
    let element_width = bbox.x_max() - bbox.x_min();
    let is_wide = page_width > 0.0 && (element_width / page_width) >= 0.55;

    match label.to_lowercase().as_str() {
        "header" => 排序标签::页眉,
        "footer" | "number" => 排序标签::页脚,
        "doc_title" | "document_title" | "title" => 排序标签::文档标题,
        "paragraph_title" | "section_header" | "figure_title" => 排序标签::段落标题,
        "figure" | "image" | "illustration" | "chart" => {
            if is_wide { 排序标签::跨栏元素 } else { 排序标签::视觉实体 }
        }
        "table" | "formula" | "isolate_formula" | "display_formula" => {
            if is_wide { 排序标签::跨栏元素 } else { 排序标签::普通文本 }
        }
        _ => 排序标签::普通文本,
    }
}

impl DeepTrackEngine {
    fn resolve_project_dir() -> PathBuf {
        let candidate = PathBuf::from("/home/zhz/ai-forge/src-tauri/crates/tools/tool-pdf-parse");
        if candidate.exists() {
            return candidate;
        }
        if let Ok(manifestdir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifestdir);
        }
        PathBuf::from("src-tauri/crates/tools/tool-pdf-parse")
    }

    async fn execute_ipc_worker(
        input_path: &Path,
        output_dir: &Path,
        pages_str: &str,
    ) -> Result<DeepTrackWorkerOutput, String> {
        let project_dir = Self::resolve_project_dir();
        let script_path = project_dir.join("scripts/deeptrack_worker.py");

        let (mut child, mut channel) = spawn_uv_worker(&project_dir, &script_path)
            .map_err(|e| format!("启动 DeepTrack Worker 失败: {e}"))?;

        let ready_msg = channel.recv().await
            .map_err(|e| format!("接收 DeepTrack Worker 就绪信号失败: {e}"))?;
            
        if ready_msg.method != "system.ready" {
            return Err(format!("Worker 响应异常，未按预期就绪: {}", ready_msg.method));
        }

        let req_payload = serde_json::json!({
            "input_path": input_path.to_string_lossy(),
            "output_dir": output_dir.to_string_lossy(),
            "pages_str": pages_str
        });

        let parse_req = IpcMessage {
            method: "parse_pdf".to_string(),
            params: req_payload,
        };

        channel.send(&parse_req).await
            .map_err(|e| format!("发送 DeepTrack 解析请求失败: {e}"))?;

        let res_msg = channel.recv().await
            .map_err(|e| format!("读取 DeepTrack 解析结果失败: {e}"))?;

        let exit_msg = IpcMessage {
            method: "exit".to_string(),
            params: serde_json::json!({}),
        };
        let _ = channel.send(&exit_msg).await;
        let _ = child.wait().await;

        if res_msg.method == "error" {
            let err_text = res_msg.params["message"].as_str().unwrap_or("未知算法推理异常");
            return Err(format!("DeepTrack 视觉算法推理失败: {err_text}"));
        }

        let raw_res: DeepTrackWorkerOutput = serde_json::from_value(res_msg.params)
            .map_err(|e| format!("解析 DeepTrackWorkerOutput 反序列化失败: {e}"))?;

        if !raw_res.success {
            return Err(raw_res.error.unwrap_or_else(|| "DeepTrack Worker 执行失败".into()));
        }

        Ok(raw_res)
    }

    pub async fn run_pages(
        input_path: &Path,
        output_dir: &Path,
        pages: Option<&[usize]>,
    ) -> Result<(String, Vec<String>), String> {
        if !input_path.exists() {
            return Err(format!("输入物理文件不存在: {:?}", input_path));
        }

        let pages_str = pages
            .map(|p| p.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","))
            .unwrap_or_default();

        let raw_res = Self::execute_ipc_worker(input_path, output_dir, &pages_str).await?;

        Self::stitch_and_assemble(raw_res.elements, raw_res.page_width, raw_res.page_height)
    }

    pub fn stitch_and_assemble(
        raw_elements: Vec<RawLayoutElement>,
        page_width: f32,
        page_height: f32,
    ) -> Result<(String, Vec<String>), String> {
        if raw_elements.is_empty() {
            return Ok((String::new(), Vec::new()));
        }

        let mut image_boxes: Vec<(usize, BoundingBox)> = Vec::new();
        for elem in &raw_elements {
            if matches!(elem.label.as_str(), "figure" | "image" | "illustration") {
                if elem.bbox.len() >= 4 {
                    let p_idx = elem.page_index.unwrap_or(0);
                    let raw_box = BoundingBox::from_coords(elem.bbox[0], elem.bbox[1], elem.bbox[2], elem.bbox[3]);
                    let padded_box = apply_class_margin(&raw_box, &elem.label);
                    image_boxes.push((p_idx, padded_box));
                }
            }
        }

        let filtered_elements: Vec<RawLayoutElement> = raw_elements
            .into_iter()
            .filter(|e| {
                if matches!(e.label.as_str(), "figure" | "image" | "illustration") {
                    return true;
                }
                if e.bbox.len() >= 4 {
                    let elem_page = e.page_index.unwrap_or(0);
                    let text_box = BoundingBox::from_coords(e.bbox[0], e.bbox[1], e.bbox[2], e.bbox[3]);

                    for (img_page, img_box) in &image_boxes {
                        if *img_page == elem_page {
                            let inter_area = text_box.intersection_area(img_box);
                            let text_area = text_box.area();
                            if text_area > 0.0 && (inter_area / text_area) > 0.6 {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .collect();

        use std::collections::BTreeMap;
        let mut page_map: BTreeMap<usize, Vec<&RawLayoutElement>> = BTreeMap::new();
        for elem in &filtered_elements {
            let p_idx = elem.page_index.unwrap_or(0);
            page_map.entry(p_idx).or_default().push(elem);
        }

        let mut doc_ast = DocumentNode::new();
        let mut extracted_tables = Vec::new();

        for (p_idx, page_elements) in page_map {
            let mut page_node = PageNode::new(p_idx, page_width, page_height);
            let mut col_group = ColumnGroupNode::new(ColumnLayoutType::SingleColumn);

            let mut page_fig_candidates: Vec<(usize, BoundingBox)> = Vec::new();
            let mut page_caption_candidates: Vec<(usize, BoundingBox, String)> = Vec::new();

            for (e_idx, e) in page_elements.iter().enumerate() {
                if e.bbox.len() >= 4 {
                    let bbox = BoundingBox::from_coords(e.bbox[0], e.bbox[1], e.bbox[2], e.bbox[3]);
                    let label = e.label.to_lowercase();
                    let txt = e.text.as_deref().unwrap_or_default().trim();

                    if matches!(label.as_str(), "figure" | "image" | "illustration") {
                        page_fig_candidates.push((e_idx, bbox));
                    } else if label == "figure_title" || txt.starts_with("Figure ") || txt.starts_with("Fig.") {
                        if !txt.is_empty() {
                            page_caption_candidates.push((e_idx, bbox, txt.to_string()));
                        }
                    }
                }
            }

            use std::collections::HashMap;
            let mut fig_to_caption_map: HashMap<usize, String> = HashMap::new();
            let mut bound_captions_set: std::collections::HashSet<usize> = std::collections::HashSet::new();

            for (fig_idx, fig_box) in &page_fig_candidates {
                let mut best_cap_idx = None;
                let mut min_dist = f32::MAX;

                for (cap_idx, cap_box, cap_txt) in &page_caption_candidates {
                    if bound_captions_set.contains(cap_idx) {
                        continue;
                    }
                    if is_within_caption_window(fig_box, cap_box) {
                        let dist = compute_knn_caption_distance(fig_box, cap_box);
                        if dist < min_dist {
                            min_dist = dist;
                            best_cap_idx = Some((*cap_idx, cap_txt.clone()));
                        }
                    }
                }

                if let Some((cap_idx, cap_txt)) = best_cap_idx {
                    fig_to_caption_map.insert(*fig_idx, cap_txt);
                    bound_captions_set.insert(cap_idx);
                }
            }

            let enhanced_elements: Vec<增强版面元素> = page_elements
                .iter()
                .map(|e| {
                    let bbox = if e.bbox.len() >= 4 {
                        let raw_box = BoundingBox::from_coords(e.bbox[0], e.bbox[1], e.bbox[2], e.bbox[3]);
                        apply_class_margin(&raw_box, &e.label)
                    } else {
                        BoundingBox::from_coords(0.0, 0.0, 0.0, 0.0)
                    };
                    增强版面元素 {
                        物理边界: bbox.clone(),
                        元素类型: map_label_to_sort_tag(&e.label, &bbox, page_width),
                        估算行数: Some(1),
                    }
                })
                .collect();

            let sorted_indices = sort_by_xycut_enhanced(&enhanced_elements, page_width, page_height);

            for (elem_rank, &idx) in sorted_indices.iter().enumerate() {
                if idx >= page_elements.len() {
                    continue;
                }

                if bound_captions_set.contains(&idx) {
                    continue;
                }

                let elem = page_elements[idx];
                let txt = elem.text.as_deref().unwrap_or_default();
                let bbox = if elem.bbox.len() >= 4 {
                    BoundingBox::from_coords(elem.bbox[0], elem.bbox[1], elem.bbox[2], elem.bbox[3])
                } else {
                    BoundingBox::from_coords(0.0, 0.0, 0.0, 0.0)
                };

                let block_type = BlockType::from_label(&elem.label);
                let block_id = format!("p{}_b{}", p_idx, elem_rank);

                let content = match elem.label.as_str() {
                    "table" => {
                        let mut cells_nodes = Vec::new();
                        let mut struct_toks = Vec::new();
                        if let (Some(tokens), Some(cells)) = (&elem.structure_tokens, &elem.cells) {
                            struct_toks = tokens.clone();
                            for (c_idx, c) in cells.iter().enumerate() {
                                let c_box = if c.bbox.len() >= 4 {
                                    BoundingBox::from_coords(c.bbox[0], c.bbox[1], c.bbox[2], c.bbox[3])
                                } else {
                                    BoundingBox::from_coords(0.0, 0.0, 0.0, 0.0)
                                };
                                cells_nodes.push(CellNode {
                                    bbox: c_box,
                                    row: c_idx,
                                    col: c_idx,
                                    row_span: 1,
                                    col_span: 1,
                                    text: c.text.clone().unwrap_or_default(),
                                });
                            }
                        }
                        BlockContent::Table {
                            structure_tokens: struct_toks,
                            cells: cells_nodes,
                            is_cross_page: false,
                        }
                    }
                    "formula" | "isolate_formula" | "display_formula" | "inline_formula" | "math" => {
                        let clean_latex = normalize_latex_formula(txt);
                        BlockContent::Formula {
                            latex: clean_latex,
                            is_display: true,
                        }
                    }
                    "figure" | "image" | "illustration" => {
                        let cap_text = fig_to_caption_map.get(&idx).cloned();
                        BlockContent::Figure {
                            image_path: txt.to_string(),
                            caption_text: cap_text,
                        }
                    }
                    _ => {
                        let stitched = join_ocr_texts(&[(&bbox, txt.to_string())], 5.0);
                        BlockContent::Text(vec![InlineNode::TextSpan {
                            text: stitched,
                            is_bold: false,
                            is_italic: false,
                        }])
                    }
                };

                let block_node = BlockNode::new(block_id, block_type, bbox, content);
                col_group.blocks.push(block_node);
            }

            page_node.column_groups.push(col_group);
            doc_ast.add_page(page_node);
        }

        let rendered_markdown = doc_ast.render_to_markdown();

        for page in &doc_ast.pages {
            for col in &page.column_groups {
                for b in &col.blocks {
                    if let BlockContent::Table { .. } = &b.content {
                        let rendered_table = b.render_markdown();
                        if !rendered_table.trim().is_empty() {
                            extracted_tables.push(rendered_table);
                        }
                    }
                }
            }
        }

        Ok((rendered_markdown, extracted_tables))
    }
}
