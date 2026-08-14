use service_doc_parse::DocParseEngine;
use service_pdfium::PdfiumEngine;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn resolve_fixture(filenames: &[&str]) -> Option<PathBuf> {
    for name in filenames {
        let p = PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", name));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[test]
fn test_super_doc_parse_tdd_pipeline() {
    let out_dir = Path::new(r"C:\dev\ai-forge\test\outs\service-doc-parse");
    fs::create_dir_all(out_dir).expect("无法创建测试输出目录");

    println!("\n🚀 ===== [service-doc-parse 超级底座 TDD 真实打靶测试] =====");

    let deep_pdf = resolve_fixture(&["pdf-parse-deep.pdf", "tool-pdf-parse-deep.pdf", "2.pdf"]);
    let fast_pdf = resolve_fixture(&["pdf-parse-fast.pdf", "tool-pdf-parse-fast.pdf", "1.pdf"]);

    let targets = vec![
        ("扫描件/复杂轨", deep_pdf),
        ("矢量轨", fast_pdf),
    ];

    let mut engine = DocParseEngine::default_engine();

    for (label, maybe_path) in targets {
        let Some(pdf_path) = maybe_path else {
            println!("⚠️ [TDD 拦截] 测试文件不存在: {}", label);
            continue;
        };

        println!("\n📄 正在对 [{}] 进行全流程渲染解析: {:?}", label, pdf_path);
        let t0 = Instant::now();

        let page_count = PdfiumEngine::get_page_count(&pdf_path).expect("获取页数失败");
        assert!(page_count > 0, "PDF 总页数不可为 0");

        for page_idx in 0..page_count.min(2) {
            let page_t0 = Instant::now();
            let dyn_img = PdfiumEngine::render_page_to_image(&pdf_path, page_idx, 300)
                .expect("300DPI 渲染失败");

            let (ast, markdown) = engine.parse_image_ast(&dyn_img, page_idx)
                .expect("超级底座 AST 排版解析失败");

            let elapsed_ms = page_t0.elapsed().as_secs_f64() * 1000.0;
            let file_stem = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
            let out_md_filename = format!("{}_p{}_super_out.md", file_stem, page_idx + 1);
            let out_md_path = out_dir.join(&out_md_filename);

            fs::write(&out_md_path, &markdown).expect("Markdown 落盘失败");

            println!("  ✅ [页码 {}/{}] RXYC++ 排序与 AST 渲染成功 ➔ 耗时: {:.2} ms | 落盘: {:?}",
                page_idx + 1, page_count, elapsed_ms, out_md_path);
            println!("  📝 提炼 Markdown 预览 (前 200 字):\n{}\n---",
                markdown.chars().take(200).collect::<String>());
        }

        println!("  ⏱️ [{}] 端到端打靶解析完成！总耗时: {:.2} ms", label, t0.elapsed().as_secs_f64() * 1000.0);
    }

    println!("\n🎉 ===== [service-doc-parse 超级底座 TDD 打靶全量成功] =====");
    println!("  💾 物理产物目录: {:?}", out_dir);
    println!("===========================================================\n");
}
