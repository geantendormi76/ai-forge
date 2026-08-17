pub mod engine;
pub mod probe;

pub use engine::{
    burn_hard_subtitles_nvenc, clean_acoustic_noise, estimate_text_width, extract_audio_from_video,
    extract_subtitle_stream, generate_ass, generate_srt, mux_soft_subtitles, parse_srt_content,
    smart_wrap_line, EngineSegment, ParsedSrtSegment,
};
pub use probe::{SubtitleSourceKind, SubtitleTrackInfo, VideoProbeResult, VideoProbeService};

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
    /// 极简双轨调度：软字幕直通 / 生肉纯音频听写 (100% 纯血 Rust 架构)
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
                tracing::info!("⚡ [双轨调度] 命中内嵌软字幕流，启动 0.05s 直通纯血 Rust 神经翻译通道");
                Self::run_soft_stream_pipeline(options, vram_guard).await
            }
            SubtitleSourceKind::RawAudio => {
                tracing::info!("🎙️ [双轨调度] 启动 MOSS 0.9B ASR 纯语音听写与翻译通道 (纯血 Rust)");
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

        let probe_res = VideoProbeService::probe(video_path).await.unwrap_or_else(|_| {
            VideoProbeResult {
                video_path: options.video_path.clone(),
                duration_sec: 0.0,
                width: 1920,
                height: 1080,
                has_audio: true,
                audio_streams_count: 1,
                soft_tracks: Vec::new(),
                recommended_mode: SubtitleSourceKind::EmbeddedSoftStream,
                elapsed_ms: 0.0,
            }
        });

        let stream_idx = if let Some(idx) = options.subtitle_stream_index {
            idx
        } else {
            probe_res.soft_tracks.first().map(|t| t.stream_index).unwrap_or(2)
        };

        let timestamp_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_srt_path = output_dir.join(format!("temp_extracted_{}.srt", timestamp_now));

        let raw_segments = engine::extract_subtitle_stream(video_path, stream_idx, &temp_srt_path).await?;
        let _ = tokio::fs::remove_file(&temp_srt_path).await;

        if raw_segments.is_empty() {
            return Err("抽离出的字幕轨道没有任何有效文本！".into());
        }

        let _vram_permit = if let Some(guard) = vram_guard {
            match guard.acquire(TaskWeight::Light).await {
                Ok(permit) => Some(permit),
                Err(_) => None,
            }
        } else {
            None
        };

        let source_texts: Vec<String> = raw_segments.iter().map(|s| s.text.clone()).collect();
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

        let mut engine_segments = Vec::new();
        let mut result_segments = Vec::new();

        for (idx, seg) in raw_segments.into_iter().enumerate() {
            let target_text = trans_res.translations.get(idx).cloned().unwrap_or_default();
            engine_segments.push(EngineSegment {
                id: seg.id,
                speaker: seg.speaker.clone(),
                start_sec: seg.start_sec,
                end_sec: seg.end_sec,
                source_text: seg.text.clone(),
                target_text: target_text.clone(),
            });
            result_segments.push(SubtitleSegmentResult {
                id: seg.id,
                speaker: seg.speaker,
                start_sec: seg.start_sec,
                end_sec: seg.end_sec,
                source_text: seg.text,
                target_text,
            });
        }

        let display_mode_str = match options.display_mode {
            DisplayMode::Bilingual => "bilingual",
            DisplayMode::TargetOnly => "target_only",
            DisplayMode::SourceOnly => "source_only",
        };

        let base_name = video_path.file_stem().and_then(|s| s.to_str()).unwrap_or("video");
        let srt_path = output_dir.join(format!("{}_subtitle.srt", base_name));
        let srt_content = engine::generate_srt(&engine_segments, display_mode_str, options.show_speaker);
        tokio::fs::write(&srt_path, srt_content).await
            .map_err(|e| format!("写入 SRT 字幕失败: {e}"))?;

        let ass_path = output_dir.join(format!("{}_subtitle.ass", base_name));
        let ass_content = engine::generate_ass(
            &engine_segments,
            display_mode_str,
            options.show_speaker,
            options.font_size_multiplier,
            probe_res.width,
            probe_res.height,
            options.mask_hardsub,
        );
        tokio::fs::write(&ass_path, ass_content).await
            .map_err(|e| format!("写入 ASS 字幕失败: {e}"))?;

        let json_path = output_dir.join(format!("{}_bilingual.json", base_name));
        let json_payload = serde_json::json!({
            "video_path": options.video_path,
            "total_segments": result_segments.len(),
            "segments": result_segments
        });
        tokio::fs::write(&json_path, serde_json::to_string_pretty(&json_payload).unwrap()).await
            .map_err(|e| format!("写入 JSON 失败: {e}"))?;

        let output_video_path = match options.output_mode {
            OutputMode::HardMp4Nvenc => {
                let out_mp4 = output_dir.join(format!("{}_zidian_burned.mp4", base_name));
                engine::burn_hard_subtitles_nvenc(video_path, &ass_path, &out_mp4).await?;
                out_mp4.to_string_lossy().to_string()
            }
            OutputMode::SoftMkv => {
                let out_mkv = output_dir.join(format!("{}_zidian_muxed.mkv", base_name));
                engine::mux_soft_subtitles(video_path, &ass_path, &out_mkv).await?;
                out_mkv.to_string_lossy().to_string()
            }
        };

        let elapsed_ms = t0.elapsed().as_millis() as f64;

        Ok(VideoSubtitleResult {
            success: true,
            output_video_path,
            srt_path: srt_path.to_string_lossy().to_string(),
            ass_path: ass_path.to_string_lossy().to_string(),
            json_path: json_path.to_string_lossy().to_string(),
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

        // 1. 纯血 Rust 调起原生 FFmpeg 提取 16k mono 音频
        engine::extract_audio_from_video(video_path, &temp_wav_path).await?;

        // 2. MOSS 0.9B ASR 原位听写
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

        let _ = tokio::fs::remove_file(&temp_wav_path).await;

        if asr_res.segments.is_empty() {
            return Err("视频中未识别出任何语音台词！".into());
        }

        let source_texts: Vec<String> = asr_res.segments.iter().map(|s| s.text.clone()).collect();
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

        // 3. Hy-MT2 1.8B 神经翻译
        let trans_req = PureTranslationRequest {
            texts: prepared_texts,
            target_lang: Some(options.target_lang.clone()),
        };

        let trans_res = TranslationService::run_translation_pipeline(trans_req)
            .await
            .map_err(|e| format!("Hy-MT2 1.8B 神经翻译流水线失败: {e}"))?;

        let mut engine_segments = Vec::new();
        let mut result_segments = Vec::new();

        for (idx, seg) in asr_res.segments.iter().enumerate() {
            let target_text = trans_res.translations.get(idx).cloned().unwrap_or_default();
            engine_segments.push(EngineSegment {
                id: seg.id,
                speaker: seg.speaker.clone(),
                start_sec: seg.start_sec,
                end_sec: seg.end_sec,
                source_text: seg.text.clone(),
                target_text: target_text.clone(),
            });
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

        let base_name = video_path.file_stem().and_then(|s| s.to_str()).unwrap_or("video");
        let srt_path = output_dir.join(format!("{}_subtitle.srt", base_name));
        let srt_content = engine::generate_srt(&engine_segments, display_mode_str, options.show_speaker);
        tokio::fs::write(&srt_path, srt_content).await
            .map_err(|e| format!("写入 SRT 字幕失败: {e}"))?;

        let ass_path = output_dir.join(format!("{}_subtitle.ass", base_name));
        let ass_content = engine::generate_ass(
            &engine_segments,
            display_mode_str,
            options.show_speaker,
            options.font_size_multiplier,
            probe_res.width,
            probe_res.height,
            options.mask_hardsub,
        );
        tokio::fs::write(&ass_path, ass_content).await
            .map_err(|e| format!("写入 ASS 字幕失败: {e}"))?;

        let json_path = output_dir.join(format!("{}_bilingual.json", base_name));
        let json_payload = serde_json::json!({
            "video_path": options.video_path,
            "total_segments": result_segments.len(),
            "segments": result_segments
        });
        tokio::fs::write(&json_path, serde_json::to_string_pretty(&json_payload).unwrap()).await
            .map_err(|e| format!("写入 JSON 失败: {e}"))?;

        let output_video_path = match options.output_mode {
            OutputMode::HardMp4Nvenc => {
                let out_mp4 = output_dir.join(format!("{}_zidian_burned.mp4", base_name));
                engine::burn_hard_subtitles_nvenc(video_path, &ass_path, &out_mp4).await?;
                out_mp4.to_string_lossy().to_string()
            }
            OutputMode::SoftMkv => {
                let out_mkv = output_dir.join(format!("{}_zidian_muxed.mkv", base_name));
                engine::mux_soft_subtitles(video_path, &ass_path, &out_mkv).await?;
                out_mkv.to_string_lossy().to_string()
            }
        };

        let elapsed_ms = t0.elapsed().as_millis() as f64;

        Ok(VideoSubtitleResult {
            success: true,
            output_video_path,
            srt_path: srt_path.to_string_lossy().to_string(),
            ass_path: ass_path.to_string_lossy().to_string(),
            json_path: json_path.to_string_lossy().to_string(),
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
    async fn test_real_physical_1_mp4_e2e() {
        let input_path = PathBuf::from(r"C:\dev\ai-forge\test\input\video-subtitle\1.mp4");
        let output_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\video-subtitle");
        let _ = tokio::fs::create_dir_all(&output_dir).await;

        if !input_path.exists() {
            eprintln!("⚠️ [跳过测试] 输入测试视频不存在: {:?}", input_path);
            return;
        }

        println!("\n🎬 ===== [1.mp4 3分钟生肉视频纯血 Rust 全流程转写与压制打靶] =====");
        println!("  输入视频: {:?}", input_path);
        println!("  输出目录: {:?}", output_dir);
        println!("  架构模式: 100% 纯血 Rust (零 Python, 零 IPC)");
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
            source_kind: None,
            subtitle_stream_index: None,
            mask_hardsub: true,
        };

        let res = VideoSubtitleTool::run_pipeline(opts, None).await
            .expect("1.mp4 双语字幕流水线执行失败");

        let total_elapsed = t0.elapsed().as_secs_f64();

        println!("\n🎉 ===== [1.mp4 纯血 Rust 双语转写全量打靶成功] =====");
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
