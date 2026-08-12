use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TensorLayout {
    #[default]
    CHW,
    HWC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorOrder {
    #[default]
    RGB,
    BGR,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LimitType {
    #[default]
    Max,
    Min,
    ResizeLong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScoreMode {
    #[default]
    Fast,
    Slow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BoxType {
    #[default]
    Quad,
    Poly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResizeType {
    Type0,
    Type1 {
        image_shape: (u32, u32),
        keep_ratio: bool,
    },
    Type2 {
        resize_long: u32,
    },
    Type3 {
        input_shape: (u32, u32, u32),
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageScaleInfo {
    pub src_h: f32,
    pub src_w: f32,
    pub ratio_h: f32,
    pub ratio_w: f32,
}

impl ImageScaleInfo {
    pub fn new(src_h: f32, src_w: f32, ratio_h: f32, ratio_w: f32) -> Self {
        Self {
            src_h,
            src_w,
            ratio_h,
            ratio_w,
        }
    }
}
