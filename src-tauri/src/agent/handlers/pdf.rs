// 🛡️ service-pdfium 物理几何与位图渲染专属处理器
use serde_json::{json, Value};
use service_pdfium::PdfiumEngine;
use shared_contracts::VramTokenGuard;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub async fn handle(
    tool_name: &str,
    params: Value,
    _vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    match tool_name {
        "pdf_get_meta" => {
            let file_path_str = params.get("file_path").and_then(Value::as_str).ok_or("缺少 file_path 参数")?;
            let path = PathBuf::from(file_path_str);
            if !path.exists() { return Err(format!("PDF 文件不存在: {file_path_str}")); }

            tokio::task::spawn_blocking(move || -> Result<Value, String> {
                let page_count = PdfiumEngine::get_page_count(&path).map_err(|e| e.to_string())?;
                let (w_pt, h_pt) = if page_count > 0 {
                    PdfiumEngine::get_page_dimensions(&path, 0).unwrap_or((0.0, 0.0))
                } else {
                    (0.0, 0.0)
                };
                Ok(json!({
                    "file_path": path.to_string_lossy(),
                    "page_count": page_count,
                    "first_page_width_pt": w_pt,
                    "first_page_height_pt": h_pt,
                    "is_landscape": w_pt > h_pt
                }))
            })
            .await
            .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        }

        "pdf_render_page_image" => {
            let file_path_str = params.get("file_path").and_then(Value::as_str).ok_or("缺少 file_path 参数")?;
            let page_index = params.get("page_index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let target_dpi = params.get("target_dpi").and_then(Value::as_u64).unwrap_or(300) as u32;
            let output_path_opt = params.get("output_path").and_then(Value::as_str).map(PathBuf::from);

            let in_path = PathBuf::from(file_path_str);
            if !in_path.exists() { return Err(format!("PDF 文件不存在: {file_path_str}")); }

            tokio::task::spawn_blocking(move || -> Result<Value, String> {
                let img = PdfiumEngine::render_page_to_image(&in_path, page_index, target_dpi).map_err(|e| e.to_string())?;
                let out_path = output_path_opt.unwrap_or_else(|| {
                    let parent = in_path.parent().unwrap_or_else(|| Path::new("."));
                    let stem = in_path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
                    parent.join(format!("{}_p{}_{}dpi.png", stem, page_index + 1, target_dpi))
                });

                if let Some(parent) = out_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                img.save(&out_path).map_err(|e| format!("保存渲染图像失败: {e}"))?;

                Ok(json!({
                    "output_image_path": out_path.to_string_lossy(),
                    "page_index": page_index,
                    "width_px": img.width(),
                    "height_px": img.height(),
                    "dpi": target_dpi
                }))
            })
            .await
            .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        }

        "pdf_extract_char_boxes" => {
            let file_path_str = params.get("file_path").and_then(Value::as_str).ok_or("缺少 file_path 参数")?;
            let page_index = params.get("page_index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let path = PathBuf::from(file_path_str);
            if !path.exists() { return Err(format!("PDF 文件不存在: {file_path_str}")); }

            tokio::task::spawn_blocking(move || -> Result<Value, String> {
                let chars = PdfiumEngine::extract_page_chars(&path, page_index).map_err(|e| e.to_string())?;
                Ok(json!({
                    "page_index": page_index,
                    "total_chars": chars.len(),
                    "chars": chars
                }))
            })
            .await
            .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        }

        "pdf_extract_raw_text" => {
            let file_path_str = params.get("file_path").and_then(Value::as_str).ok_or("缺少 file_path 参数")?;
            let page_index = params.get("page_index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let path = PathBuf::from(file_path_str);
            if !path.exists() { return Err(format!("PDF 文件不存在: {file_path_str}")); }

            tokio::task::spawn_blocking(move || -> Result<Value, String> {
                let text = PdfiumEngine::extract_page_text(&path, page_index).map_err(|e| e.to_string())?;
                Ok(json!({
                    "page_index": page_index,
                    "char_count": text.chars().count(),
                    "text": text
                }))
            })
            .await
            .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        }

        _ => Err(format!("PDF 处理器未实现算子: {tool_name}")),
    }
}
