pub mod agent;
pub mod agent_bridge;
pub mod logging;

use agent::model_hub::{GgufModelHub, GgufModelInfo};
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

pub struct AgentSessionManager {
    pub bridge: tokio::sync::Mutex<Option<agent_bridge::AgentBridge>>,
}

fn clean_path_str(p: &Path) -> String {
    p.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

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
    let deps = ModelManager::get_tool_dependencies(&base_dir, &tool_id).await;
    state.cancel_token.store(false, Ordering::SeqCst);
    for item in deps {
        if item.is_ready { continue; }
        let target_path = base_dir.join(&item.relative_path);
        let window_clone = window.clone();
        let tool_id_clone = tool_id.clone();
        let item_id = item.id.clone();
        let item_name = item.name.clone();
        let progress_cb = move |downloaded: u64, total: u64, phase: &str| {
            let percent = if total > 0 { ((downloaded as f64 / total as f64) * 100.0).min(100.0) as u32 } else { 0 };
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
        ModelManager::download_dependency_file(&target_path, &item, state.cancel_token.clone(), progress_cb).await
            .map_err(|e| format!("下载依赖 [{}] 失败: {}", item.name, e))?;
    }
    Ok(true)
}

#[tauri::command]
fn cancel_dependency_downloads(state: State<'_, AppState>) -> Result<bool, String> {
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
        let _ = window_clone.emit("upscale-progress", serde_json::json!({ "current": current, "total": total, "message": msg }));
    };
    Upscale48kTool::execute(task, state.vram_guard.clone(), Some(progress_cb)).await
}

#[tauri::command]
async fn parse_pdf(
    file_path: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<PdfParseResult, String> {
    Gatekeeper::check_permission_tokens("pdf-parse", 2).await?;
    let window_clone = window.clone();
    let progress_cb = move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit("pdf-parse-progress", serde_json::json!({ "current": current, "total": total, "message": msg }));
    };
    let pdf_input_path = Path::new(&file_path);
    let target_out_dir = pdf_input_path.parent().unwrap_or_else(|| Path::new("."));
    PdfParseService::run_parse(&file_path, target_out_dir, Some(&state.vram_guard), Some(progress_cb)).await
}

#[tauri::command]
async fn probe_video(video_path: String) -> Result<VideoProbeResult, String> {
    let path = std::path::Path::new(&video_path);
    VideoProbeService::probe(path).await
}

#[tauri::command]
async fn run_video_subtitle(
    options: VideoSubtitleOptions,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<VideoSubtitleResult, String> {
    state.cancel_token.store(false, Ordering::SeqCst);
    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit("video-subtitle-progress", serde_json::json!({ "current": current, "total": total, "message": msg }));
    });
    VideoSubtitleTool::run_pipeline_with_progress(options, Some(&state.vram_guard), state.cancel_token.clone(), Some(progress_cb)).await
}

#[tauri::command]
async fn run_translation(
    task: TranslationTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<TranslationResult, String> {
    state.cancel_token.store(false, Ordering::SeqCst);
    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit("translation-progress", serde_json::json!({ "current": current, "total": total, "message": msg }));
    });
    TranslationTool::execute(task, Some(&state.vram_guard), Some(state.cancel_token.clone()), Some(progress_cb)).await
}

#[tauri::command]
async fn run_image_translation(
    task: ImageTranslationTask,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<ImageTranslationResult, String> {
    state.cancel_token.store(false, Ordering::SeqCst);
    let window_clone = window.clone();
    let progress_cb = Arc::new(move |current: usize, total: usize, msg: &str| {
        let _ = window_clone.emit("translation-progress", serde_json::json!({ "current": current, "total": total, "message": msg }));
    });
    TranslationTool::execute_image_ocr(task, Some(&state.vram_guard), Some(state.cancel_token.clone()), Some(progress_cb)).await
}

#[tauri::command]
fn cancel_current_task(state: State<'_, AppState>) -> Result<bool, String> {
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
    let res = FormatConvertService::convert(&task);
    let _ = window.emit("format-convert-finished", &res);
    Ok(res)
}

/// 扫描本地所有 GGUF 模型资产及视觉眼球配对情况
#[tauri::command]
fn get_local_gguf_models() -> Result<Vec<GgufModelInfo>, String> {
    let active_id = GgufModelHub::get_current_active_model_id();
    Ok(GgufModelHub::scan_local_models(Some(&active_id)))
}

/// 一键热拔插切换主脑模型并秒级重载推理引擎
#[tauri::command]
async fn switch_gguf_model(
    model_id: String,
    agent_mgr: State<'_, AgentSessionManager>,
) -> Result<GgufModelInfo, String> {
    log::info!("🔄 [ModelHub] 收到一键热拔插切换主脑请求: {}", model_id);
    let target = GgufModelHub::switch_active_model(&model_id)?;

    // 1. 重启后台 llama-server.exe
    let model_p = PathBuf::from(&target.file_path);
    let mm_p = target.mmproj_path.as_ref().map(PathBuf::from);
    agent::llm_server::LlmServer::restart_with_model(&model_p, mm_p.as_deref())?;

    // 2. 释放旧的桥接进程，下一次用户提问时自动绑定新模型
    let mut guard = agent_mgr.bridge.lock().await;
    if let Some(ref bridge) = *guard {
        let _ = bridge.abort().await;
    }
    *guard = None;

    log::info!("🎉 [ModelHub] 成功热拔插切换至新模型: {}", target.id);
    Ok(target)
}

/// 辅助函数：拉起新的 Pi Bridge 实例并监听事件
async fn spawn_fresh_bridge(
    vram_guard: Arc<VramTokenGuard>,
    window: &tauri::Window,
) -> Result<agent_bridge::AgentBridge, String> {
    let ext_path = std::path::PathBuf::from(r"C:\dev\ai-forge\.pi\extensions\ai_forge_suite.ts");
    let (bridge, mut rx) = agent_bridge::AgentBridge::spawn(
        vram_guard,
        Some(&ext_path),
        None,
    )
    .await
    .map_err(|e| format!("启动智能体引擎失败: {e}"))?;

    let win_clone = window.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = win_clone.emit("agent-event", &event);
        }
    });

    Ok(bridge)
}

#[tauri::command]
async fn send_agent_prompt(
    prompt: String,
    window: tauri::Window,
    state: State<'_, AppState>,
    agent_mgr: State<'_, AgentSessionManager>,
) -> Result<bool, String> {
    log::info!("🤖 [Agent] 收到用户智能体任务: {}", prompt);
    let mut guard = agent_mgr.bridge.lock().await;

    // 1. 若当前没有桥接，立即拉起新进程
    if guard.is_none() {
        let fresh_bridge = spawn_fresh_bridge(state.vram_guard.clone(), &window).await?;
        *guard = Some(fresh_bridge);
    }

    // 2. 尝试向当前进程发送，若遭遇死管道则自动秒级自愈重启
    let mut retry_needed = false;
    if let Some(ref bridge) = *guard {
        if let Err(e) = bridge.send_prompt(&prompt, None).await {
            log::warn!("⚠️ [Agent] 检测到原有管道已失效 ({e}), 正在自动自愈重启 Pi 引擎...");
            retry_needed = true;
        }
    }

    // 3. 自愈流程：销毁旧实例，拉起全新进程重试下发
    if retry_needed {
        *guard = None;
        let fresh_bridge = spawn_fresh_bridge(state.vram_guard.clone(), &window).await?;
        fresh_bridge.send_prompt(&prompt, None).await.map_err(|e| format!("重连后下发任务依然失败: {e}"))?;
        *guard = Some(fresh_bridge);
        log::info!("✅ [Agent] Pi 引擎已成功自愈重连，任务已顺利下发！");
    }

    Ok(true)
}

#[tauri::command]
async fn abort_agent_task(
    agent_mgr: State<'_, AgentSessionManager>,
) -> Result<bool, String> {
    log::warn!("🛑 [Agent] 收到前端紧急截停智能体指令");
    let mut guard = agent_mgr.bridge.lock().await;
    if let Some(ref bridge) = *guard {
        let _ = bridge.abort().await;
    }
    *guard = None;
    Ok(true)
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

    let agent_session_manager = AgentSessionManager {
        bridge: tokio::sync::Mutex::new(None),
    };

    agent::tool_server::ToolRpcServer::start(app_state.vram_guard.clone());
    agent::llm_server::LlmServer::start();

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
        .manage(agent_session_manager)
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
            run_image_translation,
            send_agent_prompt,
            abort_agent_task,
            get_local_gguf_models,
            switch_gguf_model
        ])
        .setup(move |_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("🚨 启动 紫电 AI 桌面端失败");
}
