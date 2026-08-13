use crate::contracts::{FormulaConfig, FormulaResult};
use crate::postprocess::normalize_latex;
use crate::preprocess::FormulaPreprocessor;
use core_onnx_infer::OrtInfer;
use image::RgbImage;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tokenizers::Tokenizer;

/// ONNX 引擎与 Tokenizer 常驻单例缓存锁
static FORMULA_ENGINE: OnceLock<Mutex<FormulaEngine>> = OnceLock::new();

struct FormulaEngine {
    infer: OrtInfer,
    tokenizer: Tokenizer,
    sos_token_id: i64,
    eos_token_id: i64,
}

pub struct FormulaService;

impl FormulaService {
    /// 自动解析 ONNX 公式模型物理路径 (优先匹配用户重命名后的 PP-FormulaNet-S.onnx)
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

    /// 自动解析 FastTokenizer 物理路径
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

    /// 从 FastTokenizer 动态提取标志位 Token ID (BOS / EOS)
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

    /// 纯血 ONNX + FastTokenizer 数学公式识别管道 (支持 Session 常驻缓存)
    pub fn recognize_crop(
        image: &RgbImage,
        config: Option<FormulaConfig>,
        device_override: Option<&str>,
    ) -> Result<FormulaResult, String> {
        let t0 = Instant::now();

        // 1. 从 OnceLock 获取或懒加载常驻引擎 (零重复读盘与重复解析)
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

            let infer = OrtInfer::new(&model_path, device_override)
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

        // 2. 图像预处理生成 [1, 1, 384, 384] 4D 张量
        let cfg = config.unwrap_or_default();
        let preprocessor = FormulaPreprocessor::new(cfg);
        let batch_tensor = preprocessor.preprocess_batch(&[image.clone()])?;

        // 3. 常驻 ONNX 引擎硬件推导
        let token_ids_2d = engine
            .infer
            .infer_array2_i64(&batch_tensor)
            .map_err(|e| format!("ONNX 矩阵推导失败: {e}"))?;

        // 4. 解构 Token ID 矩阵并进行标志位截断过滤
        let row = token_ids_2d.index_axis(ndarray::Axis(0), 0);
        let vocab_size = engine.tokenizer.get_vocab_size(true) as u32;

        let tokens_to_decode: Vec<u32> = row
            .iter()
            .copied()
            .take_while(|&id| id != engine.eos_token_id)
            .filter(|&id| id >= 0 && id != engine.sos_token_id)
            .map(|id| id as u32)
            .filter(|&id| id < vocab_size)
            .collect();

        // 5. FastTokenizer 查表解码与 LaTeX 正则清洗
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_paths() {
        let model_p = FormulaService::resolve_model_path();
        let tok_p = FormulaService::resolve_tokenizer_path();
        assert!(model_p.to_string_lossy().contains("PP-FormulaNet-S.onnx") || model_p.to_string_lossy().contains("model.onnx"));
        assert!(tok_p.to_string_lossy().contains("tokenizer.json"));
    }
}
