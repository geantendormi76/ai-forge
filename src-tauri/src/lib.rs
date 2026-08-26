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
use tauri_plugin_log::{Builder as LogBuilder, RotationStrategy, Target, TargetKind};
use tauri_plugin_updater::UpdaterExt;
use upscale_48k::{Upscale48kTool, UpscaleResult, UpscaleTask};
use video_subtitle::{
    VideoProbeResult, VideoProbeService, VideoSubtitleOptions, VideoSubtitleResult,
    VideoSubtitleTool,
};
use translation::{
    ImageTranslationResult, ImageTranslationTask, TranslationResult, TranslationTask,
    TranslationTool,
};

pub struct AppState {
    pub vram_guard: Arc<VramTokenGuard>,
    pub output_dir: PathBuf,
    pub cancel_token: Arc<AtomicBool>,
}

fn clean_path_str(p: &Path) -> String {
    p.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

/// 解析日志物理根目录
pub fn resolve_log_dir_path() -> PathBuf {
    if let Some(portable_logs) = PortableEngine::resolve_data_path("logs") {
        let _ = std::fs::create_dir_all(&portable_logs);
        return portable_logs;
    }
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

#[tauri::command]
fn is_portable() -> bool {
    PortableEngine::is_portable()
}

#[tauri::command]
fn get_log_dir_path() -> Result<String, String> {
    Ok(clean_path_str(&resolve_log_dir_path()))
}

#[tauri::command]
async fn open_log_dir(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_opener::OpenerExt;
    let log_dir = resolve_log_dir_path();
    let dir_str = clean_path_str(&log_dir);
    app.opener()
        .open_path(dir_str, None::<&str>)
        .map_err(|e| format!("打开日志文件夹失败: {e}"))?;
    Ok(true)
}

#[tauri::command]
fn get_diagnostic_report() -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push("==================================================".to_string());
    lines.push("⚡ 紫电 AI 客户端白盒诊断与运行黑匣子报告".to_string());
    lines.push("==================================================".to_string());
    lines.push(format!("• 操作系统: {} ({})", std::env::consts::OS, std::env::consts::ARCH));
    lines.push(format!("• 运行模式: {}", if PortableEngine::is_portable() { "便携免安装模式 (./Data)" } else { "标准模式 (%LOCALAPPDATA%)" }));
    lines.push(format!("• 日志物理目录: {}", clean_path_str(&resolve_log_dir_path())));

    #[cfg(target_os = "windows")]
    {
        let nvcuda = Path::new(r"C:\Windows\System32\nvcuda.dll").exists();
        let vcruntime = Path::new(r"C:\Windows\System32\vcruntime140.dll").exists();
        lines.push(format!("• NVIDIA 显卡驱动: {}", if nvcuda { "✅ 正常 (nvcuda.dll 在线)" } else { "❌ 缺失/未检测到独显" }));
        lines.push(format!("• 微软 VC++ 运行库: {}", if vcruntime { "✅ 正常" } else { "⚠️ 缺失" }));
    }

    let base_dir = ModelManager::resolve_models_base_dir();
    lines.push(format!("• 模型物理基准区: {}", clean_path_str(&base_dir)));

    let cuda_runtime = ModelManager::resolve_cuda_runtime_dir();
    lines.push(format!("• CUDA 动态库目录: {}", clean_path_str(&cuda_runtime)));
    lines.push("--------------------------------------------------".to_string());

    lines.push("📜 【最近 40 行运行流水日志】:".to_string());
    let log_dir = resolve_log_dir_path();
    let mut log_files: Vec<PathBuf> = std::fs::read_dir(&log_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "log"))
                .collect()
        })
        .unwrap_or_default();

    log_files.sort_by_key(|p| p.metadata().and_then(|m| m.modified()).ok());

    if let Some(latest_log) = log_files.last() {
        if let Ok(content) = std::fs::read_to_string(latest_log) {
            let log_lines: Vec<&str> = content.lines().collect();
            let start_idx = log_lines.len().saturating_sub(40);
            for l in &log_lines[start_idx..] {
                lines.push(format!("  {}", l));
            }
        } else {
            lines.push("  (暂无流水日志内容)".to_string());
        }
    } else {
        lines.push("  (当前尚未生成日志文件)".to_string());
    }

    lines.push("==================================================".to_string());
    Ok(lines.join("\n"))
}

#[tauri::command]
async fn install_app_update(app: tauri::AppHandle) -> Result<bool, String> {
    let updater = app.updater().map_err(|e| format!("初始化更新器失败: {e}"))?;
    if let Some(update) = updater.check().await.map_err(|e| format!("版本探针探测异常: {e}"))? {
        let app_handle = app.clone();
        log::info!("🚀 [更新中台] 发现可用更新: v{}, 正在流式拉取并验签...", update.version);
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
                    log::info!("🎉 [更新中台] 升级包写入完成，准备重启进程");
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
    log::info!("🔍 [依赖检测] 目标功能: '{}', 基准目录: {}", tool_id, clean_path_str(&base_dir));
    Ok(ModelManager::get_tool_dependencies(&base_dir, &tool_id).await)
}

#[tauri::command]
async fn download_tool_dependencies(
    tool_id: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let base_dir = ModelManager::resolve_models_base_dir();
    log::info!("🌐 [依赖下载] 开始拉取依赖: '{}', 落盘目录: {}", tool_id, clean_path_str(&base_dir));
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
    log::warn!("🛑 [依赖下载] 收到前端紧急取消指令");
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
    log::info!("🚀 [4K/8K 超分] 收到前端请求: 源图像='{}'", task.input_path);
    if task.model_path.is_none() {
        let base_dir = ModelManager::resolve_models_base_dir();
        let candidate = base_dir.join("service-upscale").join("RealESRGAN_x4plus.onnx");
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
    )
    .await;
    res
}

#[tauri::command]
async fn parse_pdf(
    file_path: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<PdfParseResult, String> {
    log::info!("🚀 [PDF 解析] 收到解析请求: 文件='{}'", file_path);
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
    let target_out_dir = pdf_input_path.parent().unwrap_or_else(|| Path::new("."));
    let start_t = std::time::Instant::now();
    let res = PdfParseService::run_parse(
        &file_path,
        target_out_dir,
        Some(&state.vram_guard),
        Some(progress_cb),
    )
    .await;
    let elapsed = start_t.elapsed().as_millis() as u64;
    Gatekeeper::report_telemetry(
        "pdf-parse",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    )
    .await;
    res
}

#[tauri::command]
async fn probe_video(video_path: String) -> Result<VideoProbeResult, String> {
    log::info!("🔍 [视频探测] 探测视频元信息: '{}'", video_path);
    let path = std::path::Path::new(&video_path);
    VideoProbeService::probe(path).await
}

#[tauri::command]
async fn run_video_subtitle(
    options: VideoSubtitleOptions,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<VideoSubtitleResult, String> {
    log::info!("🚀 [视频字幕] 收到任务: 视频='{}'", options.video_path);
    let video_p = Path::new(&options.video_path);
    let probe_info = VideoProbeService::probe(video_p).await.ok();
    let duration_sec = probe_info.map(|p| p.duration_sec).unwrap_or(180.0);
    let minutes = (duration_sec / 60.0).ceil() as u32;
    let tokens_needed = (minutes * 3).max(3);
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
    )
    .await;
    let elapsed = start_t.elapsed().as_millis() as u64;
    Gatekeeper::report_telemetry(
        "video-subtitle",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    )
    .await;
    res
}

/// 🛡️ 离线高精文本翻译指令 (100% 免费 · 0 Token 扣减)
#[tauri::command]
async fn run_translation(
    task: TranslationTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<TranslationResult, String> {
    state.cancel_token.store(false, Ordering::SeqCst);
    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit(
            "translation-progress",
            serde_json::json!({
                "current": current,
                "total": total,
                "message": msg
            }),
        );
    });
    let start_t = std::time::Instant::now();
    let res = TranslationTool::execute(
        task,
        Some(&state.vram_guard),
        Some(state.cancel_token.clone()),
        Some(progress_cb),
    )
    .await;
    let elapsed = start_t.elapsed().as_millis() as u64;
    Gatekeeper::report_telemetry(
        "translation",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    )
    .await;
    res
}

/// 🛡️ 图像与剪贴板截图 OCR 高精翻译指令 (100% 免费 · 0 Token 扣减)
#[tauri::command]
async fn run_image_translation(
    task: ImageTranslationTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<ImageTranslationResult, String> {
    state.cancel_token.store(false, Ordering::SeqCst);
    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit(
            "translation-progress",
            serde_json::json!({
                "current": current,
                "total": total,
                "message": msg
            }),
        );
    });
    let start_t = std::time::Instant::now();
    let res = TranslationTool::execute_image_ocr(
        task,
        Some(&state.vram_guard),
        Some(state.cancel_token.clone()),
        Some(progress_cb),
    )
    .await;
    let elapsed = start_t.elapsed().as_millis() as u64;
    Gatekeeper::report_telemetry(
        "translation_ocr",
        elapsed,
        res.is_ok(),
        res.as_ref().err().map(|e| e.as_str()),
    )
    .await;
    res
}

#[tauri::command]
fn cancel_current_task(state: State<'_, AppState>) -> Result<bool, String> {
    log::warn!("🛑 [IPC] 收到前端紧急任务截停指令");
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
    log::info!("🚀 [格式转换] 启动转换: 源文件='{}', 目标格式='{}'", task.input_path, task.target_format);
    let res = FormatConvertService::convert(&task);
    let _ = window.emit("format-convert-finished", &res);
    Ok(res)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    PortableEngine::init();
    core_onnx_infer::config::ensure_cuda_dll_registered();

    let logs_dir = resolve_log_dir_path();
    let output_dir = PortableEngine::resolve_data_path("outputs")
        .unwrap_or_else(|| std::env::temp_dir().join("ai_forge_outputs"));
    let _ = std::fs::create_dir_all(&output_dir);

    let app_state = AppState {
        vram_guard: Arc::new(VramTokenGuard::default_rtx3060()),
        output_dir,
        cancel_token: Arc::new(AtomicBool::new(false)),
    };

    let log_plugin = LogBuilder::new()
        .level(log::LevelFilter::Info)
        .max_file_size(5_000_000)
        .rotation_strategy(RotationStrategy::KeepOne)
        .clear_targets()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::Folder {
                path: logs_dir.clone(),
                file_name: Some("zidian".into()),
            }),
        ])
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(log_plugin)
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            is_portable,
            get_quota_status,
            get_log_dir_path,
            open_log_dir,
            get_diagnostic_report,
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
            install_app_update,
            run_translation,
            run_image_translation
        ])
        .setup(move |_app| {
            log::info!("======================================================================");
            log::info!("🚀 [紫电 AI] 工业级桌面工坊已点火启动 (生产黑匣子日志已就绪)");
            log::info!("📁 [日志物理目录] {}", clean_path_str(&logs_dir));
            log::info!("🖥️ [系统环境] OS: {} | Arch: {}", std::env::consts::OS, std::env::consts::ARCH);
            log::info!("======================================================================\n");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("🚨 启动 紫电 AI 桌面端失败");
}
