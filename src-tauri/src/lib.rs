use shared_contracts::VramTokenGuard;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tool_pdf_parse::service::{PdfParseResult, PdfParseService};

pub struct AppState {
    pub vram_guard: Arc<VramTokenGuard>,
    pub output_dir: PathBuf,
}

#[tauri::command]
async fn parse_pdf(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<PdfParseResult, String> {
    tracing::info!("🚀 收到前端 PDF 解析请求: {}", file_path);
    
    PdfParseService::run_parse(&file_path, &state.output_dir, Some(&state.vram_guard))
        .await
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
        .invoke_handler(tauri::generate_handler![parse_pdf])
        .run(tauri::generate_context!())
        .expect("🚨 启动 AI-Forge 桌面端失败");
}
