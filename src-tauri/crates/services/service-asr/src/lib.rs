use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use transcribe_cpp::{CancelToken, Diarize, Error as TranscribeError, Model, ModelOptions, RunOptions, Task, TimestampKind};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const CHUNK_DURATION_SEC: f64 = 300.0;
const SAMPLE_RATE: usize = 16000;
const CHUNK_SAMPLES: usize = (CHUNK_DURATION_SEC as usize) * SAMPLE_RATE;

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

pub type AsrProgressCallback = Arc<dyn Fn(usize, usize, &str) + Send + Sync + 'static>;

pub struct AsrService;

impl AsrService {
    fn resolve_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-Q5_K_M.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-Q4_K_M.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-ASR\MOSS-Transcribe-Diarize-Q5_K_M.gguf"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-Q5_K_M.gguf")
    }

    fn load_audio_pcm_16k_mono(audio_path: &Path) -> Result<Vec<f32>, String> {
        if let Ok(mut reader) = hound::WavReader::open(audio_path) {
            let spec = reader.spec();
            if spec.sample_rate == 16000 && spec.channels == 1 {
                if spec.sample_format == hound::SampleFormat::Int {
                    let samples: Vec<f32> = reader
                        .samples::<i16>()
                        .filter_map(|s| s.ok())
                        .map(|s| s as f32 / 32768.0)
                        .collect();
                    if !samples.is_empty() {
                        return Ok(samples);
                    }
                } else if spec.sample_format == hound::SampleFormat::Float {
                    let samples: Vec<f32> = reader
                        .samples::<f32>()
                        .filter_map(|s| s.ok())
                        .collect();
                    if !samples.is_empty() {
                        return Ok(samples);
                    }
                }
            }
        }

        let mut cmd = std::process::Command::new("ffmpeg");
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd
            .arg("-y")
            .arg("-i")
            .arg(audio_path)
            .arg("-f")
            .arg("s16le")
            .arg("-ar")
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg("pipe:1")
            .output()
            .map_err(|e| format!("调起 FFmpeg 解码器失败: {e}"))?;

        if !output.status.success() {
            return Err(format!(
                "FFmpeg 音频解码失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let pcm_f32: Vec<f32> = output
            .stdout
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect();

        if pcm_f32.is_empty() {
            return Err("解码后的 PCM 音频采样数据为空！".into());
        }

        Ok(pcm_f32)
    }

    pub async fn run_asr_pipeline(options: AsrOptions) -> Result<AsrResult, String> {
        Self::run_asr_pipeline_cancellable(options, None, None).await
    }

    /// 🛡️ 纯血 Rust Native ASR 底座：具备 OutputTruncated 弹性容错、毫秒级截停与细粒度进度广播
    pub async fn run_asr_pipeline_cancellable(
        options: AsrOptions,
        cancel_token: Option<Arc<AtomicBool>>,
        on_progress: Option<AsrProgressCallback>,
    ) -> Result<AsrResult, String> {
        let audio_path_buf = PathBuf::from(&options.audio_path);
        if !audio_path_buf.exists() {
            return Err(format!("音频物理文件不存在: {}", options.audio_path));
        }
        let model_path = Self::resolve_model_path();
        if !model_path.exists() {
            return Err(format!("GGUF ASR 模型物理文件不存在: {:?}", model_path));
        }

        if let Some(ref ct) = cancel_token {
            if ct.load(Ordering::Relaxed) {
                return Err("任务已由用户主动取消".into());
            }
        }

        let t0 = Instant::now();
        let pcm_samples = Self::load_audio_pcm_16k_mono(&audio_path_buf)?;
        let total_samples = pcm_samples.len();
        let total_duration_sec = total_samples as f64 / SAMPLE_RATE as f64;

        tracing::info!(
            "🎙️ [ASR 底座] 音频解码完成: 共 {:.2} 秒 ({:.2} 分钟) | 总采样点: {}",
            total_duration_sec,
            total_duration_sec / 60.0,
            total_samples
        );

        let progress_cb = on_progress.clone();
        let all_segments = tokio::task::spawn_blocking(move || -> Result<Vec<RawSegment>, String> {
            let model = Model::load_with(&model_path, &ModelOptions::default())
                .map_err(|e| format!("纯血 C-FFI 载入 GGUF 模型失败: {e}"))?;
            let mut session = model.session()
                .map_err(|e| format!("创建 transcribe Session 失败: {e}"))?;

            let cpp_cancel = CancelToken::default();
            session.set_cancel_token(&cpp_cancel);

            let watcher_stop = Arc::new(AtomicBool::new(false));
            let watcher_stop_clone = watcher_stop.clone();
            let ct_clone = cancel_token.clone();
            let cpp_cancel_clone = cpp_cancel.clone();

            if let Some(ct) = ct_clone {
                std::thread::spawn(move || {
                    while !watcher_stop_clone.load(Ordering::Relaxed) {
                        if ct.load(Ordering::Relaxed) {
                            cpp_cancel_clone.cancel();
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                });
            }

            let mut run_opts = RunOptions::default();
            run_opts.task = Task::Transcribe;
            run_opts.timestamps = TimestampKind::Auto;
            run_opts.diarize = Diarize::On;
            if let Some(lang) = &options.language {
                if lang != "auto" && !lang.is_empty() {
                    run_opts.language = Some(lang.clone());
                }
            }

            let mut collected_segments: Vec<RawSegment> = Vec::new();
            let mut chunk_start_idx = 0;
            let mut chunk_id = 1;
            let total_chunks = (total_samples + CHUNK_SAMPLES - 1) / CHUNK_SAMPLES;

            while chunk_start_idx < total_samples {
                if let Some(ref ct) = cancel_token {
                    if ct.load(Ordering::Relaxed) {
                        watcher_stop.store(true, Ordering::Relaxed);
                        return Err("任务已由用户主动取消".into());
                    }
                }

                let chunk_end_idx = (chunk_start_idx + CHUNK_SAMPLES).min(total_samples);
                let chunk_slice = &pcm_samples[chunk_start_idx..chunk_end_idx];
                let chunk_time_offset_sec = chunk_start_idx as f64 / SAMPLE_RATE as f64;
                let chunk_end_sec = chunk_end_idx as f64 / SAMPLE_RATE as f64;

                if let Some(ref cb) = progress_cb {
                    let msg = format!(
                        "MOSS 0.9B 语音转写中 ({}/{} 块, 已识别 {:.1}s/{:.1}s)",
                        chunk_id, total_chunks, chunk_end_sec, total_duration_sec
                    );
                    cb(chunk_id, total_chunks, &msg);
                }

                tracing::info!(
                    "⚡ [ASR 切片推导] 正在执行分块 [{}/{}] | 时间偏移: {:.2}s ~ {:.2}s",
                    chunk_id,
                    total_chunks,
                    chunk_time_offset_sec,
                    chunk_end_sec
                );

                let transcript_res = session.run(chunk_slice, &run_opts);
                let transcript = match transcript_res {
                    Ok(t) => t,
                    Err(TranscribeError::OutputTruncated { partial: Some(partial_t), .. }) => {
                        tracing::warn!(
                            "⚠️ [ASR 预算截断保护] 切片 [{}/{}] 语流极密集触发预算上限，已自动提取已生成台词并继续平滑接力",
                            chunk_id, total_chunks
                        );
                        *partial_t
                    }
                    Err(e) => {
                        watcher_stop.store(true, Ordering::Relaxed);
                        if session.was_aborted() || cancel_token.as_ref().map_or(false, |ct| ct.load(Ordering::Relaxed)) {
                            return Err("任务已由用户主动取消".into());
                        }
                        return Err(format!("切片 [{chunk_id}/{total_chunks}] 推导失败: {e}"));
                    }
                };

                for seg in transcript.segments {
                    let text = seg.text.trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    let speaker = if seg.speaker_id > 0 {
                        format!("S{:02}", seg.speaker_id)
                    } else {
                        "S01".to_string()
                    };
                    let start_sec = chunk_time_offset_sec + (seg.t0_ms as f64 / 1000.0);
                    let end_sec = chunk_time_offset_sec + (seg.t1_ms as f64 / 1000.0);
                    let seg_id = collected_segments.len() + 1;
                    collected_segments.push(RawSegment {
                        id: seg_id,
                        speaker,
                        start_sec: (start_sec * 1000.0).round() / 1000.0,
                        end_sec: (end_sec * 1000.0).round() / 1000.0,
                        text,
                    });
                }

                chunk_start_idx += CHUNK_SAMPLES;
                chunk_id += 1;
            }

            watcher_stop.store(true, Ordering::Relaxed);
            Ok(collected_segments)
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        .map_err(|e| e)?;

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let audio_file = audio_path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "audio".to_string());

        Ok(AsrResult {
            audio_file,
            duration_sec: total_duration_sec,
            segments: all_segments,
            elapsed_ms,
        })
    }
}
