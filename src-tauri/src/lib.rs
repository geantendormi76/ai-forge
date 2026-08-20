use core_security::{
    gatekeeper::{Gatekeeper, QuotaStatus},
    DeviceFingerprint,
};
use format_converter::{service::FormatConvertService, FormatConvertResult, FormatConvertTask};
use pdf_parse::service::{PdfParseResult, PdfParseService};
use shared_contracts::VramTokenGuard;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State};
use upscale_48k::{Upscale48kTool, UpscaleResult, UpscaleTask};
use video_subtitle::{
    VideoProbeResult, VideoProbeService, VideoSubtitleOptions, VideoSubtitleResult,
    VideoSubtitleTool,
};

pub struct AppState {
    pub vram_guard: Arc<VramTokenGuard>,
    pub output_dir: PathBuf,
    pub cancel_token: Arc<AtomicBool>,
}

#[tauri::command]
async fn get_quota_status() -> Result<QuotaStatus, String> {
    Gatekeeper::get_quota_status().await
}

#[tauri::command]
async fn run_upscale_48k(
    task: UpscaleTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<UpscaleResult, String> {
    tracing::info!("🚀 收到前端 4K/8K 视觉超分请求: {:?}", task.input_path);
    
    // 纯 Token 结算：4K 超分 10 Tokens
    Gatekeeper::check_permission_tokens("upscale-48k", 10).await?;
    
    let window_clone = window.clone();
    let progress_cb = move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit(
            "upscale-progress",
            serde_json::json!({
                "current": current,
                "total": total,
                "message": msg
            }),
        );
    };

    let start_t = std::time::Instant::now();
    let res = Upscale48kTool::execute(task, state.vram_guard.clone(), Some(progress_cb)).await;
    let elapsed = start_t.elapsed().as_millis() as u64;

    Gatekeeper::report_telemetry(
        "upscale-48k",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    ).await;

    res
}

#[tauri::command]
async fn parse_pdf(
    file_path: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<PdfParseResult, String> {
    tracing::info!("🚀 收到前端 PDF 解析请求: {}", file_path);

    // 纯 Token 结算：PDF 解析 2 Tokens
    Gatekeeper::check_permission_tokens("pdf-parse", 2).await?;

    let window_clone = window.clone();
    let progress_cb = move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit(
            "pdf-parse-progress",
            serde_json::json!({
                "current": current,
                "total": total,
                "message": msg
            }),
        );
    };

    let pdf_input_path = Path::new(&file_path);
    let target_out_dir = pdf_input_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let start_t = std::time::Instant::now();
    let res = PdfParseService::run_parse(
        &file_path,
        target_out_dir,
        Some(&state.vram_guard),
        Some(progress_cb),
    ).await;
    let elapsed = start_t.elapsed().as_millis() as u64;

    Gatekeeper::report_telemetry(
        "pdf-parse",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    ).await;

    res
}

#[tauri::command]
async fn probe_video(video_path: String) -> Result<VideoProbeResult, String> {
    tracing::info!("🔍 [VideoProbe] 探测视频元信息: {}", video_path);
    let path = std::path::Path::new(&video_path);
    VideoProbeService::probe(path).await
}

#[tauri::command]
async fn run_video_subtitle(
    options: VideoSubtitleOptions,
    state: State<'_, AppState>,
) -> Result<VideoSubtitleResult, String> {
    tracing::info!("🚀 收到前端 视频双语字幕工坊请求: {}", options.video_path);

    let video_p = Path::new(&options.video_path);
    let probe_info = VideoProbeService::probe(video_p).await.ok();
    let duration_sec = probe_info.map(|p| p.duration_sec).unwrap_or(180.0);
    
    // 纯 Token 结算：按时长负载动态换算 Tokens
    let minutes = (duration_sec / 60.0).ceil() as u32;
    let tokens_needed = (minutes * 3).max(3);
    tracing::info!("⏱️ 视频时长: {:.1} 秒 ➔ 动态消耗 {} Tokens", duration_sec, tokens_needed);

    Gatekeeper::check_permission_tokens("video-subtitle", tokens_needed).await?;

    state.cancel_token.store(false, Ordering::SeqCst);

    let start_t = std::time::Instant::now();
    let res = VideoSubtitleTool::run_pipeline_cancellable(
        options,
        Some(&state.vram_guard),
        state.cancel_token.clone(),
    ).await;
    let elapsed = start_t.elapsed().as_millis() as u64;

    Gatekeeper::report_telemetry(
        "video-subtitle",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    ).await;

    res
}

#[tauri::command]
fn cancel_current_task(state: State<'_, AppState>) -> Result<bool, String> {
    tracing::warn!("🛑 [IPC 截停专线] 收到前端紧急截停请求");
    state.cancel_token.store(true, Ordering::SeqCst);
    Ok(true)
}

#[tauri::command]
fn get_hardware_fingerprint() -> Result<String, String> {
    let secret = "ai-forge-commercial-secret-2026";
    DeviceFingerprint::new()
        .add_cpu_info()
        .add_mac_address()
        .add_system_info()
        .generate(secret)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_format_convert(
    task: FormatConvertTask,
    window: tauri::Window,
) -> Result<FormatConvertResult, String> {
    tracing::info!("🚀 收到前端 全能格式转换请求 (0 Tokens): {:?}", task);
    let res = FormatConvertService::convert(&task);
    let _ = window.emit("format-convert-finished", &res);
    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let output_dir = std::env::temp_dir().join("ai_forge_outputs");
    let _ = std::fs::create_dir_all(&output_dir);

    let app_state = AppState {
        vram_guard: Arc::new(VramTokenGuard::default_rtx3060()),
        output_dir,
        cancel_token: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_quota_status,
            parse_pdf,
            probe_video,
            run_video_subtitle,
            cancel_current_task,
            get_hardware_fingerprint,
            run_format_convert,
            run_upscale_48k
        ])
        .run(tauri::generate_context!())
        .expect("🚨 启动 紫电 AI 桌面端失败");
}
