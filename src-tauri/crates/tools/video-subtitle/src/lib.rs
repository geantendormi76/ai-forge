pub mod probe;

pub use probe::{SubtitleSourceKind, SubtitleTrackInfo, VideoProbeResult, VideoProbeService};

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
    #[serde(default)]
    pub source_kind: Option<SubtitleSourceKind>,
    #[serde(default)]
    pub subtitle_stream_index: Option<usize>,
    /// 是否开启电影级羽化半透明遮罩 (温润遮挡原片硬字幕)
    #[serde(default)]
    pub mask_hardsub: bool,
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
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\crates\tools\video-subtitle"),
            PathBuf::from(r"C:\dev\ai-toolkit\src-tauri\crates\tools\video-subtitle"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifest_dir);
        }
        PathBuf::from("src-tauri/crates/tools/video-subtitle")
    }

    /// 极简双轨调度：软字幕直通 / 生肉纯音频听写
    pub async fn run_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<VideoSubtitleResult, String> {
        let video_path = Path::new(&options.video_path);
        if !video_path.exists() {
            return Err(format!("输入的物理视频文件不存在: {}", options.video_path));
        }

        let resolved_mode = if let Some(mode) = options.source_kind {
            mode
        } else if options.subtitle_stream_index.is_some() {
            SubtitleSourceKind::EmbeddedSoftStream
        } else {
            let probe_res = VideoProbeService::probe(video_path).await.unwrap_or_else(|_| {
                VideoProbeResult {
                    video_path: options.video_path.clone(),
                    duration_sec: 0.0,
                    width: 1920,
                    height: 1080,
                    has_audio: true,
                    audio_streams_count: 1,
                    soft_tracks: Vec::new(),
                    recommended_mode: SubtitleSourceKind::RawAudio,
                    elapsed_ms: 0.0,
                }
            });
            probe_res.recommended_mode
        };

        match resolved_mode {
            SubtitleSourceKind::EmbeddedSoftStream => {
                tracing::info!("⚡ [双轨调度] 命中内嵌软字幕流，启动 0.05s 直通神经翻译通道");
                Self::run_soft_stream_pipeline(options, vram_guard).await
            }
            SubtitleSourceKind::RawAudio => {
                tracing::info!("🎙️ [双轨调度] 启动 MOSS 0.9B ASR 纯语音听写与翻译通道");
                Self::run_raw_audio_pipeline(options, vram_guard).await
            }
        }
    }

    pub async fn run_soft_stream_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<VideoSubtitleResult, String> {
        let t0 = std::time::Instant::now();
        let video_path = Path::new(&options.video_path);
        let output_dir = if let Some(dir) = &options.output_dir {
            PathBuf::from(dir)
        } else {
            video_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        let _ = tokio::fs::create_dir_all(&output_dir).await;

        let stream_idx = if let Some(idx) = options.subtitle_stream_index {
            idx
        } else {
            let probe_res = VideoProbeService::probe(video_path).await?;
            probe_res.soft_tracks.first().map(|t| t.stream_index).unwrap_or(2)
        };

        let tool_dir = Self::resolve_tool_dir();
        let script_path = tool_dir.join("scripts/worker.py");

        let timestamp_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_srt_path = output_dir.join(format!("temp_extracted_{}.srt", timestamp_now));

        let (mut child, mut channel) = spawn_uv_worker(&tool_dir, &script_path)
            .map_err(|e| format!("启动 video-subtitle Worker 失败: {e}"))?;

        let _ready_msg = channel.recv().await
            .map_err(|e| format!("接收 Worker 就绪信号失败: {e}"))?;

        let extract_req = IpcMessage {
            method: "extract_subtitle_stream".to_string(),
            params: serde_json::json!({
                "video_path": options.video_path,
                "stream_index": stream_idx,
                "out_srt_path": temp_srt_path.to_string_lossy()
            }),
        };

        channel.send(&extract_req).await
            .map_err(|e| format!("发送字幕流提取请求失败: {e}"))?;

        let extract_res = channel.recv().await
            .map_err(|e| format!("接收字幕流提取响应失败: {e}"))?;

        let res_params = &extract_res.params;
        if res_params.get("success") != Some(&serde_json::Value::Bool(true)) {
            let err_msg = res_params.get("error").and_then(|v| v.as_str()).unwrap_or("抽离内嵌字幕流失败");
            let _ = child.kill().await;
            return Err(format!("FFmpeg 抽流错误: {err_msg}"));
        }

        let raw_segments_val = res_params.get("segments").and_then(|v| v.as_array())
            .ok_or_else(|| "Worker 未能返回有效的台词数组".to_string())?;

        if raw_segments_val.is_empty() {
            let _ = child.kill().await;
            let _ = tokio::fs::remove_file(&temp_srt_path).await;
            return Err("抽离出的字幕轨道没有任何有效文本！".into());
        }

        let mut parsed_segments = Vec::new();
        let mut source_texts = Vec::new();

        for item in raw_segments_val {
            let id = item["id"].as_u64().unwrap_or(0) as usize;
            let speaker = item["speaker"].as_str().unwrap_or("S01").to_string();
            let start_sec = item["start_sec"].as_f64().unwrap_or(0.0);
            let end_sec = item["end_sec"].as_f64().unwrap_or(0.0);
            let text = item["text"].as_str().unwrap_or("").to_string();

            source_texts.push(text.clone());
            parsed_segments.push((id, speaker, start_sec, end_sec, text));
        }

        let _vram_permit = if let Some(guard) = vram_guard {
            match guard.acquire(TaskWeight::Light).await {
                Ok(permit) => Some(permit),
                Err(_) => None,
            }
        } else {
            None
        };

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
            .map_err(|e| format!("Hy-MT2 神经翻译失败: {e}"))?;

        let mut result_segments = Vec::new();
        for (idx, (id, speaker, start_sec, end_sec, source_text)) in parsed_segments.into_iter().enumerate() {
            let target_text = trans_res.translations.get(idx).cloned().unwrap_or_default();
            result_segments.push(SubtitleSegmentResult {
                id,
                speaker,
                start_sec,
                end_sec,
                source_text,
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
                "output_mode": output_mode_str,
                "mask_hardsub": options.mask_hardsub
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

        let _ = tokio::fs::remove_file(&temp_srt_path).await;

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

    pub async fn run_raw_audio_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<VideoSubtitleResult, String> {
        let t0 = std::time::Instant::now();
        let video_path = Path::new(&options.video_path);
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
                Ok(permit) => Some(permit),
                Err(_) => None,
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
            .map_err(|e| format!("启动 video-subtitle Worker 失败: {e}"))?;

        let _ready_msg = channel.recv().await
            .map_err(|e| format!("接收 Worker 就绪信号失败: {e}"))?;

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

        let asr_opts = AsrOptions {
            audio_path: temp_wav_path.to_string_lossy().to_string(),
            language: Some("auto".into()),
            prompt: None,
            hotwords: options.hotwords.clone(),
            max_new_tokens: Some(4096),
            temperature: Some(0.0),
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
                "output_mode": output_mode_str,
                "mask_hardsub": options.mask_hardsub
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

    #[tokio::test]
    async fn test_real_physical_5_mkv_e2e() {
        let input_path = PathBuf::from(r"C:\dev\ai-forge\test\input\video-subtitle\5.mkv");
        let output_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\video-subtitle");
        let _ = tokio::fs::create_dir_all(&output_dir).await;

        if !input_path.exists() {
            eprintln!("⚠️ [跳过测试] 输入测试视频不存在: {:?}", input_path);
            return;
        }

        println!("\n🎬 ===== [5.mkv 26分钟长视频真机全流程转写与压制打靶] =====");
        println!("  输入视频: {:?}", input_path);
        println!("  输出目录: {:?}", output_dir);
        println!("  遮罩模式: 电影级柔和羽化渐晕 (mask_hardsub = true)");

        let t0 = std::time::Instant::now();

        let opts = VideoSubtitleOptions {
            video_path: input_path.to_string_lossy().to_string(),
            output_dir: Some(output_dir.to_string_lossy().to_string()),
            target_lang: "Chinese".into(),
            display_mode: DisplayMode::Bilingual,
            show_speaker: true,
            font_size_multiplier: 1.8,
            output_mode: OutputMode::SoftMkv,
            hotwords: None,
            glossary: None,
            source_kind: None, // 探针自动判定 RawAudio 听写
            subtitle_stream_index: None,
            mask_hardsub: true,
        };

        let res = VideoSubtitleTool::run_pipeline(opts, None).await
            .expect("5.mkv 双语字幕流水线执行失败");

        let total_elapsed = t0.elapsed().as_secs_f64();

        println!("\n🎉 ===== [5.mkv 26分钟长视频双语转写全量成功] =====");
        println!("  ⏱️ 全量物理耗时: {:.2} s ({:.2} 分钟) | 共生成 {} 句双语字幕", total_elapsed, total_elapsed / 60.0, res.total_segments);
        println!("  💾 1. 软字幕视频: {:?}", res.output_video_path);
        println!("  💾 2. SRT 字幕:   {:?}", res.srt_path);
        println!("  💾 3. ASS 特效字幕 (带柔和遮罩): {:?}", res.ass_path);
        println!("  💾 4. JSON 结构数据: {:?}", res.json_path);

        assert!(res.total_segments > 0, "提取台词数不可为 0！");
        assert!(Path::new(&res.output_video_path).exists(), "产物视频必须真实落盘！");

        println!("\n  👀 预览前 5 句双语对齐结果:");
        for seg in res.segments.iter().take(5) {
            println!("    [{:02}] [{:.2}s -> {:.2}s] [{}]", seg.id, seg.start_sec, seg.end_sec, seg.speaker);
            println!("        原文: {}", seg.source_text);
            println!("        译文: {}", seg.target_text);
        }
        println!("=======================================================\n");
    }
}
