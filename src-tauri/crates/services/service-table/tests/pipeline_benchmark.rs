use service_table::{recognize_table_crop, wrap_table_html_with_content};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_table_pipeline_benchmark() {
    let model_path = Path::new(r"C:\dev\ai-forge\models\service-table\SLANet_plus.onnx");
    let dict_path = Path::new(r"C:\dev\ai-forge\models\service-table\table_structure_dict.txt");
    let real_img_path = Path::new(r"C:\dev\ai-forge\test\fixtures\service-table\1.png");
    let out_dir = Path::new(r"C:\dev\ai-forge\test\outs\service-table");

    if !model_path.exists() || !dict_path.exists() {
        println!("⚠️ 未找到物理模型/字典文件，请检查模型路径");
        return;
    }

    if !real_img_path.exists() {
        println!("⚠️ 未找到真实测试图片: {:?}", real_img_path);
        return;
    }

    fs::create_dir_all(out_dir).expect("无法创建输出目录");

    println!("\n🎯 开始对真实图片 1.png 进行极速打靶测试...");

    let img = image::open(real_img_path)
        .expect("无法打开真实图片 1.png")
        .to_rgb8();

    println!("📷 真实图像尺寸: {}x{} 像素", img.width(), img.height());

    let start = Instant::now();
    let res = recognize_table_crop(&img, model_path, dict_path, Some("cpu"))
        .expect("真实图片 1.png 识别失败");
    let elapsed = start.elapsed();

    println!("⚡ 真实表格单图解析耗时: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("🎉 综合置信度得分: {:.4}", res.score);
    println!("🧩 识别到的 HTML Token 数量: {}", res.structure_tokens.len());
    println!("📐 识别到的单元格 BBox/Grid 数量: {}", res.cells.len());

    let html = wrap_table_html_with_content(&res.structure_tokens, &[]);

    let out_html_path = out_dir.join("1_structure.html");
    fs::write(&out_html_path, &html).expect("保存 HTML 失败");
    println!("✅ 结构 HTML 保存至: {:?}", out_html_path);

    let json_str = serde_json::to_string_pretty(&res).expect("序列化 JSON 失败");
    let out_json_path = out_dir.join("1_result.json");
    fs::write(&out_json_path, &json_str).expect("保存 JSON 失败");
    println!("✅ 强类型结构 JSON 保存至: {:?}", out_json_path);

    println!("\n📄 解构导出的 HTML 结构片段:\n{}", html);
}
