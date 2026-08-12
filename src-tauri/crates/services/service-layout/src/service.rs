use crate::contracts::{LayoutConfig, LayoutResult};
use crate::postprocess::LayoutPostProcess;
use crate::preprocess::LayoutPreprocess;
use core_onnx_infer::OrtInfer;
use image::RgbImage;
use std::path::Path;
use std::time::Instant;

pub struct LayoutService {
    infer: OrtInfer,
    config: LayoutConfig,
    target_size: (u32, u32),
}

impl LayoutService {
    pub fn new(
        model_path: &Path,
        device_override: Option<&str>,
        config: Option<LayoutConfig>,
    ) -> Result<Self, String> {
        let infer = OrtInfer::new(model_path, device_override)
            .map_err(|e| format!("加载 PP-DocLayout ONNX 模型失败 ({:?}): {e}", model_path))?;

        let config = config.unwrap_or_default();
        Ok(Self {
            infer,
            config,
            target_size: (800, 800),
        })
    }

    pub fn input_names(&self) -> Vec<String> {
        self.infer.input_names()
    }

    pub fn with_target_size(mut self, width: u32, height: u32) -> Self {
        self.target_size = (width, height);
        self
    }

    pub fn detect(&mut self, img: &RgbImage) -> Result<LayoutResult, String> {
        let t0 = Instant::now();
        let (width, height) = img.dimensions();

        let (input_tensor, _scale_w, _scale_h) =
            LayoutPreprocess::preprocess_image(img, self.target_size);

        let output_array2 = self
            .infer
            .infer_scale_aware_array2(&input_tensor, (width as f32, height as f32))
            .map_err(|e| format!("PP-DocLayout ONNX 2D推导失败: {e}"))?;

        let regions = LayoutPostProcess::process_pp_doclayout_2d(
            output_array2.view(),
            width as f32,
            height as f32,
            &self.config,
        );

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        Ok(LayoutResult {
            image_width: width,
            image_height: height,
            regions,
            elapsed_ms,
        })
    }

    pub fn detect_file(&mut self, image_path: &Path) -> Result<LayoutResult, String> {
        let img = image::open(image_path)
            .map_err(|e| format!("打开版面分析测试图像失败 ({:?}): {e}", image_path))?
            .to_rgb8();

        self.detect(&img)
    }
}
