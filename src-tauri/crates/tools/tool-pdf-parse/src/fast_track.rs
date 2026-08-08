use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastTrackResult {
    pub success: bool,
    pub markdown: String,
    pub images: Vec<String>,
    pub error: Option<String>,
}

pub struct FastTrackEngine;

impl FastTrackEngine {
    fn resolve_uv_binary() -> PathBuf {
        let candidates = [
            PathBuf::from("/home/zhz/.local/bin/uv"),
            PathBuf::from("/usr/local/bin/uv"),
            PathBuf::from("/usr/bin/uv"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return candidate.clone();
            }
        }

        PathBuf::from("uv")
    }

    pub async fn run_pages(
        pdf_path: &Path,
        output_dir: &Path,
        pages: Option<&[u32]>,
    ) -> Result<FastTrackResult, String> {
        let project_dir = PathBuf::from("/home/zhz/ai-toolkit/src-tauri/crates/tools/tool-pdf-parse");
        let script_path = project_dir.join("scripts/pymupdf_worker.py");
        let uv_binary = Self::resolve_uv_binary();

        if !pdf_path.exists() {
            return Err(format!("输入物理 PDF 文件不存在: {:?}", pdf_path));
        }

        let pages_str = pages
            .map(|p| p.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","))
            .unwrap_or_default();

        let output = Command::new(&uv_binary)
            .arg("run")
            .arg("--project")
            .arg(&project_dir) // 🛡️ 绑定专属 SOTA 沙箱
            .arg("python3")
            .arg(&script_path)
            .arg(pdf_path)
            .arg(output_dir)
            .arg(&pages_str)
            .output()
            .await
            .map_err(|e| format!("无法启动 PyMuPDF4LLM Worker 进程 ({:?}): {e}", uv_binary))?;

        let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();

        let json_str = match (raw_stdout.find("___JSON_START___"), raw_stdout.rfind("___JSON_END___")) {
            (Some(start), Some(end)) if start < end => {
                raw_stdout[start + "___JSON_START___".len()..end].trim()
            }
            _ => {
                return Err(format!("未找到 JSON 哨兵边界，原始输出: {raw_stdout}"));
            }
        };

        let result: FastTrackResult = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON 解析失败: {e}, 提取串: {json_str}"))?;

        if !result.success {
            return Err(result.error.unwrap_or_else(|| "未知错误".into()));
        }

        info!(
            "⚡ [Fast-Track 极速解析成功] 提炼 Markdown 字符数: {}, 图片数: {}",
            result.markdown.len(),
            result.images.len()
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_step3_fasttrack_real_execution() {
        let pdf_path = Path::new("/mnt/c/Users/52484/Pictures/3.pdf");
        let output_dir = Path::new("/mnt/c/Users/52484/Pictures");
        if !pdf_path.exists() {
            println!("⚠️ [跳过测试] 测试物理 PDF 不存在: {:?}", pdf_path);
            return;
        }

        let res = FastTrackEngine::run_pages(pdf_path, output_dir, None)
            .await
            .expect("FastTrack 引擎真实解析失败");

        assert!(res.success);
        println!("\n========= ⚡ [FastTrack CPU 极速解析成功落盘] =========");
        println!("  Markdown 提炼总字符数: {}", res.markdown.len());
        println!("  提取内嵌图片数量: {}", res.images.len());
        println!("=======================================================\n");
    }
}
