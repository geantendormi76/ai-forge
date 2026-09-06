use crate::errors::OrtInferError;
use ort::ep::ExecutionProviderDispatch;
use ort::session::builder::{GraphOptimizationLevel, SessionBuilder};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static CUDA_DLL_REGISTERED: AtomicBool = AtomicBool::new(false);

/// 🛡️ Windows Native 工业级：自愈解压 cuda12.zip 并注册 CUDA 动态链接库搜索空间
pub fn ensure_cuda_dll_registered() {
    if CUDA_DLL_REGISTERED.load(Ordering::Relaxed) {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        // 1. 自动嗅探自解压容器
        auto_extract_cuda_zip_if_needed();

        let mut candidates: Vec<PathBuf> = Vec::new();

        // A. 第一最高优先级：主程序所在目录及其 Data 目录
        if let Ok(exe_p) = std::env::current_exe() {
            if let Some(exe_dir) = exe_p.parent() {
                candidates.push(exe_dir.to_path_buf());
                candidates.push(exe_dir.join("bin").join("cuda12"));
                candidates.push(exe_dir.join("cuda12"));
                candidates.push(exe_dir.join("Data").join("bin").join("cuda12"));
                candidates.push(exe_dir.join("resources").join("bin").join("cuda12"));
            }
        }

        // B. 第二优先级：规范 LocalAppData 数据目录
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            candidates.push(PathBuf::from(&local_appdata).join("紫电AI").join("Data").join("bin").join("cuda12"));
            candidates.push(PathBuf::from(&local_appdata).join("Programs").join("紫电AI").join("bin").join("cuda12"));
            candidates.push(PathBuf::from(&local_appdata).join("Programs").join("紫电AI"));
        }

        // C. 第三优先级：全域公共共享开发目录 (C:\dev\bin\cuda12) 与历史项目备用
        candidates.push(PathBuf::from(r"C:\dev\bin\cuda12"));
        candidates.push(PathBuf::from(r"C:\dev\ai-forge\bin\cuda12"));

        // D. 第四优先级：系统 CUDA_PATH
        if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
            candidates.push(PathBuf::from(cuda_path).join("bin"));
        }

        for c in candidates {
            if c.is_dir() && c.join("cublas64_12.dll").exists() && c.join("cublasLt64_12.dll").exists() {
                if let Ok(abs_p) = c.canonicalize() {
                    let abs_str = abs_p.to_string_lossy().trim_start_matches(r"\\?\").to_string();
                    let current_path = std::env::var("PATH").unwrap_or_default();
                    if !current_path.contains(&abs_str) {
                        std::env::set_var("PATH", format!("{};{}", abs_str, current_path));
                    }
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

/// 首次运行自愈解压 cuda12.zip 容器至用户数据目录
fn auto_extract_cuda_zip_if_needed() {
    let mut zip_candidates = Vec::new();
    if let Ok(exe_p) = std::env::current_exe() {
        if let Some(exe_dir) = exe_p.parent() {
            zip_candidates.push(exe_dir.join("cuda12.zip"));
            zip_candidates.push(exe_dir.join("bin").join("cuda12.zip"));
            zip_candidates.push(exe_dir.join("resources").join("cuda12.zip"));
        }
    }
    zip_candidates.push(PathBuf::from(r"C:\dev\bin\cuda12.zip"));
    zip_candidates.push(PathBuf::from(r"C:\dev\ai-forge\bin\cuda12.zip"));

    let target_extract_dir = if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("紫电AI").join("Data").join("bin").join("cuda12")
    } else {
        PathBuf::from("Data").join("bin").join("cuda12")
    };

    if target_extract_dir.join("cublas64_12.dll").exists() && target_extract_dir.join("cublasLt64_12.dll").exists() {
        return;
    }

    for zip_p in zip_candidates {
        if zip_p.exists() {
            log::info!("📦 [容器自愈] 首次运行检测到 cuda12.zip，正在极速释放 GPU 算子底座: {:?}", zip_p);
            let _ = std::fs::create_dir_all(&target_extract_dir);
            if let Ok(file) = std::fs::File::open(&zip_p) {
                if let Ok(mut archive) = zip::ZipArchive::new(file) {
                    let _ = archive.extract(&target_extract_dir);
                    log::info!("🎉 [容器自愈] 18 个 CUDA 12 核心库已成功释放至: {:?}", target_extract_dir);
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
            optimization_level: Some(GraphOptimizationLevel::Level1),
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

        let opt_level = self.optimization_level.unwrap_or(GraphOptimizationLevel::Level1);
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
