use crate::hybrid::HybridEngine;
use crate::idp::ZipContainerExporter;
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PdfParseResult {
    pub success: bool,
    pub route: String,
    pub route_label: String,
    pub markdown: String,
    pub download_zip_url: Option<String>,
    pub elapsed_ms: u128,
    pub error: Option<String>,
}

pub struct PdfParseService;

impl PdfParseService {
    pub async fn run_parse(
        file_path_str: &str,
        output_dir: &Path,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<PdfParseResult, String> {
        let t0 = Instant::now();
        let pdf_path = Path::new(file_path_str);

        if !pdf_path.exists() {
            return Err(format!("物理文件不存在: {:?}", pdf_path));
        }

        // 🛡️ 端侧显存防爆评估
        let _permit = if let Some(guard) = vram_guard {
            match guard.try_acquire(TaskWeight::Medium) {
                Some(permit) => {
                    tracing::info!("🎮 [GPU 显存安全] 成功锁定 3,000MB 显存 Token，分配 GPU 推理");
                    Some(permit)
                }
                None => {
                    tracing::warn!("⚠️ [显存紧张/不足] 显存守卫拦截，自动降级为 CPU 安全模式运行");
                    None
                }
            }
        } else {
            None
        };

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let rand_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() % 100000;

        let job_id = format!("parse_{}_{}", now_sec, rand_suffix);
        let task_out_dir = output_dir.join(&job_id);
        let _ = tokio::fs::create_dir_all(&task_out_dir).await;

        match HybridEngine::run_pdf(pdf_path, &task_out_dir).await {
            Ok(res) => {
                let zip_filename = format!("{}.zip", job_id);
                let zip_path = output_dir.join(&zip_filename);

                let download_zip_url = if let Some(out_md) = &res.output_md_path {
                    let images_dir = task_out_dir.join("images");
                    if ZipContainerExporter::create_zip_package(out_md, &images_dir, &zip_path)
                        .await
                        .is_ok()
                    {
                        // 🛡️ 优化为本地相对 API 路径，解决 Pages 静态主机 404 坑点
                        Some(format!("/api/v1/outputs/{}", zip_filename))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let route_label = format!(
                    "🧠 智能分流: CPU 矢量轨 ({}页) + GPU 视觉轨 ({}页)",
                    res.fast_pages_count, res.deep_pages_count
                );

                Ok(PdfParseResult {
                    success: true,
                    route: "hybrid_engine".into(),
                    route_label,
                    markdown: res.markdown,
                    download_zip_url,
                    elapsed_ms: t0.elapsed().as_millis(),
                    error: None,
                })
            }
            Err(e) => Err(format!("PDF 混合解析失败: {}", e)),
        }
    }
}
