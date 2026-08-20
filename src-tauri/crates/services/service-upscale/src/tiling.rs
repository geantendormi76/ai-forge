// 🛡️ 8K 视觉超分 - 零接缝智能动态切块引擎 (tiling.rs)
// 100% 外科手术式直译自 upscale_worker.py / upscale_daemon.py

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileInfo {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pad_left: u32,
    pub pad_top: u32,
    pub pad_right: u32,
    pub pad_bottom: u32,
}

impl TileInfo {
    /// 计算包含 Padding 的输入图像裁剪区域: (crop_x, crop_y, crop_w, crop_h)
    #[inline]
    pub fn crop_rect(&self) -> (u32, u32, u32, u32) {
        let crop_x = self.x - self.pad_left;
        let crop_y = self.y - self.pad_top;
        let crop_w = self.w + self.pad_left + self.pad_right;
        let crop_h = self.h + self.pad_top + self.pad_bottom;
        (crop_x, crop_y, crop_w, crop_h)
    }

    /// 计算模型放大后的内部有效截取区域 (剔除 Padding 伪影): (valid_x, valid_y, valid_w, valid_h)
    #[inline]
    pub fn valid_scaled_rect(&self, model_scale: u32) -> (u32, u32, u32, u32) {
        let valid_x = self.pad_left * model_scale;
        let valid_y = self.pad_top * model_scale;
        let valid_w = self.w * model_scale;
        let valid_h = self.h * model_scale;
        (valid_x, valid_y, valid_w, valid_h)
    }

    /// 计算在全局 4x 目标画布上的贴图起始坐标: (paste_x, paste_y)
    #[inline]
    pub fn canvas_paste_pos(&self, model_scale: u32) -> (u32, u32) {
        (self.x * model_scale, self.y * model_scale)
    }
}

/// 智能网格切块计算器
/// 1:1 对齐 Python 原版 compute_grid 算法
pub fn compute_grid(img_w: u32, img_h: u32, tile_size: u32, tile_pad: u32) -> Vec<TileInfo> {
    let mut tiles = Vec::new();
    let mut y = 0;

    while y < img_h {
        let h = tile_size.min(img_h - y);
        let mut x = 0;

        while x < img_w {
            let w = tile_size.min(img_w - x);
            let pad_left = tile_pad.min(x);
            let pad_top = tile_pad.min(y);
            let pad_right = tile_pad.min(img_w.saturating_sub(x + w));
            let pad_bottom = tile_pad.min(img_h.saturating_sub(y + h));

            tiles.push(TileInfo {
                x,
                y,
                w,
                h,
                pad_left,
                pad_top,
                pad_right,
                pad_bottom,
            });

            x += tile_size;
        }

        y += tile_size;
    }

    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_grid_basic() {
        // 测试 500x500 图像在 tile_size=256, pad=10 时的切块分布 (应为 2x2 = 4 块)
        let tiles = compute_grid(500, 500, 256, 10);
        assert_eq!(tiles.len(), 4);

        // 第 1 块 (左上角)
        let t0 = &tiles[0];
        assert_eq!(t0.x, 0);
        assert_eq!(t0.y, 0);
        assert_eq!(t0.w, 256);
        assert_eq!(t0.h, 256);
        assert_eq!(t0.pad_left, 0);
        assert_eq!(t0.pad_top, 0);
        assert_eq!(t0.pad_right, 10);
        assert_eq!(t0.pad_bottom, 10);
        assert_eq!(t0.crop_rect(), (0, 0, 266, 266));
        assert_eq!(t0.valid_scaled_rect(4), (0, 0, 1024, 1024));
        assert_eq!(t0.canvas_paste_pos(4), (0, 0));

        // 第 4 块 (右下角尾块: 244x244)
        let t3 = &tiles[3];
        assert_eq!(t3.x, 256);
        assert_eq!(t3.y, 256);
        assert_eq!(t3.w, 244);
        assert_eq!(t3.h, 244);
        assert_eq!(t3.pad_left, 10);
        assert_eq!(t3.pad_top, 10);
        assert_eq!(t3.pad_right, 0);
        assert_eq!(t3.pad_bottom, 0);
        assert_eq!(t3.crop_rect(), (246, 246, 254, 254));
        assert_eq!(t3.valid_scaled_rect(4), (40, 40, 976, 976));
        assert_eq!(t3.canvas_paste_pos(4), (1024, 1024));
    }

    #[test]
    fn test_small_image_no_split() {
        // 小于 tile_size 的图像不分块
        let tiles = compute_grid(100, 80, 256, 10);
        assert_eq!(tiles.len(), 1);
        let t = &tiles[0];
        assert_eq!(t.w, 100);
        assert_eq!(t.h, 80);
        assert_eq!(t.pad_left, 0);
        assert_eq!(t.pad_top, 0);
        assert_eq!(t.pad_right, 0);
        assert_eq!(t.pad_bottom, 0);
    }
}
