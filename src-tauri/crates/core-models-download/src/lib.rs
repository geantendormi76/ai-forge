pub mod gguf_meta;

use core_security::PortableEngine;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("网络请求异常: {0}")]
    Network(#[from] reqwest::Error),
    #[error("文件 I/O 异常: {0}")]
    Io(#[from] std::io::Error),
    #[error("哈希校验不匹配: 期望 {expected}, 实际得到 {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("下载已由用户取消")]
    Cancelled,
    #[error("HTTP 状态码异常: {0}")]
    HttpError(u16),
    #[error("下载最终失败，已重试所有可用镜像源")]
    DownloadFailed,
}

/// 单个模型/依赖项强类型描述契约
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyItem {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub sha256: String,
    pub download_urls: Vec<String>,
    pub is_ready: bool,
}

/// 下载进度实时推送载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyProgressPayload {
    pub tool_id: String,
    pub item_id: String,
    pub item_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: u32,
    pub phase: String,
}

pub struct ModelManager;

impl ModelManager {
    pub fn resolve_models_base_dir() -> PathBuf {
        if let Some(portable_models) = PortableEngine::resolve_data_path("models") {
            let _ = std::fs::create_dir_all(&portable_models);
            return portable_models;
        }

        if Path::new("models").is_dir() {
            if let Ok(abs) = Path::new("models").canonicalize() {
                return abs;
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            let mut current = exe_path.parent();
            while let Some(p) = current {
                let candidate = p.join("models");
                if candidate.is_dir() {
                    if let Ok(abs) = candidate.canonicalize() {
                        return abs;
                    }
                }
                current = p.parent();
            }
        }

        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let base = PathBuf::from(&local_appdata);
            let modern_target = base.join("紫电AI").join("Data").join("models");
            let legacy_target = base.join("ZiDianAI").join("models");

            if legacy_target.is_dir() && !modern_target.exists() {
                let _ = std::fs::create_dir_all(&modern_target);
                if let Ok(entries) = std::fs::read_dir(&legacy_target) {
                    for entry in entries.flatten() {
                        let src = entry.path();
                        let dst = modern_target.join(entry.file_name());
                        let _ = std::fs::rename(&src, &dst);
                    }
                }
                let _ = std::fs::remove_dir_all(base.join("ZiDianAI"));
            }

            let _ = std::fs::create_dir_all(&modern_target);
            return modern_target;
        }

        let fallback = PathBuf::from("models");
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    }

    pub fn resolve_cuda_runtime_dir() -> PathBuf {
        if let Some(portable_cuda) = PortableEngine::resolve_data_path("bin/cuda12") {
            let _ = std::fs::create_dir_all(&portable_cuda);
            return portable_cuda;
        }

        if Path::new("bin").join("cuda12").join("cublas64_13.dll").exists() {
            if let Ok(abs) = Path::new("bin").join("cuda12").canonicalize() {
                return abs;
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            let mut current = exe_path.parent();
            while let Some(p) = current {
                let candidate = p.join("bin").join("cuda12");
                if candidate.join("cublas64_13.dll").exists() {
                    if let Ok(abs) = candidate.canonicalize() {
                        return abs;
                    }
                }
                current = p.parent();
            }
        }

        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let base = PathBuf::from(&local_appdata);
            let modern_target = base.join("紫电AI").join("Data").join("bin").join("cuda12");
            let _ = std::fs::create_dir_all(&modern_target);
            return modern_target;
        }

        let fallback = PathBuf::from("bin").join("cuda12");
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    }

    pub fn format_bytes(bytes: u64) -> String {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        if mb < 0.1 {
            return "0 MB".to_string();
        }
        if mb >= 1024.0 {
            format!("{:.2} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    pub async fn compute_sha256(file_path: &Path) -> Result<String, ModelError> {
        if !file_path.exists() {
            return Ok(String::new());
        }
        let mut file = fs::File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 65536];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    /// 🛡️ 毫秒级依赖极速状态探测（仅进行 0.001ms 文件元信息 stat 校验，绝不阻塞 UI 路由导航）
    pub async fn get_tool_dependencies(base_dir: &Path, tool_id: &str) -> Vec<DependencyItem> {
        let specs = get_tool_manifest_specs(tool_id);
        let mut result = Vec::with_capacity(specs.len());

        for mut item in specs {
            let full_path = base_dir.join(&item.relative_path);
            let is_ready = if full_path.exists() {
                if let Ok(meta) = fs::metadata(&full_path).await {
                    // 仅对比文件大小是否匹配，0.001ms 瞬时完成
                    meta.len() == item.size_bytes
                } else {
                    false
                }
            } else {
                false
            };

            item.is_ready = is_ready;
            result.push(item);
        }

        result
    }

    pub async fn is_tool_ready(base_dir: &Path, tool_id: &str) -> bool {
        let deps = Self::get_tool_dependencies(base_dir, tool_id).await;
        if deps.is_empty() {
            return true;
        }
        deps.iter().all(|d| d.is_ready)
    }

    pub async fn download_dependency_file<F>(
        target_path: &Path,
        item: &DependencyItem,
        cancel_token: Arc<AtomicBool>,
        on_progress: F,
    ) -> Result<PathBuf, ModelError>
    where
        F: Fn(u64, u64, &str) + Send + Sync + 'static,
    {
        if cancel_token.load(Ordering::Relaxed) {
            return Err(ModelError::Cancelled);
        }

        if target_path.exists() {
            if item.sha256.is_empty() {
                on_progress(item.size_bytes, item.size_bytes, "complete");
                return Ok(target_path.to_path_buf());
            }
            on_progress(item.size_bytes, item.size_bytes, "verifying");
            let actual_hash = Self::compute_sha256(target_path).await.unwrap_or_default();
            if actual_hash.eq_ignore_ascii_case(&item.sha256) {
                on_progress(item.size_bytes, item.size_bytes, "complete");
                return Ok(target_path.to_path_buf());
            }
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let part_path = target_path.with_extension("part");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()?;

        let mut last_error = None;

        for url in &item.download_urls {
            if cancel_token.load(Ordering::Relaxed) {
                return Err(ModelError::Cancelled);
            }

            let mut resume_from = if part_path.exists() {
                fs::metadata(&part_path).await.map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            let mut req = client.get(url).header("User-Agent", "ZiDianAI-Client/1.0");
            if resume_from > 0 {
                req = req.header("Range", format!("bytes={}-", resume_from));
            }

            let res = match req.send().await {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 206 => r,
                Ok(r) => {
                    let code = r.status().as_u16();
                    last_error = Some(ModelError::HttpError(code));
                    continue;
                }
                Err(e) => {
                    last_error = Some(ModelError::Network(e));
                    continue;
                }
            };

            let is_append = resume_from > 0 && res.status().as_u16() == 206;
            if !is_append {
                resume_from = 0;
            }

            let total_size = if is_append {
                res.content_length().unwrap_or(0) + resume_from
            } else {
                res.content_length().unwrap_or(item.size_bytes)
            };

            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(is_append)
                .truncate(!is_append)
                .open(&part_path)
                .await?;

            let mut stream = res.bytes_stream();
            let mut downloaded = resume_from;
            let mut last_emit = Instant::now();
            let mut last_percent = 0u32;

            on_progress(downloaded, total_size, "downloading");

            while let Some(chunk_res) = stream.next().await {
                if cancel_token.load(Ordering::Relaxed) {
                    let _ = file.flush().await;
                    return Err(ModelError::Cancelled);
                }

                let chunk = chunk_res?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;

                let current_percent = if total_size > 0 {
                    ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u32
                } else {
                    0
                };

                if current_percent != last_percent || last_emit.elapsed().as_millis() >= 80 {
                    last_percent = current_percent;
                    last_emit = Instant::now();
                    on_progress(downloaded, total_size, "downloading");
                }
            }

            file.flush().await?;
            drop(file);

            on_progress(downloaded, total_size, "verifying");

            if !item.sha256.is_empty() {
                let actual_hash = Self::compute_sha256(&part_path).await.unwrap_or_default();
                if !actual_hash.eq_ignore_ascii_case(&item.sha256) {
                    last_error = Some(ModelError::HashMismatch {
                        expected: item.sha256.clone(),
                        actual: actual_hash,
                    });
                    let _ = fs::remove_file(&part_path).await;
                    continue;
                }
            }

            fs::rename(&part_path, target_path).await?;
            on_progress(total_size, total_size, "complete");
            return Ok(target_path.to_path_buf());
        }

        Err(last_error.unwrap_or(ModelError::DownloadFailed))
    }
}

fn get_tool_manifest_specs(tool_id: &str) -> Vec<DependencyItem> {
    match tool_id.to_lowercase().as_str() {
        "upscale" | "upscale-48k" | "tool-upscale-48k" => vec![
            DependencyItem {
                id: "realesrgan-x4plus".to_string(),
                name: "RealESRGAN x4plus 超分大模型".to_string(),
                relative_path: "service-upscale/RealESRGAN_x4plus.onnx".to_string(),
                size_bytes: 33754215,
                size_formatted: "32.2 MB".to_string(),
                sha256: "c61f3c947c46af0c3ae188364923b3050b1d0089cc1e8394c8ce8306def2ce25".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-upscale/RealESRGAN_x4plus.onnx".to_string(),
                ],
                is_ready: false,
            },
        ],
        "asr" | "video-subtitle" | "tool-asr" => vec![
            DependencyItem {
                id: "moss-transcribe-diarize-q5".to_string(),
                name: "MOSS 语音识别与说话人分段大模型 (Q5_K_M GGUF)".to_string(),
                relative_path: "service-asr/MOSS-Transcribe-Diarize-Q5_K_M.gguf".to_string(),
                size_bytes: 700313760,
                size_formatted: "667.9 MB".to_string(),
                sha256: "52deaeff931272f3d49eb437f0f4916e42fce9f42e68db250047408241cf473c".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-asr/MOSS-Transcribe-Diarize-Q5_K_M.gguf".to_string(),
                ],
                is_ready: false,
            },
        ],
        "trans" | "translation" | "tool-translation" | "tool-trans" => vec![
            DependencyItem {
                id: "hy-mt2-1.8b-q4".to_string(),
                name: "Hy-MT2 1.8B 神经翻译大模型 (Q4 GGUF)".to_string(),
                relative_path: "service-translation/Hy-MT2-1.8B-Q4.gguf".to_string(),
                size_bytes: 1133080736,
                size_formatted: "1.08 GB".to_string(),
                sha256: "d17d2b8d0b1abac7dd8b15dfc3a50998aa4be2735987341c49a5a0b4d3b5b2ee".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-translation/Hy-MT2-1.8B-Q4.gguf".to_string(),
                ],
                is_ready: false,
            },
        ],
        "pdf" | "pdf-parse" | "tool-pdf-parse" => vec![
            DependencyItem {
                id: "pp-doclayoutv3".to_string(),
                name: "PP-DocLayoutV3 版面分析模型".to_string(),
                relative_path: "service-layout/PP-DocLayoutV3.onnx".to_string(),
                size_bytes: 130502049,
                size_formatted: "124.5 MB".to_string(),
                sha256: "45bf71750b00739a41fc209f132eb104a4d6b5bb29483c9078164d8b87cf28ba".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-layout/PP-DocLayoutV3.onnx".to_string(),
                ],
                is_ready: false,
            },
            DependencyItem {
                id: "pp-formulanet-s".to_string(),
                name: "PP-FormulaNet-S 公式识别模型".to_string(),
                relative_path: "service-formula/PP-FormulaNet-S.onnx".to_string(),
                size_bytes: 231878904,
                size_formatted: "221.1 MB".to_string(),
                sha256: "9fbcf2d7b5d7534e4c596e241f5c60be482a4d24d6a9625f2e79e140c7522877".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-formula/PP-FormulaNet-S.onnx".to_string(),
                ],
                is_ready: false,
            },
            DependencyItem {
                id: "pp-ocrv6-det".to_string(),
                name: "PP-OCRv6 文本检测模型".to_string(),
                relative_path: "service-ocr/PP-OCRv6_small/pp-ocrv6_small_det.onnx".to_string(),
                size_bytes: 9880512,
                size_formatted: "9.4 MB".to_string(),
                sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-ocr/PP-OCRv6_small/pp-ocrv6_small_det.onnx".to_string(),
                ],
                is_ready: false,
            },
            DependencyItem {
                id: "pp-ocrv6-rec".to_string(),
                name: "PP-OCRv6 文本识别模型".to_string(),
                relative_path: "service-ocr/PP-OCRv6_small/pp-ocrv6_small_rec.onnx".to_string(),
                size_bytes: 21159378,
                size_formatted: "20.2 MB".to_string(),
                sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-ocr/PP-OCRv6_small/pp-ocrv6_small_rec.onnx".to_string(),
                ],
                is_ready: false,
            },
            DependencyItem {
                id: "slanet-plus".to_string(),
                name: "SLANet_plus 表格结构还原模型".to_string(),
                relative_path: "service-table/SLANet_plus.onnx".to_string(),
                size_bytes: 7781309,
                size_formatted: "7.4 MB".to_string(),
                sha256: "e48a401a4ebcddd47fe3822427db24d867a557324f58e438692f588bbe9231de".to_string(),
                download_urls: vec![
                    "https://assets.geantendormi.top/models/service-table/SLANet_plus.onnx".to_string(),
                ],
                is_ready: false,
            },
        ],
        "format" | "format-converter" | "tool-format-convert" => vec![],
        _ => vec![],
    }
}
