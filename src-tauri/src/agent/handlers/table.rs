// 🛡️ service-table 复杂表格结构化重构专属处理器 (handlers/table.rs)
// 工业级几何缝合：SLANet 表格拓扑 + PP-OCRv6 单元格文字空间包容注入

use serde_json::{json, Value};
use service_ocr::OcrService;
use service_table::{recognize_table_crop, wrap_table_html_with_content, BBox8};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

pub fn resolve_table_model_path() -> Option<PathBuf> {
    let base_dir = core_models_download::ModelManager::resolve_models_base_dir();
    let candidates = [
        base_dir.join("service-table").join("SLANet_plus.onnx"),
        base_dir.join("service-table").join("slanet.onnx"),
        base_dir.join("service-table").join("model.onnx"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-table\SLANet_plus.onnx"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-table\slanet.onnx"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-table\model.onnx"),
        PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\SLANet\model.onnx"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub fn resolve_table_dict_path() -> Option<PathBuf> {
    let base_dir = core_models_download::ModelManager::resolve_models_base_dir();
    let candidates = [
        base_dir.join("service-table").join("table_structure_dict.txt"),
        base_dir.join("service-table").join("table_structure_dict_ch.txt"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-table\table_structure_dict.txt"),
        PathBuf::from(r"C:\dev\ai-forge\models\service-table\table_structure_dict_ch.txt"),
        PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\SLANet\table_structure_dict_ch.txt"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

/// 计算 BBox8 四边形的外接轴对齐矩形 (AABB)
fn get_bbox8_aabb(b: &BBox8) -> (f32, f32, f32, f32) {
    let xs = [b[0], b[2], b[4], b[6]];
    let ys = [b[1], b[3], b[5], b[7]];
    let min_x = xs.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_x = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min_y = ys.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_y = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    (min_x, min_y, max_x, max_y)
}

pub async fn handle(
    _tool_name: &str,
    params: Value,
    vram_guard: Arc<VramTokenGuard>,
) -> Result<Value, String> {
    let img_path_str = params.get("image_path").and_then(Value::as_str).ok_or("缺少 image_path 参数")?;
    let img_p = PathBuf::from(img_path_str);
    if !img_p.exists() { return Err(format!("表格图像文件不存在: {img_path_str}")); }

    let model_path = resolve_table_model_path()
        .ok_or_else(|| "未找到 SLANet 表格模型 (SLANet_plus.onnx / slanet.onnx)".to_string())?;
    let dict_path = resolve_table_dict_path()
        .ok_or_else(|| "未找到 SLANet 表格字典 (table_structure_dict.txt)".to_string())?;

    let _permit = vram_guard.acquire(TaskWeight::Light).await?;

    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let t0 = Instant::now();

        // 1. 读取原图
        let dynamic_img = image::open(&img_p).map_err(|e| format!("读取表格图片失败: {e}"))?;
        let rgb_img = dynamic_img.to_rgb8();

        // 2. SLANet 提取表格拓扑结构与单元格坐标
        let table_res = recognize_table_crop(&rgb_img, &model_path, &dict_path, None)
            .map_err(|e| format!("SLANet 神经推导失败: {e}"))?;

        // 3. PP-OCRv6 提取图像内所有文本行与几何中心
        let mut ocr_engine = OcrService::default_engine().map_err(|e| format!("加载 OCR 引擎失败: {e}"))?;
        let ocr_regions = ocr_engine.process_image(dynamic_img).map_err(|e| format!("OCR 推理失败: {e}"))?;

        // 预先计算各 OCR 文本块的几何中心 (cx, cy)
        let ocr_items: Vec<(f32, f32, String)> = ocr_regions
            .into_iter()
            .filter(|r| r.score >= 0.3 && !r.text.trim().is_empty())
            .map(|r| {
                let n = r.bbox.points.len() as f32;
                let sum_x: f32 = r.bbox.points.iter().map(|p| p.x).sum();
                let sum_y: f32 = r.bbox.points.iter().map(|p| p.y).sum();
                (sum_x / n.max(1.0), sum_y / n.max(1.0), r.text.trim().to_string())
            })
            .collect();

        // 4. 2D 几何空间包容性缝合：为每个 SLANet Cell 搜寻命中的 OCR 文字
        let padding = 3.0f32; // 容差像素缓冲
        let mut filled_texts: Vec<Option<String>> = Vec::with_capacity(table_res.cells.len());

        for cell in &table_res.cells {
            let (min_x, min_y, max_x, max_y) = get_bbox8_aabb(&cell.bbox);
            let mut matched_lines = Vec::new();

            for (cx, cy, text) in &ocr_items {
                if *cx >= (min_x - padding)
                    && *cx <= (max_x + padding)
                    && *cy >= (min_y - padding)
                    && *cy <= (max_y + padding)
                {
                    matched_lines.push((*cy, text.clone()));
                }
            }

            if matched_lines.is_empty() {
                filled_texts.push(None);
            } else {
                // 按纵向 y 坐标从上至下排序合并
                matched_lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let combined = matched_lines.into_iter().map(|(_, t)| t).collect::<Vec<_>>().join(" ");
                filled_texts.push(Some(combined));
            }
        }

        // 5. 组装饱满的 HTML 表格代码
        let html_structure = wrap_table_html_with_content(&table_res.structure_tokens, &filled_texts);
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        Ok(json!({
            "image_path": img_p.to_string_lossy(),
            "total_cells": table_res.cells.len(),
            "html_structure": html_structure,
            "score": table_res.score,
            "cells": table_res.cells,
            "filled_count": filled_texts.iter().filter(|t| t.is_some()).count(),
            "elapsed_ms": elapsed_ms
        }))
    })
    .await
    .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
}
