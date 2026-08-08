use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

pub const TOTAL_VRAM_POOL_MB: usize = 11_000;
pub const TOKEN_UNIT_MB: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskWeight {
    Light = 1,   // ~1,000 MB
    Medium = 3,  // ~3,000 MB (PDF 深度解析)
    Heavy = 8,   // ~8,000 MB (MOSS ASR 视频翻译)
}

impl TaskWeight {
    pub fn required_mb(&self) -> usize {
        match self {
            TaskWeight::Light => 1000,
            TaskWeight::Medium => 3000,
            TaskWeight::Heavy => 8000,
        }
    }

    pub fn permits(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone)]
pub struct VramTokenGuard {
    semaphore: Arc<Semaphore>,
    total_tokens: usize,
    used_vram_mb: Arc<AtomicUsize>,
}

impl VramTokenGuard {
    pub fn new(total_tokens: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(total_tokens)),
            total_tokens,
            used_vram_mb: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn default_rtx3060() -> Self {
        Self::new(TOTAL_VRAM_POOL_MB / TOKEN_UNIT_MB)
    }

    pub async fn acquire(&self, weight: TaskWeight) -> Result<VramPermit<'_>, String> {
        let permits_needed = weight.permits();
        let mb_needed = weight.required_mb();

        let permit = self
            .semaphore
            .acquire_many(permits_needed)
            .await
            .map_err(|e| format!("显存守卫已关闭: {e}"))?;

        self.used_vram_mb.fetch_add(mb_needed, Ordering::SeqCst);

        Ok(VramPermit {
            _permit: permit,
            guard: self,
            weight,
        })
    }

    /// 🛡️ 端侧核心：非阻塞尝试申请！直接借用 self.semaphore 避免临时变量生命周期悬垂
    pub fn try_acquire(&self, weight: TaskWeight) -> Option<VramPermit<'_>> {
        let permits_needed = weight.permits();
        let mb_needed = weight.required_mb();

        match self.semaphore.try_acquire_many(permits_needed) {
            Ok(permit) => {
                self.used_vram_mb.fetch_add(mb_needed, Ordering::SeqCst);
                Some(VramPermit {
                    _permit: permit,
                    guard: self,
                    weight,
                })
            }
            Err(_) => None, // 显存不足，触发 CPU 降级防护
        }
    }

    pub fn available_vram_mb(&self) -> usize {
        let total_mb = self.total_tokens * TOKEN_UNIT_MB;
        let used = self.used_vram_mb.load(Ordering::SeqCst);
        total_mb.saturating_sub(used)
    }
}

pub struct VramPermit<'a> {
    _permit: SemaphorePermit<'a>,
    guard: &'a VramTokenGuard,
    pub weight: TaskWeight,
}

impl<'a> Drop for VramPermit<'a> {
    fn drop(&mut self) {
        let mb = self.weight.required_mb();
        self.guard.used_vram_mb.fetch_sub(mb, Ordering::SeqCst);
    }
}
