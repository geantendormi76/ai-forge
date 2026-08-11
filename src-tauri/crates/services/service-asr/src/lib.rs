use core_ipc::{spawn_uv_worker, IpcMessage};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// MOSS 0.9B 官方规范底层参数契约（纯净无领域污染）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrOptions {
    pub audio_path: String,
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub hotwords: Option<String>,
    pub max_new_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawSegment {
    pub id: usize,
    pub speaker: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub audio_file: String,
    pub duration_sec: f64,
    pub segments: Vec<RawSegment>,
    pub elapsed_ms: f64,
}

pub struct AsrService;

impl AsrService {
    fn resolve_service_dir() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\crates\services\service-asr"),
            PathBuf::from(r"C:\dev\ai-toolkit\src-tauri\crates\services\service-asr"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifest_dir);
        }
        PathBuf::from("src-tauri/crates/services/service-asr")
    }

    /// 纯净听写底座管道：单一职责原则，仅输出原子级 RawSegment
    pub async fn run_asr_pipeline(
        options: AsrOptions,
    ) -> Result<AsrResult, String> {
        let audio_path = Path::new(&options.audio_path);
        if !audio_path.exists() {
            return Err(format!("音频物理文件不存在: {}", options.audio_path));
        }

        let service_dir = Self::resolve_service_dir();
        let script_path = service_dir.join("scripts/worker.py");

        let (mut child, mut channel) = spawn_uv_worker(&service_dir, &script_path)
            .map_err(|e| format!("启动 service-asr 进程失败: {e}"))?;

        let ready_msg = channel.recv().await
            .map_err(|e| format!("接收 ASR Worker 就绪信号失败: {e}"))?;
            
        if ready_msg.method != "system.ready" {
            tracing::warn!("⚠️ 收到非预期就绪信号: {}", ready_msg.method);
        }

        let req_payload = serde_json::json!({
            "audio_path": options.audio_path,
            "language": options.language.unwrap_or_else(|| "auto".into()),
            "prompt": options.prompt,
            "hotwords": options.hotwords,
            "max_new_tokens": options.max_new_tokens.unwrap_or(2048),
            "temperature": options.temperature.unwrap_or(0.0)
        });

        let req_msg = IpcMessage {
            method: "transcribe".to_string(),
            params: req_payload,
        };

        let t0 = std::time::Instant::now();
        channel.send(&req_msg).await
            .map_err(|e| format!("发送 ASR 转写请求失败: {e}"))?;

        let res_msg = channel.recv().await
            .map_err(|e| format!("读取 ASR 转写响应失败: {e}"))?;

        let exit_msg = IpcMessage {
            method: "exit".to_string(),
            params: serde_json::json!({}),
        };
        let _ = channel.send(&exit_msg).await;
        let _ = child.wait().await;

        if res_msg.method == "error" {
            let err_msg = res_msg.params["message"].as_str().unwrap_or("ASR 推理发生错误");
            return Err(format!("MOSS ASR 转写失败: {err_msg}"));
        }

        let elapsed_ms = t0.elapsed().as_millis() as f64;

        let result_val = &res_msg.params["result"];
        let raw_segs: Vec<serde_json::Value> = serde_json::from_value(result_val["segments"].clone())
            .map_err(|e| format!("解析 segments 失败: {e}"))?;

        let mut segments = Vec::new();
        for (idx, item) in raw_segs.iter().enumerate() {
            let speaker = item["speaker"].as_str().unwrap_or("S01").to_string();
            let start_sec = item["start"].as_f64().unwrap_or(0.0);
            let end_sec = item["end"].as_f64().unwrap_or(0.0);
            let text = item["text"].as_str().unwrap_or("").to_string();

            segments.push(RawSegment {
                id: idx + 1,
                speaker,
                start_sec,
                end_sec,
                text,
            });
        }

        let duration_sec = result_val["duration_sec"].as_f64().unwrap_or(0.0);
        let audio_file = result_val["audio_file"].as_str().unwrap_or("audio").to_string();

        Ok(AsrResult {
            audio_file,
            duration_sec,
            segments,
            elapsed_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asr_options_purity() {
        let opts = AsrOptions {
            audio_path: "/tmp/test.wav".into(),
            language: None,
            prompt: None,
            hotwords: None,
            max_new_tokens: None,
            temperature: None,
        };
        let json_str = serde_json::to_string(&opts).unwrap();
        assert!(!json_str.contains("mode"));
        assert!(!json_str.contains("srt_text"));
        println!("\n✅ [service-asr 底座纯净性测试通过] 纯粹官方 API 契约: {}", json_str);
    }

    fn resolve_fixture_path(filename: &str) -> PathBuf {
        let candidates = [
            PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", filename)),
            PathBuf::from(format!(r"C:\dev\ai-toolkit\test\fixtures\{}", filename)),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", filename))
    }

    fn resolve_outs_dir() -> PathBuf {
        let candidate = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-asr");
        let _ = std::fs::create_dir_all(&candidate);
        candidate
    }

    #[tokio::test]
    async fn test_service_asr_official_spec() {
        let fixture_audio = resolve_fixture_path("ASR_英语_餐厅就餐.mp3");
        if !fixture_audio.exists() {
            println!("⚠️ [跳过测试] 英文基准音频不存在: {:?}", fixture_audio);
            return;
        }

        let outs_dir = resolve_outs_dir();
        println!("\n🚀 [TDD 官方纯净底座测试] 正在测试 service-asr: {:?}", fixture_audio);

        let opts = AsrOptions {
            audio_path: fixture_audio.to_string_lossy().to_string(),
            language: Some("en".into()),
            prompt: None,
            hotwords: None,
            max_new_tokens: Some(2048),
            temperature: Some(0.0),
        };

        let res = AsrService::run_asr_pipeline(opts)
            .await
            .expect("MOSS 0.9B ASR 官方规范打靶失败");

        println!("\n🎉 ===== [MOSS 0.9B 官方纯净底座 ASR 识别完成] =====");
        println!("  ⏱️ 耗时: {:.2} ms ({:.2} s) | 音频时长: {:.2}s | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.duration_sec, res.segments.len());

        assert!(res.segments.len() > 0, "转写台词数不可为 0！");

        let out_json_path = outs_dir.join("asr_english_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 官方纯净底座 JSON 成功落盘: {:?}", out_json_path);
    }
}
