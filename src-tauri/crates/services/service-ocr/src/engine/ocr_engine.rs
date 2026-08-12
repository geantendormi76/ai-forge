use crate::core::errors::OCRError;
use crate::engine::crnn_rec_model::CRNNRecModel;
use crate::engine::db_det_model::DBDetModel;
use crate::models::text_region::TextRegion;
use crate::processors::crop_text_region;
use image::{DynamicImage, RgbImage};
use std::path::Path;

pub struct OcrEngine {
    det_model: DBDetModel,
    rec_model: CRNNRecModel,
}

impl OcrEngine {
    pub fn new(
        det_model_path: &Path,
        rec_model_path: &Path,
        dict_path: &Path,
    ) -> Result<Self, OCRError> {
        let det_model = DBDetModel::new(det_model_path, None)?;
        let rec_model = CRNNRecModel::new(rec_model_path, dict_path)?;

        Ok(Self {
            det_model,
            rec_model,
        })
    }

    pub fn process_image(&mut self, img: DynamicImage) -> Result<Vec<TextRegion>, OCRError> {
        let rgb_img = img.to_rgb8();
        let (boxes, _scores) = self.det_model.detect(DynamicImage::ImageRgb8(rgb_img.clone()))?;

        if boxes.is_empty() {
            return Ok(Vec::new());
        }

        let mut valid_crops: Vec<(usize, crate::models::text_region::BoundingBox, RgbImage)> =
            Vec::with_capacity(boxes.len());

        for (orig_idx, bbox) in boxes.into_iter().enumerate() {
            let crop = crop_text_region(&rgb_img, &bbox);
            if crop.width() >= 4 && crop.height() >= 4 {
                valid_crops.push((orig_idx, bbox, crop));
            }
        }

        if valid_crops.is_empty() {
            return Ok(Vec::new());
        }

        // 🛡️ SOTA 算法 1：按文字行物理宽度升序排序（Width Bucketing）
        valid_crops.sort_by_key(|(_, _, c)| c.width());

        // 🛡️ SOTA 算法 2：微批次分桶推导 (Micro-Batching, 6 行一组)，避免黑边 padding 引发 O(W^2) 计算量暴胀
        let chunk_size = 6;
        let mut indexed_results = Vec::with_capacity(valid_crops.len());

        for chunk in valid_crops.chunks(chunk_size) {
            let crops_chunk: Vec<RgbImage> = chunk.iter().map(|(_, _, c)| c.clone()).collect();
            let batch_res = self.rec_model.recognize_crops_batch(&crops_chunk)?;

            for (item, (text, score)) in chunk.iter().zip(batch_res.into_iter()) {
                indexed_results.push((item.0, item.1.clone(), text, score));
            }
        }

        // 🛡️ SOTA 算法 3：按原始检测框顺序还原
        indexed_results.sort_by_key(|(orig_idx, _, _, _)| *orig_idx);

        let mut regions = Vec::with_capacity(indexed_results.len());
        for (_, bbox, text, score) in indexed_results {
            let trimmed = text.trim();
            if !trimmed.is_empty() && score >= 0.3 {
                regions.push(TextRegion {
                    bbox,
                    text: trimmed.to_string(),
                    score,
                });
            }
        }

        Ok(regions)
    }
}
