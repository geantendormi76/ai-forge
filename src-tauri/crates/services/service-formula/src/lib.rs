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

        println!("\n=== 🚀 [service-formula] 纯血 ONNX 4 图打靶压测启动 ===");

        for i in 1..=4 {
            let img_name = format!("{}.png", i);
            let img_path = fixture_dir.join(&img_name);
            if !img_path.exists() {
                println!("⚠️ [跳过] 测试图片不存在: {:?}", img_path);
                continue;
            }

            let img = open(&img_path).expect("读取图片失败").to_rgb8();
            let result = FormulaService::recognize_crop(&img, None, None)
                .unwrap_or_else(|e| panic!("图片 {} 识别失败: {}", img_name, e));

            println!(
                "🎯 图片 {} 识别成功 ➔ LaTeX:\n   \"{}\"\n   ⏱️ 耗时: {:.2} ms | Tokens: {}",
                img_name,
                result.latex,
                result.elapsed_ms,
                result.raw_token_ids.len()
            );

            let out_json_path = output_dir.join(format!("{}_formula.json", i));
            let json_content = serde_json::to_string_pretty(&result).unwrap();
            fs::write(&out_json_path, json_content).unwrap();
        }

        println!("💾 全部 4 张公式图打靶产物物理落盘至: {:?}", output_dir);
    }
}
