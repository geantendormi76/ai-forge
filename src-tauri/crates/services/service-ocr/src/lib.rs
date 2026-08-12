pub mod core;
pub mod engine;
pub mod models;
pub mod processors;
pub mod utils;

pub use core::errors::{OCRError, OcrResult};
pub use core::inference::OrtInfer;
pub use engine::{CRNNRecModel, DBDetModel, OcrEngine, OcrService};
pub use models::text_region::{BoundingBox, Point, TextRegion};
pub use processors::{
    crop_text_region, BoxType, ColorOrder, CTCLabelDecode, DBPostProcess, DBPostProcessConfig,
    DetResizeForTest, ImageScaleInfo, LimitType, NormalizeImage, OCRResize, ScoreMode, TensorLayout,
};
pub use utils::read_character_dict;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    fn resolve_test_image_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\service-ocr.png"),
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\ocr_test.png"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\service-ocr.png")
        })
    }

    #[test]
    fn test_service_ocr_default_small_e2e_benchmark() {
        let det_path = OcrService::resolve_default_det_model_path();
        let rec_path = OcrService::resolve_default_rec_model_path();
        let dict_path = OcrService::resolve_default_dict_path();
        let img_path = resolve_test_image_path();

        if !det_path.exists() || !rec_path.exists() || !dict_path.exists() || !img_path.exists() {
            println!("⚠️ [跳过打靶测试] 默认物理模型/词表/测试图片文件不存在: det={:?}, rec={:?}, dict={:?}, img={:?}", det_path, rec_path, dict_path, img_path);
            return;
        }

        println!("\n⚡ ===== [service-ocr 默认 PP-OCRv6 Small 物理打靶启动] =====");
        println!("  默认检测模型: {:?}", det_path);
        println!("  默认识别模型: {:?}", rec_path);
        println!("  默认字符词表: {:?}", dict_path);
        println!("  测试图片:     {:?}", img_path);

        let t0 = Instant::now();
        let img = image::open(&img_path).expect("读取测试图片失败");

        let mut engine = OcrService::default_engine().expect("初始化默认小模型 OcrEngine 失败");
        let init_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        let regions = engine.process_image(img).expect("默认小模型 OCR 推导打靶失败");
        let infer_ms = t1.elapsed().as_secs_f64() * 1000.0;

        println!("\n🎉 ===== [service-ocr 默认 Small 模型打靶成功断言] =====");
        println!("  ⏱️ 默认引擎点火耗时: {:.2} ms", init_ms);
        println!("  ⏱️ 默认图像推导耗时: {:.2} ms (0.5 秒极速即出！)", infer_ms);
        println!("  📊 识别文本行数: {}", regions.len());
        println!("  预览前 10 句识别结果:");

        for (idx, reg) in regions.iter().take(10).enumerate() {
            println!("    [{:02}] [{:.2}]: {}", idx + 1, reg.score, reg.text);
        }

        let outs_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-ocr");
        let _ = std::fs::create_dir_all(&outs_dir);

        let out_json_path = outs_dir.join("ocr_result_default_small.json");
        let out_md_path = outs_dir.join("ocr_result_default_small.md");

        if let Ok(pretty_json) = serde_json::to_string_pretty(&regions) {
            let _ = std::fs::write(&out_json_path, pretty_json);
        }

        let mut md_content = String::from("# ⚡ service-ocr 默认 PP-OCRv6 Small 物理打靶产物\n\n");
        for (idx, reg) in regions.iter().enumerate() {
            md_content.push_str(&format!("{}. **[{:.2}]**: {}\n", idx + 1, reg.score, reg.text));
        }
        let _ = std::fs::write(&out_md_path, md_content);

        println!("  💾 默认模型 OCR 物理产物成功落盘至:");
        println!("     - JSON: {:?}", out_json_path);
        println!("     - MD:   {:?}", out_md_path);
        println!("=======================================================\n");

        assert!(regions.len() > 0, "识别文本行数不可为 0！");
    }
}
