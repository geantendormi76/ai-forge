// 🛡️ service-translation Hy-MT2 38 语种神经机器翻译专属处理器 (handlers/translation.rs)
use serde_json::{json, Value};
use service_translation::{PureTranslationRequest, TranslationService};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::sync::Arc;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    // 智能兼顾单句字符串与多句数组输入
    let texts: Vec<String> = if let Some(arr) = params.get("texts").and_then(Value::as_array) {
        arr.iter().filter_map(Value::as_str).map(str::to_string).collect()
    } else if let Some(single_text) = params.get("text").and_then(Value::as_str) {
        vec![single_text.to_string()]
    } else {
        return Err("缺少待翻译文本参数 (texts 或 text)".into());
    };

    if texts.is_empty() {
        return Err("待翻译文本列表不能为空".into());
    }

    let target_lang_str = params.get("target_lang")
        .and_then(Value::as_str)
        .unwrap_or("Chinese")
        .to_string();

    let _permit = vram_guard.acquire(TaskWeight::Light).await?;

    let req = PureTranslationRequest {
        texts,
        target_lang: Some(target_lang_str.clone()),
    };

    let trans_res = TranslationService::run_translation_pipeline_with_progress(req, None).await
        .map_err(|e| format!("Hy-MT2 神经翻译推导失败: {e}"))?;

    Ok(json!({
        "total_segments": trans_res.translations.len(),
        "target_lang": target_lang_str,
        "translations": trans_res.translations,
        "elapsed_ms": trans_res.elapsed_ms
    }))
}
