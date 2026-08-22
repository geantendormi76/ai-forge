pub mod engine;
pub mod probe;

pub use engine::{
    assign_overlap_lanes, burn_hard_subtitles_nvenc, clean_acoustic_noise, estimate_text_width,
    extract_audio_from_video, extract_subtitle_stream, generate_ass, generate_srt,
    mux_soft_subtitles, parse_srt_content, smart_wrap_line, EngineSegment, ParsedSrtSegment,
};
pub use probe::{SubtitleSourceKind, SubtitleTrackInfo, VideoProbeResult, VideoProbeService};

use serde::{Deserialize, Serialize};
use service_asr::{AsrOptions, AsrService};
use service_translation::{PureTranslationRequest, TranslationService};
use shared_contracts::{TaskWeight, VramTokenGuard};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

pub type SubtitleProgressCallback = Arc<dyn Fn(usize, usize, &str) + Send + Sync + 'static>;

pub struct VideoSubtitleTool;

impl VideoSubtitleTool {
    pub async fn run_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
    ) -> Result<VideoSubtitleResult, String> {
        Self::run_pipeline_with_progress(options, vram_guard, Arc::new(AtomicBool::new(false)), None).await
    }

    pub async fn run_pipeline_cancellable(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<VideoSubtitleResult, String> {
        Self::run_pipeline_with_progress(options, vram_guard, cancel_token, None).await
    }

    /// 🛡️ 全生命周期白盒进度调度流水线
    pub async fn run_pipeline_with_progress(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
        cancel_token: Arc<AtomicBool>,
        on_progress: Option<SubtitleProgressCallback>,
    ) -> Result<VideoSubtitleResult, String> {
        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }
        let video_path = Path::new(&options.video_path);
        if !video_path.exists() {
            return Err(format!("输入的物理视频文件不存在: {}", options.video_path));
        }

        if let Some(ref cb) = on_progress {
            cb(2, 100, "正在探测视频多模态轨道...");
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
                Self::run_soft_stream_pipeline(options, vram_guard, cancel_token, on_progress).await
            }
            SubtitleSourceKind::RawAudio => {
                tracing::info!("🎙️ [双轨调度] 启动 MOSS 0.9B ASR 纯语音听写与翻译通道 (纯血 Rust)");
                Self::run_raw_audio_pipeline(options, vram_guard, cancel_token, on_progress).await
            }
        }
    }

    pub async fn run_soft_stream_pipeline(
        options: VideoSubtitleOptions,
        vram_guard: Option<&VramTokenGuard>,
        cancel_token: Arc<AtomicBool>,
        on_progress: Option<SubtitleProgressCallback>,
    ) -> Result<VideoSubtitleResult, String> {
        let t0 = std::time::Instant::now();
        let video_path = Path::new(&options.video_path);
        let output_dir = if let Some(dir) = &options.output_dir {
            PathBuf::from(dir)
        } else {
            video_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        let _ = tokio::fs::create_dir_all(&output_dir).await;

        if let Some(ref cb) = on_progress {
            cb(10, 100, "正在抽离内嵌软字幕轨道...");
        }

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

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }
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
        let trans_req = PureTranslationRequest {
            texts: source_texts,
            target_lang: Some(options.target_lang.clone()),
        };

        let prog_cb_clone = on_progress.clone();
        let trans_progress: Option<service_translation::TranslationProgressCallback> = prog_cb_clone.map(|cb| {
            Arc::new(move |cur: usize, total: usize, _msg: &str| {
                let scaled_pct = 20 + ((cur as f64 / total as f64) * 60.0) as usize;
                let msg = format!("Hy-MT2 神经翻译中 ({}/{})", cur, total);
                cb(scaled_pct, 100, &msg);
            }) as service_translation::TranslationProgressCallback
        });

        let trans_res = TranslationService::run_translation_pipeline_with_progress(trans_req, trans_progress)
            .await
            .map_err(|e| format!("Hy-MT2 神经翻译失败: {e}"))?;

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }

        if let Some(ref cb) = on_progress {
            cb(85, 100, "正在生成 ASS 特效字幕与排版...");
        }

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

        if let Some(ref cb) = on_progress {
            cb(92, 100, "正在封装交付最终视频...");
        }

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

        if let Some(ref cb) = on_progress {
            cb(100, 100, "双语字幕处理完成！");
        }

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
        cancel_token: Arc<AtomicBool>,
        on_progress: Option<SubtitleProgressCallback>,
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

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }

        if let Some(ref cb) = on_progress {
            cb(5, 100, "FFmpeg 抽取 16kHz 高保真音频流...");
        }

        let timestamp_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_wav_path = output_dir.join(format!("temp_zidian_{}.wav", timestamp_now));
        engine::extract_audio_from_video(video_path, &temp_wav_path).await?;

        if cancel_token.load(Ordering::Relaxed) {
            let _ = tokio::fs::remove_file(&temp_wav_path).await;
            return Err("任务已由用户主动取消".into());
        }

        let asr_opts = AsrOptions {
            audio_path: temp_wav_path.to_string_lossy().to_string(),
            language: Some("auto".into()),
            prompt: None,
            hotwords: options.hotwords.clone(),
            max_new_tokens: Some(4096),
            temperature: Some(0.0),
        };

        let prog_cb_for_asr = on_progress.clone();
        let asr_progress: Option<service_asr::AsrProgressCallback> = prog_cb_for_asr.map(|cb| {
            Arc::new(move |cur: usize, total: usize, msg: &str| {
                let scaled_pct = 10 + ((cur as f64 / total as f64) * 45.0) as usize;
                cb(scaled_pct, 100, msg);
            }) as service_asr::AsrProgressCallback
        });

        let asr_res = AsrService::run_asr_pipeline_cancellable(asr_opts, Some(cancel_token.clone()), asr_progress)
            .await
            .map_err(|e| format!("MOSS 0.9B ASR 听写流水线失败: {e}"))?;

        let _ = tokio::fs::remove_file(&temp_wav_path).await;

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }
        if asr_res.segments.is_empty() {
            return Err("视频中未识别出任何语音台词！".into());
        }

        let source_texts: Vec<String> = asr_res.segments.iter().map(|s| s.text.clone()).collect();
        let trans_req = PureTranslationRequest {
            texts: source_texts,
            target_lang: Some(options.target_lang.clone()),
        };

        let prog_cb_for_trans = on_progress.clone();
        let trans_progress: Option<service_translation::TranslationProgressCallback> = prog_cb_for_trans.map(|cb| {
            Arc::new(move |cur: usize, total: usize, _msg: &str| {
                let scaled_pct = 58 + ((cur as f64 / total as f64) * 28.0) as usize;
                let msg = format!("Hy-MT2 神经翻译中 ({}/{})", cur, total);
                cb(scaled_pct, 100, &msg);
            }) as service_translation::TranslationProgressCallback
        });

        let trans_res = TranslationService::run_translation_pipeline_with_progress(trans_req, trans_progress)
            .await
            .map_err(|e| format!("Hy-MT2 1.8B 神经翻译流水线失败: {e}"))?;

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }

        if let Some(ref cb) = on_progress {
            cb(88, 100, "正在生成双语字幕与 ASS 特效胶囊...");
        }

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

        if let Some(ref cb) = on_progress {
            cb(92, 100, "正在封装/压制最终双语视频...");
        }

        if cancel_token.load(Ordering::Relaxed) {
            return Err("任务已由用户主动取消".into());
        }

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

        if let Some(ref cb) = on_progress {
            cb(100, 100, "双语视频字幕生成完成！");
        }

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
