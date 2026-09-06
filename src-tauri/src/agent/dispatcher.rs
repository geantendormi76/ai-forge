// 🛡️ 极简统一算子路由器 (dispatcher.rs)
use super::handlers;
use serde_json::Value;
use shared_contracts::VramTokenGuard;
use std::sync::Arc;

pub async fn dispatch(
    tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    tracing::info!("⚡ [原子总线分发] 目标算子: {} | 显存余量: {} MB", tool_name, vram_guard.available_vram_mb());
    match tool_name {
        // 1. PDF 物理几何与位图
        "pdf_get_meta" | "pdf_render_page_image" | "pdf_extract_char_boxes" | "pdf_extract_raw_text" => {
            handlers::pdf::handle(tool_name, params, vram_guard).await
        }
        // 2. 版面分析与阅读序
        "layout_detect_regions" => {
            handlers::layout::handle(tool_name, params, vram_guard).await
        }
        // 3. OCR 文字与坐标提取
        "ocr_recognize_image" | "native_ocr_recognize" => {
            handlers::ocr::handle(tool_name, params, vram_guard).await
        }
        // 4. 数学公式与 LaTeX 重构
        "formula_recognize_latex" => {
            handlers::formula::handle(tool_name, params, vram_guard).await
        }
        // 5. 复杂表格拓扑与 HTML 重构
        "table_recognize_structure" => {
            handlers::table::handle(tool_name, params, vram_guard).await
        }
        // 6. 视觉超分
        "native_upscale_image" => {
            handlers::vision::handle(tool_name, params, vram_guard).await
        }
        // 7. 语音识别
        "native_asr_transcribe" => {
            handlers::audio::handle(tool_name, params, vram_guard).await
        }
        // 8. 神经翻译
        "native_neural_translate" => {
            handlers::translation::handle(tool_name, params, vram_guard).await
        }
        // 9. 格式转换
        "native_format_convert" => {
            handlers::converter::handle(tool_name, params, vram_guard).await
        }
        // 10. 多模态文档复合流水线
        "native_doc_parse" => {
            handlers::doc::handle(tool_name, params, vram_guard).await
        }
        _ => Err(format!("未知的原子底座算子: {tool_name}")),
    }
}
