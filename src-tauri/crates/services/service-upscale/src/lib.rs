// 🛡️ 8K 视觉超分 - 工业级服务门面 (lib.rs)
// 100% 对齐原版 UpscaleEngine / UpscaleTask 契约

pub mod pipeline;
pub mod processor;
pub mod tiling;

pub use pipeline::{RealESRGANModel, UpscalePipeline};
pub use processor::{RealESRGANPostprocessor, RealESRGANPreprocessConfig, RealESRGANPreprocessor};
pub use tiling::{compute_grid, TileInfo};

use serde::{Deserialize, Serialize};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpscaleTask {
    pub input_path: String,
    pub output_path: String,
    pub model_path: Option<String>,
    pub target_scale: Option<f32>,
    pub max_output_side: Option<u32>,
    pub tile_size: Option<u32>,
    pub tile_pad: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpscaleConfig {
    pub model_path: PathBuf,
    pub target_scale: f32,
    pub max_output_side: u32,
    pub tile_size: u32,
    pub tile_pad: u32,
}

impl Default for UpscaleConfig {
    fn default() -> Self {
        Self {
            model_path: UpscaleService::resolve_default_model_path(),
            target_scale: 4.0,
            max_output_side: 8192,
            tile_size: 256,
            tile_pad: 10,
        }
    }
}

impl UpscaleConfig {
    /// 4K 极速模式：3840px 黄金封顶，兼顾超清观感与秒级完成
    pub fn fast_4k() -> Self {
        Self {
            model_path: UpscaleService::resolve_default_model_path(),
            target_scale: 4.0,
            max_output_side: 3840,
            tile_size: 256,
            tile_pad: 10,
        }
    }

    /// 8K 旗舰模式：8192px 物理封顶
    pub fn flagship_8k() -> Self {
        Self {
            model_path: UpscaleService::resolve_default_model_path(),
            target_scale: 4.0,
            max_output_side: 8192,
            tile_size: 256,
            tile_pad: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpscaleResult {
    pub success: bool,
    pub input_path: String,
    pub output_path: String,
    pub original_size: (u32, u32),
    pub output_size: (u32, u32),
    pub actual_scale: f32,
    pub elapsed_ms: u64,
    pub total_tiles: usize,
}

pub struct UpscaleService;

impl UpscaleService {
    pub fn resolve_default_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-upscale\RealESRGAN_x4plus.onnx"),
            PathBuf::from(r"models\service-upscale\RealESRGAN_x4plus.onnx"),
            PathBuf::from(r"..\models\service-upscale\RealESRGAN_x4plus.onnx"),
            PathBuf::from(r"..\..\models\service-upscale\RealESRGAN_x4plus.onnx"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return candidate.clone();
            }
        }

        PathBuf::from(r"C:\dev\ai-forge\models\service-upscale\RealESRGAN_x4plus.onnx")
    }

    pub async fn run_upscale<F>(
        task: &UpscaleTask,
        vram_guard: Option<&VramTokenGuard>,
        progress_cb: Option<F>,
    ) -> Result<UpscaleResult, String>
    where
        F: Fn(usize, usize, &str) + Send + Sync + 'static,
    {
        let input_path = Path::new(&task.input_path);
        if !input_path.exists() {
            return Err(format!("输入图像文件不存在: {:?}", task.input_path));
        }

        let output_path = Path::new(&task.output_path);
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let model_path = task
            .model_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(Self::resolve_default_model_path);

        if !model_path.exists() {
            return Err(format!("ONNX 模型文件未就绪: {:?}", model_path));
        }

        let config = UpscaleConfig {
            model_path: model_path.clone(),
            target_scale: task.target_scale.unwrap_or(4.0),
            max_output_side: task.max_output_side.unwrap_or(8192),
            tile_size: task.tile_size.unwrap_or(256),
            tile_pad: task.tile_pad.unwrap_or(10),
        };

        let _permit = if let Some(guard) = vram_guard {
            Some(guard.acquire(TaskWeight::Medium).await?)
        } else {
            None
        };

        let input_buf = input_path.to_path_buf();
        let output_buf = output_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let mut pipeline = UpscalePipeline::new(&model_path)?;
            pipeline.run(&input_buf, &output_buf, &config, progress_cb)
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
    }
}
