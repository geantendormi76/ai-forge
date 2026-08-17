use anyhow::{Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::PathBuf;
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
            PathBuf::from(r"C:\dev\ai-forge\models\service-translation\Hy-MT2-1.8B-Q4.gguf"),
            PathBuf::from(r"C:\dev\ai-toolkit\models\service-translation\Hy-MT2-1.8B-Q4.gguf"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\service-translation\Hy-MT2-1.8B-Q4.gguf")
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
