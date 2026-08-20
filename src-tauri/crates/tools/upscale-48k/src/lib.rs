// 🛡️ 4K/8K 视觉超分 - 独立工具适配与 IPC 网关门面 (lib.rs)
// 遵循单一职责原则：编排 service-upscale、VramTokenGuard 显存锁与 Gatekeeper 算力鉴权

pub use service_upscale::{UpscaleConfig, UpscaleResult, UpscaleTask};
use core_security::gatekeeper::Gatekeeper;
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
        tracing::info!("🛡️ [upscale-48k] 触发算力门控与显存守卫: {:?}", task.input_path);
        Gatekeeper::check_permission("tool-upscale-48k").await?;
        UpscaleService::run_upscale(&task, Some(&vram_guard), progress_cb).await
    }
}
