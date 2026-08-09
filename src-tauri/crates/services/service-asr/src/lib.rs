use core_ipc::{spawn_uv_worker, IpcMessage};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrOptions {
    pub audio_path: String,
    pub mode: Option<String>,
    pub language: Option<String>,
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
            PathBuf::from("/home/zhz/ai-forge/src-tauri/crates/services/service-asr"),
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

    /// 纯净听写管道：只输出物理句段信息，零业务/格式污染
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
            "mode": options.mode.unwrap_or_else(|| "verbatim".into()),
            "language": options.language.unwrap_or_else(|| "auto".into())
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
            mode: None,
            language: None,
        };
        let json_str = serde_json::to_string(&opts).unwrap();
        assert!(!json_str.contains("srt_text"));
        assert!(!json_str.contains("ass_text"));
        println!("\n✅ [service-asr 纯净性测试通过] 零领域污染数据契约: {}", json_str);
    }

    fn resolve_fixture_path(filename: &str) -> PathBuf {
        let candidates = [
            PathBuf::from(format!(r"C:\dev\ai-forge\test\fixtures\{}", filename)),
            PathBuf::from(format!(r"C:\dev\ai-toolkit\test\fixtures\{}", filename)),
            PathBuf::from(format!("/home/zhz/ai-forge/test/fixtures/{}", filename)),
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
    async fn test_service_asr_e2e_english_pure() {
        let fixture_audio = resolve_fixture_path("ASR_英语_餐厅就餐.mp3");
        if !fixture_audio.exists() {
            println!("⚠️ [跳过测试] 英文基准音频不存在: {:?}", fixture_audio);
            return;
        }

        let outs_dir = resolve_outs_dir();

        println!("\n🚀 [TDD 纯净打靶 - 英文] 正在测试 service-asr: {:?}", fixture_audio);

        let opts = AsrOptions {
            audio_path: fixture_audio.to_string_lossy().to_string(),
            mode: Some("verbatim".into()),
            language: Some("en".into()),
        };

        let res = AsrService::run_asr_pipeline(opts)
            .await
            .expect("MOSS 0.9B ASR 英文语音打靶失败");

        println!("\n🎉 ===== [MOSS 0.9B 纯净 ASR 英文识别完成] =====");
        println!("  ⏱️ 耗时: {:.2} ms ({:.2} s) | 音频时长: {:.2}s | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.duration_sec, res.segments.len());

        assert!(res.segments.len() > 0, "转写台词数不可为 0！");

        let out_json_path = outs_dir.join("asr_english_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 英文纯净 JSON 成功落盘: {:?}", out_json_path);
        println!("  预览前 2 句: ");
        for seg in res.segments.iter().take(2) {
            println!("     [{:.2}s -> {:.2}s] ({}): {}", seg.start_sec, seg.end_sec, seg.speaker, seg.text);
        }
    }

    #[tokio::test]
    async fn test_service_asr_e2e_japanese_pure() {
        let fixture_audio = resolve_fixture_path("ASR_日语_购物核实年龄.mp3");
        if !fixture_audio.exists() {
            println!("⚠️ [跳过测试] 日文基准音频不存在: {:?}", fixture_audio);
            return;
        }

        let outs_dir = resolve_outs_dir();

        println!("\n🚀 [TDD 纯净打靶 - 日文] 正在测试 service-asr: {:?}", fixture_audio);

        let opts = AsrOptions {
            audio_path: fixture_audio.to_string_lossy().to_string(),
            mode: Some("verbatim".into()),
            language: Some("ja".into()),
        };

        let res = AsrService::run_asr_pipeline(opts)
            .await
            .expect("MOSS 0.9B ASR 日文语音打靶失败");

        println!("\n🎉 ===== [MOSS 0.9B 纯净 ASR 日文识别完成] =====");
        println!("  ⏱️ 耗时: {:.2} ms ({:.2} s) | 音频时长: {:.2}s | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.duration_sec, res.segments.len());

        assert!(res.segments.len() > 0, "转写台词数不可为 0！");

        let out_json_path = outs_dir.join("asr_japanese_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 日文纯净 JSON 成功落盘: {:?}", out_json_path);
        println!("  预览前 2 句: ");
        for seg in res.segments.iter().take(2) {
            println!("     [{:.2}s -> {:.2}s] ({}): {}", seg.start_sec, seg.end_sec, seg.speaker, seg.text);
        }
    }
}
