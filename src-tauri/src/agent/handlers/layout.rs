// 🛡️ service-layout 视觉版面分析专属处理器
use serde_json::{json, Value};
use service_layout::{LayoutConfig, LayoutService};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::PathBuf;
use std::sync::Arc;

pub fn resolve_layout_model_path() -> Option<PathBuf> {
    let base_dir = core_models_download::ModelManager::resolve_models_base_dir();
    let candidates = [
        base_dir.join("service-layout").join("PP-DocLayoutV3.onnx"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-layout\PP-DocLayoutV3.onnx"),
        PathBuf::from(r"models\service-layout\PP-DocLayoutV3.onnx"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let img_path_str = params.get("image_path").and_then(Value::as_str).ok_or("缺少 image_path 参数")?;
    let img_p = PathBuf::from(img_path_str);
    if !img_p.exists() { return Err(format!("图像文件不存在: {img_path_str}")); }

    let score_threshold = params.get("score_threshold").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(0.15);
    let filter_categories: Option<Vec<String>> = params.get("filter_categories")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).map(|s| s.to_lowercase()).collect());

    let model_path = resolve_layout_model_path()
        .ok_or_else(|| "未找到 PP-DocLayoutV3.onnx 模型文件".to_string())?;

    let _permit = vram_guard.acquire(TaskWeight::Light).await?;

    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let config = LayoutConfig {
            score_threshold,
            ..Default::default()
        };
        let mut service = LayoutService::new(&model_path, None, Some(config))
            .map_err(|e| format!("初始化 PP-DocLayout 失败: {e}"))?;

        let layout_res = service.detect_file(&img_p)
            .map_err(|e| format!("执行版面分析推导失败: {e}"))?;

        let filtered_regions: Vec<service_layout::LayoutRegion> = if let Some(filters) = filter_categories {
            layout_res.regions.into_iter().filter(|r| {
                filters.contains(&r.label.to_lowercase())
            }).collect()
        } else {
            layout_res.regions
        };

        Ok(json!({
            "image_width": layout_res.image_width,
            "image_height": layout_res.image_height,
            "elapsed_ms": layout_res.elapsed_ms,
            "total_regions": filtered_regions.len(),
            "regions": filtered_regions
        }))
    })
    .await
    .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
}
