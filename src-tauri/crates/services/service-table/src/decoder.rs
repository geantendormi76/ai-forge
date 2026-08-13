use crate::types::{CellGridInfo, BBox8, TableCellPredict, TableStructureResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableError {
    #[error("字典文件读取失败: {0}")]
    DictLoad(String),
    #[error("Tensor 解构或 Shape 校验失败: {0}")]
    InvalidTensor(String),
    #[error("图像预处理失败: {0}")]
    Preprocess(String),
    #[error("ONNX 推理失败: {0}")]
    Infer(String),
}

pub type TableResult<T> = Result<T, TableError>;

pub fn parse_cell_grid_info(tokens: &[String]) -> Vec<CellGridInfo> {
    let mut cells = Vec::new();
    let mut current_row: usize = 0;
    let mut current_col: usize = 0;
    let mut idx = 0usize;
    let mut occupied: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    while idx < tokens.len() {
        let token = tokens[idx].as_str();

        if token == "<tr>" {
            current_col = 0;
            while occupied.contains(&(current_row, current_col)) {
                current_col += 1;
            }
            idx += 1;
            continue;
        }

        if token == "</tr>" {
            current_row += 1;
            idx += 1;
            continue;
        }

        if token == "<td></td>" {
            while occupied.contains(&(current_row, current_col)) {
                current_col += 1;
            }
            cells.push(CellGridInfo {
                row: current_row,
                col: current_col,
                row_span: 1,
                col_span: 1,
            });
            current_col += 1;
            idx += 1;
            continue;
        }

        if token.starts_with("<td") {
            let parsed = parse_td_tag(tokens, idx);

            while occupied.contains(&(current_row, current_col)) {
                current_col += 1;
            }

            cells.push(CellGridInfo {
                row: current_row,
                col: current_col,
                row_span: parsed.row_span,
                col_span: parsed.col_span,
            });

            if parsed.row_span > 1 {
                for r in 1..parsed.row_span {
                    for c in 0..parsed.col_span {
                        occupied.insert((current_row + r, current_col + c));
                    }
                }
            }

            current_col += parsed.col_span;
            idx = parsed.next_index;
            continue;
        }

        idx += 1;
    }

    cells
}

struct ParsedTdTag {
    row_span: usize,
    col_span: usize,
    next_index: usize,
}

fn parse_span_attr(token: &str, attr: &str) -> Option<usize> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = token.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = token[value_start..].find('"') {
            if let Ok(value) = token[value_start..value_start + end].parse::<usize>() {
                return Some(value);
            }
        }
    }
    None
}

fn parse_td_tag(tokens: &[String], start_idx: usize) -> ParsedTdTag {
    let mut col_span = 1usize;
    let mut row_span = 1usize;

    if let Some(start_token) = tokens.get(start_idx) {
        if let Some(stripped) = start_token.strip_prefix("<td") {
            if let Some(before_gt) = stripped.split('>').next() {
                if !before_gt.is_empty() {
                    if let Some(v) = parse_span_attr(before_gt, "colspan") {
                        col_span = v;
                    }
                    if let Some(v) = parse_span_attr(before_gt, "rowspan") {
                        row_span = v;
                    }
                }
            }
        }
    }

    let mut idx = start_idx + 1;

    while idx < tokens.len() {
        let token = tokens[idx].as_str();

        if token == ">"
            || token == "</td>"
            || token.starts_with("<td")
            || token == "<tr>"
            || token == "</tr>"
        {
            break;
        }

        if let Some(v) = parse_span_attr(token, "colspan") {
            col_span = v;
        }
        if let Some(v) = parse_span_attr(token, "rowspan") {
            row_span = v;
        }

        idx += 1;
    }

    let mut next_index = idx;
    while next_index < tokens.len() {
        let token = tokens[next_index].as_str();
        if token == "</td>" {
            next_index += 1;
            break;
        }
        if token.starts_with("<td") || token == "<tr>" || token == "</tr>" {
            break;
        }
        next_index += 1;
    }

    ParsedTdTag {
        row_span,
        col_span,
        next_index: next_index.max(start_idx + 1),
    }
}

pub fn wrap_table_html_with_content(tokens: &[String], cell_texts: &[Option<String>]) -> String {
    let mut result = Vec::new();
    let mut td_index = 0;
    let mut idx = 0usize;

    result.push("<html><body>".to_string());

    let has_table_tag = tokens.first().map(|t| t.contains("<table")).unwrap_or(false);
    if !has_table_tag {
        result.push("<table>".to_string());
    }

    while idx < tokens.len() {
        let tag = tokens[idx].as_str();

        if tag == "<td></td>" {
            result.push("<td>".to_string());
            if let Some(Some(text)) = cell_texts.get(td_index) {
                result.push(text.clone());
            }
            result.push("</td>".to_string());
            td_index += 1;
            idx += 1;
            continue;
        }

        if tag.starts_with("<td") {
            let parsed = parse_td_tag(tokens, idx);
            result.push("<td>".to_string());
            if let Some(Some(text)) = cell_texts.get(td_index) {
                result.push(text.clone());
            }
            result.push("</td>".to_string());
            td_index += 1;
            idx = parsed.next_index;
            continue;
        }

        result.push(tokens[idx].clone());
        idx += 1;
    }

    if !has_table_tag {
        result.push("</table>".to_string());
    }
    result.push("</body></html>".to_string());

    result.join("")
}

#[derive(Debug, Clone)]
pub struct TableStructureDecode {
    pub character_dict: Vec<String>,
    pub ignored_tokens: Vec<usize>,
    pub td_token_indices: Vec<usize>,
    pub end_idx: usize,
}

impl TableStructureDecode {
    pub fn from_dict_str(dict_content: &str) -> TableResult<Self> {
        let mut character_dict: Vec<String> = dict_content
            .lines()
            .map(|l| l.trim_end().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        if !character_dict.contains(&"<td></td>".to_string()) {
            character_dict.push("<td></td>".to_string());
        }
        if let Some(pos) = character_dict.iter().position(|s| s == "<td>") {
            character_dict.remove(pos);
        }

        let beg_str = "sos";
        let end_str = "eos";

        let mut final_dict = Vec::with_capacity(character_dict.len() + 2);
        final_dict.push(beg_str.to_string());
        final_dict.extend(character_dict);
        final_dict.push(end_str.to_string());

        let start_idx = 0;
        let end_idx = final_dict.len() - 1;
        let ignored_tokens = vec![start_idx, end_idx];

        let td_tokens = ["<td>", "<td", "<td></td>"];
        let td_token_indices: Vec<usize> = td_tokens
            .iter()
            .filter_map(|&token| final_dict.iter().position(|s| s == token))
            .collect();

        Ok(Self {
            character_dict: final_dict,
            ignored_tokens,
            td_token_indices,
            end_idx,
        })
    }

    pub fn from_dict_file(dict_path: &std::path::Path) -> TableResult<Self> {
        let content = std::fs::read_to_string(dict_path).map_err(|e| {
            TableError::DictLoad(format!("无法读取字典文件 '{}': {}", dict_path.display(), e))
        })?;
        Self::from_dict_str(&content)
    }

    pub fn decode_single(
        &self,
        structure_logits: &ndarray::Array2<f32>,
        bbox_preds: &ndarray::Array2<f32>,
        orig_shape: (f32, f32),
        target_size: f32,
    ) -> TableResult<TableStructureResult> {
        let seq_len = structure_logits.shape()[0];
        let (orig_h, orig_w) = orig_shape;

        let scale = target_size / orig_h.max(orig_w);
        let longest_side = if scale > 0.0 { target_size / scale } else { orig_h.max(orig_w) };

        let mut structure_tokens = Vec::new();
        let mut cell_boxes: Vec<BBox8> = Vec::new();
        let mut scores = Vec::new();

        for seq_idx in 0..seq_len {
            let (token_idx, token_prob) = self.argmax_at_seq(structure_logits, seq_idx);

            if seq_idx > 0 && token_idx == self.end_idx {
                break;
            }

            if self.ignored_tokens.contains(&token_idx) {
                continue;
            }

            let token = self
                .character_dict
                .get(token_idx)
                .cloned()
                .unwrap_or_else(|| format!("UNK_{}", token_idx));

            structure_tokens.push(token);
            scores.push(token_prob);

            if self.td_token_indices.contains(&token_idx) {
                let mut bbox = [0.0f32; 8];
                for coord_idx in 0..8 {
                    let norm_coord = bbox_preds[[seq_idx, coord_idx]];
                    let mut coord = norm_coord * longest_side;
                    if coord_idx % 2 == 0 {
                        coord = coord.clamp(0.0, orig_w);
                    } else {
                        coord = coord.clamp(0.0, orig_h);
                    }
                    bbox[coord_idx] = coord;
                }
                cell_boxes.push(bbox);
            }
        }

        let mean_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f32>() / (scores.len() as f32)
        };

        let grid_infos = parse_cell_grid_info(&structure_tokens);

        let cells = grid_infos
            .into_iter()
            .zip(cell_boxes.into_iter())
            .map(|(grid, bbox)| TableCellPredict { grid, bbox })
            .collect();

        Ok(TableStructureResult {
            structure_tokens,
            cells,
            score: mean_score,
        })
    }

    fn argmax_at_seq(&self, logits: &ndarray::Array2<f32>, seq_idx: usize) -> (usize, f32) {
        let vocab_size = logits.shape()[1];
        let mut max_idx = 0;
        let mut max_val = f32::NEG_INFINITY;

        for v_idx in 0..vocab_size {
            let val = logits[[seq_idx, v_idx]];
            if val > max_val {
                max_val = val;
                max_idx = v_idx;
            }
        }

        (max_idx, max_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cell_grid_info_simple() {
        let tokens = vec![
            "<tr>".to_string(),
            "<td></td>".to_string(),
            "<td></td>".to_string(),
            "</tr>".to_string(),
            "<tr>".to_string(),
            "<td></td>".to_string(),
            "<td></td>".to_string(),
            "</tr>".to_string(),
        ];

        let grid = parse_cell_grid_info(&tokens);
        assert_eq!(grid.len(), 4);
        assert_eq!(grid[0], CellGridInfo { row: 0, col: 0, row_span: 1, col_span: 1 });
        assert_eq!(grid[1], CellGridInfo { row: 0, col: 1, row_span: 1, col_span: 1 });
        assert_eq!(grid[2], CellGridInfo { row: 1, col: 0, row_span: 1, col_span: 1 });
        assert_eq!(grid[3], CellGridInfo { row: 1, col: 1, row_span: 1, col_span: 1 });
    }

    #[test]
    fn test_parse_cell_grid_info_spans() {
        let tokens = vec![
            "<tr>".to_string(),
            "<td colspan=\"2\">".to_string(),
            "</td>".to_string(),
            "</tr>".to_string(),
            "<tr>".to_string(),
            "<td></td>".to_string(),
            "<td></td>".to_string(),
            "</tr>".to_string(),
        ];

        let grid = parse_cell_grid_info(&tokens);
        assert_eq!(grid.len(), 3);
        assert_eq!(grid[0], CellGridInfo { row: 0, col: 0, row_span: 1, col_span: 2 });
        assert_eq!(grid[1], CellGridInfo { row: 1, col: 0, row_span: 1, col_span: 1 });
        assert_eq!(grid[2], CellGridInfo { row: 1, col: 1, row_span: 1, col_span: 1 });
    }
}
