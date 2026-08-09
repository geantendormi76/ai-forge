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
    use std::path::{Path, PathBuf};

    fn resolve_fixture_path(filename: &str) -> PathBuf {
        let candidates = [
            PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", filename)),
            PathBuf::from(format!(r"C:\dev\ai-toolkit\test\fixtures\{}", filename)),
            PathBuf::from(format!("/home/zhz/ai-forge/test/fixtures/{}", filename)),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", filename))
    }

    fn resolve_outs_dir() -> PathBuf {
        let candidate = PathBuf::from(r"C:\dev\ai-forge\test\outs\tool-pdf-parse");
        let _ = std::fs::create_dir_all(&candidate);
        candidate
    }

    #[tokio::test]
    async fn test_pdf_parse_e2e_with_vram_guard() {
        let fast_pdf_buf = resolve_fixture_path("tool-pdf-parse-fast.pdf");
        let deep_pdf_buf = resolve_fixture_path("tool-pdf-parse-deep.pdf");
        let out_dir = resolve_outs_dir();

        if !fast_pdf_buf.exists() || !deep_pdf_buf.exists() {
            println!("⚠️ [跳过测试] PDF 测试基准文件不存在");
            return;
        }

        let fast_pdf = fast_pdf_buf.to_string_lossy().to_string();
        let deep_pdf = deep_pdf_buf.to_string_lossy().to_string();

        let vram_guard = VramTokenGuard::default_rtx3060();

        println!("\n🔍 [1/2] 正在带显存守卫压测 CPU 矢量轨: {} ...", fast_pdf);
        let res_fast = PdfParseService::run_parse(&fast_pdf, &out_dir, Some(&vram_guard)).await;
        assert!(res_fast.is_ok(), "CPU 矢量轨解析失败: {:?}", res_fast.err());
        let fast_val = res_fast.unwrap();
        assert!(fast_val.success);
        println!("✅ [CPU 矢量轨成功] 耗时: {}ms, 剩余显存额度: {}MB", fast_val.elapsed_ms, vram_guard.available_vram_mb());

        println!("🔍 [2/2] 正在带显存守卫压测 GPU 视觉轨: {} ...", deep_pdf);
        let res_deep = PdfParseService::run_parse(&deep_pdf, &out_dir, Some(&vram_guard)).await;
        assert!(res_deep.is_ok(), "GPU 视觉轨解析失败: {:?}", res_deep.err());
        let deep_val = res_deep.unwrap();
        assert!(deep_val.success);
        println!("✅ [GPU 视觉轨成功] 耗时: {}ms, 剩余显存额度: {}MB", deep_val.elapsed_ms, vram_guard.available_vram_mb());
    }
}
