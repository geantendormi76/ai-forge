use crate::core::OCRError;
use crate::processors::types::{ColorOrder, TensorLayout};
use image::{DynamicImage, RgbImage};
use ndarray::Array4;
use rayon::prelude::*;

#[derive(Debug)]
pub struct NormalizeImage {
    pub alpha: Vec<f32>,
    pub beta: Vec<f32>,
    pub order: TensorLayout,
    pub color_order: ColorOrder,
}

impl NormalizeImage {
    const PARALLEL_NORMALIZE_MIN_BYTES: usize = 1_048_576;

    fn should_parallelize(batch_size: usize, total_output_bytes: usize) -> bool {
        batch_size > 1 && total_output_bytes > Self::PARALLEL_NORMALIZE_MIN_BYTES
    }

    fn src_channels(&self) -> [usize; 3] {
        match self.color_order {
            ColorOrder::RGB => [0, 1, 2],
            ColorOrder::BGR => [2, 1, 0],
        }
    }

    fn image_len(width: u32, height: u32, channels: usize) -> usize {
        width as usize * height as usize * channels
    }

    pub fn new(
        scale: Option<f32>,
        mean: Option<Vec<f32>>,
        std: Option<Vec<f32>>,
        order: Option<TensorLayout>,
        color_order: Option<ColorOrder>,
    ) -> Result<Self, OCRError> {
        Self::with_color_order(scale, mean, std, order, color_order)
    }

    pub fn with_color_order(
        scale: Option<f32>,
        mean: Option<Vec<f32>>,
        std: Option<Vec<f32>>,
        order: Option<TensorLayout>,
        color_order: Option<ColorOrder>,
    ) -> Result<Self, OCRError> {
        let scale = scale.unwrap_or(1.0 / 255.0);
        let mean = mean.unwrap_or_else(|| vec![0.485, 0.456, 0.406]);
        let std = std.unwrap_or_else(|| vec![0.229, 0.224, 0.225]);
        let order = order.unwrap_or(TensorLayout::CHW);
        let color_order = color_order.unwrap_or_default();

        if scale <= 0.0 {
            return Err(OCRError::ConfigError("Scale must be greater than 0".into()));
        }
        if mean.len() != 3 || std.len() != 3 {
            return Err(OCRError::ConfigError("Mean/Std must have exactly 3 elements".into()));
        }
        for (i, &s) in std.iter().enumerate() {
            if s <= 0.0 {
                return Err(OCRError::ConfigError(format!("Std at index {i} must be > 0")));
            }
        }

        let alpha: Vec<f32> = std.iter().map(|s| scale / s).collect();
        let beta: Vec<f32> = mean.iter().zip(&std).map(|(m, s)| -m / s).collect();

        Ok(Self {
            alpha,
            beta,
            order,
            color_order,
        })
    }

    pub fn apply(&self, imgs: Vec<DynamicImage>) -> Vec<Vec<f32>> {
        imgs.into_iter().map(|img| self.normalize(img)).collect()
    }

    fn normalize(&self, img: DynamicImage) -> Vec<f32> {
        let rgb_img = into_rgb8_no_copy(img);
        self.normalize_rgb(&rgb_img)
    }

    fn normalize_rgb(&self, rgb_img: &RgbImage) -> Vec<f32> {
        let (width, height) = rgb_img.dimensions();
        let mut result = vec![0.0f32; Self::image_len(width, height, 3)];
        self.normalize_rgb_into(rgb_img, &mut result);
        result
    }

    fn normalize_rgb_into(&self, rgb_img: &RgbImage, out: &mut [f32]) {
        let (width, height) = rgb_img.dimensions();
        let (w, h) = (width as usize, height as usize);
        let src_ch = self.src_channels();
        let alpha = [self.alpha[0], self.alpha[1], self.alpha[2]];
        let beta = [self.beta[0], self.beta[1], self.beta[2]];
        let rgb = rgb_img.as_raw();

        let hw = w * h;
        match self.order {
            TensorLayout::CHW => {
                for y in 0..h {
                    for x in 0..w {
                        let i = y * w + x;
                        let pixel_idx = i * 3;
                        let r = rgb[pixel_idx];
                        let g = rgb[pixel_idx + 1];
                        let b = rgb[pixel_idx + 2];
                        let pixels = [r as f32, g as f32, b as f32];

                        out[0 * hw + i] = pixels[src_ch[0]] * alpha[0] + beta[0];
                        out[1 * hw + i] = pixels[src_ch[1]] * alpha[1] + beta[1];
                        out[2 * hw + i] = pixels[src_ch[2]] * alpha[2] + beta[2];
                    }
                }
            }
            TensorLayout::HWC => {
                for y in 0..h {
                    for x in 0..w {
                        let i = y * w + x;
                        let pixel_idx = i * 3;
                        let r = rgb[pixel_idx];
                        let g = rgb[pixel_idx + 1];
                        let b = rgb[pixel_idx + 2];
                        let pixels = [r as f32, g as f32, b as f32];

                        let out_idx = i * 3;
                        out[out_idx] = pixels[src_ch[0]] * alpha[0] + beta[0];
                        out[out_idx + 1] = pixels[src_ch[1]] * alpha[1] + beta[1];
                        out[out_idx + 2] = pixels[src_ch[2]] * alpha[2] + beta[2];
                    }
                }
            }
        }
    }

    pub fn normalize_to(&self, img: DynamicImage) -> Result<Array4<f32>, OCRError> {
        let rgb_img = into_rgb8_no_copy(img);
        let (width, height) = rgb_img.dimensions();
        let (w, h) = (width as usize, height as usize);
        let result = self.normalize_rgb(&rgb_img);

        match self.order {
            TensorLayout::CHW => Array4::from_shape_vec((1, 3, h, w), result)
                .map_err(|e| OCRError::InvalidInput(format!("CHW shape error: {e}"))),
            TensorLayout::HWC => Array4::from_shape_vec((1, h, w, 3), result)
                .map_err(|e| OCRError::InvalidInput(format!("HWC shape error: {e}"))),
        }
    }

    pub fn normalize_batch_to(&self, imgs: Vec<DynamicImage>) -> Result<Array4<f32>, OCRError> {
        if imgs.is_empty() {
            return Ok(Array4::zeros((0, 0, 0, 0)));
        }

        let batch_size = imgs.len();
        let rgb_imgs: Vec<_> = imgs.into_iter().map(into_rgb8_no_copy).collect();
        let (first_w, first_h) = rgb_imgs[0].dimensions();

        for (i, img) in rgb_imgs.iter().enumerate() {
            if img.dimensions() != (first_w, first_h) {
                return Err(OCRError::InvalidInput(format!(
                    "Batch images must have same dimensions. Image 0: {first_w}x{first_h}, Image {i}: {}x{}",
                    img.width(), img.height()
                )));
            }
        }

        let (w, h) = (first_w as usize, first_h as usize);
        let img_size = Self::image_len(first_w, first_h, 3);
        let mut result = vec![0.0f32; batch_size * img_size];

        let use_parallel = Self::should_parallelize(batch_size, result.len() * std::mem::size_of::<f32>());
        if !use_parallel {
            for (rgb_img, batch_slice) in rgb_imgs.iter().zip(result.chunks_mut(img_size)) {
                self.normalize_rgb_into(rgb_img, batch_slice);
            }
        } else {
            result
                .par_chunks_mut(img_size)
                .zip(rgb_imgs.par_iter())
                .for_each(|(batch_slice, rgb_img)| {
                    self.normalize_rgb_into(rgb_img, batch_slice);
                });
        }

        let shape = match self.order {
            TensorLayout::CHW => (batch_size, 3, h, w),
            TensorLayout::HWC => (batch_size, h, w, 3),
        };
        Array4::from_shape_vec(shape, result)
            .map_err(|e| OCRError::InvalidInput(format!("Batch normalization shape error: {e}")))
    }
}

fn into_rgb8_no_copy(img: DynamicImage) -> RgbImage {
    match img {
        DynamicImage::ImageRgb8(img) => img,
        img => img.to_rgb8(),
    }
}
