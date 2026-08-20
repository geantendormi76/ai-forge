use crate::errors::OrtInferError;
use ort::ep::ExecutionProviderDispatch;
use ort::session::builder::{GraphOptimizationLevel, SessionBuilder};
use std::path::Path;
use std::sync::Once;

static INIT_CUDA_DLL_DIR: Once = Once::new();

/// 🛡️ Windows Native 专属：自动探测并向进程注入 bin/cuda12 动态链接库搜索目录
pub fn ensure_cuda_dll_registered() {
    INIT_CUDA_DLL_DIR.call_once(|| {
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                r"C:\dev\ai-forge\bin\cuda12",
                r"bin\cuda12",
                r"..\bin\cuda12",
                r"..\..\bin\cuda12",
            ];
            for c in candidates {
                let p = Path::new(c);
                if p.exists() {
                    if let Ok(abs_p) = p.canonicalize() {
                        let abs_str = abs_p.to_string_lossy().trim_start_matches(r"\\?\").to_string();
                        let current_path = std::env::var("PATH").unwrap_or_default();
                        if !current_path.contains(&abs_str) {
                            std::env::set_var("PATH", format!("{};{}", abs_str, current_path));
                            tracing::info!("🎮 [CUDA DLL 自动并网成功] 已将动态库目录注入进程 PATH: {}", abs_str);
                        }
                    }
                    break;
                }
            }
        }
    });
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
                OrtExecutionProvider::CPU,
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
                OrtExecutionProvider::CPU,
            ],
            ..Default::default()
        });
    }

    if dev.starts_with("directml") || dev.starts_with("dml") {
        let id = dev
            .strip_prefix("directml:")
            .or_else(|| dev.strip_prefix("dml:"))
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        return Ok(OrtSessionConfig {
            execution_providers: vec![
                OrtExecutionProvider::DirectML { device_id: id },
                OrtExecutionProvider::CPU,
            ],
            parallel_execution: Some(false),
            ..Default::default()
        });
    }

    Ok(OrtSessionConfig::default())
}

impl OrtSessionConfig {
    /// 🛡️ 为自回归控制流模型定制的安全配置
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
        // 1. 自动挂载注册 Windows CUDA 动态库寻址目录
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

        if let Some(intra) = self.intra_threads {
            builder = builder.with_intra_threads(intra).map_err(|e| OrtInferError::ModelLoad {
                path: "ONNX SessionBuilder".into(),
                context: format!("Failed to set intra threads: {e}"),
            })?;
        }

        if let Some(inter) = self.inter_threads {
            builder = builder.with_inter_threads(inter).map_err(|e| OrtInferError::ModelLoad {
                path: "ONNX SessionBuilder".into(),
                context: format!("Failed to set inter threads: {e}"),
            })?;
        }

        if let Some(par) = self.parallel_execution {
            builder = builder.with_parallel_execution(par).map_err(|e| OrtInferError::ModelLoad {
                path: "ONNX SessionBuilder".into(),
                context: format!("Failed to set parallel execution: {e}"),
            })?;
        }

        let mut dispatches: Vec<ExecutionProviderDispatch> = Vec::new();
        for ep in &self.execution_providers {
            match ep {
                OrtExecutionProvider::CPU => {
                    dispatches.push(ort::ep::CPU::default().build());
                }
                #[cfg(feature = "cuda")]
                OrtExecutionProvider::CUDA { device_id } => {
                    use ort::ep::cuda::ConvAlgorithmSearch;
                    let mut cuda_ep = ort::ep::CUDA::default().with_device_id(*device_id);
                    cuda_ep = cuda_ep.with_conv_algorithm_search(ConvAlgorithmSearch::Heuristic);
                    dispatches.push(cuda_ep.build());
                }
                #[cfg(feature = "directml")]
                OrtExecutionProvider::DirectML { device_id } => {
                    let dml_ep = ort::ep::DirectML::default().with_device_id(*device_id);
                    dispatches.push(dml_ep.build());
                }
                #[cfg(not(feature = "cuda"))]
                OrtExecutionProvider::CUDA { .. } => {}
                #[cfg(not(feature = "directml"))]
                OrtExecutionProvider::DirectML { .. } => {}
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
