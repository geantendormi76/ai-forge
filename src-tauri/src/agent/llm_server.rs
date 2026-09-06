// 🛡️ 紫电 AI - 本地大模型推理基座后台守护服务 (llm_server.rs)
// 2026 SOTA 热拔插生命周期中枢：动态权重切换、显存秒级释放与多模态眼球动态挂载
use super::model_hub::GgufModelHub;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static CURRENT_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);

pub struct LlmServer;

impl LlmServer {
    /// 自动定位本地打包或系统全局的 llama-server.exe 执行程序
    pub fn resolve_exe_path() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from(r"C:\dev\bin\llama\llama-server.exe"),
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\llama-server.exe"),
            PathBuf::from(r"C:\dev\ai-forge\bin\llama-server.exe"),
            PathBuf::from(r"bin\llama-server.exe"),
            PathBuf::from(r"llama-server.exe"),
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    /// 热拔插核心：终止旧推理引擎，释放显存，按新模型物理路径与眼球拉起新服务
    pub fn restart_with_model(model_path: &Path, mmproj_path: Option<&Path>) -> Result<(), String> {
        let exe_path = Self::resolve_exe_path()
            .ok_or_else(|| "未找到 llama-server.exe 执行文件，请检查 bin 目录".to_string())?;

        let mut lock = CURRENT_PROCESS.lock().map_err(|e| format!("抢占 LlmServer 进程锁失败: {e}"))?;

        // 1. 安全杀死旧进程并归还显存
        if let Some(mut old_child) = lock.take() {
            let pid = old_child.id();
            tracing::info!("🛑 [LlmServer] 正在终止旧推理引擎进程 (PID: {})...", pid);
            let _ = old_child.kill();
            let _ = old_child.wait();
            // 毫秒级留白，确保 Windows 显卡驱动释放 CUDA 上下文
            std::thread::sleep(std::time::Duration::from_millis(600));
            tracing::info!("✅ [LlmServer] 旧显存与端口已完全释放");
        }

        tracing::info!("🚀 [LlmServer] 正在拉起新模型推理引擎 (127.0.0.1:8000)...");
        tracing::info!("🧠 [LlmServer] 主脑权重: {:?}", model_path);
        if let Some(mm) = mmproj_path {
            tracing::info!("👁️ [LlmServer] 挂载多模态视觉眼球: {:?}", mm);
        } else {
            tracing::info!("💬 [LlmServer] 未挂载视觉眼球，以纯语言模式推导");
        }

        let mut cmd = std::process::Command::new(&exe_path);
        let mut args: Vec<String> = vec![
            "-m".into(), model_path.to_string_lossy().to_string(),
            "--n-gpu-layers".into(), "99".into(),
            "--load-mode".into(), "none".into(),
            "--jinja".into(),
            "--cache-type-k".into(), "q4_0".into(),
            "--cache-type-v".into(), "q4_0".into(),
            "--flash-attn".into(), "on".into(),
            "--temp".into(), "0.6".into(),
            "--top-p".into(), "0.95".into(),
            "--top-k".into(), "20".into(),
            "--min-p".into(), "0.05".into(),
            "--parallel".into(), "1".into(),
            "--presence-penalty".into(), "0.0".into(),
            "--repeat-penalty".into(), "1.05".into(),
            "-n".into(), "-1".into(),
            "--host".into(), "127.0.0.1".into(),
            "--port".into(), "8000".into(),
            "--timeout".into(), "36000".into(),
            "--sse-ping-interval".into(), "15".into(),
            "-t".into(), "6".into(),
            "-tb".into(), "6".into(),
            "--ctx-size".into(), "16384".into(),
            "-b".into(), "2048".into(),
            "-ub".into(), "1024".into(),
        ];

        if let Some(mm) = mmproj_path {
            args.push("--mmproj".into());
            args.push(mm.to_string_lossy().to_string());
        }

        cmd.args(&args);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW: 静默后台运行
        }

        let child = cmd.spawn().map_err(|e| format!("启动 llama-server 失败: {e}"))?;
        let pid = child.id();
        crate::agent::process_guardian::protect_child_pid(Some(pid));
        tracing::info!("✅ [LlmServer] 新模型推理进程已点火运行 (PID: {})", pid);

        *lock = Some(child);
        Ok(())
    }

    /// 应用冷启动：自动根据当前持久化选中的模型 ID 拉起后台推理服务
    pub fn start() {
        std::thread::spawn(move || {
            let active_id = GgufModelHub::get_current_active_model_id();
            let models = GgufModelHub::scan_local_models(Some(&active_id));

            let target_model = models.iter().find(|m| m.id == active_id)
                .or_else(|| models.first());

            if let Some(m) = target_model {
                let model_path = PathBuf::from(&m.file_path);
                let mmproj_path = m.mmproj_path.as_ref().map(PathBuf::from);
                if let Err(e) = Self::restart_with_model(&model_path, mmproj_path.as_deref()) {
                    tracing::error!("🚨 [LlmServer] 启动模型失败: {e}");
                }
            } else {
                tracing::warn!("⚠️ [LlmServer] 未在 models 目录中检测到任何可用 GGUF 主脑模型");
            }
        });
    }
}
