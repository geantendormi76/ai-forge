#![allow(non_snake_case, dead_code, non_camel_case_types)]

use crate::config::{BoundingBox, CellGridInfo};
use std::cmp::Ordering;
use std::collections::HashSet;

pub fn calculate_ioa(inner_bbox: &[f32; 4], container_bbox: &[f32; 4]) -> f32 {
    let inter_x1 = inner_bbox[0].max(container_bbox[0]);
    let inter_y1 = inner_bbox[1].max(container_bbox[1]);
    let inter_x2 = inner_bbox[2].min(container_bbox[2]);
    let inter_y2 = inner_bbox[3].min(container_bbox[3]);

    let inter_w = (inter_x2 - inter_x1).max(0.0);
    let inter_h = (inter_y2 - inter_y1).max(0.0);
    let inter_area = inter_w * inter_h;

    let inner_area = (inner_bbox[2] - inner_bbox[0]).max(0.0) * (inner_bbox[3] - inner_bbox[1]).max(0.0);
    if inner_area <= 0.0 {
        0.0
    } else {
        inter_area / inner_area
    }
}

pub fn is_inside_box(inner_bbox: &[f32; 4], container_bbox: &[f32; 4], ioa_threshold: f32) -> bool {
    calculate_ioa(inner_bbox, container_bbox) >= ioa_threshold
}

pub fn normalize_latex_formula(latex: &str) -> String {
    let mut result = latex.replace("\\ ", " ");
    result = result.replace('Ġ', " ").replace(' ', " ");

    result = result.replace("\\_", "_");
    result = result.replace("\\^", "^");
    while result.contains("^^") {
        result = result.replace("^^", "^");
    }
    result = result.replace("{{\\}}", "").replace("{\\}}", "");
    result = result.replace("^{\\star}", "*");
    result = result.replace("^{\\ast}", "*");

    let cmd_prefixes = [
        "\\mathrm{",
        "\\operatorname*{",
        "\\operatorname{",
        "\\mathbf{",
        "\\text{",
        "\\mathit{",
    ];
    for prefix in &cmd_prefixes {
        let mut search_pos = 0;
        while let Some(start_idx) = result[search_pos..].find(prefix) {
            let abs_start = search_pos + start_idx + prefix.len();
            if let Some(end_rel) = result[abs_start..].find('}') {
                let abs_end = abs_start + end_rel;
                let inner = &result[abs_start..abs_end];
                let cleaned_inner = inner.replace(' ', "");
                result = format!("{}{}{}", &result[..abs_start], cleaned_inner, &result[abs_end..]);
                search_pos = abs_start + cleaned_inner.len() + 1;
            } else {
                break;
            }
        }
    }

    let invalid_single_cmds = [
        "\\A ", "\\B ", "\\C ", "\\D ", "\\F ", "\\G ", "\\H ", "\\I ", "\\J ", "\\K ", "\\L ",
        "\\M ", "\\N ", "\\O ", "\\P ", "\\Q ", "\\R ", "\\S ", "\\T ", "\\U ", "\\V ", "\\W ",
        "\\X ", "\\Y ", "\\Z ",
    ];
    for invalid_cmd in &invalid_single_cmds {
        result = result.replace(invalid_cmd, &invalid_cmd[1..]);
    }

    result = result.replace("\\left.", "").replace("\\right.", "");

    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result.trim().to_string()
}

pub fn normalize_checkbox_symbols(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() != 1 {
        return text.to_string();
    }
    match trimmed.chars().next().unwrap_or_default() {
        'ü' | 'Ü' | '√' | '☑' | 'v' | 'V' => "✓".to_string(),
        '✕' | '✖' | '☒' | 'x' | 'X' => "✗".to_string(),
        _ => text.to_string(),
    }
}

pub fn dehyphenate(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '-' && i > 0 && i + 1 < len && chars[i + 1] == '\n' {
            i += 2;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

pub fn is_sentence_ending_punctuation(c: char) -> bool {
    matches!(c, '.' | '。' | '?' | '？' | '!' | '！' | ':' | '：')
}

pub fn should_merge_paragraph_boxes(
    b1: &BoundingBox,
    text1: &str,
    b2: &BoundingBox,
    font_size: f32,
) -> bool {
    let delta_y = (b2.y_min() - b1.y_max()).max(0.0);
    let max_gap = (font_size * 1.5).max(12.0);

    if delta_y > max_gap {
        return false;
    }

    let trimmed = text1.trim_end();
    if let Some(last_char) = trimmed.chars().next_back() {
        if is_sentence_ending_punctuation(last_char) {
            return false;
        }
    }

    true
}

pub fn is_within_caption_window(parent_box: &BoundingBox, caption_box: &BoundingBox) -> bool {
    let win_x1 = parent_box.x_min() - 300.0;
    let win_x2 = parent_box.x_max() + 300.0;
    let win_y1 = parent_box.y_min() - 300.0;
    let win_y2 = parent_box.y_max() + 600.0;

    let cx = caption_box.center().x;
    let cy = caption_box.center().y;

    cx >= win_x1 && cx <= win_x2 && cy >= win_y1 && cy <= win_y2
}

pub fn compute_knn_caption_distance(parent_box: &BoundingBox, caption_box: &BoundingBox) -> f32 {
    let delta_y = (caption_box.y_min() - parent_box.y_max()).abs();
    let delta_cx = (caption_box.center().x - parent_box.center().x).abs();
    delta_y + 0.5 * delta_cx
}

pub fn cluster_y_positions(positions: &[f32], tolerance: f32) -> Vec<f32> {
    if positions.is_empty() {
        return Vec::new();
    }
    let mut sorted = positions.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mut clustered = Vec::new();
    let mut current_cluster = vec![sorted[0]];

    for &pos in sorted.iter().skip(1) {
        if (pos - *current_cluster.last().unwrap_or(&pos)).abs() <= tolerance {
            current_cluster.push(pos);
        } else {
            let mean = current_cluster.iter().sum::<f32>() / (current_cluster.len() as f32);
            clustered.push(mean);
            current_cluster.clear();
            current_cluster.push(pos);
        }
    }
    if !current_cluster.is_empty() {
        let mean = current_cluster.iter().sum::<f32>() / (current_cluster.len() as f32);
        clustered.push(mean);
    }
    clustered
}

pub fn find_nearest_row_index(positions: &[f32], value: f32) -> usize {
    positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (*a - value).abs();
            let db = (*b - value).abs();
            da.partial_cmp(&db).unwrap_or(Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

pub fn is_same_visual_line(b1: &BoundingBox, b2: &BoundingBox, same_line_tolerance: f32) -> bool {
    let h1 = (b1.y_max() - b1.y_min()).max(1.0);
    let h2 = (b2.y_max() - b2.y_min()).max(1.0);
    let inter_h = (b1.y_max().min(b2.y_max()) - b1.y_min().max(b2.y_min())).max(0.0);
    let overlap_ratio = inter_h / h1.min(h2);
    if overlap_ratio >= 0.6 {
        return true;
    }
    let center_delta = (b1.center().y - b2.center().y).abs();
    center_delta <= (h1.min(h2) * 0.5).max(same_line_tolerance)
}

pub fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u)
        || (0x3400..=0x4DBF).contains(&u)
        || (0x20000..=0x2A6DF).contains(&u)
        || (0x2A700..=0x2B73F).contains(&u)
        || (0x2B740..=0x2B81F).contains(&u)
}

pub fn join_ocr_texts(texts: &[(&BoundingBox, String)], same_line_tolerance: f32) -> String {
    if texts.is_empty() {
        return String::new();
    }
    let 探测起点 = std::time::Instant::now();
    let mut c_x_min = f32::INFINITY;
    let mut c_y_min = f32::INFINITY;
    let mut c_x_max = f32::NEG_INFINITY;
    let mut c_y_max = f32::NEG_INFINITY;
    for (bbox, _) in texts {
        c_x_min = c_x_min.min(bbox.x_min());
        c_y_min = c_y_min.min(bbox.y_min());
        c_x_max = c_x_max.max(bbox.x_max());
        c_y_max = c_y_max.max(bbox.y_max());
    }
    let container_width = c_x_max - c_x_min;
    let has_valid_container = texts.len() >= 2 && container_width > 50.0;
    let mut sorted_texts = texts.to_vec();
    let mut y_coords = Vec::with_capacity(sorted_texts.len());
    for (bbox, _) in &sorted_texts {
        y_coords.push(bbox.y_min());
    }
    let y_positions = cluster_y_positions(&y_coords, same_line_tolerance);
    sorted_texts.sort_by(|(b1, _), (b2, _)| {
        let row1 = find_nearest_row_index(&y_positions, b1.y_min());
        let row2 = find_nearest_row_index(&y_positions, b2.y_min());
        match row1.cmp(&row2) {
            Ordering::Equal => b1.x_min().partial_cmp(&b2.x_min()).unwrap_or(Ordering::Equal),
            other => other,
        }
    });
    let mut result = String::new();
    let mut last_box: Option<&BoundingBox> = None;
    for (bbox, text) in &sorted_texts {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if result.is_empty() {
            result.push_str(&格式化内联公式_helper(trimmed));
        } else {
            let last_char = last_non_whitespace_char_helper(&result);
            let first_char = trimmed.chars().next();
            let mut need_space = true;
            if let (Some(lc), Some(fc)) = (last_char, first_char) {
                if is_cjk(lc) || is_cjk(fc) || lc.is_ascii_punctuation() || fc.is_ascii_punctuation() {
                    need_space = false;
                }
            }
            let is_new_line = if let Some(last) = last_box {
                !is_same_visual_line(last, bbox, same_line_tolerance)
            } else {
                false
            };
            if is_new_line {
                let mut add_newline = false;
                let mut is_line_wrap = false;
                if has_valid_container {
                    if let Some(last) = last_box {
                        let right_gap = c_x_max - last.x_max();
                        let ends_with_non_break_punct = last_char.map_or(false, is_non_break_line_end_punctuation);
                        let threshold = if last_char.map_or(false, |c| c.is_ascii_alphabetic()) {
                            0.5
                        } else {
                            0.3
                        };
                        if !ends_with_non_break_punct && right_gap > container_width * threshold {
                            add_newline = true;
                        } else {
                            is_line_wrap = true;
                        }
                    } else {
                        add_newline = true;
                    }
                } else {
                    add_newline = true;
                }
                let processed_text = 格式化内联公式_helper(trimmed);
                let prev_ends_hyphen = result.ends_with('-');
                if prev_ends_hyphen && is_line_wrap {
                    result.pop();
                    result.push_str(&processed_text);
                } else if add_newline {
                    result.push('\n');
                    result.push_str(&processed_text);
                } else {
                    if need_space && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(&processed_text);
                }
            } else {
                if need_space && !result.ends_with(' ') {
                    result.push(' ');
                }
                result.push_str(&格式化内联公式_helper(trimmed));
            }
        }
        last_box = Some(bbox);
    }
    let 耗时_微秒 = 探测起点.elapsed().as_micros();
    tracing::info!("📊 [Aegis IDP Probe] join_ocr_texts 缝合耗时: {} 微秒, 碎片数: {}", 耗时_微秒, texts.len());
    result
}

pub fn last_non_whitespace_char_helper(text: &str) -> Option<char> {
    text.chars().rev().find(|c| !c.is_whitespace())
}

pub fn is_non_break_line_end_punctuation(c: char) -> bool {
    matches!(c, ',' | '，' | '、' | ';' | '；' | ':' | '：')
}

pub fn 格式化内联公式_helper(text: &str) -> String {
    let clean = normalize_latex_formula(text);
    let trimmed = clean.trim();
    let already_wrapped = trimmed.starts_with('$') || trimmed.starts_with("\\(") || trimmed.starts_with("\\[");
    if already_wrapped {
        return trimmed.to_string();
    }

    let is_real_latex =
        (trimmed.contains('\\') && (
            trimmed.contains("\\frac") || trimmed.contains("\\sqrt") ||
            trimmed.contains("\\alpha") || trimmed.contains("\\beta") ||
            trimmed.contains("\\theta") || trimmed.contains("\\pi") ||
            trimmed.contains("\\sigma") || trimmed.contains("\\sum") ||
            trimmed.contains("\\int") || trimmed.contains("\\times") ||
            trimmed.contains("\\left") || trimmed.contains("\\right")
        )) ||
        (trimmed.contains('^') && !trimmed.ends_with('^')) ||
        (trimmed.contains("_{") && trimmed.contains('}')) ||
        (trimmed.contains("f(x)") && trimmed.contains('=')) ||
        (trimmed.contains("y =") && trimmed.contains('x')) ||
        (trimmed.contains("x^") && (trimmed.contains('+') || trimmed.contains('-') || trimmed.contains('=')));

    if is_real_latex {
        format!("${}$", trimmed)
    } else {
        trimmed.to_string()
    }
}

pub fn match_table_cells_with_structure_rows(
    cells: &mut [CellGridInfo],
    cell_bboxes: &[BoundingBox],
    structure_tokens: &[String],
    ocr_candidates: &[(&BoundingBox, String)],
    row_y_tolerance: f32,
) -> Option<(Vec<Option<usize>>, HashSet<usize>)> {
    if cells.is_empty() || structure_tokens.is_empty() || ocr_candidates.is_empty() {
        return None;
    }

    let mut by_y: Vec<usize> = (0..cell_bboxes.len()).collect();
    by_y.sort_by(|&a, &b| {
        cell_bboxes[a].y_min().partial_cmp(&cell_bboxes[b].y_min()).unwrap_or(Ordering::Equal)
    });

    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut current_row = Vec::new();
    let mut current_y: Option<f32> = None;

    for idx in by_y {
        let y = cell_bboxes[idx].y_min();
        match current_y {
            None => {
                current_row.push(idx);
                current_y = Some(y);
            }
            Some(row_y) if (y - row_y).abs() <= row_y_tolerance => {
                current_row.push(idx);
            }
            Some(_) => {
                current_row.sort_by(|&a, &b| {
                    cell_bboxes[a].x_min().partial_cmp(&cell_bboxes[b].x_min()).unwrap_or(Ordering::Equal)
                });
                rows.push(current_row);
                current_row = vec![idx];
                current_y = Some(y);
            }
        }
    }
    if !current_row.is_empty() {
        current_row.sort_by(|&a, &b| {
            cell_bboxes[a].x_min().partial_cmp(&cell_bboxes[b].x_min()).unwrap_or(Ordering::Equal)
        });
        rows.push(current_row);
    }

    let mut cell_sorted_indices = Vec::with_capacity(cells.len());
    let mut cell_row_flags = vec![0];
    for r in rows {
        cell_sorted_indices.extend(r.iter().copied());
        let next = cell_row_flags.last().copied().unwrap_or(0) + r.len();
        cell_row_flags.push(next);
    }

    let mut row_start_index = Vec::new();
    let mut current_index = 0usize;
    let mut inside_row = false;
    for token in structure_tokens {
        if token == "<tr>" {
            inside_row = true;
        } else if token == "</tr>" {
            inside_row = false;
        } else if is_td_end_token(token) && inside_row {
            row_start_index.push(current_index);
            inside_row = false;
        }
        if is_td_end_token(token) {
            current_index += 1;
        }
    }

    let mut cell_aligned = Vec::with_capacity(row_start_index.len());
    let mut i = 0usize;
    let mut max_value = None;
    for &row_start in &row_start_index {
        while i < cell_row_flags.len() && cell_row_flags[i] <= row_start {
            max_value = Some(max_value.map_or(cell_row_flags[i], |v: usize| v.max(cell_row_flags[i])));
            i += 1;
        }
        cell_aligned.push(max_value.unwrap_or(row_start));
    }
    cell_aligned.push(cell_sorted_indices.len());

    let mut globally_matched_ocr = HashSet::new();
    let mut td_to_cell_mapping = Vec::new();

    let mut td_index = 0;
    let mut td_count = 0;
    let mut matched_row_idx = 0;

    for tag in structure_tokens {
        if tag == "<tr>" {
            td_index = 0;
            continue;
        }
        if !is_td_end_token(tag) {
            continue;
        }

        let mapped_cell_idx = cell_aligned.get(matched_row_idx).copied().and_then(|row_start| {
            let sorted_pos = row_start + td_index;
            cell_sorted_indices.get(sorted_pos).copied()
        });

        td_to_cell_mapping.push(mapped_cell_idx);

        if let Some(cell_idx) = mapped_cell_idx {
            let cell_box = &cell_bboxes[cell_idx];

            for (ocr_idx, &(ocr_box, _)) in ocr_candidates.iter().enumerate() {
                if globally_matched_ocr.contains(&ocr_idx) {
                    continue;
                }

                let inter = ocr_box.intersection_area(cell_box);
                let ocr_area = ocr_box.area();
                let ioa = if ocr_area > 0.0 { inter / ocr_area } else { 0.0 };

                if ioa > 0.70 {
                    globally_matched_ocr.insert(ocr_idx);
                }
            }
        }

        td_index += 1;
        td_count += 1;
        if matched_row_idx + 1 < row_start_index.len() && td_count >= row_start_index[matched_row_idx + 1] {
            matched_row_idx += 1;
        }
    }

    Some((td_to_cell_mapping, globally_matched_ocr))
}

pub fn is_td_end_token(token: &str) -> bool {
    token == "<td></td>"
        || token == "</td>"
        || (token.contains("<td") && token.contains("</td>"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BoundingBox;

    #[test]
    fn test_latex_bpe_space_collapsing() {
        let raw_latex = "\\mathrm{D i s t i l l} = \\operatorname*{m a x}(x) + \\left. 1 \\right.";
        let cleaned = normalize_latex_formula(raw_latex);
        assert_eq!(cleaned, "\\mathrm{Distill} = \\operatorname*{max}(x) + 1");
    }

    #[test]
    fn test_block_merger_and_caption_binder() {
        let b1 = BoundingBox::from_coords(100.0, 100.0, 500.0, 120.0);
        let b2 = BoundingBox::from_coords(100.0, 125.0, 500.0, 145.0);

        assert!(should_merge_paragraph_boxes(&b1, "This is paragraph line 1", &b2, 10.0));
        assert!(!should_merge_paragraph_boxes(&b1, "This is paragraph line 1.", &b2, 10.0));

        let img_box = BoundingBox::from_coords(100.0, 100.0, 500.0, 400.0);
        let caption_box = BoundingBox::from_coords(150.0, 410.0, 450.0, 430.0);
        assert!(is_within_caption_window(&img_box, &caption_box));
    }

    #[test]
    fn test_cjk_english_no_extra_space() {
        let b1 = BoundingBox::from_coords(0.0, 0.0, 30.0, 15.0);
        let b2 = BoundingBox::from_coords(32.0, 0.0, 60.0, 15.0);
        let texts = vec![(&b1, "紫电".to_string()), (&b2, "RPA".to_string())];
        let joined = join_ocr_texts(&texts, 10.0);
        assert_eq!(joined, "紫电RPA", "中英文拼接产生了多余空格！");
    }

    #[test]
    fn test_dehyphenation() {
        let b1 = BoundingBox::from_coords(0.0, 0.0, 80.0, 15.0);
        let b2 = BoundingBox::from_coords(0.0, 20.0, 40.0, 35.0);
        let texts = vec![(&b1, "multi-".to_string()), (&b2, "column".to_string())];
        let joined = join_ocr_texts(&texts, 10.0);
        assert_eq!(joined, "multicolumn", "去连字符自愈失败！");
    }

    #[test]
    fn test_inline_latex_sniffing() {
        let b1 = BoundingBox::from_coords(0.0, 0.0, 100.0, 20.0);
        let texts = vec![(&b1, "f(x) = x^2 + 1".to_string())];
        let joined = join_ocr_texts(&texts, 10.0);
        assert_eq!(joined, "$f(x) = x^2 + 1$", "内联 Latex 包裹自愈失败！");
    }
}
