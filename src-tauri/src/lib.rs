use core_security::{gatekeeper::Gatekeeper, DeviceFingerprint};
use format_converter::{service::FormatConvertService, FormatConvertResult, FormatConvertTask};
use pdf_parse::service::{PdfParseResult, PdfParseService};
use shared_contracts::VramTokenGuard;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};
use tool_video_subtitle::{VideoSubtitleOptions, VideoSubtitleResult, VideoSubtitleTool};

pub struct AppState {
    pub vram_guard: Arc<VramTokenGuard>,
    pub output_dir: PathBuf,
}

#[tauri::command]
async fn parse_pdf(
    file_path: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<PdfParseResult, String> {
    tracing::info!("🚀 收到前端 紫电 AI 本地 PDF 解析请求: {}", file_path);

    // 🛡️ [Gatekeeper 算力收费站] 鉴权与指纹检查
    Gatekeeper::check_permission("tool-pdf-parse").await?;

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

    PdfParseService::run_parse(
        &file_path,
        &state.output_dir,
        Some(&state.vram_guard),
        Some(progress_cb),
    )
    .await
}

#[tauri::command]
async fn run_video_subtitle(
    options: VideoSubtitleOptions,
    state: State<'_, AppState>,
) -> Result<VideoSubtitleResult, String> {
    tracing::info!("🚀 收到前端 紫电 AI 视频双语字幕工坊请求: {}", options.video_path);

    Gatekeeper::check_permission("tool-video-subtitle").await?;

    VideoSubtitleTool::run_pipeline(options, Some(&state.vram_guard)).await
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
    tracing::info!("🚀 收到前端 紫电 AI 全能格式转换请求: {:?}", task);
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
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            parse_pdf,
            run_video_subtitle,
            get_hardware_fingerprint,
            run_format_convert
        ])
        .run(tauri::generate_context!())
        .expect("🚨 启动 紫电 AI 桌面端失败");
}
