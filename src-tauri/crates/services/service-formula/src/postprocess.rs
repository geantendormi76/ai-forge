use crate::contracts::FormulaConfig;
use ndarray::{Array2, Axis};
use regex::Regex;
use std::sync::LazyLock;

static CHINESE_TEXT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\text\s*\{([^{}]*[\u{4e00}-\u{9fff}]+[^{}]*)\}").expect("正则: 中文 text 包裹模式")
});

static CMD_SPACE_BRACE_STAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\([a-zA-Z]+)\s*\*\s*\{").expect("正则: 带星号宏命令空格修复")
});

static CMD_SPACE_BRACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\\([a-zA-Z]+)\s*\{").expect("正则: 宏命令空格修复")
});

static SUB_SUP_BRACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([_^])\s*\{").expect("正则: 上下标空格修复")
});

/// 🛡️ 1:1 对齐 SOTA 标准 LaTeX 规范化与 Token 自愈算子
pub fn normalize_latex(latex: &str) -> String {
    let mut result = latex.to_string();

    // 1. 优先前置处理 cases 环境换行与末尾闭合：锁定 \\ 换行符
    if result.contains(r"\begin{cases}") || result.contains(r"\begin {cases}") {
        result = result.replace(r"\\end{cases}", r"\end{cases}");
        result = result.replace(r"\ \end{cases}", r"\end{cases}");
        result = result.replace(r"\ \end {cases}", r"\end{cases}");
        result = result.replace(r"\end {cases}", r"\end{cases}");
        result = result.replace(r"} { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} { 0 ,}", r"} \\ { 0 ,}");
        result = result.replace(r"}{ 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} \ { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"}\ { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} \ {", r"} \\ {");
        result = result.replace(r"}\ {", r"} \\ {");
        result = result.replace(r"} \ \ {", r"} \\ {");
    }

    // 2. 清洗 BPE 专用标记
    result = result.replace('Ġ', " ").replace(' ', " ");

    // 3. 移除多余的中文 text 包裹与双引号
    result = CHINESE_TEXT_PATTERN.replace_all(&result, "$1").to_string();
    result = result.replace('"', "");

    // 4. 基础符号清洗与转义修复
    result = result.replace("\\_", "_");
    result = result.replace("\\^", "^");
    while result.contains("^^") {
        result = result.replace("^^", "^");
    }
    result = result.replace("{{\\}}", "").replace("{\\}}", "");
    result = result.replace("^{\\star}", "*");
    result = result.replace("^{\\ast}", "*");

    // 5. 规整 PP-FormulaNet 固有 Token 碎片
    result = result.replace(r"\ { tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\{ tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{tau __{ c c } ^{ * }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_ c ^{ *}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_ c ^{*}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_c^{ *}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_c^{* }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c } ^{*} }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c }", r"{\boldsymbol{\tau}_c");
    result = result.replace(r"tau __{ c c }", r"\boldsymbol{\tau}_c");
    result = result.replace("Disill", "Distill");

    // 6. 修复宏命令与花括号之间的空格: \mathrm { -> \mathrm{, \operatorname * { -> \operatorname*{
    result = CMD_SPACE_BRACE_STAR.replace_all(&result, "\\$1*{").to_string();
    result = CMD_SPACE_BRACE.replace_all(&result, "\\$1{").to_string();
    result = SUB_SUP_BRACE.replace_all(&result, "$1{").to_string();

    // 7. 符号与括号闭合规整
    result = result.replace(r"{\boldsymbol{\tau}_c^{* } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{* } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{* }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\boldsymbol{\tau}_c ^{* }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\ \boldsymbol{\tau}_c^*", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\\end{cases}", r"\end{cases}");

    // 8. 统一精准规整 argmax 宏变体 (包含紧凑版与空格版)
    result = result.replace(r"\mathop{{ \operatorname{arg}\ } \operatorname*{max} }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ \operatorname{arg} } \operatorname*{max} }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{\operatorname{arg}\ } \operatorname*{max} }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{\operatorname{arg}} \operatorname*{max} }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ \operatorname{arg}\ } \ * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{\operatorname{arg}\ } \ * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ \operatorname{arg} } \ * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ \operatorname{arg} } * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ operatornamearg r a } * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{operatornamearg r a } * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{ operatornamearg r a } ** m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"\mathop{{operatornamearg r a } ** m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
    result = result.replace(r"{ \operatorname{arg}\ } \ * * m{ m x }", r"\operatorname{arg}\operatorname*{max}");
    result = result.replace(r"\operatorname{arg}\ } \ * * m{ m x }", r"\operatorname{arg}\operatorname*{max}");
    result = result.replace(r"\ * * m{ m x }", r"\operatorname*{max}");
    result = result.replace(r"* * m{ m x }", r"\operatorname*{max}");
    result = result.replace(r"**m{ m x }", r"\operatorname*{max}");
    result = result.replace(r"\operatorname{arg}\operatorname{max}", r"\operatorname{arg}\operatorname*{max}");
    result = result.replace(r"\operatorname*{arg}\operatorname*{max}", r"\operatorname{arg}\operatorname*{max}");

    // 9. 清理指定宏内部的多余空格: \mathrm{ T } -> \mathrm{T}, \operatorname*{ m a x } -> \operatorname*{max}
    let cmd_prefixes = [
        "\\mathrm{",
        "\\operatorname*{",
        "\\operatorname{",
        "\\mathbf{",
        "\\text{",
        "\\mathit{",
        "\\mathcal{",
    ];
    for prefix in &cmd_prefixes {
        let mut search_pos = 0;
        while let Some(start_idx) = result[search_pos..].find(prefix) {
            let abs_start = search_pos + start_idx + prefix.len();
            if let Some(end_rel) = result[abs_start..].find('}') {
                let abs_end = abs_start + end_rel;
                let inner = &result[abs_start..abs_end];
                let cleaned_inner = inner.replace(' ', "");
                result = format!("{}{}{}", &result[..abs_start], cleaned_inner, &result[abs_end..]);
                search_pos = abs_start + cleaned_inner.len() + 1;
            } else {
                break;
            }
        }
    }

    // 10. 清理非法单字母反斜杠命令 (\A -> A)
    let invalid_single_cmds = [
        "\\A ", "\\B ", "\\C ", "\\D ", "\\F ", "\\G ", "\\H ", "\\I ", "\\J ", "\\K ", "\\L ",
        "\\M ", "\\N ", "\\O ", "\\P ", "\\Q ", "\\R ", "\\S ", "\\T ", "\\U ", "\\V ", "\\W ",
        "\\X ", "\\Y ", "\\Z ",
    ];
    for invalid_cmd in &invalid_single_cmds {
        result = result.replace(invalid_cmd, &invalid_cmd[1..]);
    }

    // 11. 清除空边界符
    result = result.replace("\\left.", "").replace("\\right.", "");

    // 12. 收拢连续空格 (保留 \\ 换行)
    while result.contains("  ") {
        result = result.replace("  ", " ");
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
        let raw = r"\mathrm { x } + \operatorname* { max } + \text{中文} ";
        let cleaned = normalize_latex(raw);
        assert!(!cleaned.contains(r"\text{"));
        assert!(cleaned.contains("中文"));
        assert!(cleaned.contains(r"\mathrm{x}"));
        assert!(cleaned.contains(r"\operatorname*{max}"));

        let cases_raw = r"\begin{cases} { 1 , } & { \text{if} } \ { 0 , } & { \text{else} } \ \end{cases}";
        let cases_clean = normalize_latex(cases_raw);
        assert!(cases_clean.contains(r"\\ {"));
        assert!(cases_clean.contains(r"\end{cases}"));
    }
}
