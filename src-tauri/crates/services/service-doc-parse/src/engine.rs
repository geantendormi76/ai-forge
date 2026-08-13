use crate::markdown::render_gfm_markdown;
use crate::sorting::sort_doc_elements;
use crate::stitching::calculate_ioa;
use crate::types::{DocElement, LayoutType, ParsedDocument};
use image::DynamicImage;
use service_formula::FormulaService;
use service_layout::{LayoutCategory, LayoutService};
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

    pub fn parse_image(&mut self, img: &DynamicImage) -> Result<(ParsedDocument, String), String> {
        let rgb_img = img.to_rgb8();
        let (img_w, img_h) = (img.width() as f32, img.height() as f32);

        let layout_model_path = self.models_root.join(r"service-layout\PP-DocLayoutV3.onnx");
        let mut doc_elements = Vec::new();

        if layout_model_path.exists() {
            if let Ok(mut layout_svc) = LayoutService::new(&layout_model_path, None, None) {
                if let Ok(layout_res) = layout_svc.detect(&rgb_img) {
                    for region in layout_res.regions {
                        let l_type = match region.category {
                            LayoutCategory::Title | LayoutCategory::Section => LayoutType::DocTitle,
                            LayoutCategory::Abstract | LayoutCategory::Keyword | LayoutCategory::Author | LayoutCategory::Affiliation => LayoutType::ParagraphTitle,
                            LayoutCategory::Text | LayoutCategory::Catalog | LayoutCategory::Code | LayoutCategory::Reference => LayoutType::Text,
                            LayoutCategory::Table | LayoutCategory::TableCaption => LayoutType::Table,
                            LayoutCategory::Formula | LayoutCategory::EquationNumber => LayoutType::Formula,
                            LayoutCategory::Figure | LayoutCategory::FigureCaption | LayoutCategory::Caption => LayoutType::Image,
                            LayoutCategory::Header => LayoutType::Header,
                            LayoutCategory::Footer | LayoutCategory::Footnote => LayoutType::Footer,
                            LayoutCategory::List => LayoutType::List,
                            LayoutCategory::Seal => LayoutType::Seal,
                            LayoutCategory::Sidebar => LayoutType::AsideText,
                            _ => LayoutType::Text,
                        };

                        let bbox = [region.bbox.x1, region.bbox.y1, region.bbox.x2, region.bbox.y2];
                        doc_elements.push(DocElement {
                            bbox,
                            layout_type: l_type,
                            order_index: 0,
                            content: String::new(),
                            score: region.score,
                        });
                    }
                }
            }
        }

        if doc_elements.is_empty() {
            doc_elements.push(DocElement {
                bbox: [0.0, 0.0, img_w, img_h],
                layout_type: LayoutType::Text,
                order_index: 1,
                content: String::new(),
                score: 1.0,
            });
        }

        let table_model = self.models_root.join(r"service-table\SLANet_plus.onnx");
        let table_dict = self.models_root.join(r"service-table\table_structure_dict.txt");

        let ocr_lines = if let Ok(mut ocr_engine) = OcrService::default_engine() {
            ocr_engine.process_image(img.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        for elem in doc_elements.iter_mut() {
            let crop_bbox = elem.bbox;
            let crop_x = crop_bbox[0].max(0.0) as u32;
            let crop_y = crop_bbox[1].max(0.0) as u32;
            let crop_w = (crop_bbox[2] - crop_bbox[0]).max(1.0).min(img_w - crop_x as f32) as u32;
            let crop_h = (crop_bbox[3] - crop_bbox[1]).max(1.0).min(img_h - crop_y as f32) as u32;

            match elem.layout_type {
                LayoutType::Table => {
                    if table_model.exists() && table_dict.exists() && crop_w > 10 && crop_h > 10 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(table_res) = recognize_table_crop(&crop_img, &table_model, &table_dict, Some("cpu")) {
                            let mut cell_texts = Vec::new();
                            for cell in &table_res.cells {
                                let cell_page_bbox = [
                                    cell.bbox[0] + crop_x as f32,
                                    cell.bbox[1] + crop_y as f32,
                                    cell.bbox[2] + crop_x as f32,
                                    cell.bbox[5] + crop_y as f32,
                                ];

                                let mut matched_words = Vec::new();
                                for line in &ocr_lines {
                                    let (lx1, ly1, lx2, ly2) = line.bbox.aabb();
                                    let line_bbox = [lx1, ly1, lx2, ly2];
                                    if calculate_ioa(&line_bbox, &cell_page_bbox) > 0.3 {
                                        matched_words.push(line.text.as_str());
                                    }
                                }
                                if matched_words.is_empty() {
                                    cell_texts.push(None);
                                } else {
                                    cell_texts.push(Some(matched_words.join(" ")));
                                }
                            }

                            elem.content = service_table::wrap_table_html_with_content(&table_res.structure_tokens, &cell_texts);
                        }
                    }
                }
                LayoutType::Formula => {
                    if crop_w > 5 && crop_h > 5 {
                        let crop_img = image::imageops::crop_imm(&rgb_img, crop_x, crop_y, crop_w, crop_h).to_image();
                        if let Ok(f_res) = FormulaService::recognize_crop(&crop_img, None, None) {
                            elem.content = format!("$$\n{}\n$$", f_res.latex);
                        }
                    }
                }
                _ => {
                    let mut matched_lines = Vec::new();
                    for line in &ocr_lines {
                        let (lx1, ly1, lx2, ly2) = line.bbox.aabb();
                        let line_bbox = [lx1, ly1, lx2, ly2];
                        if calculate_ioa(&line_bbox, &elem.bbox) > 0.3 {
                            matched_lines.push(line.text.as_str());
                        }
                    }
                    elem.content = matched_lines.join("\n");
                }
            }
        }

        sort_doc_elements(&mut doc_elements, img_w, img_h);

        let parsed_doc = ParsedDocument {
            elements: doc_elements,
            page_width: img_w,
            page_height: img_h,
        };

        let markdown = render_gfm_markdown(&parsed_doc);
        Ok((parsed_doc, markdown))
    }
}
