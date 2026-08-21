use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),
    #[error("文件系统 IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("哈希校验不匹配，预期: {expected}, 实际: {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("下载已由用户主动取消")]
    Cancelled,
    #[error("服务器响应异常 [HTTP {0}]: 资源未就绪")]
    HttpError(u16),
    #[error("模型下载中断或网络流异常终止")]
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
    pub phase: String, // "waiting" | "downloading" | "verifying" | "complete"
}

pub struct ModelManager;

impl ModelManager {
    /// 🛡️ 三级自愈模型基准目录寻址器
    pub fn resolve_models_base_dir() -> PathBuf {
        // 1. 就近探测：当前工作目录直接存在 models/
        if Path::new("models").is_dir() {
            if let Ok(abs) = Path::new("models").canonicalize() {
                return abs;
            }
            return PathBuf::from("models");
        }

        // 2. 向上回溯：若从 target/release 等子目录启动，向上逐级回溯寻找 models/
        if let Ok(exe_path) = std::env::current_exe() {
            let mut current = exe_path.parent();
            for _ in 0..5 {
                if let Some(p) = current {
                    let candidate = p.join("models");
                    if candidate.is_dir() {
                        if let Ok(abs) = candidate.canonicalize() {
                            return abs;
                        }
                        return candidate;
                    }
                    current = p.parent();
                } else {
                    break;
                }
            }
        }

        // 3. 生产安全隔离区：使用 %LOCALAPPDATA%\ZiDianAI\models（确保拥有 100% 读写主权）
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let target = PathBuf::from(local_appdata).join("ZiDianAI").join("models");
            let _ = std::fs::create_dir_all(&target);
            return target;
        }

        PathBuf::from("models")
    }

    /// 格式化字节大小
    pub fn format_bytes(bytes: u64) -> String {
        if bytes == 0 {
            return "0 MB".to_string();
        }
        let mb = bytes as f64 / (1024.0 * 1024.0);
        if mb >= 1024.0 {
            format!("{:.2} GB", mb / 1024.0)
        } else {
            format!("{:.1} MB", mb)
        }
    }

    /// 计算文件的 SHA256 哈希值 (使用堆内存 buffer 规避栈溢出)
    pub async fn compute_sha256(file_path: &Path) -> Result<String, ModelError> {
        if !file_path.exists() {
            return Ok(String::new());
        }
        let mut file = File::open(file_path).await?;
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

    /// 获取特定工具算子所需的全部依赖清单
    pub async fn get_tool_dependencies(base_dir: &Path, tool_id: &str) -> Vec<DependencyItem> {
        let manifest_items = Self::get_tool_manifest_specs(tool_id);
        let mut result = Vec::new();

        for mut item in manifest_items {
            let full_path = base_dir.join(&item.relative_path);
            let mut is_ready = false;

            if full_path.exists() {
                if item.sha256.is_empty() {
                    is_ready = true;
                } else if let Ok(meta) = fs::metadata(&full_path).await {
                    if meta.len() == item.size_bytes {
                        is_ready = true;
                    }
                }
            }

            item.is_ready = is_ready;
            item.size_formatted = Self::format_bytes(item.size_bytes);
            result.push(item);
        }

        result
    }

    /// 检查指定工具的全部依赖是否均已就绪
    pub async fn is_tool_ready(base_dir: &Path, tool_id: &str) -> bool {
        let deps = Self::get_tool_dependencies(base_dir, tool_id).await;
        deps.iter().all(|d| d.is_ready)
    }

    /// 工具 ➔ 13 项依赖模型精准契约注册表 (统合“4K/8K 图像超分”标准命名)
    fn get_tool_manifest_specs(tool_id: &str) -> Vec<DependencyItem> {
        match tool_id {
            "upscale" | "upscale-48k" | "tool-upscale-48k" => vec![
                DependencyItem {
                    id: "realesrgan_x4plus".into(),
                    name: "4K/8K 图像超分模型".into(),
                    relative_path: "service-upscale/RealESRGAN_x4plus.onnx".into(),
                    size_bytes: 33_754_215,
                    size_formatted: "32.2 MB".into(),
                    sha256: "c61f3c947c46af0c3ae188364923b3050b1d0089cc1e8394c8ce8306def2ce25".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-upscale/RealESRGAN_x4plus.onnx".into(),
                    ],
                    is_ready: false,
                }
            ],
            "asr" | "video-subtitle" | "tool-ASR" => vec![
                DependencyItem {
                    id: "moss_asr_09b".into(),
                    name: "MOSS 0.9B ASR 语音模型".into(),
                    relative_path: "service-asr/MOSS-Transcribe-Diarize-Q5_K_M.gguf".into(),
                    size_bytes: 700_313_760,
                    size_formatted: "667.9 MB".into(),
                    sha256: "52deaeff931272f3d49eb437f0f4916e42fce9f42e68db250047408241cf473c".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-asr/MOSS-Transcribe-Diarize-Q5_K_M.gguf".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "hymt2_translation_18b".into(),
                    name: "Hy-MT2 1.8B 神经翻译模型".into(),
                    relative_path: "service-translation/Hy-MT2-1.8B-Q4.gguf".into(),
                    size_bytes: 1_133_080_736,
                    size_formatted: "1.08 GB".into(),
                    sha256: "d17d2b8d0b1abac7dd8b15dfc3a50998aa4be2735987341c49a5a0b4d3b5b2ee".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-translation/Hy-MT2-1.8B-Q4.gguf".into(),
                    ],
                    is_ready: false,
                }
            ],
            "pdf" | "pdf-parse" | "tool-pdf-parse" => vec![
                DependencyItem {
                    id: "pp_doclayout_v3".into(),
                    name: "PP-DocLayoutV3 版面重构模型".into(),
                    relative_path: "service-layout/PP-DocLayoutV3.onnx".into(),
                    size_bytes: 130_502_049,
                    size_formatted: "124.5 MB".into(),
                    sha256: "45bf71750b00739a41fc209f132eb104a4d6b5bb29483c9078164d8b87cf28ba".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-layout/PP-DocLayoutV3.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_ocrv6_rec".into(),
                    name: "PP-OCRv6 视觉识别模型".into(),
                    relative_path: "service-ocr/PP-OCRv6_small/pp-ocrv6_small_rec.onnx".into(),
                    size_bytes: 21_159_378,
                    size_formatted: "20.2 MB".into(),
                    sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-ocr/PP-OCRv6_small/pp-ocrv6_small_rec.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_ocrv6_det".into(),
                    name: "PP-OCRv6 文本检测模型".into(),
                    relative_path: "service-ocr/PP-OCRv6_small/pp-ocrv6_small_det.onnx".into(),
                    size_bytes: 9_880_512,
                    size_formatted: "9.4 MB".into(),
                    sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-ocr/PP-OCRv6_small/pp-ocrv6_small_det.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_ocrv6_dict".into(),
                    name: "PP-OCRv6 字典".into(),
                    relative_path: "service-ocr/PP-OCRv6_small/ppocrv6_dict.txt".into(),
                    size_bytes: 93_655,
                    size_formatted: "0.1 MB".into(),
                    sha256: "769e7fa79bb297b5f18d8dbd149e364a45bc61f2b3f574e5ea836f0b261c23a6".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-ocr/PP-OCRv6_small/ppocrv6_dict.txt".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_formulanet_s".into(),
                    name: "PP-FormulaNet 公式识别模型".into(),
                    relative_path: "service-formula/PP-FormulaNet-S.onnx".into(),
                    size_bytes: 231_878_904,
                    size_formatted: "221.1 MB".into(),
                    sha256: "9fbcf2d7b5d7534e4c596e241f5c60be482a4d24d6a9625f2e79e140c7522877".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-formula/PP-FormulaNet-S.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_formula_tokenizer".into(),
                    name: "PP-Formula 分词器".into(),
                    relative_path: "service-formula/tokenizer.json".into(),
                    size_bytes: 2_240_079,
                    size_formatted: "2.1 MB".into(),
                    sha256: "23d8fa64d068f450b61d6581dac0fe234441770365d64ba39ba74e34079bc022".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-formula/tokenizer.json".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "pp_formula_manifest".into(),
                    name: "PP-Formula 配置".into(),
                    relative_path: "service-formula/manifest.json".into(),
                    size_bytes: 639,
                    size_formatted: "0.0 MB".into(),
                    sha256: "ad2b17acc58c32ea9651963ea86d9a56391f57f0644e06768df8e6958f9a5289".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-formula/manifest.json".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "slanet_plus".into(),
                    name: "SLANet+ 多维表格结构模型".into(),
                    relative_path: "service-table/SLANet_plus.onnx".into(),
                    size_bytes: 7_781_309,
                    size_formatted: "7.4 MB".into(),
                    sha256: "e48a401a4ebcddd47fe3822427db24d867a557324f58e438692f588bbe9231de".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-table/SLANet_plus.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "table_model".into(),
                    name: "Table 辅助模型".into(),
                    relative_path: "service-table/model.onnx".into(),
                    size_bytes: 7_781_309,
                    size_formatted: "7.4 MB".into(),
                    sha256: "e48a401a4ebcddd47fe3822427db24d867a557324f58e438692f588bbe9231de".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-table/model.onnx".into(),
                    ],
                    is_ready: false,
                },
                DependencyItem {
                    id: "table_dict".into(),
                    name: "Table 结构字典".into(),
                    relative_path: "service-table/table_structure_dict.txt".into(),
                    size_bytes: 624,
                    size_formatted: "0.0 MB".into(),
                    sha256: "e43477e1819efb450e33a2a37d78ba408951bb1c1357901eb62f35dbfa382652".into(),
                    download_urls: vec![
                        "https://assets.geantendormi.top/models/service-table/table_structure_dict.txt".into(),
                    ],
                    is_ready: false,
                }
            ],
            // 全能格式转换：纯血 Rust 算法直出，0 外部模型依赖
            "format" | "format-converter" | "tool-format-convert" => vec![],
            _ => vec![],
        }
    }

    /// 断点续传流式下载 (带毫秒级节流阀与栈内存防爆)
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

        // 1. 目标文件已存在时的极速哈希断言
        if target_path.exists() {
            if item.sha256.is_empty() {
                on_progress(item.size_bytes, item.size_bytes, "complete");
                return Ok(target_path.to_path_buf());
            }
            on_progress(item.size_bytes, item.size_bytes, "verifying");
            let actual_hash = Self::compute_sha256(target_path).await?;
            if actual_hash.eq_ignore_ascii_case(&item.sha256) {
                tracing::info!("✅ 模型物理哈希校验一致: {:?}", target_path);
                on_progress(item.size_bytes, item.size_bytes, "complete");
                return Ok(target_path.to_path_buf());
            }
            let _ = fs::remove_file(target_path).await;
        }

        // 2. 确保父级完整目录树物理存在
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let part_path = target_path.with_extension("part");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
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

            if resume_from > item.size_bytes && item.size_bytes > 0 {
                let _ = fs::remove_file(&part_path).await;
                resume_from = 0;
            }

            let mut req = client.get(url);
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
            let mut downloaded = if is_append { resume_from } else { 0 };
            let total_size = if item.size_bytes > 0 {
                item.size_bytes
            } else {
                res.content_length().unwrap_or(0) + downloaded
            };

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(is_append)
                .truncate(!is_append)
                .open(&part_path)
                .await?;

            let mut stream = res.bytes_stream();
            let mut success = true;

            // 初始触发一次通知
            on_progress(downloaded, total_size, "downloading");

            // 🛡️ 毫秒级节流状态机 (每 80ms 或每 1% 才发射一次 IPC 事件，彻底杜绝主线程栈溢出)
            let mut last_emit = std::time::Instant::now();
            let mut last_percent = if total_size > 0 {
                ((downloaded as f64 / total_size as f64) * 100.0) as u32
            } else {
                0
            };

            while let Some(chunk_res) = stream.next().await {
                if cancel_token.load(Ordering::Relaxed) {
                    let _ = file.flush().await;
                    return Err(ModelError::Cancelled);
                }

                match chunk_res {
                    Ok(chunk) => {
                        file.write_all(&chunk).await?;
                        downloaded += chunk.len() as u64;

                        let current_percent = if total_size > 0 {
                            ((downloaded as f64 / total_size as f64) * 100.0) as u32
                        } else {
                            0
                        };

                        if current_percent != last_percent || last_emit.elapsed().as_millis() >= 80 {
                            last_percent = current_percent;
                            last_emit = std::time::Instant::now();
                            on_progress(downloaded, total_size, "downloading");
                        }
                    }
                    Err(e) => {
                        tracing::error!("🚨 下载网络流中断: {}", e);
                        success = false;
                        last_error = Some(ModelError::Network(e));
                        break;
                    }
                }
            }

            if success {
                file.flush().await?;
                drop(file);

                on_progress(downloaded, total_size, "verifying");

                if !item.sha256.is_empty() {
                    let actual_hash = Self::compute_sha256(&part_path).await?;
                    if !actual_hash.eq_ignore_ascii_case(&item.sha256) {
                        tracing::error!("🚨 SHA256 哈希校验失败，预期: {}, 实际: {}", item.sha256, actual_hash);
                        let _ = fs::remove_file(&part_path).await;
                        last_error = Some(ModelError::HashMismatch {
                            expected: item.sha256.clone(),
                            actual: actual_hash,
                        });
                        continue;
                    }
                }

                fs::rename(&part_path, target_path).await?;
                tracing::info!("🎉 模型物理落盘并就绪: {:?}", target_path);
                on_progress(total_size, total_size, "complete");
                return Ok(target_path.to_path_buf());
            }
        }

        Err(last_error.unwrap_or(ModelError::DownloadFailed))
    }
}
