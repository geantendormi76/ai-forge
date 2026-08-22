// 🛡️ 4K/8K 视觉超分 - 独立工具适配与 IPC 网关门面 (lib.rs)
// 遵循单一职责原则：编排 service-upscale 与 VramTokenGuard 显存锁

pub use service_upscale::{UpscaleConfig, UpscaleResult, UpscaleTask};
use service_upscale::UpscaleService;
use shared_contracts::VramTokenGuard;
use std::sync::Arc;

pub struct Upscale48kTool;

impl Upscale48kTool {
    pub async fn execute<F>(
        task: UpscaleTask,
        vram_guard: Arc<VramTokenGuard>,
        progress_cb: Option<F>,
    ) -> Result<UpscaleResult, String>
    where
        F: Fn(usize, usize, &str) + Send + Sync + 'static,
    {
        tracing::info!("🛡️ [upscale-48k] 启动硬件超分流水线: {:?}", task.input_path);
        UpscaleService::run_upscale(&task, Some(&vram_guard), progress_cb).await
    }
}
