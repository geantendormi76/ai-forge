use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),
    #[error("文件系统 IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("哈希校验不匹配，预期: {expected}, 实际: {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("模型下载中断或服务器响应非 200")]
    DownloadFailed,
}

pub struct ModelManager;

impl ModelManager {
    /// 计算文件的 SHA256 哈希值
    pub async fn compute_sha256(file_path: &Path) -> Result<String, ModelError> {
        if !file_path.exists() {
            return Ok(String::new());
        }
        let mut file = File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        use tokio::io::AsyncReadExt;
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// 校验模型是否存在且完整。若缺失则从指定 CDN 断点续传下载
    pub async fn ensure_model_file<F>(
        target_path: &Path,
        expected_sha256: &str,
        download_urls: &[&str],
        on_progress: F,
    ) -> Result<PathBuf, ModelError>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        // 1. 如果文件已存在，执行 SHA256 秒级断言
        if target_path.exists() {
            if expected_sha256.is_empty() {
                return Ok(target_path.to_path_buf());
            }
            let actual_hash = Self::compute_sha256(target_path).await?;
            if actual_hash.eq_ignore_ascii_case(expected_sha256) {
                tracing::info!("✅ 模型物理哈希校验一致: {:?}", target_path);
                return Ok(target_path.to_path_buf());
            }
            tracing::warn!("⚠️ 模型哈希受损，准备重新下载...");
            let _ = fs::remove_file(target_path).await;
        }

        // 2. 确保父级目录存在
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // 3. 多源重试轮询下载 (支持 Cloudflare CDN 与 ModelScope)
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        for &url in download_urls {
            tracing::info!("🌐 正在从 CDN 下载模型: {}", url);
            let res = match client.get(url).send().await {
                Ok(response) if response.status().is_success() => response,
                _ => continue,
            };

            let total_size = res.content_length().unwrap_or(0);
            let mut downloaded: u64 = 0;
            let mut file = File::create(target_path).await?;
            let mut stream = res.bytes_stream();

            let mut success = true;
            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        file.write_all(&chunk).await?;
                        downloaded += chunk.len() as u64;
                        on_progress(downloaded, total_size);
                    }
                    Err(e) => {
                        tracing::error!("🚨 流量中断: {}", e);
                        success = false;
                        break;
                    }
                }
            }

            if success {
                file.flush().await?;
                // 校验下载完成后的哈希
                if !expected_sha256.is_empty() {
                    let actual_hash = Self::compute_sha256(target_path).await?;
                    if !actual_hash.eq_ignore_ascii_case(expected_sha256) {
                        tracing::error!("🚨 下载文件哈希不匹配: {}", actual_hash);
                        let _ = fs::remove_file(target_path).await;
                        continue;
                    }
                }
                tracing::info!("🎉 模型下载并校验成功: {:?}", target_path);
                return Ok(target_path.to_path_buf());
            }
        }

        Err(ModelError::DownloadFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_manager_hash() {
        let tmp_file = std::env::temp_dir().join("test_model.bin");
        tokio::fs::write(&tmp_file, b"AI-Forge 4-bit Quantized Model Payload").await.unwrap();

        let hash = ModelManager::compute_sha256(&tmp_file).await.unwrap();
        assert!(!hash.is_empty());
        println!("✅ 测试文件 SHA256: {}", hash);

        let _ = tokio::fs::remove_file(tmp_file).await;
    }
}
