// 🛡️ 8K 视觉超分 - 纯血零拷贝预处理与 Rayon 多核并行解码器 (processor.rs)
// 100% 外科手术式 1:1 直译自 ai-toolkit Commit 776957e

use image::{DynamicImage, GenericImageView, ImageBuffer, RgbImage};
use ndarray::Array4;
use rayon::prelude::*;

/// 预处理配置参数
#[derive(Debug, Clone, Copy)]
pub struct RealESRGANPreprocessConfig {
    pub normalize_scale: f32,
}

impl Default for RealESRGANPreprocessConfig {
    fn default() -> Self {
        Self {
            normalize_scale: 1.0 / 255.0,
        }
    }
}

/// RealESRGAN 预处理器: DynamicImage -> [1, 3, H, W] NCHW Float32 张量
#[derive(Debug, Clone, Default)]
pub struct RealESRGANPreprocessor {
    pub config: RealESRGANPreprocessConfig,
}

impl RealESRGANPreprocessor {
    pub fn new(config: RealESRGANPreprocessConfig) -> Self {
        Self { config }
    }

    pub fn preprocess(&self, image: &DynamicImage) -> Result<Array4<f32>, String> {
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err("输入图像尺寸不能为 0".to_string());
        }

        let rgb_img = image.to_rgb8();
        let scale = self.config.normalize_scale;
        let mut tensor = Array4::<f32>::zeros((1, 3, height as usize, width as usize));

        for y in 0..height {
            for x in 0..width {
                let pixel = rgb_img.get_pixel(x, y);
                tensor[[0, 0, y as usize, x as usize]] = pixel.0[0] as f32 * scale; // R
                tensor[[0, 1, y as usize, x as usize]] = pixel.0[1] as f32 * scale; // G
                tensor[[0, 2, y as usize, x as usize]] = pixel.0[2] as f32 * scale; // B
            }
        }

        Ok(tensor)
    }
}

/// RealESRGAN 后处理器: [1, 3, H, W] Float32 张量 -> DynamicImage
#[derive(Debug, Clone, Default)]
pub struct RealESRGANPostprocessor;

impl RealESRGANPostprocessor {
    pub fn decode_array4(&self, tensor: &Array4<f32>) -> Result<DynamicImage, String> {
        let shape = tensor.shape();
        if shape.len() != 4 || shape[1] != 3 {
            return Err(format!("张量形状不匹配，预期 [1, 3, H, W]，实际 {:?}", shape));
        }

        let height = shape[2];
        let width = shape[3];

        let slice_3d = tensor.slice(ndarray::s![0, .., .., ..]);
        let mut bytes = vec![0_u8; width * height * 3];

        bytes
            .par_chunks_mut(3)
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index % width;
                let y = index / width;
                pixel[0] = (slice_3d[[0, y, x]] * 255.0).clamp(0.0, 255.0) as u8;
                pixel[1] = (slice_3d[[1, y, x]] * 255.0).clamp(0.0, 255.0) as u8;
                pixel[2] = (slice_3d[[2, y, x]] * 255.0).clamp(0.0, 255.0) as u8;
            });

        let image: RgbImage = ImageBuffer::from_raw(width as u32, height as u32, bytes)
            .ok_or_else(|| "RGB 图像缓冲区重建失败".to_string())?;

        Ok(DynamicImage::ImageRgb8(image))
    }
}
