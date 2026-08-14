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
    use std::path::PathBuf;

    #[test]
    fn test_layout_preprocess_shape() {
        let img = image::RgbImage::new(100, 200);
        let (tensor, scale_w, scale_h) = LayoutPreprocess::preprocess_image(&img, (800, 800));

        assert_eq!(tensor.shape(), &[1, 3, 800, 800]);
        assert_eq!(scale_w, 8.0);
        assert_eq!(scale_h, 4.0);
    }

    #[test]
    fn test_page1_real_layout_detection() {
        let model_path = PathBuf::from(r"C:\dev\ai-forge\models\service-layout\PP-DocLayoutV3.onnx");
        let img_path = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-pdfium\page_1_300dpi.png");

        if !model_path.exists() || !img_path.exists() {
            println!("⚠️ [跳过测试] 未找到模型或第 1 页测试图像: {:?}, {:?}", model_path, img_path);
            return;
        }

        let mut service = LayoutService::new(&model_path, None, None).expect("加载 ONNX 失败");
        let result = service.detect_file(&img_path).expect("执行版面分析失败");

        println!("\n🚀 ===== [service-layout TDD 单一打靶测试] =====");
        println!("  🖼️ 测试图像: {:?} ({}x{})", img_path, result.image_width, result.image_height);
        println!("  ⏱️ 推理耗时: {:.2} ms", result.elapsed_ms);
        println!("  📊 检测到区块总数: {}", result.regions.len());

        for r in &result.regions {
            println!("    - [{:02}] {:<18} (类别: {:02}) | 得分: {:.3} | 坐标: [{:.1}, {:.1}, {:.1}, {:.1}]",
                r.id, r.label, r.category as usize, r.score, r.bbox.x1, r.bbox.y1, r.bbox.x2, r.bbox.y2);
        }

        let has_doc_title = result.regions.iter().any(|r| r.category == LayoutCategory::DocTitle);
        println!("  🎯 大标题 doc_title 检出断言: {}", if has_doc_title { "✅ 成功检出" } else { "❌ 未检出" });
        assert!(has_doc_title, "必须准确检测到大标题 doc_title (类别 6)！");
        println!("=================================================\n");
    }
}
