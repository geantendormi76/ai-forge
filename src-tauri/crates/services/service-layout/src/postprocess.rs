use crate::contracts::{LayoutBox, LayoutCategory, LayoutConfig, LayoutRegion};
use ndarray::ArrayView2;

pub struct LayoutPostProcess;

impl LayoutPostProcess {
    /// 从 PP-DocLayoutV3 2D 张量 [300, 7] 中直接解构版面区块
    /// [0]: class_id, [1]: score, [2..5]: x1, y1, x2, y2 (原图绝对像素坐标), [6]: read_order
    pub fn process_pp_doclayout_2d(
        predictions: ArrayView2<f32>,
        orig_width: f32,
        orig_height: f32,
        config: &LayoutConfig,
    ) -> Vec<LayoutRegion> {
        let shape = predictions.shape();
        if shape.len() != 2 || shape[1] < 7 {
            return Vec::new();
        }

        let num_boxes = shape[0];
        let mut filtered: Vec<(LayoutBox, usize, f32, f32)> = Vec::new(); // (bbox, cls, score, read_order)

        for box_idx in 0..num_boxes {
            let class_id = predictions[[box_idx, 0]] as i32;
            let score = predictions[[box_idx, 1]];
            let x1 = predictions[[box_idx, 2]];
            let y1 = predictions[[box_idx, 3]];
            let x2 = predictions[[box_idx, 4]];
            let y2 = predictions[[box_idx, 5]];
            let read_order = predictions[[box_idx, 6]];

            if score < config.score_threshold || class_id < 0 {
                continue;
            }

            let class_idx = class_id as usize;
            let sx1 = x1.clamp(0.0, orig_width);
            let sy1 = y1.clamp(0.0, orig_height);
            let sx2 = x2.clamp(0.0, orig_width);
            let sy2 = y2.clamp(0.0, orig_height);

            if sx2 <= sx1 || sy2 <= sy1 {
                continue;
            }

            filtered.push((
                LayoutBox::new(sx1, sy1, sx2, sy2),
                class_idx,
                score,
                read_order,
            ));
        }

        // 按模型预测出的阅读顺序 (read_order) 正向排序
        filtered.sort_by(|a, b| a.3.total_cmp(&b.3));

        filtered
            .into_iter()
            .enumerate()
            .map(|(idx, (bbox, cls_idx, score, _))| {
                let category = LayoutCategory::from_id(cls_idx);
                LayoutRegion {
                    id: idx + 1,
                    category,
                    label: category.as_str().to_string(),
                    score,
                    bbox,
                }
            })
            .collect()
    }
}
