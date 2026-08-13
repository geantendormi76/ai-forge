use service_doc_parse::DocParseEngine;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[test]
fn test_doc_parse_pipeline_benchmark() {
    let models_root = Path::new(r"C:\dev\ai-forge\models");
    let pages_dir = Path::new(r"C:\dev\ai-forge\test\fixtures\pdf_pages");
    let out_dir = Path::new(r"C:\dev\ai-forge\test\outs\service-doc-parse");

    if !pages_dir.exists() {
        println!("⚠️ 未找到 pdf_pages 测试测试目录: {:?}", pages_dir);
        return;
    }

    fs::create_dir_all(out_dir).expect("无法创建输出目录");

    println!("\n🚀 开始 IDP 智能排版引擎 (service-doc-parse) 4 页 PDF 全量真实打靶测试...");

    let mut engine = DocParseEngine::default_engine();

    for i in 1..=4 {
        let img_name = format!("page_{}.png", i);
        let img_path = pages_dir.join(&img_name);
        if !img_path.exists() {
            continue;
        }

        let dyn_img = image::open(&img_path).expect("无法打开图片");

        let start = Instant::now();
        let (doc, markdown) = engine.parse_image(&dyn_img).expect("IDP 页解析失败");
        let elapsed = start.elapsed();

        println!("\n📄 [页面 {}/4] 图像: {} | 耗时: {:.2} ms | 提取排版块: {} 个",
            i, img_name, elapsed.as_secs_f64() * 1000.0, doc.elements.len());

        for (idx, elem) in doc.elements.iter().enumerate() {
            let sample_content = elem.content.lines().next().unwrap_or("<空>");
            let trunc_content = if sample_content.len() > 40 { &sample_content[..40] } else { sample_content };
            println!("   [{:02}] 类别: {:<15} | 顺序: {:>2} | 预览: {}",
                idx + 1, elem.layout_type.as_str(), elem.order_index, trunc_content);
        }

        let out_md_path = out_dir.join(format!("page_{}_doc.md", i));
        fs::write(&out_md_path, &markdown).expect("保存 Markdown 失败");

        let json_str = serde_json::to_string_pretty(&doc).expect("序列化 JSON 失败");
        let out_json_path = out_dir.join(format!("page_{}_doc.json", i));
        fs::write(&out_json_path, &json_str).expect("保存 JSON 失败");
    }

    println!("\n💾 4 页 PDF 真实文档全量 IDP 排版成果已成功落盘至: {:?}", out_dir);
}
