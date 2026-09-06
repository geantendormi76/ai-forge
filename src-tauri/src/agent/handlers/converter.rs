// 🛡️ service-converter 全能格式转换与解密专属处理器 (handlers/converter.rs)
use format_converter::{service::FormatConvertService, FormatConvertTask};
use serde_json::{json, Value};
use shared_contracts::VramTokenGuard;
use std::path::Path;
use std::sync::Arc;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    _vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let input_path_str = params.get("input_path").and_then(Value::as_str).ok_or("缺少 input_path 参数")?;
    let in_p = Path::new(input_path_str);
    if !in_p.exists() {
        return Err(format!("待转换源文件不存在: {input_path_str}"));
    }

    let target_format = params.get("target_format").and_then(Value::as_str).ok_or("缺少 target_format 参数")?.to_string();
    let output_dir = params.get("output_dir").and_then(Value::as_str).map(str::to_string);

    let task = FormatConvertTask {
        input_path: input_path_str.to_string(),
        target_format,
        output_dir,
    };

    tokio::task::spawn_blocking(move || {
        let res = FormatConvertService::convert(&task);
        Ok(json!({
            "success": res.success,
            "input_path": res.input_path,
            "output_path": res.output_path,
            "detected_format": res.detected_format,
            "message": res.message
        }))
    })
    .await
    .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
}
