use crate::contracts::FormulaConfig;
use image::imageops::{overlay, resize, FilterType};
use image::{DynamicImage, RgbImage};
use ndarray::Array4;

/// PP-FormulaNet 公式图像预处理算子
pub struct FormulaPreprocessor {
    config: FormulaConfig,
}

impl FormulaPreprocessor {
    pub fn new(config: FormulaConfig) -> Self {
        Self { config }
    }

    /// 批量预处理，生成模型所需的 4D 张量 [Batch, 1, Height, Width]
    pub fn preprocess_batch(&self, images: &[RgbImage]) -> Result<Array4<f32>, String> {
        if images.is_empty() {
            return Err("输入的图像列表为空！".to_string());
        }

        let mut normalized_grays = Vec::with_capacity(images.len());

        for img in images {
            let cropped = self.crop_margin(img);
            let resized = self.resize_and_pad(&cropped);
            let gray = self.normalize_and_to_grayscale(&resized);
            normalized_grays.push(gray);
        }

        self.format_to_tensor(normalized_grays)
    }

    /// 去除背景空白边缘：二值化前景探测与边界裁切
    fn crop_margin(&self, img: &RgbImage) -> RgbImage {
        let gray = DynamicImage::ImageRgb8(img.clone()).to_luma8();
        let (width, height) = gray.dimensions();

        let mut min_val = u8::MAX;
        let mut max_val = u8::MIN;
        for pixel in gray.pixels() {
            let val = pixel[0];
            min_val = min_val.min(val);
            max_val = max_val.max(val);
        }

        // 若图像颜色单一，原样返回
        if max_val == min_val {
            return img.clone();
        }

        let range = (max_val - min_val) as f32;
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;

        for (x, y, pixel) in gray.enumerate_pixels() {
            let normalized = ((pixel[0] as f32 - min_val as f32) / range * 255.0) as u8;
            if normalized < self.config.crop_threshold {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        if min_x >= max_x || min_y >= max_y {
            return img.clone();
        }

        let crop_w = max_x - min_x + 1;
        let crop_h = max_y - min_y + 1;
        image::imageops::crop_imm(img, min_x, min_y, crop_w, crop_h).to_image()
    }

    /// 等比例缩放并居中贴合并黑底填充
    fn resize_and_pad(&self, img: &RgbImage) -> RgbImage {
        let (target_width, target_height) = self.config.target_size;
        let (img_width, img_height) = img.dimensions();

        if img_width == 0 || img_height == 0 {
            return RgbImage::new(target_width, target_height);
        }

        let min_size = target_width.min(target_height);
        let scale = (min_size as f32) / (img_width.max(img_height) as f32);
        let new_width = (img_width as f32 * scale) as u32;
        let new_height = (img_height as f32 * scale) as u32;

        let final_width = new_width.min(target_width);
        let final_height = new_height.min(target_height);

        let resized = resize(img, final_width, final_height, FilterType::Triangle);

        let delta_width = target_width - final_width;
        let delta_height = target_height - final_height;
        let pad_left = delta_width / 2;
        let pad_top = delta_height / 2;

        let mut padded = RgbImage::from_pixel(target_width, target_height, image::Rgb([0, 0, 0]));
        overlay(&mut padded, &resized, pad_left as i64, pad_top as i64);

        padded
    }

    /// 归一化并转为单通道灰度矩阵 (UniMERNet/PP-FormulaNet 标准)
    fn normalize_and_to_grayscale(&self, img: &RgbImage) -> ndarray::Array2<f32> {
        let (width, height) = img.dimensions();

        const SCALE: f32 = 1.0 / 255.0;
        let mean = [0.7931f32, 0.7931f32, 0.7931f32];
        let std = [0.1738f32, 0.1738f32, 0.1738f32];

        let mut grayscale = ndarray::Array2::<f32>::zeros((height as usize, width as usize));

        for (x, y, pixel) in img.enumerate_pixels() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // OpenCV BGR 顺序归一化
            let norm_b = (b * SCALE - mean[0]) / std[0];
            let norm_g = (g * SCALE - mean[1]) / std[1];
            let norm_r = (r * SCALE - mean[2]) / std[2];

            // 标准亮度转换
            let y_val = 0.114 * norm_b + 0.587 * norm_g + 0.299 * norm_r;
            grayscale[[y as usize, x as usize]] = y_val;
        }

        grayscale
    }

    /// 组装为对齐尺寸的 4D 张量 [batch, 1, padded_height, padded_width]
    fn format_to_tensor(&self, grays: Vec<ndarray::Array2<f32>>) -> Result<Array4<f32>, String> {
        let (target_width, target_height) = self.config.target_size;
        let batch_size = grays.len();

        let multiple = self.config.padding_multiple as f32;
        let padded_height = ((target_height as f32 / multiple).ceil() * multiple) as usize;
        let padded_width = ((target_width as f32 / multiple).ceil() * multiple) as usize;

        let mut tensor = Array4::<f32>::from_elem((batch_size, 1, padded_height, padded_width), 1.0f32);

        for (batch_idx, gray) in grays.iter().enumerate() {
            for y in 0..target_height as usize {
                for x in 0..target_width as usize {
                    tensor[[batch_idx, 0, y, x]] = gray[[y, x]];
                }
            }
        }

        Ok(tensor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn test_formula_preprocessor_shape() {
        let config = FormulaConfig::default();
        let processor = FormulaPreprocessor::new(config);

        let img = RgbImage::from_pixel(200, 100, Rgb([255, 255, 255]));
        let tensor = processor.preprocess_batch(&[img]).expect("预处理必须成功");

        assert_eq!(tensor.shape(), &[1, 1, 384, 384]);
    }
}
