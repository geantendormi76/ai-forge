use core_ipc::{spawn_uv_worker, IpcMessage};
use serde::{Deserialize, Serialize};
use service_asr::{AsrOptions, AsrService};
use service_translation::{PureTranslationRequest, TranslationService};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Bilingual,
    TargetOnly,
    SourceOnly,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Bilingual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    SoftMkv,
    HardMp4Nvenc,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::SoftMkv
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryTerm {
    pub source_term: String,
    pub target_term: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSubtitleOptions {
    pub video_path: String,
    pub output_dir: Option<String>,
    pub target_lang: String,
    pub display_mode: DisplayMode,
    pub show_speaker: bool,
    pub font_size_multiplier: f32,
    pub output_mode: OutputMode,
    pub hotwords: Option<String>,
    pub glossary: Option<Vec<GlossaryTerm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubtitleSegmentResult {
    pub id: usize,
    pub speaker: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub source_text: String,
    pub target_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSubtitleResult {
    pub success: bool,
    pub output_video_path: String,
    pub srt_path: String,
    pub ass_path: String,
    pub json_path: String,
    pub total_segments: usize,
    pub segments: Vec<SubtitleSegmentResult>,
    pub elapsed_ms: f64,
    pub error: Option<String>,
}

pub struct VideoSubtitleTool;

impl VideoSubtitleTool {
    fn resolve_tool_dir() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\crates\tools\tool-video-subtitle"),
            PathBuf::from(r"C:\dev\ai-toolkit\src-tauri\crates\tools\tool-video-subtitle"),
            PathBuf::from("/home/zhz/ai-forge/src-tauri/crates/tools/tool-video-subtitle"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifest_dir);
        }
        PathBuf::from("src-tauri/crates/tools/tool-video-subtitle")
    }

    pub async fn run_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<VideoSubtitleResult, String> {
        let t0 = std::time::Instant::now();
        let video_path = Path::new(&options.video_path);
        if !video_path.exists() {
            return Err(format!("输入的物理视频文件不存在: {}", options.video_path));
        }

        let output_dir = if let Some(dir) = &options.output_dir {
            PathBuf::from(dir)
        } else {
            video_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        let _ = tokio::fs::create_dir_all(&output_dir).await;

        let tool_dir = Self::resolve_tool_dir();
        let script_path = tool_dir.join("scripts/worker.py");

        let _vram_permit = if let Some(guard) = vram_guard {
            match guard.acquire(TaskWeight::Heavy).await {
                Ok(permit) => {
                    tracing::info!("🎮 [显存金库得手] 锁住 8,000MB 显存 Token，进入紫电 AI 视频字幕流水线");
                    Some(permit)
                }
                Err(e) => {
                    tracing::warn!("⚠️ 显存不足或申请失败: {e}，切入降级防护");
                    None
                }
            }
        } else {
            None
        };

        let timestamp_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_wav_path = output_dir.join(format!("temp_zidian_{}.wav", timestamp_now));
        
        let (mut child, mut channel) = spawn_uv_worker(&tool_dir, &script_path)
            .map_err(|e| format!("启动 tool-video-subtitle Worker 失败: {e}"))?;

        let ready_msg = channel.recv().await
            .map_err(|e| format!("接收 Worker 就绪信号失败: {e}"))?;
            
        if ready_msg.method != "system.ready" {
            tracing::warn!("⚠️ Worker 响应非预期信号: {}", ready_msg.method);
        }

        let extract_req = IpcMessage {
            method: "extract_audio".to_string(),
            params: serde_json::json!({
                "video_path": options.video_path,
                "out_wav_path": temp_wav_path.to_string_lossy()
            }),
        };

        channel.send(&extract_req).await
            .map_err(|e| format!("发送音频提取请求失败: {e}"))?;

        let extract_res = channel.recv().await
            .map_err(|e| format!("接收音频提取响应失败: {e}"))?;

        if extract_res.params.get("success") != Some(&serde_json::Value::Bool(true)) {
            let err_msg = extract_res.params.get("error").and_then(|v| v.as_str()).unwrap_or("提取音频失败");
            return Err(format!("FFmpeg 音频提取错误: {err_msg}"));
        }

        // 🛡️ 语种自动识别：传入 "auto" 给 MOSS 0.9B 自由侦测
        let asr_opts = AsrOptions {
            audio_path: temp_wav_path.to_string_lossy().to_string(),
            mode: Some("verbatim".into()),
            language: Some("auto".into()),
        };

        let asr_res = AsrService::run_asr_pipeline(asr_opts)
            .await
            .map_err(|e| format!("MOSS 0.9B ASR 听写流水线失败: {e}"))?;

        if asr_res.segments.is_empty() {
            let _ = tokio::fs::remove_file(&temp_wav_path).await;
            return Err("视频中未识别出任何语音台词！".into());
        }

        let mut source_texts = Vec::new();
        for seg in &asr_res.segments {
            source_texts.push(seg.text.clone());
        }

        let mut prepared_texts = Vec::new();
        if let Some(terms) = &options.glossary {
            if !terms.is_empty() {
                let mut term_prompt = String::from("参考下面的翻译：\n");
                for t in terms {
                    term_prompt.push_str(&format!("{} 翻译成 {}\n", t.source_term, t.target_term));
                }
                for txt in &source_texts {
                    prepared_texts.push(format!("{}\n{}", term_prompt.trim_end(), txt));
                }
            } else {
                prepared_texts = source_texts.clone();
            }
        } else {
            prepared_texts = source_texts.clone();
        }

        let trans_req = PureTranslationRequest {
            texts: prepared_texts,
            target_lang: Some(options.target_lang.clone()),
        };

        let trans_res = TranslationService::run_translation_pipeline(trans_req)
            .await
            .map_err(|e| format!("Hy-MT2 1.8B 神经翻译流水线失败: {e}"))?;

        let mut result_segments = Vec::new();
        for (idx, seg) in asr_res.segments.iter().enumerate() {
            let target_text = trans_res.translations.get(idx).cloned().unwrap_or_default();
            result_segments.push(SubtitleSegmentResult {
                id: seg.id,
                speaker: seg.speaker.clone(),
                start_sec: seg.start_sec,
                end_sec: seg.end_sec,
                source_text: seg.text.clone(),
                target_text,
            });
        }

        let display_mode_str = match options.display_mode {
            DisplayMode::Bilingual => "bilingual",
            DisplayMode::TargetOnly => "target_only",
            DisplayMode::SourceOnly => "source_only",
        };

        let output_mode_str = match options.output_mode {
            OutputMode::SoftMkv => "soft_mkv",
            OutputMode::HardMp4Nvenc => "hard_mp4_nvenc",
        };

        let render_req = IpcMessage {
            method: "render_and_mux".to_string(),
            params: serde_json::json!({
                "video_path": options.video_path,
                "output_dir": output_dir.to_string_lossy(),
                "segments": result_segments,
                "display_mode": display_mode_str,
                "show_speaker": options.show_speaker,
                "font_size_multiplier": options.font_size_multiplier,
                "output_mode": output_mode_str
            }),
        };

        channel.send(&render_req).await
            .map_err(|e| format!("发送渲染挂载请求失败: {e}"))?;

        let render_res = channel.recv().await
            .map_err(|e| format!("接收渲染挂载响应失败: {e}"))?;

        let exit_msg = IpcMessage {
            method: "exit".to_string(),
            params: serde_json::json!({}),
        };
        let _ = channel.send(&exit_msg).await;
        let _ = child.wait().await;

        let _ = tokio::fs::remove_file(&temp_wav_path).await;

        let res_params = &render_res.params;
        if res_params.get("success") != Some(&serde_json::Value::Bool(true)) {
            let err_msg = res_params.get("error").and_then(|v| v.as_str()).unwrap_or("视频渲染压制失败");
            return Err(format!("字幕渲染压制错误: {err_msg}"));
        }

        let srt_path = res_params.get("srt_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let ass_path = res_params.get("ass_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let json_path = res_params.get("json_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let output_video_path = res_params.get("output_video_path").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let elapsed_ms = t0.elapsed().as_millis() as f64;

        Ok(VideoSubtitleResult {
            success: true,
            output_video_path,
            srt_path,
            ass_path,
            json_path,
            total_segments: result_segments.len(),
            segments: result_segments,
            elapsed_ms,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_video_fixture_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\tool-video-subtitle.mp4"),
            PathBuf::from(r"C:\dev\ai-toolkit\test\fixtures\tool-video-subtitle.mp4"),
            PathBuf::from(r"D:\视频\教材\欧美专区\05.adriana chechik.mp4"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\test\fixtures\tool-video-subtitle.mp4")
    }

    #[test]
    fn test_video_subtitle_dto_purity() {
        let opts = VideoSubtitleOptions {
            video_path: "C:\\test\\video.mp4".into(),
            output_dir: None,
            target_lang: "Chinese".into(),
            display_mode: DisplayMode::Bilingual,
            show_speaker: false,
            font_size_multiplier: 1.8,
            output_mode: OutputMode::SoftMkv,
            hotwords: Some("Tauri, Rust".into()),
            glossary: Some(vec![GlossaryTerm {
                source_term: "AI-Forge".into(),
                target_term: "紫电AI桌面工坊".into(),
            }]),
        };

        let json_str = serde_json::to_string_pretty(&opts).unwrap();
        assert!(json_str.contains("font_size_multiplier"));
        assert!(json_str.contains("bilingual"));
        println!("\n✅ [tool-video-subtitle DTO 契约测试通过]:\n{}", json_str);
    }

    #[tokio::test]
    async fn test_real_video_subtitle_pipeline() {
        let video_path_buf = resolve_video_fixture_path();
        if !video_path_buf.exists() {
            println!("⚠️ [跳过测试] 目标 TDD 基准视频不存在: {:?}", video_path_buf);
            return;
        }

        let out_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\tool-video-subtitle");
        let _ = std::fs::create_dir_all(&out_dir);

        let video_path = video_path_buf.to_string_lossy().to_string();
        println!("\n🚀 [TDD 标准基准视频打靶启动] 视频物理路径: {}", video_path);

        let opts = VideoSubtitleOptions {
            video_path: video_path.clone(),
            output_dir: Some(out_dir.to_string_lossy().to_string()),
            target_lang: "Chinese".into(),
            display_mode: DisplayMode::Bilingual,
            show_speaker: false,
            font_size_multiplier: 1.8,
            output_mode: OutputMode::SoftMkv,
            hotwords: None,
            glossary: None,
        };

        let vram_guard = VramTokenGuard::default_rtx3060();

        let res = VideoSubtitleTool::run_pipeline(opts, Some(&vram_guard))
            .await
            .expect("紫电 AI 视频双语字幕工坊 TDD 打靶失败！");

        println!("\n🎉 ===== [紫电 AI 视频双语字幕工坊 TDD 打靶完成] =====");
        println!("  ⏱️ 端到端物理总耗时: {:.2} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms / 1000.0);
        println!("  📊 共转写并翻译台词: {} 句", res.total_segments);
        println!("  🎬 封装产物视频物理路径: {}", res.output_video_path);
        println!("  📄 SRT 字幕物理路径: {}", res.srt_path);
        println!("  🎨 ASS 字幕物理路径: {}", res.ass_path);
        println!("  📝 JSON 结构化物理路径: {}", res.json_path);
        println!("  预览前 3 句双语字幕对照效果:");
        for seg in res.segments.iter().take(3) {
            println!("     [{:.2}s -> {:.2}s] [{}] 原文: {} ➔ 译文: {}", seg.start_sec, seg.end_sec, seg.speaker, seg.source_text, seg.target_text);
        }
        println!("==================================================\n");

        assert!(res.success);
        assert!(Path::new(&res.output_video_path).exists());
        assert!(Path::new(&res.json_path).exists());
    }
}
