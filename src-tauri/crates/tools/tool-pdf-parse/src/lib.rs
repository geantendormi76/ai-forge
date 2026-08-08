pub mod deep_track;
pub mod fast_track;
pub mod formula;
pub mod hybrid;
pub mod idp;
pub mod probe;
pub mod service;

pub use service::PdfParseService;

#[cfg(test)]
mod tests {
    use super::*;
    use shared_contracts::VramTokenGuard;
    use std::path::Path;

    #[tokio::test]
    async fn test_pdf_parse_e2e_with_vram_guard() {
        let fast_pdf = "/home/zhz/ai-forge/test/fixtures/tool-pdf-parse-fast.pdf";
        let deep_pdf = "/home/zhz/ai-forge/test/fixtures/tool-pdf-parse-deep.pdf";
        let out_dir = Path::new("/home/zhz/ai-forge/test/tool-pdf-parse/outs");

        // 初始化端侧显存守卫
        let vram_guard = VramTokenGuard::default_rtx3060();

        println!("\n🔍 [1/2] 正在带显存守卫压测 CPU 矢量轨: tool-pdf-parse-fast.pdf ...");
        let res_fast = PdfParseService::run_parse(fast_pdf, out_dir, Some(&vram_guard)).await;
        assert!(res_fast.is_ok(), "CPU 矢量轨解析失败: {:?}", res_fast.err());
        let fast_val = res_fast.unwrap();
        assert!(fast_val.success);
        println!("✅ [CPU 矢量轨成功] 耗时: {}ms, 剩余显存额度: {}MB", fast_val.elapsed_ms, vram_guard.available_vram_mb());

        println!("🔍 [2/2] 正在带显存守卫压测 GPU 视觉轨: tool-pdf-parse-deep.pdf ...");
        let res_deep = PdfParseService::run_parse(deep_pdf, out_dir, Some(&vram_guard)).await;
        assert!(res_deep.is_ok(), "GPU 视觉轨解析失败: {:?}", res_deep.err());
        let deep_val = res_deep.unwrap();
        assert!(deep_val.success);
        println!("✅ [GPU 视觉轨成功] 耗时: {}ms, 剩余显存额度: {}MB", deep_val.elapsed_ms, vram_guard.available_vram_mb());
    }
}
