use anyhow::{Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::mtmd::{MtmdBitmap, MtmdContext};
use llama_cpp_2::sampling::LlamaSampler;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time::Instant;

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

pub fn parse_asr_transcript(raw_text: &str) -> Vec<RawSegment> {
    let mut segments = Vec::new();
    let mut curr = raw_text;
    let mut idx = 1;

    while let Some(start_bracket) = curr.find('[') {
        let after_start_bracket = &curr[start_bracket + 1..];
        let start_close = match after_start_bracket.find(']') {
            Some(i) => i,
            None => break,
        };
        let start_str = &after_start_bracket[..start_close];
        let start_sec: f64 = match start_str.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                curr = &after_start_bracket[start_close + 1..];
                continue;
            }
        };

        let after_start = &after_start_bracket[start_close + 1..];
        let spk_open = match after_start.find('[') {
            Some(i) => i,
            None => break,
        };
        let after_spk_open = &after_start[spk_open + 1..];
        let spk_close = match after_spk_open.find(']') {
            Some(i) => i,
            None => break,
        };
        let speaker = after_spk_open[..spk_close].trim().to_string();

        let after_spk = &after_spk_open[spk_close + 1..];
        let end_open = match after_spk.find('[') {
            Some(i) => i,
            None => {
                let text = after_spk.trim().to_string();
                if !text.is_empty() {
                    segments.push(RawSegment {
                        id: idx,
                        speaker,
                        start_sec,
                        end_sec: start_sec + 3.0,
                        text,
                    });
                }
                break;
            }
        };

        let text = after_spk[..end_open].trim().to_string();
        let after_end_open = &after_spk[end_open + 1..];
        let end_close = match after_end_open.find(']') {
            Some(i) => i,
            None => break,
        };
        let end_str = &after_end_open[..end_close];
        let end_sec: f64 = match end_str.trim().parse() {
            Ok(v) => v,
            Err(_) => start_sec + 2.0,
        };

        if !text.is_empty() {
            segments.push(RawSegment {
                id: idx,
                speaker,
                start_sec,
                end_sec,
                text,
            });
            idx += 1;
        }

        curr = &after_end_open[end_close + 1..];
    }

    if segments.is_empty() && !raw_text.trim().is_empty() {
        segments.push(RawSegment {
            id: 1,
            speaker: "S01".to_string(),
            start_sec: 0.0,
            end_sec: 5.0,
            text: raw_text.trim().to_string(),
        });
    }

    segments
}

pub struct AsrService;

impl AsrService {
    fn resolve_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-0.9B-Q4_K_M.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-0.9B-F16.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-ASR\MOSS-Transcribe-Diarize-0.9B-Q4_K_M.gguf"),
            PathBuf::from(r"C:\dev\ai-toolkit\models\tool-ASR\MOSS-Transcribe-Diarize-0.9B-Q4_K_M.gguf"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-0.9B-Q4_K_M.gguf")
    }

    /// 纯血 C-FFI 语音转写管道：原位 GPU 直推，零 Python、零沙箱、零网络端口
    pub async fn run_asr_pipeline(
        options: AsrOptions,
    ) -> Result<AsrResult, String> {
        let audio_path_buf = PathBuf::from(&options.audio_path);
        if !audio_path_buf.exists() {
            return Err(format!("音频物理文件不存在: {}", options.audio_path));
        }

        let t0 = Instant::now();
        let model_path = Self::resolve_model_path();
        if !model_path.exists() {
            return Err(format!("GGUF ASR 模型物理文件不存在: {:?}", model_path));
        }

        let mode = options.mode.unwrap_or_else(|| "verbatim".to_string());
        let language = options.language.unwrap_or_else(|| "auto".to_string());
        let audio_path_str = options.audio_path.clone();

        let raw_transcript = tokio::task::spawn_blocking(move || -> Result<String, String> {
            let backend = LlamaBackend::init()
                .map_err(|e| format!("初始化 LlamaBackend 失败: {e}"))?;

            // 1. 使用 MtmdContext 多模态加载器同时装载语音编码器与文本解码器
            let mtmd_ctx = MtmdContext::load_from_file(&backend, &model_path, 1000)
                .map_err(|e| format!("多模态 MtmdContext 载入失败: {e}"))?;

            let mut prompt_content = String::from(
                "请将音频转写为文本，每一段需以起始时间戳和说话人编号（[S01]、[S02]、[S03]…）开头，正文为对应的语音内容，并在段末标注结束时间戳，以清晰标明该段语音范围。"
            );
            if language != "auto" && !language.is_empty() {
                prompt_content.push_str(&format!(" 主要语言为：{}。", language));
            }

            // 2. 将音频直推入多模态 C++ 算子
            let res = mtmd_ctx.eval_audio_file(&audio_path_str, &prompt_content)
                .map_err(|e| format!("Mtmd 多模态语音识别推导失败: {e}"))?;

            Ok(res)
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        .map_err(|e| e)?;

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let segments = parse_asr_transcript(&raw_transcript);

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
            mode: None,
            language: None,
        };
        let json_str = serde_json::to_string(&opts).unwrap();
        assert!(!json_str.contains("srt_text"));
        assert!(!json_str.contains("ass_text"));
        println!("\n✅ [service-asr 纯净性测试通过] 零领域污染数据契约: {}", json_str);
    }

    #[test]
    fn test_parse_asr_transcript() {
        let sample = "[0.00][S01] Can we get a table for two? [2.50][2.50][S02] Sure, right this way. [5.10]";
        let segs = parse_asr_transcript(sample);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].speaker, "S01");
        assert_eq!(segs[0].start_sec, 0.00);
        assert_eq!(segs[0].end_sec, 2.50);
        assert_eq!(segs[0].text, "Can we get a table for two?");
        assert_eq!(segs[1].speaker, "S02");
        assert_eq!(segs[1].start_sec, 2.50);
        assert_eq!(segs[1].end_sec, 5.10);
        assert_eq!(segs[1].text, "Sure, right this way.");
        println!("\n✅ [ASR 状态机解析算子测试通过]: {:?}", segs);
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
            .expect("MOSS 0.9B ASR 纯血 C-FFI 英文语音打靶失败");

        println!("\n🎉 ===== [MOSS 0.9B 纯血 C-FFI ASR 英文识别完成] =====");
        println!("  ⏱️ 耗时: {:.2} ms ({:.2} s) | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.segments.len());

        assert!(res.segments.len() > 0, "转写台词数不可为 0！");

        let out_json_path = outs_dir.join("asr_english_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 英文纯净 JSON 成功落盘: {:?}", out_json_path);
    }
}
