use crate::contracts::FormulaConfig;
use ndarray::{Array2, Axis};
use regex::Regex;
use std::sync::LazyLock;

static CHINESE_TEXT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\text\s*\{([^{}]*[\u{4e00}-\u{9fff}]+[^{}]*)\}").expect("正则: 中文 text 包裹模式")
});

static TEXT_COMMAND_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\\(operatorname|mathrm|text|mathbf)\s?\*?\s*\{.*?\})").expect("正则: LaTeX 文本命令模式")
});

static LETTER_TO_NONLETTER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-zA-Z])\s+([^a-zA-Z])").expect("正则: 字母到非字母空格模式")
});

/// 标准 LaTeX 规范化与空格清洗算子
pub fn normalize_latex(latex: &str) -> String {
    let mut result = latex.to_string();

    // 1. 移除多余的中文 text 包裹与双引号
    result = CHINESE_TEXT_PATTERN.replace_all(&result, "$1").to_string();
    result = result.replace('"', "");

    // 2. 清理 LaTeX 宏命令内部的空格
    let mut names = Vec::new();
    for mat in TEXT_COMMAND_PATTERN.find_iter(&result) {
        let text = mat.as_str();
        let cleaned = text.replace(' ', "");
        names.push(cleaned);
    }

    if !names.is_empty() {
        let mut names_iter = names.into_iter();
        result = TEXT_COMMAND_PATTERN
            .replace_all(&result, |_: &regex::Captures| {
                names_iter.next().unwrap_or_default()
            })
            .to_string();
    }

    // 3. 消除非必要的连写空格 (保留 LaTeX 细空格 '\ ')
    let mut prev_result = String::new();
    let max_iterations = 10;
    let mut iterations = 0;

    while prev_result != result && iterations < max_iterations {
        prev_result = result.clone();

        let mut temp = String::new();
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '\\' && chars[i + 1] == ' ' {
                temp.push(chars[i]);
                i += 1;
            } else if i + 1 < chars.len() && chars[i + 1].is_whitespace() {
                let is_noletter_current = !chars[i].is_ascii_alphabetic();
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() {
                    let is_noletter_next = !chars[j].is_ascii_alphabetic();
                    if is_noletter_current && is_noletter_next {
                        temp.push(chars[i]);
                        i = j;
                    } else if is_noletter_current && chars[j].is_ascii_alphabetic() {
                        temp.push(chars[i]);
                        i = j;
                    } else {
                        temp.push(chars[i]);
                        i += 1;
                    }
                } else {
                    temp.push(chars[i]);
                    i += 1;
                }
            } else {
                temp.push(chars[i]);
                i += 1;
            }
        }
        result = temp;

        result = LETTER_TO_NONLETTER_PATTERN
            .replace_all(&result, "$1$2")
            .to_string();

        iterations += 1;
    }

    result.trim().to_string()
}

/// PP-FormulaNet 公式后处理解码器
#[derive(Debug, Default)]
pub struct FormulaPostprocessor;

impl FormulaPostprocessor {
    pub fn new() -> Self {
        Self
    }

    /// 截断过滤 Token ID 序列
    pub fn filter_tokens(&self, token_ids: &Array2<i64>, config: &FormulaConfig) -> Vec<Vec<u32>> {
        let batch_size = token_ids.shape()[0];
        let mut filtered = Vec::with_capacity(batch_size);

        for batch_idx in 0..batch_size {
            let row = token_ids.index_axis(Axis(0), batch_idx);
            let tokens: Vec<u32> = row
                .iter()
                .copied()
                .take_while(|&id| id != config.eos_token_id)
                .take_while(|&id| id >= 0 && id < config.vocab_size)
                .filter(|&id| id >= 0 && id != config.sos_token_id)
                .map(|id| id as u32)
                .collect();

            filtered.push(tokens);
        }

        filtered
    }

    /// 根据词表映射解码 Token 并进行 LaTeX 规范化
    pub fn decode_and_normalize(
        &self,
        token_ids: &Array2<i64>,
        dict: &[String],
        config: &FormulaConfig,
    ) -> Vec<String> {
        let batch_tokens = self.filter_tokens(token_ids, config);
        let mut results = Vec::with_capacity(batch_tokens.len());

        for tokens in batch_tokens {
            let mut raw_str = String::new();
            for &token_id in &tokens {
                if let Some(word) = dict.get(token_id as usize) {
                    raw_str.push_str(word);
                }
            }
            let normalized = normalize_latex(&raw_str);
            results.push(normalized);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_filter_tokens_stops_at_eos() {
        let post = FormulaPostprocessor::new();
        let config = FormulaConfig::default();
        let token_ids = arr2(&[[0, 10, 20, 2, 30]]);

        let filtered = post.filter_tokens(&token_ids, &config);
        assert_eq!(filtered, vec![vec![10, 20]]);
    }

    #[test]
    fn test_normalize_latex_clean() {
        let raw = r"\mathrm { x } + \text{中文} ";
        let cleaned = normalize_latex(raw);
        // 确认成功剥离了 \text{ 包裹，且清理了 \mathrm 内部空格
        assert!(!cleaned.contains(r"\text{"));
        assert!(cleaned.contains("中文"));
        assert!(cleaned.contains(r"\mathrm{x}"));
    }
}
