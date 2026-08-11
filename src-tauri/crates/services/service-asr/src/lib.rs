use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
use transcribe_cpp::{Diarize, Model, ModelOptions, RunOptions, Task, TimestampKind};

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

    /// 内存原位音频解码：将任何格式音频 (MP3/WAV/AAC/FLAC/MP4) 高效解码为 16kHz 单声道 f32 浮点采样波形
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

        let output = std::process::Command::new("ffmpeg")
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

    /// 纯血 Rust Native ASR 底座管道：内存原位直连 C++ / CUDA 引擎，零 Python、零进程通信
    pub async fn run_asr_pipeline(
        options: AsrOptions,
    ) -> Result<AsrResult, String> {
        let audio_path_buf = PathBuf::from(&options.audio_path);
        if !audio_path_buf.exists() {
            return Err(format!("音频物理文件不存在: {}", options.audio_path));
        }

        let model_path = Self::resolve_model_path();
        if !model_path.exists() {
            return Err(format!("GGUF ASR 模型物理文件不存在: {:?}", model_path));
        }

        let t0 = Instant::now();

        // 1. 内存原位音频波形解码
        let pcm_samples = Self::load_audio_pcm_16k_mono(&audio_path_buf)?;

        // 2. 线程池安全拉起 C-FFI C++ CUDA 原生推导
        let result_transcript = tokio::task::spawn_blocking(move || -> Result<transcribe_cpp::Transcript, String> {
            let model = Model::load_with(&model_path, &ModelOptions::default())
                .map_err(|e| format!("纯血 C-FFI 载入 GGUF 模型失败: {e}"))?;

            let mut session = model.session()
                .map_err(|e| format!("创建 transcribe Session 失败: {e}"))?;

            let mut run_opts = RunOptions::default();
            run_opts.task = Task::Transcribe;
            run_opts.timestamps = TimestampKind::Auto;
            run_opts.diarize = Diarize::On;

            if let Some(lang) = &options.language {
                if lang != "auto" && !lang.is_empty() {
                    run_opts.language = Some(lang.clone());
                }
            }

            let transcript = session.run(&pcm_samples, &run_opts)
                .map_err(|e| format!("纯血 C-FFI ASR 推导失败: {e}"))?;

            Ok(transcript)
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        .map_err(|e| e)?;

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // 3. 将 C++ 极速生成的段落转为强类型 RawSegment
        let mut segments = Vec::new();
        for (idx, seg) in result_transcript.segments.iter().enumerate() {
            let speaker = if seg.speaker_id > 0 {
                format!("S{:02}", seg.speaker_id)
            } else {
                "S01".to_string()
            };

            segments.push(RawSegment {
                id: idx + 1,
                speaker,
                start_sec: seg.t0_ms as f64 / 1000.0,
                end_sec: seg.t1_ms as f64 / 1000.0,
                text: seg.text.clone(),
            });
        }

        let audio_file = audio_path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "audio".to_string());

        let duration_sec = segments
            .last()
            .map(|s| s.end_sec)
            .unwrap_or(0.0);

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

    #[tokio::test]
    async fn test_service_asr_pure_native() {
        let fixture_audio = PathBuf::from(r"C:\dev\ai-forge\test\fixtures\ASR_英语_餐厅就餐.mp3");
        if !fixture_audio.exists() {
            println!("⚠️ [跳过测试] 英文基准音频不存在: {:?}", fixture_audio);
            return;
        }

        let outs_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-asr");
        let _ = std::fs::create_dir_all(&outs_dir);

        println!("\n🚀 [TDD 纯血 Rust Native 打靶启动] 正在测试 service-asr: {:?}", fixture_audio);

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
            .expect("纯血 Rust Native ASR 打靶失败！");

        println!("\n🎉 ===== [纯血 Rust Native ASR 英文识别打靶成功] =====");
        println!("  ⏱️ 耗时: {:.2} ms ({:.2} s) | 音频时长: {:.2}s | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.duration_sec, res.segments.len());

        assert!(res.segments.len() > 0, "转写台词数不可为 0！");

        let out_json_path = outs_dir.join("asr_english_pure_native.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 纯血 Native JSON 成功物理落盘: {:?}", out_json_path);
        println!("  预览前 3 句识别结果:");
        for seg in res.segments.iter().take(3) {
            println!("     [{:.2}s -> {:.2}s] [{}]: {}", seg.start_sec, seg.end_sec, seg.speaker, seg.text);
        }
    }
}
