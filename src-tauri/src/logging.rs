//! 🛡️ 紫电 AI - 2026 SOTA 工业级日志与硬件黑匣子中台 (logging.rs)
//! 支持双轨输出 (Console + 每日自动切片落盘)、非阻塞高性能队列与开机硬件自愈诊断

use core_security::PortableEngine;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

/// 获取规范化的日志物理存储根目录 (%LOCALAPPDATA%\紫电AI\Data\logs 或 ./Data/logs)
pub fn resolve_logs_dir() -> PathBuf {
    // 1. 便携模式最高优先级 (./Data/logs)
    if let Some(portable_logs) = PortableEngine::resolve_data_path("logs") {
        let _ = std::fs::create_dir_all(&portable_logs);
        return portable_logs;
    }

    // 2. 标准模式：%LOCALAPPDATA%\紫电AI\Data\logs
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let log_dir = PathBuf::from(&local_appdata)
            .join("紫电AI")
            .join("Data")
            .join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        return log_dir;
    }

    let fallback = PathBuf::from("logs");
    let _ = std::fs::create_dir_all(&fallback);
    fallback
}

/// 初始化全局双轨日志系统 (在主程序 run() 开头第 0 秒调用)
pub fn init_logging() -> Option<WorkerGuard> {
    let logs_dir = resolve_logs_dir();

    // 每日自动滚动切片文件 (例如: app.2026-08-26.log)
    let file_appender = tracing_appender::rolling::daily(&logs_dir, "app");
    let (non_blocking_file, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_thread_names(true)
        .with_writer(non_blocking_file)
        .with_filter(EnvFilter::new("info,ort=warn,core_onnx_infer=info,service_upscale=info"));

    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_filter(EnvFilter::new("info,ort=warn"));

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);

    if subscriber.try_init().is_ok() {
        write_blackbox_header(&logs_dir);
        Some(guard)
    } else {
        None
    }
}

/// 写入开机硬件黑匣子诊断头
fn write_blackbox_header(logs_dir: &std::path::Path) {
    tracing::info!("======================================================================");
    tracing::info!("🚀 [紫电 AI] 工业级桌面工坊已点火启动 (生产黑匣子已激活)");
    tracing::info!("📁 [日志物理路径] {:?}", logs_dir);
    tracing::info!("🖥️ [系统环境] OS: {} | Arch: {}", std::env::consts::OS, std::env::consts::ARCH);

    #[cfg(target_os = "windows")]
    {
        let sys32_nvcuda = std::path::Path::new(r"C:\Windows\System32\nvcuda.dll");
        if sys32_nvcuda.exists() {
            tracing::info!("🎮 [显卡驱动] NVIDIA 驱动核心 (nvcuda.dll) 在线");
        } else {
            tracing::warn!("⚠️ [显卡驱动] 未检测到 System32/nvcuda.dll，可能未安装 NVIDIA 独显驱动");
        }

        let sys32_vc = std::path::Path::new(r"C:\Windows\System32\vcruntime140.dll");
        if sys32_vc.exists() {
            tracing::info!("🧩 [VC++ 运行库] 微软 VC++ 2015-2022 基础库已就绪");
        } else {
            tracing::warn!("⚠️ [VC++ 运行库] 未检测到 vcruntime140.dll，可能需要安装运行库");
        }
    }
    tracing::info!("======================================================================\n");
}

/// 一键生成当前系统的白盒诊断报告文本 (用于前端一键复制/用户反馈)
pub fn generate_diagnostic_summary() -> String {
    let mut lines = Vec::new();
    lines.push("=== 📋 紫电 AI 客户端诊断报告 ===".to_string());
    lines.push(format!("• 操作系统: {} / {}", std::env::consts::OS, std::env::consts::ARCH));
    lines.push(format!("• 便携模式: {}", if PortableEngine::is_portable() { "已激活 (./Data)" } else { "标准模式 (%LOCALAPPDATA%)" }));
    lines.push(format!("• 日志目录: {:?}", resolve_logs_dir()));

    #[cfg(target_os = "windows")]
    {
        let nvcuda = std::path::Path::new(r"C:\Windows\System32\nvcuda.dll").exists();
        let vcruntime = std::path::Path::new(r"C:\Windows\System32\vcruntime140.dll").exists();
        lines.push(format!("• NVIDIA 驱动: {}", if nvcuda { "✅ 正常 (nvcuda.dll 在线)" } else { "❌ 异常/缺失" }));
        lines.push(format!("• VC++ 运行库: {}", if vcruntime { "✅ 正常" } else { "⚠️ 缺失" }));
    }

    let base_dir = core_models_download::ModelManager::resolve_models_base_dir();
    lines.push(format!("• 模型存储基准区: {:?}", base_dir));

    let cuda_runtime = core_models_download::ModelManager::resolve_cuda_runtime_dir();
    lines.push(format!("• CUDA 运行时目录: {:?}", cuda_runtime));

    lines.push("=================================".to_string());
    lines.join("\n")
}
