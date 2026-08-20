use crate::contracts::FormulaConfig;
use image::imageops::{overlay, resize, FilterType};
use image::{DynamicImage, RgbImage};
use ndarray::Array4;

pub struct FormulaPreprocessor {
    config: FormulaConfig,
}

impl FormulaPreprocessor {
    pub fn new(config: FormulaConfig) -> Self {
        Self { config }
    }

    pub fn preprocess_batch(&self, images: &[RgbImage]) -> Result<Array4<f32>, String> {
        if images.is_empty() {
            return Err("输入的图像列表为空！".to_string());
        }

        let (target_width, target_height) = self.config.target_size;
        let batch_size = images.len();
        let mut tensor = Array4::<f32>::zeros((batch_size, 1, target_height as usize, target_width as usize));

        for (b_idx, img) in images.iter().enumerate() {
            let cropped = self.crop_margin(img);
            let padded = self.resize_and_pad(&cropped);
            let gray_2d = self.normalize_and_to_grayscale(&padded);

            for y in 0..target_height as usize {
                for x in 0..target_width as usize {
                    tensor[[b_idx, 0, y, x]] = gray_2d[[y, x]];
                }
            }
        }

        Ok(tensor)
    }

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

        if max_val <= min_val {
            return img.clone();
        }

        let range = (max_val - min_val) as f32;
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut has_content = false;

        for (x, y, pixel) in gray.enumerate_pixels() {
            let normalized = ((pixel[0] as f32 - min_val as f32) / range * 255.0) as u8;
            if normalized < self.config.crop_threshold {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                has_content = true;
            }
        }

        if !has_content || min_x >= max_x || min_y >= max_y {
            return img.clone();
        }

        let crop_w = (max_x - min_x + 1).min(width - min_x);
        let crop_h = (max_y - min_y + 1).min(height - min_y);
        image::imageops::crop_imm(img, min_x, min_y, crop_w, crop_h).to_image()
    }

    fn resize_and_pad(&self, img: &RgbImage) -> RgbImage {
        let (target_width, target_height) = self.config.target_size;
        let (img_width, img_height) = img.dimensions();

        if img_width == 0 || img_height == 0 {
            return RgbImage::from_pixel(target_width, target_height, image::Rgb([255, 255, 255]));
        }

        let scale = (target_width as f32 / img_width as f32).min(target_height as f32 / img_height as f32);
        let new_width = ((img_width as f32 * scale).round() as u32).clamp(1, target_width);
        let new_height = ((img_height as f32 * scale).round() as u32).clamp(1, target_height);

        let resized = resize(img, new_width, new_height, FilterType::Triangle);

        let delta_width = target_width - new_width;
        let delta_height = target_height - new_height;
        let pad_left = delta_width / 2;
        let pad_top = delta_height / 2;

        let mut padded = RgbImage::from_pixel(target_width, target_height, image::Rgb([255, 255, 255]));
        overlay(&mut padded, &resized, pad_left as i64, pad_top as i64);

        padded
    }

    fn normalize_and_to_grayscale(&self, img: &RgbImage) -> ndarray::Array2<f32> {
        let (width, height) = img.dimensions();
        const SCALE: f32 = 1.0 / 255.0;
        let mean = 0.7931f32;
        let std = 0.1738f32;

        let mut grayscale = ndarray::Array2::<f32>::zeros((height as usize, width as usize));

        for (x, y, pixel) in img.enumerate_pixels() {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            let norm_b = (b * SCALE - mean) / std;
            let norm_g = (g * SCALE - mean) / std;
            let norm_r = (r * SCALE - mean) / std;

            let y_val = 0.114 * norm_b + 0.587 * norm_g + 0.299 * norm_r;
            grayscale[[y as usize, x as usize]] = y_val;
        }

        grayscale
    }
}
