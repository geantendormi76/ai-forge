// 🛡️ service-doc-parse 多模态文档与图像通用深度重构总成 (handlers/doc.rs)
// 100% 串联 5 大纯血底座：PP-DocLayoutV3 + SLANet + PP-FormulaNet + PP-OCRv6 + XY-Cut 空间编织

use pdf_parse::service::PdfParseService;
use serde_json::{json, Value};
use service_doc_parse::DocParseEngine;
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let file_path_str = params.get("file_path").and_then(Value::as_str).ok_or("缺少 file_path 参数")?;
    let in_p = PathBuf::from(file_path_str);
    if !in_p.exists() {
        return Err(format!("待解析物理文件不存在: {file_path_str}"));
    }

    let target_out_dir = if let Some(out_dir_str) = params.get("output_dir").and_then(Value::as_str) {
        PathBuf::from(out_dir_str)
    } else {
        in_p.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    };

    let ext = in_p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "webp");

    if is_image {
        // ==================== 1. 单张图片 / 复杂单据全模态排版流水线 ====================
        tracing::info!("🖼️ [多模态总成] 命中单图排版通道, 启动 5 大模型底座全景重构: {:?}", in_p);
        let _permit = vram_guard.acquire(TaskWeight::Medium).await?;

        tokio::task::spawn_blocking(move || -> Result<Value, String> {
            let t0 = Instant::now();
            let dyn_img = image::open(&in_p).map_err(|e| format!("打开图像失败: {e}"))?;

            // 调起统一排版大脑
            let mut engine = DocParseEngine::default_engine();
            let (_doc_ast, markdown) = engine.parse_image_ast(&dyn_img, 0)
                .map_err(|e| format!("多模态图像重构推导失败: {e}"))?;

            // 将生成的完整 Markdown 写入输出目录
            let stem = in_p.file_stem().and_then(|s| s.to_str()).unwrap_or("parsed_image");
            let output_md_path = target_out_dir.join(format!("{}_parsed.md", stem));
            let _ = std::fs::create_dir_all(&target_out_dir);
            std::fs::write(&output_md_path, &markdown)
                .map_err(|e| format!("保存 Markdown 文件失败: {e}"))?;

            let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
            tracing::info!("✅ [多模态总成] 图像全量排版完成, 耗时: {:.2}ms, 产物: {:?}", elapsed_ms, output_md_path);

            Ok(json!({
                "success": true,
                "markdown": markdown,
                "output_md_path": output_md_path.to_string_lossy(),
                "task_out_dir": target_out_dir.to_string_lossy(),
                "images_dir": target_out_dir.join("images").to_string_lossy(),
                "route_label": "🧠 全血单图多模态重构轨 (Layout + SLANet + Formula + OCR + XY-Cut)",
                "elapsed_ms": elapsed_ms
            }))
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
    } else {
        // ==================== 2. PDF 多页文档双轨重构流水线 ====================
        tracing::info!("📄 [多模态总成] 命中 PDF 重构通道, 调起双轨漏斗引擎: {:?}", in_p);
        let res = PdfParseService::run_parse(
            file_path_str,
            &target_out_dir,
            Some(&vram_guard),
            None::<fn(usize, usize, &str)>,
        )
        .await
        .map_err(|e| format!("多模态 PDF 深度重构失败: {e}"))?;

        Ok(json!({
            "success": res.success,
            "markdown": res.markdown,
            "output_md_path": res.output_md_path,
            "task_out_dir": res.task_out_dir,
            "images_dir": res.images_dir,
            "route_label": res.route_label,
            "elapsed_ms": res.elapsed_ms
        }))
    }
}
