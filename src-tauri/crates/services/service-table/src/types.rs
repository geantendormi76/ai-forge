use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CellGridInfo {
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
}

pub type BBox8 = [f32; 8];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCellPredict {
    pub grid: CellGridInfo,
    pub bbox: BBox8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructureResult {
    pub structure_tokens: Vec<String>,
    pub cells: Vec<TableCellPredict>,
    pub score: f32,
}
