pub mod contracts;
pub mod postprocess;
pub mod preprocess;
pub mod service;

pub use contracts::{FormulaConfig, FormulaResult};
pub use postprocess::{normalize_latex, FormulaPostprocessor};
pub use preprocess::FormulaPreprocessor;
pub use service::FormulaService;

#[cfg(test)]
mod tests {
    use super::*;
    use image::open;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_formula_pipeline_real_images() {
        let fixture_dir = PathBuf::from(r"C:\dev\ai-forge\test\fixtures\service-formula");
        let output_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-formula");
        let _ = fs::create_dir_all(&output_dir);

        println!("\n=== 🚀 [service-formula] 纯血 ONNX 单图 1.png 打靶压测启动 ===");

        let img_path = fixture_dir.join("1.png");
        assert!(img_path.exists(), "测试图片 1.png 物理文件必须存在");

        let img = open(&img_path).expect("读取 1.png 失败").to_rgb8();

        let result = FormulaService::recognize_crop(&img, None, None).expect("1.png 识别必须成功");

        println!(
            "🎯 图片 1.png 识别成功 ➔ LaTeX: \"{}\" | 耗时: {:.2} ms | Tokens 数量: {}",
            result.latex,
            result.elapsed_ms,
            result.raw_token_ids.len()
        );

        // 落盘 JSON 产物
        let out_json_path = output_dir.join("1_formula.json");
        let json_content = serde_json::to_string_pretty(&result).unwrap();
        fs::write(&out_json_path, json_content).unwrap();

        println!("💾 单图打靶产物物理落盘至: {:?}", out_json_path);
    }
}
