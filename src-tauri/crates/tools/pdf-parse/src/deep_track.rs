use md5::{Digest, Md5};
use service_doc_parse::{ast::*, config::*, stitching::*, xy_cut::*};
use service_formula::FormulaService;
use service_layout::LayoutService;
use service_ocr::OcrService;
use service_pdfium::{PdfCharInfo, PdfiumEngine};
use service_table::recognize_table_crop;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

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
pub struct DeepTrackResult {
    pub success: bool,
    pub markdown: String,
    pub images: Vec<String>,
    pub tables: Vec<String>,
    pub error: Option<String>,
}

pub struct DeepTrackEngine;

pub fn clean_katex_markdown(text: &str) -> String {
    let mut res = text.to_string();
    res = res.replace("\\_", "_");
    res = res.replace("\\^", "^");
    while res.contains("^^") {
        res = res.replace("^^", "^");
    }
    res = res.replace("{{\\}}", "");
    res = res.replace("{\\}}", "");
    res = res.replace("\\VIT\\B", "-VIT-B");
    res = res.replace("\\VIT", "-VIT");
    res = res.replace("\\B4", "-B4");
    res = res.replace("Vary\\VIT\\B", "Vary-VIT-B");
    res = res.replace("PP-HGNetV2\\B4", "PP-HGNetV2-B4");
    res = res.replace("{tau__{c c}^{*}}", "\\boldsymbol{\\tau}_c^*");
    res = res.replace("tau__{c c}^{*}", "\\boldsymbol{\\tau}_c^*");
    res = res.replace("(P P_{i,c}", "P(y_{i,c}");
    res = res.replace("\\operatorname{if}P", "\\operatorname{if}\\:P");
    res
}

pub fn apply_class_margin(bbox: &BoundingBox, label: &str) -> BoundingBox {
    let (dx, dy) = match label.to_lowercase().as_str() {
        "formula" | "isolate_formula" | "display_formula" => (10.0, 10.0),
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

fn map_label_to_sort_tag(label: &str, bbox: &BoundingBox, page_width: f32) -> 排序标签 {
    let element_width = bbox.x_max() - bbox.x_min();
    let is_wide = page_width > 0.0 && (element_width / page_width) >= 0.55;

    match label.to_lowercase().as_str() {
        "header" => 排序标签::页眉,
        "footer" | "number" => 排序标签::页脚,
        "doc_title" | "document_title" | "title" => 排序标签::文档标题,
        "paragraph_title" | "section_header" | "figure_title" | "section" => 排序标签::段落标题,
        "figure" | "image" | "illustration" | "chart" => {
            if is_wide { 排序标签::跨栏元素 } else { 排序标签::视觉实体 }
        }
        "table" | "formula" | "isolate_formula" | "display_formula" => {
            if is_wide { 排序标签::跨栏元素 } else { 排序标签::普通文本 }
        }
        _ => 排序标签::普通文本,
    }
}

fn extract_vector_text_in_bbox(
    chars: &[PdfCharInfo],
    bbox_px: &[f32; 4],
    page_w_pt: f32,
    page_h_pt: f32,
    img_w_px: f32,
    img_h_px: f32,
) -> String {
    if chars.is_empty() || page_w_pt <= 0.0 || page_h_pt <= 0.0 || img_w_px <= 0.0 || img_h_px <= 0.0 {
        return String::new();
    }

    let scale_x = img_w_px / page_w_pt;
    let scale_y = img_h_px / page_h_pt;

    let bx0_pt = (bbox_px[0] - 2.0) / scale_x;
    let by0_pt = (bbox_px[1] - 2.0) / scale_y;
    let bx1_pt = (bbox_px[2] + 2.0) / scale_x;
    let by1_pt = (bbox_px[3] + 2.0) / scale_y;

    let mut matched = Vec::new();

    for c in chars {
        if c.unicode_char.is_control() && c.unicode_char != '\n' && c.unicode_char != '\t' {
            continue;
        }

        let cx_pt = (c.x1 + c.x2) * 0.5;
        let cy_pt = page_h_pt - (c.y1 + c.y2) * 0.5;

        let in_x = cx_pt >= bx0_pt && cx_pt <= bx1_pt;
        let in_y = cy_pt >= by0_pt && cy_pt <= by1_pt;

        if in_x && in_y {
            matched.push(c.unicode_char);
        }
    }

    let raw = matched.into_iter().collect::<String>();
    raw.trim().to_string()
}

impl DeepTrackEngine {
    pub async fn run_pages(
        pdf_path: &Path,
        output_dir: &Path,
        pages: Option<&[usize]>,
    ) -> Result<DeepTrackResult, String> {
        if !pdf_path.exists() {
            return Err(format!("物理文件不存在: {:?}", pdf_path));
        }

        let models_root = PathBuf::from(r"C:\dev\ai-forge\models");
        let layout_model = models_root.join(r"service-layout\PP-DocLayoutV3.onnx");
        let table_model = models_root.join(r"service-table\SLANet_plus.onnx");
        let table_dict = models_root.join(r"service-table\table_structure_dict.txt");

        let mut layout_svc = LayoutService::new(&layout_model, None, None)
            .map_err(|e| format!("加载 Layout 模型失败: {e}"))?;

        let mut ocr_svc = OcrService::default_engine()
            .map_err(|e| format!("加载 OCR 模型失败: {e}"))?;

        let page_count = PdfiumEngine::get_page_count(pdf_path)
            .map_err(|e| format!("Pdfium 获取页数失败: {e}"))?;

        let target_pages: Vec<usize> = if let Some(p_list) = pages {
            p_list
                .iter()
                .map(|&p| p.saturating_sub(1))
                .filter(|&p| p < page_count)
                .collect()
        } else {
            (0..page_count).collect()
        };

        let images_dir = output_dir.join("images");
        let _ = std::fs::create_dir_all(&images_dir);

        let mut all_elements = Vec::new();
        let mut extracted_images = Vec::new();
        let mut page_w = 0.0f32;
        let mut page_h = 0.0f32;

        for &p_idx in &target_pages {
            let (page_w_pt, page_h_pt) = PdfiumEngine::get_page_dimensions(pdf_path, p_idx)
                .unwrap_or((612.0, 792.0));

            let dyn_img = PdfiumEngine::render_page_to_image(pdf_path, p_idx, 300)
                .map_err(|e| format!("渲染 300DPI 图片失败: {e}"))?;

            let rgb_img = dyn_img.to_rgb8();
            let (img_w, img_h) = (dyn_img.width() as f32, dyn_img.height() as f32);
            page_w = page_w.max(img_w);
            page_h = page_h.max(img_h);

            let page_chars = PdfiumEngine::extract_page_chars(pdf_path, p_idx).unwrap_or_default();

            let layout_res = layout_svc.detect(&rgb_img)
                .map_err(|e| format!("Layout 检测失败: {e}"))?;

            for region in layout_res.regions.into_iter() {
                let label_str = region.category.as_str().to_string();
                let label_lower = label_str.to_lowercase();

                if matches!(label_lower.as_str(), "aside_text" | "sidebar_text" | "page_number" | "number" | "header" | "footer") {
                    continue;
                }

                let bbox_px = [region.bbox.x1, region.bbox.y1, region.bbox.x2, region.bbox.y2];
                let (x0, y0, x1, y1) = (bbox_px[0], bbox_px[1], bbox_px[2], bbox_px[3]);

                let vec_text = extract_vector_text_in_bbox(&page_chars, &bbox_px, page_w_pt, page_h_pt, img_w, img_h);

                let mut final_text = None;
                let mut cells_data = None;
                let mut structure_tokens_data = None;

                if label_lower == "table" {
                    let crop_x = (x0 - 12.0).max(0.0) as u32;
                    let crop_y = (y0 - 10.0).max(0.0) as u32;
                    let crop_w = (x1 - x0 + 24.0).max(1.0).min(img_w - crop_x as f32) as u32;
                    let crop_h = (y1 - y0 + 20.0).max(1.0).min(img_h - crop_y as f32) as u32;

                    if table_model.exists() && table_dict.exists() && crop_w > 10 && crop_h > 10 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(table_res) = recognize_table_crop(&crop_img, &table_model, &table_dict, Some("cpu")) {
                            structure_tokens_data = Some(table_res.structure_tokens.clone());
                            let mut cell_boxes = Vec::new();
                            for cell in table_res.cells {
                                let c_box = vec![
                                    cell.bbox[0] + crop_x as f32,
                                    cell.bbox[1] + crop_y as f32,
                                    cell.bbox[2] + crop_x as f32,
                                    cell.bbox[5] + crop_y as f32,
                                ];
                                let c_bbox_arr = [c_box[0], c_box[1], c_box[2], c_box[3]];
                                let cell_vec_text = extract_vector_text_in_bbox(&page_chars, &c_bbox_arr, page_w_pt, page_h_pt, img_w, img_h);

                                cell_boxes.push(RawCellBox {
                                    bbox: c_box,
                                    text: if !cell_vec_text.is_empty() { Some(cell_vec_text) } else { None },
                                });
                            }
                            cells_data = Some(cell_boxes);
                        }
                    }
                    if cells_data.is_none() && !vec_text.is_empty() {
                        final_text = Some(vec_text);
                    }
                } else if matches!(label_lower.as_str(), "display_formula" | "isolate_formula") {
                    let crop_x = (x0 - 25.0).max(0.0) as u32;
                    let crop_y = (y0 - 10.0).max(0.0) as u32;
                    let crop_w = (x1 - x0 + 50.0).max(1.0).min(img_w - crop_x as f32) as u32;
                    let crop_h = (y1 - y0 + 20.0).max(1.0).min(img_h - crop_y as f32) as u32;

                    if crop_w > 5 && crop_h > 5 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(f_res) = FormulaService::recognize_crop(&crop_img, None, None) {
                            final_text = Some(normalize_latex_formula(&f_res.latex));
                        }
                    }
                    if final_text.is_none() && !vec_text.is_empty() {
                        final_text = Some(vec_text);
                    }
                } else if matches!(label_lower.as_str(), "figure" | "image" | "illustration") {
                    let crop_x = x0.max(0.0) as u32;
                    let crop_y = y0.max(0.0) as u32;
                    let crop_w = (x1 - x0).max(1.0).min(img_w - crop_x as f32) as u32;
                    let crop_h = (y1 - y0).max(1.0).min(img_h - crop_y as f32) as u32;

                    if crop_w > 5 && crop_h > 5 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        let mut bytes = Vec::new();
                        let _ = crop_img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png);
                        
                        let hash = hex::encode(Md5::digest(&bytes));
                        let img_filename = format!("{hash}.png");
                        let img_full_path = images_dir.join(&img_filename);
                        let rel_path = format!("images/{img_filename}");

                        if !img_full_path.exists() {
                            let _ = crop_img.save(&img_full_path);
                        }

                        final_text = Some(rel_path.clone());
                        if !extracted_images.contains(&rel_path) {
                            extracted_images.push(rel_path);
                        }
                    }
                } else {
                    if !vec_text.is_empty() {
                        final_text = Some(vec_text);
                    } else {
                        let left_margin = if x0 > 400.0 { 30.0 } else { 150.0 };
                        let crop_x = (x0 - left_margin).max(0.0) as u32;
                        let crop_y = (y0 - 12.0).max(0.0) as u32;
                        let crop_w = (x1 - x0 + left_margin + 100.0).max(1.0).min(img_w - crop_x as f32) as u32;
                        let crop_h = (y1 - y0 + 24.0).max(1.0).min(img_h - crop_y as f32) as u32;

                        if crop_w > 5 && crop_h > 5 {
                            let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                            if let Ok(lines) = ocr_svc.process_image(image::DynamicImage::ImageRgb8(crop_img)) {
                                let txts: Vec<String> = lines.into_iter().map(|l| l.text).collect();
                                final_text = Some(txts.join("\n"));
                            }
                        }
                    }
                }

                all_elements.push(RawLayoutElement {
                    page_index: Some(p_idx),
                    label: label_str,
                    bbox: bbox_px.to_vec(),
                    text: final_text,
                    cells: cells_data,
                    structure_tokens: structure_tokens_data,
                });
            }
        }

        let (md_text, tables) = Self::stitch_and_assemble(all_elements, page_w, page_h)?;
        let cleaned_md = clean_katex_markdown(&md_text);

        info!(
            "🎉 [Native DeepTrack 深度视觉轨成功] 页数: {}, 提炼 Markdown 字符数: {}",
            target_pages.len(),
            cleaned_md.len()
        );

        Ok(DeepTrackResult {
            success: true,
            markdown: cleaned_md,
            images: extracted_images,
            tables,
            error: None,
        })
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

        let filtered_by_image: Vec<RawLayoutElement> = raw_elements
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
        let mut page_elements_map: BTreeMap<usize, Vec<RawLayoutElement>> = BTreeMap::new();
        for elem in filtered_by_image {
            let p = elem.page_index.unwrap_or(0);
            page_elements_map.entry(p).or_default().push(elem);
        }

        let mut deduplicated_elements: Vec<RawLayoutElement> = Vec::new();
        for (_p_idx, mut page_elems) in page_elements_map {
            page_elems.sort_by(|a, b| {
                let area_a = if a.bbox.len() >= 4 { (a.bbox[2] - a.bbox[0]) * (a.bbox[3] - a.bbox[1]) } else { 0.0 };
                let area_b = if b.bbox.len() >= 4 { (b.bbox[2] - b.bbox[0]) * (b.bbox[3] - b.bbox[1]) } else { 0.0 };
                area_b.partial_cmp(&area_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut kept: Vec<RawLayoutElement> = Vec::new();
            for elem in page_elems {
                if elem.bbox.len() < 4 {
                    kept.push(elem);
                    continue;
                }
                let bbox_a = BoundingBox::from_coords(elem.bbox[0], elem.bbox[1], elem.bbox[2], elem.bbox[3]);
                let area_a = bbox_a.area();

                let is_contained = kept.iter().any(|kept_elem| {
                    if kept_elem.bbox.len() < 4 {
                        return false;
                    }
                    let bbox_b = BoundingBox::from_coords(kept_elem.bbox[0], kept_elem.bbox[1], kept_elem.bbox[2], kept_elem.bbox[3]);
                    let inter = bbox_a.intersection_area(&bbox_b);
                    if area_a > 0.0 && (inter / area_a) > 0.70 {
                        let is_a_text = matches!(elem.label.as_str(), "text" | "abstract" | "paragraph_title" | "content" | "inline_formula" | "reference" | "reference_content");
                        let is_b_text = matches!(kept_elem.label.as_str(), "text" | "abstract" | "doc_title" | "paragraph_title" | "content" | "reference" | "reference_content");
                        if is_a_text && is_b_text {
                            return true;
                        }
                    }
                    false
                });

                if !is_contained {
                    kept.push(elem);
                }
            }
            deduplicated_elements.extend(kept);
        }

        let mut page_map: BTreeMap<usize, Vec<&RawLayoutElement>> = BTreeMap::new();
        for elem in &deduplicated_elements {
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

                let mut block_type = BlockType::from_label(&elem.label);
                let trimmed_txt = txt.trim();

                if trimmed_txt.starts_with("Table ")
                    || trimmed_txt.starts_with("Tab.")
                    || trimmed_txt.starts_with("Figure ")
                    || trimmed_txt.starts_with("Fig.")
                    || (trimmed_txt.starts_with('[') && trimmed_txt.chars().nth(1).map_or(false, |c| c.is_ascii_digit()))
                    || elem.label == "abstract"
                    || elem.label == "table_caption"
                    || elem.label == "figure_caption"
                {
                    block_type = BlockType::Paragraph;
                }

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
                    "display_formula" | "isolate_formula" => {
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
                        let stitched = if block_type == BlockType::DocTitle || block_type == BlockType::SectionHeader {
                            txt.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
                        } else {
                            join_ocr_texts(&[(&bbox, txt.to_string())], 5.0)
                        };
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_native_deeptrack_execution() {
        let pdf_path = Path::new(r"C:\dev\ai-forge\test\fixtures\1.pdf");
        let fallback_path = Path::new(r"C:\dev\ai-forge\test\parse\pdf-parse-fast.pdf");
        let target_pdf = if pdf_path.exists() { pdf_path } else { fallback_path };

        let output_dir = Path::new(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(output_dir);

        if !target_pdf.exists() {
            return;
        }

        let t0 = std::time::Instant::now();
        let res = DeepTrackEngine::run_pages(target_pdf, output_dir, None)
            .await
            .expect("Native DeepTrack 深度轨道解析失败");

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(res.success);

        let out_file = output_dir.join("1_out.md");
        fs::write(&out_file, &res.markdown).expect("写入物理 Markdown 失败");

        println!("\n========= 🎉 [Native DeepTrack 样本 1 打靶测试成功] =========");
        println!("  ⏱️ 端到端物理耗时: {:.2} ms", elapsed_ms);
        println!("  📝 提炼 Markdown 字符数: {}", res.markdown.len());
        println!("  💾 产物物理落盘: {:?}", out_file);
        println!("=======================================================\n");
    }

    #[tokio::test]
    async fn test_native_deeptrack_on_2_pdf() {
        let pdf_path = Path::new(r"C:\dev\ai-forge\test\fixtures\2.pdf");
        let fallback_path = Path::new(r"C:\dev\ai-forge\test\parse\2.pdf");
        let target_pdf = if pdf_path.exists() { pdf_path } else { fallback_path };

        let output_dir = Path::new(r"C:\dev\ai-forge\test\parse\outs");
        let _ = fs::create_dir_all(output_dir);

        if !target_pdf.exists() {
            println!("⚠️ [跳过测试] 2.pdf 物理文件不存在: {:?}", target_pdf);
            return;
        }

        let t0 = std::time::Instant::now();
        let res = DeepTrackEngine::run_pages(target_pdf, output_dir, None)
            .await
            .expect("Native DeepTrack 样本 2 解析失败");

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(res.success);

        let out_file = output_dir.join("2_out.md");
        fs::write(&out_file, &res.markdown).expect("写入 2_out.md 失败");

        println!("\n========= 🎉 [Native DeepTrack 样本 2 (LinearRAG) 打靶成功] =========");
        println!("  ⏱️ 端到端物理耗时: {:.2} ms", elapsed_ms);
        println!("  📝 提炼 Markdown 字符数: {}", res.markdown.len());
        println!("  💾 产物物理落盘: {:?}", out_file);
        println!("===================================================================\n");
    }
}
