use crate::contracts::{FormulaConfig, FormulaResult};
use crate::postprocess::normalize_latex;
use crate::preprocess::FormulaPreprocessor;
use core_onnx_infer::{OrtInfer, OrtSessionConfig};
use image::RgbImage;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tokenizers::Tokenizer;

static FORMULA_ENGINE: OnceLock<Mutex<FormulaEngine>> = OnceLock::new();

struct FormulaEngine {
    infer: OrtInfer,
    tokenizer: Tokenizer,
    sos_token_id: i64,
    eos_token_id: i64,
}

pub struct FormulaService;

impl FormulaService {
    pub fn resolve_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-formula\PP-FormulaNet-S.onnx"),
            PathBuf::from(r"C:\dev\ai-forge\models\service-formula\model.onnx"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\PP-FormulaNet-S\model.onnx"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\service-formula\PP-FormulaNet-S.onnx")
    }

    pub fn resolve_tokenizer_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-formula\tokenizer.json"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\PP-FormulaNet-S\tokenizer.json"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(r"C:\dev\ai-forge\models\service-formula\tokenizer.json")
    }

    fn extract_special_tokens(tokenizer: &Tokenizer) -> (i64, i64) {
        let bos_candidates = ["<s>", "[BOS]", "<bos>", "[CLS]"];
        let eos_candidates = ["</s>", "[EOS]", "<eos>", "[SEP]"];

        let sos_id = bos_candidates
            .iter()
            .find_map(|&token| tokenizer.token_to_id(token))
            .map(|id| id as i64)
            .unwrap_or(0);

        let eos_id = eos_candidates
            .iter()
            .find_map(|&token| tokenizer.token_to_id(token))
            .map(|id| id as i64)
            .unwrap_or(2);

        (sos_id, eos_id)
    }

    pub fn recognize_crop(
        image: &RgbImage,
        config: Option<FormulaConfig>,
        device_override: Option<&str>,
    ) -> Result<FormulaResult, String> {
        let t0 = Instant::now();

        let engine_mutex = FORMULA_ENGINE.get_or_init(|| {
            let model_path = Self::resolve_model_path();
            if !model_path.exists() {
                panic!("ONNX 公式模型物理文件不存在: {:?}", model_path);
            }

            let tokenizer_path = Self::resolve_tokenizer_path();
            if !tokenizer_path.exists() {
                panic!("FastTokenizer 物理文件不存在: {:?}", tokenizer_path);
            }

            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .unwrap_or_else(|e| panic!("加载 FastTokenizer 失败 ({tokenizer_path:?}): {e}"));
            let (sos_token_id, eos_token_id) = Self::extract_special_tokens(&tokenizer);

            let onnx_config = if let Some(dev) = device_override {
                core_onnx_infer::parse_device_config(dev)
                    .unwrap_or_else(|_| OrtSessionConfig::for_control_flow())
            } else {
                OrtSessionConfig::for_control_flow()
            };

            let infer = OrtInfer::new_with_config(&model_path, onnx_config)
                .unwrap_or_else(|e| panic!("创建 ONNX 公式推理 Session 失败 ({model_path:?}): {e}"));

            Mutex::new(FormulaEngine {
                infer,
                tokenizer,
                sos_token_id,
                eos_token_id,
            })
        });

        let mut engine = engine_mutex
            .lock()
            .map_err(|e| format!("FormulaEngine 互斥锁竞争异常: {e}"))?;

        let cfg = config.unwrap_or_default();
        let preprocessor = FormulaPreprocessor::new(cfg);
        let batch_tensor = preprocessor.preprocess_batch(&[image.clone()])?;

        let token_ids_2d = engine
            .infer
            .infer_array2_i64(&batch_tensor)
            .map_err(|e| format!("ONNX 矩阵推导失败: {e}"))?;

        let row = token_ids_2d.index_axis(ndarray::Axis(0), 0);
        let raw_slice: Vec<i64> = row.iter().copied().collect();
        let vocab_size = engine.tokenizer.get_vocab_size(true) as u32;

        let tokens_to_decode: Vec<u32> = raw_slice
            .iter()
            .copied()
            .take_while(|&id| id != engine.eos_token_id)
            .filter(|&id| id > 0 && id != engine.sos_token_id)
            .map(|id| id as u32)
            .filter(|&id| id < vocab_size)
            .collect();

        let raw_text = engine
            .tokenizer
            .decode(&tokens_to_decode, true)
            .map_err(|e| format!("FastTokenizer 解码失败: {e}"))?;

        let clean_latex = normalize_latex(&raw_text);
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

        Ok(FormulaResult {
            latex: clean_latex,
            raw_token_ids: tokens_to_decode,
            elapsed_ms,
        })
    }
}
