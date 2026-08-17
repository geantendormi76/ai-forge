use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemporalOcrSegment {
    pub id: usize,
    pub speaker: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

pub struct HardSubOcrPipeline;

impl HardSubOcrPipeline {
    /// 计算两个字符串的相似度 (Normalized Levenshtein Ratio: 0.0 ~ 1.0)
    pub fn text_similarity(s1: &str, s2: &str) -> f64 {
        let t1 = s1.trim();
        let t2 = s2.trim();
        if t1 == t2 {
            return 1.0;
        }
        if t1.is_empty() || t2.is_empty() {
            return 0.0;
        }

        let len1 = t1.chars().count();
        let len2 = t2.chars().count();
        let max_len = len1.max(len2);
        if max_len == 0 {
            return 1.0;
        }

        let v1: Vec<char> = t1.chars().collect();
        let v2: Vec<char> = t2.chars().collect();
        let mut matrix = vec![vec![0usize; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        let dist = matrix[len1][len2];
        1.0 - (dist as f64 / max_len as f64)
    }

    /// 核心时域平滑聚类算法：将离散抽样帧的 OCR 识别文本聚合为连续台词段落
    pub fn aggregate_temporal_frames(
        raw_frames: Vec<(f64, String)>,
        sample_interval_sec: f64,
        min_duration_sec: f64,
    ) -> Vec<TemporalOcrSegment> {
        let mut segments: Vec<TemporalOcrSegment> = Vec::new();
        let mut current_segment: Option<(f64, f64, String)> = None;

        for (timestamp, text) in raw_frames {
            let clean = text.trim().to_string();
            
            if clean.is_empty() {
                // 画面无字幕：结算当前活跃段落
                if let Some((start, end, seg_text)) = current_segment.take() {
                    if (end - start) >= min_duration_sec {
                        let seg_id = segments.len() + 1;
                        segments.push(TemporalOcrSegment {
                            id: seg_id,
                            speaker: "S01".to_string(),
                            start_sec: (start * 1000.0).round() / 1000.0,
                            end_sec: (end * 1000.0).round() / 1000.0,
                            text: seg_text,
                        });
                    }
                }
                continue;
            }

            let mut is_continuation = false;
            if let Some((_start, ref mut end, ref mut seg_text)) = current_segment {
                let sim = Self::text_similarity(seg_text, &clean);
                if sim >= 0.75 {
                    *end = timestamp + sample_interval_sec;
                    if clean.chars().count() > seg_text.chars().count() {
                        *seg_text = clean.clone();
                    }
                    is_continuation = true;
                }
            }

            if !is_continuation {
                // 切换为新台词：先结算并取走旧段落
                if let Some((start, end, seg_text)) = current_segment.take() {
                    if (end - start) >= min_duration_sec {
                        let seg_id = segments.len() + 1;
                        segments.push(TemporalOcrSegment {
                            id: seg_id,
                            speaker: "S01".to_string(),
                            start_sec: (start * 1000.0).round() / 1000.0,
                            end_sec: (end * 1000.0).round() / 1000.0,
                            text: seg_text,
                        });
                    }
                }
                current_segment = Some((timestamp, timestamp + sample_interval_sec, clean));
            }
        }

        // 结算最后残留段落
        if let Some((start, end, seg_text)) = current_segment {
            if (end - start) >= min_duration_sec {
                let seg_id = segments.len() + 1;
                segments.push(TemporalOcrSegment {
                    id: seg_id,
                    speaker: "S01".to_string(),
                    start_sec: (start * 1000.0).round() / 1000.0,
                    end_sec: (end * 1000.0).round() / 1000.0,
                    text: seg_text,
                });
            }
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_ocr_frame_aggregation() {
        // 模拟 2 fps 采样率（每帧间隔 0.5 秒）的离散 OCR 原始识别帧流
        let mock_raw_frames = vec![
            (0.0, "".to_string()),
            (0.5, "Welcome to AI-Forge".to_string()),
            (1.0, "Welcome to AI-Forge".to_string()),
            (1.5, "Welcome to AI-Forge!".to_string()), // 轻微标点抖动
            (2.0, "".to_string()),
            (2.5, "".to_string()),
            (3.0, "This is hardcoded subtitle".to_string()),
            (3.5, "This is hardcoded subtitle".to_string()),
            (4.0, "This is hardcoded subtitle".to_string()),
            (4.5, "".to_string()),
        ];

        let segments = HardSubOcrPipeline::aggregate_temporal_frames(mock_raw_frames, 0.5, 0.4);

        assert_eq!(segments.len(), 2, "应聚类合并为 2 句完整台词");

        // 段落 1 断言 (0.5s -> 2.0s)
        assert_eq!(segments[0].id, 1);
        assert_eq!(segments[0].start_sec, 0.5);
        assert_eq!(segments[0].end_sec, 2.0);
        assert!(segments[0].text.contains("Welcome to AI-Forge"));

        // 段落 2 断言 (3.0s -> 4.5s)
        assert_eq!(segments[1].id, 2);
        assert_eq!(segments[1].start_sec, 3.0);
        assert_eq!(segments[1].end_sec, 4.5);
        assert_eq!(segments[1].text, "This is hardcoded subtitle");

        println!("\n✅ [TDD 断言通过] 管道 C 时域离散帧平滑聚类 100% 达成精准时间轴！");
    }
}
