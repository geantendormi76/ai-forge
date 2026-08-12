use crate::core::errors::OCRError;
use crate::core::inference::OrtInfer;
use crate::models::text_region::BoundingBox;
use crate::processors::{ColorOrder, DBPostProcess, DBPostProcessConfig, DetResizeForTest, NormalizeImage, TensorLayout};
use image::DynamicImage;
use std::path::Path;

pub struct DBDetModel {
    infer: OrtInfer,
    resizer: DetResizeForTest,
    normalizer: NormalizeImage,
    postprocess: DBPostProcess,
    config: Option<DBPostProcessConfig>,
}

impl DBDetModel {
    pub fn new(model_path: &Path, config: Option<DBPostProcessConfig>) -> Result<Self, OCRError> {
        let infer = OrtInfer::new(model_path, None)?;
        let resizer = DetResizeForTest::new(None, None, None, None, None, None, None);
        let normalizer = NormalizeImage::with_color_order(
            Some(1.0 / 255.0),
            Some(vec![0.485, 0.456, 0.406]),
            Some(vec![0.229, 0.224, 0.225]),
            Some(TensorLayout::CHW),
            Some(ColorOrder::BGR),
        )?;
        let postprocess = DBPostProcess::new(None, None, None, None, None, None, None);

        Ok(Self {
            infer,
            resizer,
            normalizer,
            postprocess,
            config,
        })
    }

    pub fn detect(&mut self, img: DynamicImage) -> Result<(Vec<BoundingBox>, Vec<f32>), OCRError> {
        let (resized_imgs, scale_infos) = self.resizer.apply(vec![img], None, None, None);
        let batch_tensor = self.normalizer.normalize_batch_to(resized_imgs)?;
        let preds = self.infer.infer_array4(&batch_tensor)?;
        let (boxes_batch, scores_batch) = self.postprocess.apply(&preds, scale_infos, self.config.as_ref());

        let boxes = boxes_batch.into_iter().next().unwrap_or_default();
        let scores = scores_batch.into_iter().next().unwrap_or_default();

        Ok((boxes, scores))
    }
}
