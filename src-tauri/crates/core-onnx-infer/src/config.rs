use crate::errors::OrtInferError;
use ort::ep::ExecutionProviderDispatch;
use ort::session::builder::{GraphOptimizationLevel, SessionBuilder};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static CUDA_DLL_REGISTERED: AtomicBool = AtomicBool::new(false);

/// 🛡️ Windows Native 工业级：多级自愈探测并向 Windows 内核注册 CUDA 动态链接库搜索空间
pub fn ensure_cuda_dll_registered() {
    if CUDA_DLL_REGISTERED.load(Ordering::Relaxed) {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let mut candidates: Vec<PathBuf> = Vec::new();

        // 1. 🌟 第一最高优先级：当前主程序 exe 所在同级根目录
        if let Ok(exe_p) = std::env::current_exe() {
            if let Some(exe_dir) = exe_p.parent() {
                candidates.push(exe_dir.to_path_buf());
                candidates.push(exe_dir.join("bin").join("cuda12"));
                candidates.push(exe_dir.join("cuda12"));
                candidates.push(exe_dir.join("Data").join("bin").join("cuda12"));
                candidates.push(exe_dir.join("resources").join("bin").join("cuda12"));
            }
        }

        // 2. 第二优先级：便携与规范统一路径
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local_appdata).join("紫电AI").join("Data").join("bin").join("cuda12"));
            candidates.push(PathBuf::from(&local_appdata).join("Programs").join("紫电AI"));
            candidates.push(PathBuf::from(&local_appdata).join("ZiDianAI").join("bin").join("cuda12"));
        }

        // 3. 第三优先级：开发绝对工作区目录
        candidates.push(PathBuf::from(r"C:\dev\ai-forge\bin\cuda12"));
        candidates.push(PathBuf::from(r"bin\cuda12"));
        candidates.push(PathBuf::from(r"..\bin\cuda12"));

        // 4. 第四优先级：系统全局 CUDA_PATH 环境变量
        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            candidates.push(PathBuf::from(cuda_path).join("bin"));
        }

        for c in candidates {
            if c.is_dir() && (c.join("cublas64_13.dll").exists() || c.join("cublas64_12.dll").exists()) {
                if let Ok(abs_p) = c.canonicalize() {
                    let abs_str = abs_p.to_string_lossy().trim_start_matches(r"\\?\").to_string();

                    // A. 注入环境变量 PATH
                    let current_path = std::env::var("PATH").unwrap_or_default();
                    if !current_path.contains(&abs_str) {
                        std::env::set_var("PATH", format!("{};{}", abs_str, current_path));
                    }

                    // B. 使用 Windows 核心 API SetDllDirectoryW 强行注入动态链接库搜索列表
                    #[cfg(windows)]
                    unsafe {
                        use std::os::windows::ffi::OsStrExt;
                        let wide_path: Vec<u16> = std::ffi::OsStr::new(&abs_str)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();
                        extern "system" {
                            fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
                        }
                        let _ = SetDllDirectoryW(wide_path.as_ptr());
                    }

                    CUDA_DLL_REGISTERED.store(true, Ordering::SeqCst);
                    log::info!("🎮 [CUDA 深度并网] 成功将动态库目录注入进程搜索空间: {}", abs_str);
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrtExecutionProvider {
    CPU,
    CUDA { device_id: i32 },
    DirectML { device_id: i32 },
}

#[derive(Debug, Clone)]
pub struct OrtSessionConfig {
    pub execution_providers: Vec<OrtExecutionProvider>,
    pub intra_threads: Option<usize>,
    pub inter_threads: Option<usize>,
    pub parallel_execution: Option<bool>,
    pub optimization_level: Option<GraphOptimizationLevel>,
    pub enable_memory_pattern: Option<bool>,
}

impl Default for OrtSessionConfig {
    fn default() -> Self {
        Self {
            execution_providers: vec![
                OrtExecutionProvider::CUDA { device_id: 0 },
            ],
            intra_threads: None,
            inter_threads: None,
            parallel_execution: Some(false),
            optimization_level: Some(GraphOptimizationLevel::Level3),
            enable_memory_pattern: Some(false),
        }
    }
}

pub fn parse_device_config(device: &str) -> Result<OrtSessionConfig, OrtInferError> {
    let dev = device.to_lowercase();
    if dev == "cpu" {
        return Ok(OrtSessionConfig {
            execution_providers: vec![OrtExecutionProvider::CPU],
            ..Default::default()
        });
    }
    if dev.starts_with("cuda") {
        let id = dev
            .strip_prefix("cuda:")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        return Ok(OrtSessionConfig {
            execution_providers: vec![
                OrtExecutionProvider::CUDA { device_id: id },
            ],
            ..Default::default()
        });
    }
    Ok(OrtSessionConfig::default())
}

impl OrtSessionConfig {
    pub fn for_control_flow() -> Self {
        Self {
            execution_providers: vec![OrtExecutionProvider::CPU],
            intra_threads: None,
            inter_threads: None,
            parallel_execution: Some(false),
            optimization_level: Some(GraphOptimizationLevel::Level1),
            enable_memory_pattern: Some(false),
        }
    }

    pub fn build_session_builder(&self) -> Result<SessionBuilder, OrtInferError> {
        ensure_cuda_dll_registered();

        let mut builder = SessionBuilder::new().map_err(|e| OrtInferError::ModelLoad {
            path: "ONNX SessionBuilder".into(),
            context: format!("Failed to create builder: {e}"),
        })?;

        let opt_level = self.optimization_level.unwrap_or(GraphOptimizationLevel::Level3);
        builder = builder
            .with_optimization_level(opt_level)
            .map_err(|e| OrtInferError::ModelLoad {
                path: "ONNX SessionBuilder".into(),
                context: format!("Failed to set optimization level: {e}"),
            })?;

        if let Some(enable_mem) = self.enable_memory_pattern {
            builder = builder
                .with_memory_pattern(enable_mem)
                .map_err(|e| OrtInferError::ModelLoad {
                    path: "ONNX SessionBuilder".into(),
                    context: format!("Failed to set memory pattern: {e}"),
                })?;
        }

        let mut dispatches: Vec<ExecutionProviderDispatch> = Vec::new();
        for ep in &self.execution_providers {
            match ep {
                OrtExecutionProvider::CPU => {
                    dispatches.push(ort::ep::CPU::default().build());
                    log::warn!("⚠️ [ONNX Session] 未启用 CUDA，使用 CPU 软解模式");
                }
                #[cfg(feature = "cuda")]
                OrtExecutionProvider::CUDA { device_id } => {
                    use ort::ep::cuda::ConvAlgorithmSearch;
                    let mut cuda_ep = ort::ep::CUDA::default().with_device_id(*device_id);
                    cuda_ep = cuda_ep.with_conv_algorithm_search(ConvAlgorithmSearch::Heuristic);
                    dispatches.push(cuda_ep.build());
                    log::info!("🎮 [ONNX Session] 成功挂载纯血 CUDA Execution Provider (Device ID: {device_id})");
                }
                _ => {}
            }
        }

        if !dispatches.is_empty() {
            builder = builder
                .with_execution_providers(dispatches)
                .map_err(|e| OrtInferError::ModelLoad {
                    path: "ONNX SessionBuilder".into(),
                    context: format!("Failed to set execution providers: {e}"),
                })?;
        }

        Ok(builder)
    }
}
