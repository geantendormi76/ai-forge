// 🛡️ service-upscale 4K/8K 图像无损超分专属处理器 (handlers/vision.rs)
use serde_json::Value;
use service_upscale::{UpscaleService, UpscaleTask};
use shared_contracts::VramTokenGuard;
use std::path::Path;
use std::sync::Arc;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let input_path_str = params.get("input_path").and_then(Value::as_str).ok_or("缺少 input_path 参数")?;
    let in_p = Path::new(input_path_str);
    if !in_p.exists() {
        return Err(format!("输入图像文件不存在: {input_path_str}"));
    }

    let target_scale = params.get("target_scale").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(4.0);
    let max_output_side = params.get("max_output_side").and_then(Value::as_u64).map(|v| v as u32).unwrap_or(3840);
    let tile_size = params.get("tile_size").and_then(Value::as_u64).map(|v| v as u32);
    let tile_pad = params.get("tile_pad").and_then(Value::as_u64).map(|v| v as u32);

    let output_path_str = params.get("output_path").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| {
        let parent = in_p.parent().unwrap_or_else(|| Path::new("."));
        let stem = in_p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let ext = in_p.extension().and_then(|s| s.to_str()).unwrap_or("png");
        parent.join(format!("{}_upscaled_{:.0}x.{}", stem, target_scale, ext)).to_string_lossy().to_string()
    });

    let model_path = params.get("model_path").and_then(Value::as_str).map(str::to_string);

    let task = UpscaleTask {
        input_path: input_path_str.to_string(),
        output_path: output_path_str,
        model_path,
        target_scale: Some(target_scale),
        max_output_side: Some(max_output_side),
        tile_size,
        tile_pad,
    };

    let res = UpscaleService::run_upscale(&task, Some(&vram_guard), None::<fn(usize, usize, &str)>).await
        .map_err(|e| format!("RealESRGAN 超分推导失败: {e}"))?;

    Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
}
