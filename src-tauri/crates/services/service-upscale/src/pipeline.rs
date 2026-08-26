use crate::processor::{RealESRGANPostprocessor, RealESRGANPreprocessor};
use crate::tiling::compute_grid;
use crate::{UpscaleConfig, UpscaleResult};
use core_onnx_infer::OrtInfer;
use image::{imageops, DynamicImage, ImageBuffer, Rgb, RgbImage, Rgba, RgbaImage};
use std::path::Path;
use std::time::Instant;

pub struct RealESRGANModel {
    infer: OrtInfer,
    preprocessor: RealESRGANPreprocessor,
    postprocessor: RealESRGANPostprocessor,
    scale: u32,
}

impl RealESRGANModel {
    pub fn new(model_path: &Path, scale: u32) -> Result<Self, String> {
        let infer = OrtInfer::new(model_path, Some("cuda:0"))
            .map_err(|e| format!("初始化 RealESRGAN ONNX 引擎失败: {e} ({:?})", model_path))?;
        Ok(Self {
            infer,
            preprocessor: RealESRGANPreprocessor::default(),
            postprocessor: RealESRGANPostprocessor::default(),
            scale,
        })
    }

    pub fn scale(&self) -> u32 {
        self.scale
    }

    pub fn forward_tile(&mut self, tile_image: &DynamicImage) -> Result<DynamicImage, String> {
        let tensor = self.preprocessor.preprocess(tile_image)?;
        let out_tensor = self
            .infer
            .infer_array4(&tensor)
            .map_err(|e| format!("RealESRGAN 模型推理失败: {e}"))?;
        self.postprocessor.decode_array4(&out_tensor)
    }
}

pub struct UpscalePipeline {
    model: RealESRGANModel,
}

impl UpscalePipeline {
    pub fn new(model_path: &Path) -> Result<Self, String> {
        let model = RealESRGANModel::new(model_path, 4)?;
        Ok(Self { model })
    }

    pub fn run<F>(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        config: &UpscaleConfig,
        progress_cb: Option<F>,
    ) -> Result<UpscaleResult, String>
    where
        F: Fn(usize, usize, &str) + Send + Sync,
    {
        let t0 = Instant::now();
        if let Some(ref cb) = progress_cb {
            cb(5, 100, "正在加载输入图像与分析物理尺寸...");
        }

        let image = image::open(input_path)
            .map_err(|e| format!("打开图像文件失败: {e} ({:?})", input_path))?;
        let (img_w, img_h) = (image.width(), image.height());
        if img_w == 0 || img_h == 0 {
            return Err("输入图像宽高不能为 0".to_string());
        }

        let max_in_side = img_w.max(img_h) as f32;
        let req_scale = config.target_scale;
        let actual_scale = if (max_in_side * req_scale) > config.max_output_side as f32 {
            let capped = (config.max_output_side as f32 / max_in_side).max(1.0);
            capped
        } else {
            req_scale
        };

        let target_w = ((img_w as f32) * actual_scale).round() as u32;
        let target_h = ((img_h as f32) * actual_scale).round() as u32;

        let has_alpha = image.color().has_alpha();
        let (rgb_image, alpha_channel) = if has_alpha {
            let rgba = image.to_rgba8();
            let mut rgb_buf = ImageBuffer::new(img_w, img_h);
            let mut alpha_buf = ImageBuffer::new(img_w, img_h);
            for (x, y, pixel) in rgba.enumerate_pixels() {
                rgb_buf.put_pixel(x, y, Rgb([pixel[0], pixel[1], pixel[2]]));
                alpha_buf.put_pixel(x, y, image::Luma([pixel[3]]));
            }
            (DynamicImage::ImageRgb8(rgb_buf), Some(DynamicImage::ImageLuma8(alpha_buf)))
        } else {
            (DynamicImage::ImageRgb8(image.to_rgb8()), None)
        };

        let tile_size = if config.tile_size == 0 { 256 } else { config.tile_size };
        let tile_pad = config.tile_pad;
        let tiles = compute_grid(img_w, img_h, tile_size, tile_pad);
        let total_tiles = tiles.len();
        let model_scale = self.model.scale();

        log::info!(
            "🚀 [8K 超分流水线] 输入: {}x{}, 目标输出: {}x{} ({:.2}x), 切块总数: {} 块",
            img_w, img_h, target_w, target_h, actual_scale, total_tiles
        );

        let mut canvas_4x: RgbImage = ImageBuffer::new(img_w * model_scale, img_h * model_scale);

        for (idx, tile) in tiles.iter().enumerate() {
            if let Some(ref cb) = progress_cb {
                let msg = format!("RTX 显卡超分推理中 ({}/{})", idx + 1, total_tiles);
                cb(idx + 1, total_tiles, &msg);
            }

            let crop_x = tile.x - tile.pad_left;
            let crop_y = tile.y - tile.pad_top;
            let crop_w = tile.w + tile.pad_left + tile.pad_right;
            let crop_h = tile.h + tile.pad_top + tile.pad_bottom;
            let sub_img = rgb_image.crop_imm(crop_x, crop_y, crop_w, crop_h);

            let t_tile_start = Instant::now();
            let tile_out = self.model.forward_tile(&sub_img)?;
            let tile_ms = t_tile_start.elapsed().as_millis();

            if idx == 0 || (idx + 1) == total_tiles || (idx + 1) % 5 == 0 {
                log::info!("  ⚡ [GPU 推演切块] 第 {}/{} 块完成 (耗时: {} ms)", idx + 1, total_tiles, tile_ms);
            }

            let valid_x = tile.pad_left * model_scale;
            let valid_y = tile.pad_top * model_scale;
            let valid_w = tile.w * model_scale;
            let valid_h = tile.h * model_scale;
            let tile_rgb = tile_out.to_rgb8();
            let cropped_rgb = imageops::crop_imm(&tile_rgb, valid_x, valid_y, valid_w, valid_h).to_image();
            imageops::overlay(
                &mut canvas_4x,
                &cropped_rgb,
                (tile.x * model_scale) as i64,
                (tile.y * model_scale) as i64,
            );
        }

        if let Some(ref cb) = progress_cb {
            cb(total_tiles, total_tiles, "正在调整至目标分辨率并融合透明通道...");
        }

        let raw_4x_image = DynamicImage::ImageRgb8(canvas_4x);
        let final_rgb = raw_4x_image.resize_exact(
            target_w,
            target_h,
            image::imageops::FilterType::Triangle,
        );

        if let Some(alpha) = alpha_channel {
            let scaled_alpha = alpha.resize_exact(
                target_w,
                target_h,
                image::imageops::FilterType::Triangle,
            );
            let luma_alpha = scaled_alpha.to_luma8();
            let rgb_buf = final_rgb.to_rgb8();
            let mut rgba_buf: RgbaImage = ImageBuffer::new(target_w, target_h);
            for (x, y, pixel) in rgba_buf.enumerate_pixels_mut() {
                let rgb_p = rgb_buf.get_pixel(x, y);
                let a_p = luma_alpha.get_pixel(x, y);
                *pixel = Rgba([rgb_p[0], rgb_p[1], rgb_p[2], a_p[0]]);
            }
            DynamicImage::ImageRgba8(rgba_buf)
                .save(output_path)
                .map_err(|e| format!("保存 RGBA 图像失败: {e} ({:?})", output_path))?;
        } else {
            final_rgb
                .save(output_path)
                .map_err(|e| format!("保存 RGB 图像失败: {e} ({:?})", output_path))?;
        }

        let elapsed_ms = t0.elapsed().as_millis() as u64;
        log::info!(
            "🏆 [8K 超分交付] 成功落盘: {:?}, 端到端总耗时: {} ms ({:.2} s)",
            output_path, elapsed_ms, elapsed_ms as f64 / 1000.0
        );

        Ok(UpscaleResult {
            success: true,
            input_path: input_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            original_size: (img_w, img_h),
            output_size: (target_w, target_h),
            actual_scale,
            elapsed_ms,
            total_tiles,
        })
    }
}
