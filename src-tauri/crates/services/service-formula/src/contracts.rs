use serde::{Deserialize, Serialize};

/// 数学公式识别算法配置契约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaConfig {
    /// 目标图像尺寸 (宽, 高)，默认 (384, 384)
    pub target_size: (u32, u32),
    /// 裁切背景白边的阈值 (0-255)，默认 200
    pub crop_threshold: u8,
    /// 张量维度对齐倍数，默认 16
    pub padding_multiple: usize,
    /// 起始 Token ID (sos)，默认 0
    pub sos_token_id: i64,
    /// 结束 Token ID (eos)，默认 2
    pub eos_token_id: i64,
    /// 词表有效 ID 上限，默认 50000
    pub vocab_size: i64,
}

impl Default for FormulaConfig {
    fn default() -> Self {
        Self {
            target_size: (384, 384),
            crop_threshold: 200,
            padding_multiple: 16,
            sos_token_id: 0,
            eos_token_id: 2,
            vocab_size: 50_000,
        }
    }
}

/// 数学公式识别单项结果契约
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FormulaResult {
    /// 经过规范化清洗后的标准 LaTeX 字符串
    pub latex: String,
    /// 原始解码的 Token ID 列表
    pub raw_token_ids: Vec<u32>,
    /// 推理物理耗时 (毫秒)
    pub elapsed_ms: f64,
}
