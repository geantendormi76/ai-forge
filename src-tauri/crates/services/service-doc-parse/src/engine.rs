use crate::ast::*;
use crate::config::*;
use crate::stitching::normalize_latex_formula;
use crate::xy_cut::sort_by_xycut_enhanced;
use image::DynamicImage;
use service_formula::FormulaService;
use service_layout::LayoutService;
use service_ocr::OcrService;
use service_table::recognize_table_crop;
use std::path::PathBuf;

pub struct DocParseEngine {
    models_root: PathBuf,
}

impl DocParseEngine {
    pub fn new(models_root: impl Into<PathBuf>) -> Self {
        Self {
            models_root: models_root.into(),
        }
    }

    pub fn default_engine() -> Self {
        Self::new(r"C:\dev\ai-forge\models")
    }

    pub fn parse_image_ast(&mut self, img: &DynamicImage, page_index: usize) -> Result<(DocumentNode, String), String> {
        let rgb_img = img.to_rgb8();
        let (img_w, img_h) = (img.width() as f32, img.height() as f32);

        let layout_model_path = self.models_root.join(r"service-layout\PP-DocLayoutV3.onnx");
        let mut raw_regions = Vec::new();

        if layout_model_path.exists() {
            if let Ok(mut layout_svc) = LayoutService::new(&layout_model_path, None, None) {
                if let Ok(layout_res) = layout_svc.detect(&rgb_img) {
                    for region in layout_res.regions {
                        let label_str = region.category.as_str();
                        let bbox = BoundingBox::from_coords(region.bbox.x1, region.bbox.y1, region.bbox.x2, region.bbox.y2);
                        raw_regions.push((label_str.to_string(), bbox, region.score));
                    }
                }
            }
        }

        if raw_regions.is_empty() {
            raw_regions.push(("text".to_string(), BoundingBox::from_coords(0.0, 0.0, img_w, img_h), 1.0));
        }

        let table_model = self.models_root.join(r"service-table\SLANet_plus.onnx");
        let table_dict = self.models_root.join(r"service-table\table_structure_dict.txt");

        let ocr_lines = if let Ok(mut ocr_engine) = OcrService::default_engine() {
            ocr_engine.process_image(img.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        let enhanced_elements: Vec<增强版面元素> = raw_regions
            .iter()
            .map(|(label, bbox, _)| {
                let sort_tag = match label.to_lowercase().as_str() {
                    "header" => 排序标签::页眉,
                    "footer" | "number" => 排序标签::页脚,
                    "doc_title" | "document_title" | "title" => 排序标签::文档标题,
                    "paragraph_title" | "section_header" | "figure_title" | "section" | "abstract" | "reference" | "table_caption" | "figure_caption" => 排序标签::段落标题,
                    "figure" | "image" | "illustration" | "chart" => 排序标签::视觉实体,
                    "table" | "formula" | "isolate_formula" | "display_formula" => 排序标签::跨栏元素,
                    _ => 排序标签::普通文本,
                };
                增强版面元素 {
                    物理边界: bbox.clone(),
                    元素类型: sort_tag,
                    估算行数: Some(1),
                }
            })
            .collect();

        let sorted_indices = sort_by_xycut_enhanced(&enhanced_elements, img_w, img_h);

        let mut page_node = PageNode::new(page_index, img_w, img_h);
        let mut col_group = ColumnGroupNode::new(ColumnLayoutType::SingleColumn);

        for (rank, &idx) in sorted_indices.iter().enumerate() {
            if idx >= raw_regions.len() {
                continue;
            }

            let (label, bbox, _score) = &raw_regions[idx];
            let crop_x = bbox.x_min().max(0.0) as u32;
            let crop_y = bbox.y_min().max(0.0) as u32;
            let crop_w = (bbox.x_max() - bbox.x_min()).max(1.0).min(img_w - crop_x as f32) as u32;
            let crop_h = (bbox.y_max() - bbox.y_min()).max(1.0).min(img_h - crop_y as f32) as u32;

            let block_type = BlockType::from_label(label);
            let block_id = format!("p{}_b{}", page_index, rank);

            let content = match label.to_lowercase().as_str() {
                "table" => {
                    let mut struct_toks = Vec::new();
                    let mut cells_nodes = Vec::new();

                    if table_model.exists() && table_dict.exists() && crop_w > 10 && crop_h > 10 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(table_res) = recognize_table_crop(&crop_img, &table_model, &table_dict, Some("cpu")) {
                            struct_toks = table_res.structure_tokens.clone();
                            for (c_idx, cell) in table_res.cells.iter().enumerate() {
                                let c_box = BoundingBox::from_coords(
                                    cell.bbox[0] + crop_x as f32,
                                    cell.bbox[1] + crop_y as f32,
                                    cell.bbox[2] + crop_x as f32,
                                    cell.bbox[5] + crop_y as f32,
                                );
                                let mut matched_words = Vec::new();
                                for line in &ocr_lines {
                                    let (lx1, ly1, lx2, ly2) = line.bbox.aabb();
                                    let line_bbox = [lx1, ly1, lx2, ly2];
                                    let cell_bbox_arr = [c_box.x_min(), c_box.y_min(), c_box.x_max(), c_box.y_max()];
                                    if crate::stitching::calculate_ioa(&line_bbox, &cell_bbox_arr) > 0.3 {
                                        matched_words.push(line.text.as_str());
                                    }
                                }
                                cells_nodes.push(CellNode {
                                    bbox: c_box,
                                    row: c_idx,
                                    col: c_idx,
                                    row_span: 1,
                                    col_span: 1,
                                    text: matched_words.join(" "),
                                });
                            }
                        }
                    }

                    BlockContent::Table {
                        structure_tokens: struct_toks,
                        cells: cells_nodes,
                        is_cross_page: false,
                    }
                }
                "formula" | "isolate_formula" | "display_formula" | "inline_formula" | "math" => {
                    let mut latex_text = String::new();
                    if crop_w > 5 && crop_h > 5 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(f_res) = FormulaService::recognize_crop(&crop_img, None, None) {
                            latex_text = f_res.latex;
                        }
                    }
                    BlockContent::Formula {
                        latex: normalize_latex_formula(&latex_text),
                        is_display: true,
                    }
                }
                "figure" | "image" | "illustration" => {
                    BlockContent::Figure {
                        image_path: format!("images/crop_p{}_{}.png", page_index, rank),
                        caption_text: None,
                    }
                }
                _ => {
                    let mut matched_lines = Vec::new();
                    let elem_bbox_arr = [bbox.x_min(), bbox.y_min(), bbox.x_max(), bbox.y_max()];
                    for line in &ocr_lines {
                        let (lx1, ly1, lx2, ly2) = line.bbox.aabb();
                        let line_bbox = [lx1, ly1, lx2, ly2];
                        if crate::stitching::calculate_ioa(&line_bbox, &elem_bbox_arr) > 0.3 {
                            matched_lines.push(line.text.as_str());
                        }
                    }
                    let full_text = matched_lines.join("\n");
                    BlockContent::Text(vec![InlineNode::TextSpan {
                        text: full_text,
                        is_bold: false,
                        is_italic: false,
                    }])
                }
            };

            let block_node = BlockNode::new(block_id, block_type, bbox.clone(), content);
            col_group.blocks.push(block_node);
        }

        page_node.column_groups.push(col_group);

        let mut doc_ast = DocumentNode::new();
        doc_ast.add_page(page_node);

        let markdown = doc_ast.render_to_markdown();
        Ok((doc_ast, markdown))
    }
}
