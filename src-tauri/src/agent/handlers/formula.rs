// 🛡️ service-formula 数学公式与 LaTeX 视觉提取专属处理器 (handlers/formula.rs)
use serde_json::{json, Value};
use service_formula::FormulaService;
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let img_path_str = params.get("image_path").and_then(Value::as_str).ok_or("缺少 image_path 参数")?;
    let img_p = PathBuf::from(img_path_str);
    if !img_p.exists() { return Err(format!("公式图像文件不存在: {img_path_str}")); }

    let _permit = vram_guard.acquire(TaskWeight::Light).await?;

    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let img = image::open(&img_p)
            .map_err(|e| format!("读取公式图片失败: {e}"))?
            .to_rgb8();

        let res = FormulaService::recognize_crop(&img, None, None)
            .map_err(|e| format!("PP-FormulaNet 神经推导失败: {e}"))?;

        Ok(json!({
            "image_path": img_p.to_string_lossy(),
            "latex": res.latex,
            "raw_tokens_count": res.raw_token_ids.len(),
            "elapsed_ms": res.elapsed_ms
        }))
    })
    .await
    .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
}
