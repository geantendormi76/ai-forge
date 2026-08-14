#![allow(non_snake_case, dead_code, non_camel_case_types)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinAreaRect {
    pub center: Point,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub points: Vec<Point>,
}

impl BoundingBox {
    pub fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub fn from_coords(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let points = vec![
            Point::new(x1, y1),
            Point::new(x2, y1),
            Point::new(x2, y2),
            Point::new(x1, y2),
        ];
        Self { points }
    }

    pub fn area(&self) -> f32 {
        let x_min = self.x_min();
        let y_min = self.y_min();
        let x_max = self.x_max();
        let y_max = self.y_max();
        ((x_max - x_min) * (y_max - y_min)).max(0.0)
    }

    pub fn perimeter(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut perimeter = 0.0f32;
        let n = self.points.len();
        for i in 0..n {
            let p1 = &self.points[i];
            let p2 = &self.points[(i + 1) % n];
            let dx = p1.x - p2.x;
            let dy = p1.y - p2.y;
            perimeter += (dx * dx + dy * dy).sqrt();
        }
        perimeter
    }

    pub fn x_min(&self) -> f32 {
        self.points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min)
    }

    pub fn y_min(&self) -> f32 {
        self.points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min)
    }

    pub fn x_max(&self) -> f32 {
        self.points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn y_max(&self) -> f32 {
        self.points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn center(&self) -> Point {
        if self.points.is_empty() {
            return Point::new(0.0, 0.0);
        }
        let sum_x: f32 = self.points.iter().map(|p| p.x).sum();
        let sum_y: f32 = self.points.iter().map(|p| p.y).sum();
        let count = self.points.len() as f32;
        Point::new(sum_x / count, sum_y / count)
    }

    pub fn intersection_area(&self, other: &BoundingBox) -> f32 {
        let inter_x_min = self.x_min().max(other.x_min());
        let inter_y_min = self.y_min().max(other.y_min());
        let inter_x_max = self.x_max().min(other.x_max());
        let inter_y_max = self.y_max().min(other.y_max());
        let w = (inter_x_max - inter_x_min).max(0.0);
        let h = (inter_y_max - inter_y_min).max(0.0);
        w * h
    }

    pub fn union(&self, other: &Self) -> Self {
        Self::from_coords(
            self.x_min().min(other.x_min()),
            self.y_min().min(other.y_min()),
            self.x_max().max(other.x_max()),
            self.y_max().max(other.y_max()),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellGridInfo {
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum 排序标签 {
    #[serde(rename = "页眉")]
    页眉,
    #[serde(rename = "页脚")]
    页脚,
    #[serde(rename = "文档标题")]
    文档标题,
    #[serde(rename = "段落标题")]
    段落标题,
    #[serde(rename = "视觉实体")]
    视觉实体,
    #[serde(rename = "视觉标题")]
    视觉标题,
    #[serde(rename = "无序占位")]
    无序占位,
    #[serde(rename = "普通文本")]
    普通文本,
    #[serde(rename = "跨栏元素")]
    跨栏元素,
    #[serde(rename = "跨栏引用")]
    跨栏引用,
    #[serde(rename = "文献引用")]
    文献引用,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct 增强版面元素 {
    pub 物理边界: BoundingBox,
    pub 元素类型: 排序标签,
    pub 估算行数: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct 排序数据块 {
    pub 物理边界: BoundingBox,
    pub 原始索引: usize,
    pub 标签: 排序标签,
    pub 主方向: SortDirection,
    pub 总行数: u32,
    pub 行高: f32,
}

impl 排序数据块 {
    pub fn new(物理边界: BoundingBox, 原始索引: usize, 标签: 排序标签, 估算行数: Option<u32>) -> Self {
        let 宽 = 物理边界.x_max() - 物理边界.x_min();
        let 高 = 物理边界.y_max() - 物理边界.y_min();
        let 主方向 = if 宽 >= 高 {
            SortDirection::Horizontal
        } else {
            SortDirection::Vertical
        };
        let 总行数 = 估算行数.unwrap_or(1).max(1);
        Self {
            物理边界,
            原始索引,
            标签,
            主方向,
            总行数,
            行高: 高 / 总行数 as f32,
        }
    }
    pub fn 宽度(&self) -> f32 {
        self.物理边界.x_max() - self.物理边界.x_min()
    }
    pub fn 高度(&self) -> f32 {
        self.物理边界.y_max() - self.物理边界.y_min()
    }
    pub fn 面积(&self) -> f32 {
        self.宽度() * self.高度()
    }
    pub fn 中心点(&self) -> (f32, f32) {
        (
            (self.物理边界.x_min() + self.物理边界.x_max()) / 2.0,
            (self.物理边界.y_min() + self.物理边界.y_max()) / 2.0,
        )
    }
    pub fn 长边长度(&self) -> f32 {
        self.宽度().max(self.高度())
    }
}
