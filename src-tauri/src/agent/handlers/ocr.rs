// 🛡️ service-ocr 双阶段文字识别专属处理器
use serde_json::{json, Value};
use service_ocr::OcrService;
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let img_path_str = params.get("image_path").and_then(Value::as_str).ok_or("缺少 image_path 参数")?;
    let img_p = PathBuf::from(img_path_str);
    if !img_p.exists() { return Err(format!("图像文件不存在: {img_path_str}")); }

    let min_score = params.get("min_score").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(0.3);

    let _permit = vram_guard.acquire(TaskWeight::Light).await?;

    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let t0 = Instant::now();
        let dynamic_img = image::open(&img_p).map_err(|e| format!("读取图片失败: {e}"))?;
        let mut engine = OcrService::default_engine().map_err(|e| e.to_string())?;
        let raw_regions = engine.process_image(dynamic_img).map_err(|e| e.to_string())?;

        let filtered: Vec<service_ocr::TextRegion> = raw_regions.into_iter().filter(|r| r.score >= min_score).collect();
        let full_text = filtered.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("\n");
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        Ok(json!({
            "image_path": img_p.to_string_lossy(),
            "total_lines": filtered.len(),
            "full_text": full_text,
            "regions": filtered,
            "elapsed_ms": elapsed_ms
        }))
    })
    .await
    .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
}
