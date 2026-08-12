use serde::{Deserialize, Serialize};

/// 官方 PP-DocLayoutV3 全量 24 种版面物理区块类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutCategory {
    Title,
    Text,
    Header,
    Footer,
    Figure,
    Table,
    Formula,
    List,
    Caption,
    Footnote,
    Catalog,
    Reference,
    Code,
    Abstract,
    Author,
    Affiliation,
    Keyword,
    Section,
    EquationNumber,
    TableCaption,
    FigureCaption,
    Seal,
    Sidebar,
    Other,
}

impl LayoutCategory {
    pub fn from_id(id: usize) -> Self {
        match id {
            0 => Self::Title,
            1 => Self::Text,
            2 => Self::Header,
            3 => Self::Footer,
            4 => Self::Figure,
            5 => Self::Table,
            6 => Self::Formula,
            7 => Self::List,
            8 => Self::Caption,
            9 => Self::Footnote,
            10 => Self::Catalog,
            11 => Self::Reference,
            12 => Self::Code,
            13 => Self::Abstract,
            14 => Self::Author,
            15 => Self::Affiliation,
            16 => Self::Keyword,
            17 => Self::Section,
            18 => Self::EquationNumber,
            19 => Self::TableCaption,
            20 => Self::FigureCaption,
            21 => Self::Seal,
            22 => Self::Sidebar,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Text => "text",
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Figure => "figure",
            Self::Table => "table",
            Self::Formula => "formula",
            Self::List => "list",
            Self::Caption => "caption",
            Self::Footnote => "footnote",
            Self::Catalog => "catalog",
            Self::Reference => "reference",
            Self::Code => "code",
            Self::Abstract => "abstract",
            Self::Author => "author",
            Self::Affiliation => "affiliation",
            Self::Keyword => "keyword",
            Self::Section => "section",
            Self::EquationNumber => "equation_number",
            Self::TableCaption => "table_caption",
            Self::FigureCaption => "figure_caption",
            Self::Seal => "seal",
            Self::Sidebar => "sidebar",
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
            score_threshold: 0.15, // 官方黄金门限：0.15 解构完整版面元素
            nms_threshold: 0.5,
            max_detections: 100,
            num_classes: 24,
        }
    }
}
