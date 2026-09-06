// 🛡️ service-asr MOSS 语音大模型转写专属处理器 (handlers/audio.rs)
use serde_json::{json, Value};
use service_asr::{AsrOptions, AsrService};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::Path;
use std::sync::Arc;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let audio_path_str = params.get("audio_path").and_then(Value::as_str).ok_or("缺少 audio_path 参数")?;
    let audio_p = Path::new(audio_path_str);
    if !audio_p.exists() {
        return Err(format!("音频/视频物理文件不存在: {audio_path_str}"));
    }

    let language = params.get("language").and_then(Value::as_str).map(str::to_string);
    let prompt = params.get("prompt").and_then(Value::as_str).map(str::to_string);
    let hotwords = params.get("hotwords").and_then(Value::as_str).map(str::to_string);
    let temperature = params.get("temperature").and_then(Value::as_f64).map(|v| v as f32);

    let options = AsrOptions {
        audio_path: audio_path_str.to_string(),
        language,
        prompt,
        hotwords,
        max_new_tokens: None,
        temperature,
    };

    let _permit = vram_guard.acquire(TaskWeight::Heavy).await?;

    let asr_res = AsrService::run_asr_pipeline_cancellable(options, None, None).await
        .map_err(|e| format!("MOSS 语音大模型转写推导失败: {e}"))?;

    let full_transcript = asr_res.segments.iter()
        .map(|s| format!("[{} {:.1}s~{:.1}s]: {}", s.speaker, s.start_sec, s.end_sec, s.text))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(json!({
        "audio_file": asr_res.audio_file,
        "duration_sec": asr_res.duration_sec,
        "total_segments": asr_res.segments.len(),
        "full_transcript": full_transcript,
        "segments": asr_res.segments,
        "elapsed_ms": asr_res.elapsed_ms
    }))
}
