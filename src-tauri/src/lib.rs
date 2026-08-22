use core_models_download::{DependencyItem, DependencyProgressPayload, ModelManager};
use core_security::{
    gatekeeper::{Gatekeeper, QuotaStatus},
    DeviceFingerprint, PortableEngine,
};
use format_converter::{service::FormatConvertService, FormatConvertResult, FormatConvertTask};
use pdf_parse::service::{PdfParseResult, PdfParseService};
use shared_contracts::VramTokenGuard;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State};
use tauri_plugin_updater::UpdaterExt;
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
fn is_portable() -> bool {
    PortableEngine::is_portable()
}

#[tauri::command]
async fn install_app_update(app: tauri::AppHandle) -> Result<bool, String> {
    let updater = app.updater().map_err(|e| format!("初始化更新器失败: {e}"))?;
    if let Some(update) = updater.check().await.map_err(|e| format!("版本探针探测异常: {e}"))? {
        let app_handle = app.clone();
        tracing::info!("🚀 发现可用更新: v{}, 正在由 Rust 核心流式拉取并验签...", update.version);
        let mut downloaded_bytes: u64 = 0;
        update
            .download_and_install(
                move |chunk_length, content_length| {
                    downloaded_bytes += chunk_length as u64;
                    let total = content_length.unwrap_or(0);
                    let percent = if total > 0 {
                        ((downloaded_bytes as f64 / total as f64) * 100.0) as u32
                    } else {
                        0
                    };
                    let _ = app_handle.emit(
                        "app-update-progress",
                        serde_json::json!({
                            "downloaded": downloaded_bytes,
                            "total": total,
                            "percent": percent
                        }),
                    );
                },
                || {
                    tracing::info!("🎉 [Rust Updater] 升级包写入完成，准备重启进程");
                },
            )
            .await
            .map_err(|e| format!("下载与安装更新包失败: {e}"))?;
        app.restart();
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn check_tool_dependencies(tool_id: String) -> Result<Vec<DependencyItem>, String> {
    let base_dir = ModelManager::resolve_models_base_dir();
    tracing::info!("🔍 [依赖检测] 命中模型基准寻址根目录: {:?}", base_dir);
    Ok(ModelManager::get_tool_dependencies(&base_dir, &tool_id).await)
}

#[tauri::command]
async fn download_tool_dependencies(
    tool_id: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let base_dir = ModelManager::resolve_models_base_dir();
    tracing::info!("🌐 [依赖下载] 目标落盘基准目录: {:?}", base_dir);
    let deps = ModelManager::get_tool_dependencies(&base_dir, &tool_id).await;
    state.cancel_token.store(false, Ordering::SeqCst);
    for item in deps {
        if item.is_ready {
            continue;
        }
        let target_path = base_dir.join(&item.relative_path);
        let window_clone = window.clone();
        let tool_id_clone = tool_id.clone();
        let item_id = item.id.clone();
        let item_name = item.name.clone();
        let progress_cb = move |downloaded: u64, total: u64, phase: &str| {
            let percent = if total > 0 {
                ((downloaded as f64 / total as f64) * 100.0).min(100.0) as u32
            } else {
                0
            };
            let payload = DependencyProgressPayload {
                tool_id: tool_id_clone.clone(),
                item_id: item_id.clone(),
                item_name: item_name.clone(),
                downloaded_bytes: downloaded,
                total_bytes: total,
                percent,
                phase: phase.to_string(),
            };
            let _ = window_clone.emit("dependency-download-progress", payload);
        };
        ModelManager::download_dependency_file(
            &target_path,
            &item,
            state.cancel_token.clone(),
            progress_cb,
        )
        .await
        .map_err(|e| format!("下载依赖 [{}] 失败: {}", item.name, e))?;
    }
    Ok(true)
}

#[tauri::command]
fn cancel_dependency_downloads(state: State<'_, AppState>) -> Result<bool, String> {
    tracing::warn!("🛑 [依赖下载] 收到前端紧急取消下载指令");
    state.cancel_token.store(true, Ordering::SeqCst);
    Ok(true)
}

#[tauri::command]
async fn get_quota_status() -> Result<QuotaStatus, String> {
    Gatekeeper::get_quota_status().await
}

#[tauri::command]
async fn run_upscale_48k(
    mut task: UpscaleTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<UpscaleResult, String> {
    tracing::info!("🚀 收到前端 4K/8K 视觉超分请求: {:?}", task.input_path);
    if task.model_path.is_none() {
        let base_dir = ModelManager::resolve_models_base_dir();
        let candidate = base_dir.join("service-upscale/RealESRGAN_x4plus.onnx");
        if candidate.exists() {
            task.model_path = Some(candidate.to_string_lossy().to_string());
        }
    }
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
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<VideoSubtitleResult, String> {
    tracing::info!("🚀 收到前端 视频双语字幕工坊请求: {}", options.video_path);
    let video_p = Path::new(&options.video_path);
    let probe_info = VideoProbeService::probe(video_p).await.ok();
    let duration_sec = probe_info.map(|p| p.duration_sec).unwrap_or(180.0);
    let minutes = (duration_sec / 60.0).ceil() as u32;
    let tokens_needed = (minutes * 3).max(3);
    tracing::info!("⏱️ 视频时长: {:.1} 秒 ➔ 动态消耗 {} Tokens", duration_sec, tokens_needed);
    Gatekeeper::check_permission_tokens("video-subtitle", tokens_needed).await?;
    state.cancel_token.store(false, Ordering::SeqCst);

    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit(
            "video-subtitle-progress",
            serde_json::json!({
                "current": current,
                "total": total,
                "message": msg
            }),
        );
    });

    let start_t = std::time::Instant::now();
    let res = VideoSubtitleTool::run_pipeline_with_progress(
        options,
        Some(&state.vram_guard),
        state.cancel_token.clone(),
        Some(progress_cb),
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
    PortableEngine::init();
    let output_dir = PortableEngine::resolve_data_path("outputs")
        .unwrap_or_else(|| std::env::temp_dir().join("ai_forge_outputs"));
    let _ = std::fs::create_dir_all(&output_dir);
    let app_state = AppState {
        vram_guard: Arc::new(VramTokenGuard::default_rtx3060()),
        output_dir,
        cancel_token: Arc::new(AtomicBool::new(false)),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            is_portable,
            get_quota_status,
            parse_pdf,
            probe_video,
            run_video_subtitle,
            cancel_current_task,
            get_hardware_fingerprint,
            run_format_convert,
            run_upscale_48k,
            check_tool_dependencies,
            download_tool_dependencies,
            cancel_dependency_downloads,
            install_app_update
        ])
        .run(tauri::generate_context!())
        .expect("🚨 启动 紫电 AI 桌面端失败");
}
