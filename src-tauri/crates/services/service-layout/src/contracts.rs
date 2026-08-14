use serde::{Deserialize, Serialize};

/// 官方 PP-DocLayoutV3 全量 25 种版面物理区块类别 (1:1 对齐 inference.yml)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutCategory {
    Abstract,         // 0
    Algorithm,        // 1
    AsideText,        // 2
    Chart,            // 3
    Content,          // 4
    DisplayFormula,   // 5
    DocTitle,         // 6
    FigureTitle,      // 7
    Footer,           // 8
    FooterImage,      // 9
    Footnote,         // 10
    FormulaNumber,    // 11
    Header,           // 12
    HeaderImage,      // 13
    Image,            // 14
    InlineFormula,    // 15
    Number,           // 16
    ParagraphTitle,   // 17
    Reference,        // 18
    ReferenceContent, // 19
    Seal,             // 20
    Table,            // 21
    Text,             // 22
    VerticalText,     // 23
    VisionFootnote,   // 24
    Other,
}

impl LayoutCategory {
    pub fn from_id(id: usize) -> Self {
        match id {
            0 => Self::Abstract,
            1 => Self::Algorithm,
            2 => Self::AsideText,
            3 => Self::Chart,
            4 => Self::Content,
            5 => Self::DisplayFormula,
            6 => Self::DocTitle,
            7 => Self::FigureTitle,
            8 => Self::Footer,
            9 => Self::FooterImage,
            10 => Self::Footnote,
            11 => Self::FormulaNumber,
            12 => Self::Header,
            13 => Self::HeaderImage,
            14 => Self::Image,
            15 => Self::InlineFormula,
            16 => Self::Number,
            17 => Self::ParagraphTitle,
            18 => Self::Reference,
            19 => Self::ReferenceContent,
            20 => Self::Seal,
            21 => Self::Table,
            22 => Self::Text,
            23 => Self::VerticalText,
            24 => Self::VisionFootnote,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Abstract => "abstract",
            Self::Algorithm => "algorithm",
            Self::AsideText => "aside_text",
            Self::Chart => "chart",
            Self::Content => "content",
            Self::DisplayFormula => "display_formula",
            Self::DocTitle => "doc_title",
            Self::FigureTitle => "figure_title",
            Self::Footer => "footer",
            Self::FooterImage => "footer_image",
            Self::Footnote => "footnote",
            Self::FormulaNumber => "formula_number",
            Self::Header => "header",
            Self::HeaderImage => "header_image",
            Self::Image => "image",
            Self::InlineFormula => "inline_formula",
            Self::Number => "number",
            Self::ParagraphTitle => "paragraph_title",
            Self::Reference => "reference",
            Self::ReferenceContent => "reference_content",
            Self::Seal => "seal",
            Self::Table => "table",
            Self::Text => "text",
            Self::VerticalText => "vertical_text",
            Self::VisionFootnote => "vision_footnote",
            Self::Other => "other",
        }
    }
}

/// 2D 矩形框物理坐标（原图像素单位 [x1, y1, x2, y2]）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl LayoutBox {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn area(&self) -> f32 {
        ((self.x2 - self.x1).max(0.0)) * ((self.y2 - self.y1).max(0.0))
    }

    pub fn iou(&self, other: &Self) -> f32 {
        let ix1 = self.x1.max(other.x1);
        let iy1 = self.y1.max(other.y1);
        let ix2 = self.x2.min(other.x2);
        let iy2 = self.y2.min(other.y2);

        if ix2 <= ix1 || iy2 <= iy1 {
            return 0.0;
        }

        let inter_area = (ix2 - ix1) * (iy2 - iy1);
        let union_area = self.area() + other.area() - inter_area;

        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }
}

/// 版面识别原子区块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRegion {
    pub id: usize,
    pub category: LayoutCategory,
    pub label: String,
    pub score: f32,
    pub bbox: LayoutBox,
}

/// 版面解析全量结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub image_width: u32,
    pub image_height: u32,
    pub regions: Vec<LayoutRegion>,
    pub elapsed_ms: f64,
}

/// 版面分析后处理配置
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub score_threshold: f32,
    pub nms_threshold: f32,
    pub max_detections: usize,
    pub num_classes: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            score_threshold: 0.15, // 官方黄金门限：0.15
            nms_threshold: 0.5,
            max_detections: 100,
            num_classes: 25,
        }
    }
}
