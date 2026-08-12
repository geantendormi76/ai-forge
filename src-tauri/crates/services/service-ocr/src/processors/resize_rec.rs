use crate::core::OCRError;
use crate::processors::normalization::NormalizeImage;
use image::{imageops, DynamicImage, RgbImage};
use ndarray::Array4;

#[derive(Debug, Clone)]
pub struct OCRResize {
    pub target_height: u32,
    pub max_width: u32,
}

impl OCRResize {
    pub fn new(target_height: Option<u32>, max_width: Option<u32>) -> Self {
        Self {
            target_height: target_height.unwrap_or(48),
            max_width: max_width.unwrap_or(960),
        }
    }

    pub fn resize(&self, img: &RgbImage) -> Result<RgbImage, OCRError> {
        let (orig_w, orig_h) = img.dimensions();
        if orig_w == 0 || orig_h == 0 {
            return Err(OCRError::InvalidInput("Empty image for recognition resize".into()));
        }

        let ratio = orig_w as f32 / orig_h.max(1) as f32;
        let mut target_w = (self.target_height as f32 * ratio).ceil() as u32;
        target_w = target_w.clamp(1, self.max_width);

        let resized = imageops::resize(
            img,
            target_w,
            self.target_height,
            imageops::FilterType::Triangle,
        );

        let mut padded = RgbImage::from_pixel(target_w, self.target_height, image::Rgb([0, 0, 0]));
        imageops::overlay(&mut padded, &resized, 0, 0);

        Ok(padded)
    }

    pub fn resize_and_pack_batch_tensor(
        &self,
        crops: &[RgbImage],
        normalizer: &NormalizeImage,
    ) -> Result<Array4<f32>, OCRError> {
        if crops.is_empty() {
            return Ok(Array4::zeros((0, 3, self.target_height as usize, 32)));
        }

        let mut resized_items = Vec::with_capacity(crops.len());
        let mut max_batch_w = 32u32;

        for img in crops {
            let (orig_w, orig_h) = img.dimensions();
            if orig_w == 0 || orig_h == 0 {
                continue;
            }

            let ratio = orig_w as f32 / orig_h.max(1) as f32;
            let target_w = ((self.target_height as f32 * ratio).ceil() as u32).clamp(16, self.max_width);
            max_batch_w = max_batch_w.max(target_w);

            let resized = imageops::resize(
                img,
                target_w,
                self.target_height,
                imageops::FilterType::Triangle,
            );
            resized_items.push((resized, target_w));
        }

        if resized_items.is_empty() {
            return Ok(Array4::zeros((0, 3, self.target_height as usize, 32)));
        }

        let mut padded_batch = Vec::with_capacity(resized_items.len());
        for (resized_img, _w) in resized_items {
            let mut padded = RgbImage::from_pixel(max_batch_w, self.target_height, image::Rgb([0, 0, 0]));
            imageops::overlay(&mut padded, &resized_img, 0, 0);
            padded_batch.push(DynamicImage::ImageRgb8(padded));
        }

        normalizer.normalize_batch_to(padded_batch)
    }
}
