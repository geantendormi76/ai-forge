use crate::core::errors::OCRError;
use crate::core::inference::OrtInfer;
use crate::processors::{CTCLabelDecode, ColorOrder, NormalizeImage, OCRResize, TensorLayout};
use crate::utils::read_character_dict;
use image::{DynamicImage, RgbImage};
use std::path::Path;

pub struct CRNNRecModel {
    infer: OrtInfer,
    resizer: OCRResize,
    normalizer: NormalizeImage,
    decoder: CTCLabelDecode,
}

impl CRNNRecModel {
    pub fn new(model_path: &Path, dict_path: &Path) -> Result<Self, OCRError> {
        let infer = OrtInfer::new(model_path, None)?;
        let resizer = OCRResize::new(Some(48), Some(960));
        let normalizer = NormalizeImage::with_color_order(
            Some(1.0 / 255.0),
            Some(vec![0.5, 0.5, 0.5]),
            Some(vec![0.5, 0.5, 0.5]),
            Some(TensorLayout::CHW),
            Some(ColorOrder::BGR),
        )?;

        let dict = read_character_dict(dict_path)?;
        let decoder = CTCLabelDecode::new(Some(&dict), true);

        Ok(Self {
            infer,
            resizer,
            normalizer,
            decoder,
        })
    }

    pub fn recognize_crop(&mut self, crop_img: &RgbImage) -> Result<(String, f32), OCRError> {
        let resized = self.resizer.resize(crop_img)?;
        let batch_tensor = self.normalizer.normalize_to(DynamicImage::ImageRgb8(resized))?;
        let preds = self.infer.infer_array3(&batch_tensor)?;
        let (texts, scores) = self.decoder.apply(&preds);

        let text = texts.into_iter().next().unwrap_or_default();
        let score = scores.into_iter().next().unwrap_or(0.0);

        Ok((text, score))
    }

    pub fn recognize_crops_batch(&mut self, crops: &[RgbImage]) -> Result<Vec<(String, f32)>, OCRError> {
        if crops.is_empty() {
            return Ok(Vec::new());
        }

        let batch_tensor = self.resizer.resize_and_pack_batch_tensor(crops, &self.normalizer)?;
        let preds = self.infer.infer_array3(&batch_tensor)?;
        let (texts, scores) = self.decoder.apply(&preds);

        let mut results = Vec::with_capacity(texts.len());
        for (t, s) in texts.into_iter().zip(scores.into_iter()) {
            results.push((t, s));
        }

        Ok(results)
    }
}
