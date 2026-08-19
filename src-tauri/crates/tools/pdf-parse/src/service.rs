use crate::hybrid::HybridEngine;
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 🖥️ 桌面端 Dedicated PDF 解析交付契约 (零 Web / 零 ZIP 遗毒)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PdfParseResult {
    pub success: bool,
    pub markdown: String,
    pub output_md_path: String,
    pub task_out_dir: String,
    pub images_dir: String,
    pub route_label: String,
    pub elapsed_ms: u128,
    pub error: Option<String>,
}

pub struct PdfParseService;

impl PdfParseService {
    pub async fn run_parse<F>(
        file_path_str: &str,
        output_dir: &Path,
        vram_guard: Option<&VramTokenGuard>,
        _on_progress: Option<F>,
    ) -> Result<PdfParseResult, String>
    where
        F: Fn(usize, usize, &str) + Send + Sync + 'static,
    {
        let t0 = Instant::now();
        let pdf_path = Path::new(file_path_str);

        if !pdf_path.exists() {
            return Err(format!("物理文件不存在: {:?}", pdf_path));
        }

        // 🛡️ 端侧显存防爆评估与并发锁保护
        let _permit = if let Some(guard) = vram_guard {
            match guard.try_acquire(TaskWeight::Medium) {
                Some(permit) => {
                    tracing::info!("🎮 [GPU 显存安全] 成功锁定 3,000MB 显存 Token");
                    Some(permit)
                }
                None => {
                    tracing::warn!("⚠️ [显存不足] 显存守卫拦截，自动降级为 CPU 安全模式运行");
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
                let out_md_path_str = res
                    .output_md_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let images_dir_str = task_out_dir.join("images").to_string_lossy().to_string();

                let route_label = format!(
                    "🧠 智能分流: 矢量轨 ({}页) + 扫描轨 ({}页)",
                    res.fast_pages_count, res.deep_pages_count
                );

                Ok(PdfParseResult {
                    success: true,
                    markdown: res.markdown,
                    output_md_path: out_md_path_str,
                    task_out_dir: task_out_dir.to_string_lossy().to_string(),
                    images_dir: images_dir_str,
                    route_label,
                    elapsed_ms: t0.elapsed().as_millis(),
                    error: None,
                })
            }
            Err(e) => Err(format!("PDF 混合解析失败: {}", e)),
        }
    }
}
