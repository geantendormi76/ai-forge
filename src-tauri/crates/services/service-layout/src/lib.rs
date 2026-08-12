pub mod contracts;
pub mod postprocess;
pub mod preprocess;
pub mod service;

pub use contracts::*;
pub use postprocess::LayoutPostProcess;
pub use preprocess::LayoutPreprocess;
pub use service::LayoutService;

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;
    use ndarray::Array2;

    #[test]
    fn test_layout_preprocess_shape() {
        let img = RgbImage::new(100, 200);
        let (tensor, scale_w, scale_h) = LayoutPreprocess::preprocess_image(&img, (800, 800));

        assert_eq!(tensor.shape(), &[1, 3, 800, 800]);
        assert_eq!(scale_w, 4.0);
        assert_eq!(scale_h, 4.0);
    }

    #[test]
    fn test_layout_postprocess_dummy_data() {
        let mut predictions = Array2::<f32>::zeros((2, 7));
        predictions[[0, 0]] = 0.0;
        predictions[[0, 1]] = 0.9;
        predictions[[0, 2]] = 100.0;
        predictions[[0, 3]] = 100.0;
        predictions[[0, 4]] = 500.0;
        predictions[[0, 5]] = 200.0;
        predictions[[0, 6]] = 1.0;

        predictions[[1, 0]] = 5.0;
        predictions[[1, 1]] = 0.85;
        predictions[[1, 2]] = 100.0;
        predictions[[1, 3]] = 300.0;
        predictions[[1, 4]] = 900.0;
        predictions[[1, 5]] = 800.0;
        predictions[[1, 6]] = 2.0;

        let config = LayoutConfig::default();
        let regions = LayoutPostProcess::process_pp_doclayout_2d(predictions.view(), 1000.0, 1000.0, &config);

        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].category, LayoutCategory::Title);
        assert_eq!(regions[0].score, 0.9);
        assert_eq!(regions[0].bbox.x1, 100.0);
        assert_eq!(regions[0].bbox.y1, 100.0);

        assert_eq!(regions[1].category, LayoutCategory::Table);
        assert_eq!(regions[1].score, 0.85);
    }

    #[test]
    fn test_batch_pdf_pages_layout_inference() {
        let model_path = std::path::PathBuf::from(r"C:\dev\ai-forge\models\service-layout\PP-DocLayoutV3.onnx");
        let pages_dir = std::path::PathBuf::from(r"C:\dev\ai-forge\test\fixtures\pdf_pages");
        let out_dir = std::path::PathBuf::from(r"C:\dev\ai-forge\test\outs\service-layout");

        if !model_path.exists() || !pages_dir.exists() {
            println!("⚠️ 跳过批处理打靶：未找到模型 {:?} 或页面目录 {:?}", model_path, pages_dir);
            return;
        }

        let _ = std::fs::create_dir_all(&out_dir);
        let mut service = LayoutService::new(&model_path, None, None).expect("加载 ONNX 失败");

        println!("\n🚀 开始执行 4 页 PDF 渲染图全量版面分析打靶...");

        for i in 1..=4 {
            let img_name = format!("page_{}.png", i);
            let img_path = pages_dir.join(&img_name);
            if !img_path.exists() {
                continue;
            }

            let result = service.detect_file(&img_path).expect("执行版面分析失败");

            println!("\n  📄 [页面 {}/4] 图像: {} ({}x{}) | 耗时: {:.2} ms | 检测到 {} 个版面区块",
                i, img_name, result.image_width, result.image_height, result.elapsed_ms, result.regions.len());

            for r in &result.regions {
                println!("    - [{}] 类别: {:<12} | 得分: {:.2} | 坐标: [{:.1}, {:.1}, {:.1}, {:.1}]",
                    r.id, r.label, r.score, r.bbox.x1, r.bbox.y1, r.bbox.x2, r.bbox.y2);
            }

            let out_json_path = out_dir.join(format!("page_{}_layout.json", i));
            let json_data = serde_json::to_string_pretty(&result).expect("序列化 JSON 失败");
            std::fs::write(&out_json_path, json_data).expect("落盘 JSON 失败");
        }

        println!("\n💾 4 页 PDF 版面分析结果已全量落盘至: {:?}", out_dir);
    }
}
