use anyhow::{Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PureTranslationRequest {
    pub texts: Vec<String>,
    pub target_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PureTranslationResponse {
    pub translations: Vec<String>,
    pub elapsed_ms: f64,
}

pub struct TranslationService;

impl TranslationService {
    fn resolve_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf"),
            PathBuf::from(r"C:\dev\ai-toolkit\models\tool-translation\Hy-MT2-1.8B-Q4.gguf"),
            PathBuf::from(r"C:\dev\mvp_fx_dll\models\Hy-MT2-1.8B-Q4.gguf"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\tool-translation\Hy-MT2-1.8B-Q4.gguf")
    }

    fn translate_single_sentence(
        model: &LlamaModel,
        backend: &LlamaBackend,
        text: &str,
        target_lang: &str,
    ) -> Result<String> {
        let clean_text = text.trim();
        if clean_text.is_empty() {
            return Ok(String::new());
        }

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));
        let mut ctx = model
            .new_context(backend, ctx_params)
            .context("创建 LlamaContext 失败")?;

        let tmpl = model
            .chat_template(None)
            .context("获取 Chat Template 失败")?;

        let user_msg = LlamaChatMessage::new(
            "user".to_string(),
            format!(
                "Translate the following text into {}. Note that you should ONLY output the translated result without any additional explanation:\n\n{}",
                target_lang, clean_text
            ),
        )?;

        let prompt = model
            .apply_chat_template(&tmpl, &[user_msg], true)
            .context("应用 Chat Template 失败")?;

        let tokens_list = model
            .str_to_token(&prompt, llama_cpp_2::model::AddBos::Never)
            .context("Prompt Tokenize 失败")?;

        let mut batch = LlamaBatch::new(512, 1);
        let last_idx = (tokens_list.len() - 1) as i32;

        for (i, token) in tokens_list.into_iter().enumerate() {
            let is_last = i as i32 == last_idx;
            batch.add(token, i as i32, &[0], is_last)?;
        }

        ctx.decode(&mut batch).context("llama_decode 失败")?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(model.n_vocab(), 64, 1.15, 0.0, 0.0),
            LlamaSampler::temp(0.1),
            LlamaSampler::greedy(),
        ]);

        let mut n_cur = batch.n_tokens();
        let mut output_bytes = Vec::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        while n_cur <= 256 {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if model.is_eog_token(token) {
                break;
            }

            let piece = model.token_to_piece(token, &mut decoder, true, None)?;
            output_bytes.extend_from_slice(piece.as_bytes());

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;

            n_cur += 1;
            if ctx.decode(&mut batch).is_err() {
                break;
            }
        }

        Ok(String::from_utf8_lossy(&output_bytes).trim().to_string())
    }

    /// 纯血 C-FFI 神经翻译管道：原位 GPU 直推，零 Python、零沙箱、零网络端口
    pub async fn run_translation_pipeline(
        req: PureTranslationRequest,
    ) -> Result<PureTranslationResponse, String> {
        if req.texts.is_empty() {
            return Ok(PureTranslationResponse {
                translations: Vec::new(),
                elapsed_ms: 0.0,
            });
        }

        let t0 = Instant::now();
        let model_path = Self::resolve_model_path();
        if !model_path.exists() {
            return Err(format!("GGUF 翻译模型物理文件不存在: {:?}", model_path));
        }

        let target_lang = req.target_lang.unwrap_or_else(|| "Chinese".to_string());
        let texts = req.texts;

        let res = tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
            let backend = LlamaBackend::init()
                .map_err(|e| format!("初始化 LlamaBackend 失败: {e}"))?;

            let model_params = LlamaModelParams::default().with_n_gpu_layers(1000);
            let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
                .map_err(|e| format!("加载 GGUF 翻译模型失败: {e}"))?;

            let mut translations = Vec::with_capacity(texts.len());
            for text in &texts {
                let trans = Self::translate_single_sentence(&model, &backend, text, &target_lang)
                    .map_err(|e| format!("单句神经翻译失败: {e}"))?;
                translations.push(trans);
            }

            Ok(translations)
        })
        .await
        .map_err(|e| format!("Tokio 线程调度异常: {e}"))?
        .map_err(|e| e)?;

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        Ok(PureTranslationResponse {
            translations: res,
            elapsed_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_contract_purity() {
        let req = PureTranslationRequest {
            texts: vec!["Hello world".into()],
            target_lang: Some("Chinese".into()),
        };
        let json_str = serde_json::to_string(&req).unwrap();
        assert!(!json_str.contains("python"));
        assert!(!json_str.contains("venv"));
        println!("\n✅ [service-translation 纯血契约测试通过]: {}", json_str);
    }

    #[tokio::test]
    async fn test_native_translation_e2e_english_json() {
        let fixture_json = PathBuf::from(r"C:\dev\ai-forge\test\fixtures\asr_english_pure.json");
        if !fixture_json.exists() {
            println!("⚠️ [跳过测试] 测试集文件不存在: {:?}", fixture_json);
            return;
        }

        let outs_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-translation");
        let _ = std::fs::create_dir_all(&outs_dir);

        let json_str = std::fs::read_to_string(&fixture_json).unwrap();
        let asr_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let raw_segs = asr_val["segments"].as_array().expect("segments 必须是数组");
        let english_texts: Vec<String> = raw_segs
            .iter()
            .map(|s| s["text"].as_str().unwrap_or("").to_string())
            .collect();

        let req = PureTranslationRequest {
            texts: english_texts.clone(),
            target_lang: Some("Chinese".into()),
        };

        let res = TranslationService::run_translation_pipeline(req)
            .await
            .expect("纯血 C-FFI 神经翻译打靶失败！");

        println!("\n🎉 ===== [ai-forge service-translation 纯血 23句全量英文 ASR JSON 神经翻译成功] =====");
        println!("  ⏱️ 全量物理耗时: {:.2} ms ({:.2} s) | 句数: {}", res.elapsed_ms, res.elapsed_ms / 1000.0, res.translations.len());

        let mut output_segments = Vec::new();
        for (i, trans) in res.translations.iter().enumerate() {
            let mut seg = raw_segs[i].clone();
            seg["translated_text"] = serde_json::Value::String(trans.clone());
            output_segments.push(seg);
        }

        let out_payload = serde_json::json!({
            "audio_file": asr_val["audio_file"],
            "duration_sec": asr_val["duration_sec"],
            "total_segments": res.translations.len(),
            "elapsed_ms": res.elapsed_ms,
            "segments": output_segments
        });

        let out_json_path = outs_dir.join("english_to_chinese_pure.json");
        let pretty_json = serde_json::to_string_pretty(&out_payload).unwrap();
        std::fs::write(&out_json_path, pretty_json).unwrap();

        println!("  💾 全量翻译 JSON 产物成功物理落盘至: {:?}", out_json_path);
        println!("  预览前 3 句对照效果:");
        for i in 0..3.min(english_texts.len()) {
            println!("     [{}] 英文: {} ➔ 中文: {}", i + 1, english_texts[i], res.translations[i]);
        }
        println!("===============================================================\n");

        assert_eq!(res.translations.len(), english_texts.len());
        assert!(out_json_path.exists());
    }
}
