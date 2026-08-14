#![allow(non_snake_case, dead_code, non_camel_case_types)]

use crate::config::{BoundingBox, CellGridInfo};

pub fn compute_col_width_similarity(widths1: &[f32], widths2: &[f32]) -> f32 {
    if widths1.len() != widths2.len() || widths1.is_empty() {
        return 0.0;
    }
    let sum1: f32 = widths1.iter().sum();
    let sum2: f32 = widths2.iter().sum();
    if sum1 <= 0.0 || sum2 <= 0.0 {
        return 0.0;
    }

    let norm1: Vec<f32> = widths1.iter().map(|w| w / sum1).collect();
    let norm2: Vec<f32> = widths2.iter().map(|w| w / sum2).collect();

    let mut diff_sum = 0.0f32;
    for i in 0..norm1.len() {
        diff_sum += (norm1[i] - norm2[i]).abs();
    }

    (1.0 - (diff_sum / 2.0)).max(0.0)
}

pub fn can_merge_cross_page_tables(
    table1_cols: &[f32],
    table2_cols: &[f32],
    table1_page: usize,
    table2_page: usize,
) -> bool {
    if table2_page != table1_page + 1 {
        return false;
    }
    if table1_cols.len() != table2_cols.len() || table1_cols.is_empty() {
        return false;
    }
    let sim = compute_col_width_similarity(table1_cols, table2_cols);
    sim >= 0.90
}

pub fn merge_table_html_tokens(
    html1: &str,
    html2: &str,
) -> String {
    let h1_clean = html1
        .trim_end()
        .trim_end_matches("</tbody>\n</table>")
        .trim_end_matches("</tbody></table>")
        .trim_end_matches("</table>")
        .trim_end();

    let h2_content = if let Some(start_idx) = html2.find("<tbody>") {
        &html2[start_idx + "<tbody>".len()..]
    } else {
        html2
    };

    let h2_clean = h2_content
        .trim_end()
        .trim_end_matches("</tbody>\n</table>")
        .trim_end_matches("</tbody></table>")
        .trim_end_matches("</table>")
        .trim_end();

    format!("{}\n{}\n</tbody>\n</table>", h1_clean, h2_clean)
}

pub fn combine_rectangles_kmeans(rectangles: &[BoundingBox], target_n: usize) -> Vec<BoundingBox> {
    let num_rects = rectangles.len();
    if num_rects <= target_n || target_n == 0 {
        return rectangles.to_vec();
    }
    let points: Vec<(f32, f32)> = rectangles
        .iter()
        .map(|r| {
            let cx = (r.x_min() + r.x_max()) * 0.5;
            let cy = (r.y_min() + r.y_max()) * 0.5;
            (cx, cy)
        })
        .collect();
    let mut centers = kmeans_maxdist_init(&points, target_n);
    let mut labels = vec![0; num_rects];
    let max_iters = 10;
    for _ in 0..max_iters {
        let mut changed = false;
        for (i, &(px, py)) in points.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut best_center = 0;
            for (c_idx, &(cx, cy)) in centers.iter().enumerate() {
                let dist = (px - cx).powi(2) + (py - cy).powi(2);
                if dist < min_dist {
                    min_dist = dist;
                    best_center = c_idx;
                }
            }
            if labels[i] != best_center {
                labels[i] = best_center;
                changed = true;
            }
        }
        if !changed {
            break;
        }
        let mut sums = vec![(0.0f32, 0.0f32, 0usize); target_n];
        for (i, &label) in labels.iter().enumerate() {
            sums[label].0 += points[i].0;
            sums[label].1 += points[i].1;
            sums[label].2 += 1;
        }
        for (c_idx, center) in centers.iter_mut().enumerate() {
            let (sx, sy, count) = sums[c_idx];
            if count > 0 {
                *center = (sx / count as f32, sy / count as f32);
            }
        }
    }
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); target_n];
    for (i, &label) in labels.iter().enumerate() {
        groups[label].push(i);
    }
    let mut combined = Vec::new();
    for group in groups {
        if group.is_empty() {
            continue;
        }
        let first_rect = &rectangles[group[0]];
        let mut min_x = first_rect.x_min();
        let mut min_y = first_rect.y_min();
        let mut max_x = first_rect.x_max();
        let mut max_y = first_rect.y_max();
        for &idx in group.iter().skip(1) {
            let rect = &rectangles[idx];
            min_x = min_x.min(rect.x_min());
            min_y = min_y.min(rect.y_min());
            max_x = max_x.max(rect.x_max());
            max_y = max_y.max(rect.y_max());
        }
        combined.push(BoundingBox::from_coords(min_x, min_y, max_x, max_y));
    }
    if combined.is_empty() {
        rectangles.to_vec()
    } else {
        combined
    }
}

fn kmeans_maxdist_init(points: &[(f32, f32)], k: usize) -> Vec<(f32, f32)> {
    if points.is_empty() || k == 0 {
        return Vec::new();
    }
    if k >= points.len() {
        return points.to_vec();
    }
    let mut centers = Vec::new();
    let mut sorted_by_x: Vec<usize> = (0..points.len()).collect();
    sorted_by_x.sort_by(|&a, &b| points[a].0.partial_cmp(&points[b].0).unwrap());
    let first_idx = sorted_by_x[sorted_by_x.len() / 2];
    centers.push(points[first_idx]);
    while centers.len() < k {
        let mut max_idx = 0;
        let mut max_dist = -1.0f32;
        for (i, &pt) in points.iter().enumerate() {
            if centers.contains(&pt) {
                continue;
            }
            let mut min_dist_sq = f32::MAX;
            for &c in &centers {
                let dist_sq = (pt.0 - c.0).powi(2) + (pt.1 - c.1).powi(2);
                min_dist_sq = min_dist_sq.min(dist_sq);
            }
            if min_dist_sq > max_dist {
                max_dist = min_dist_sq;
                max_idx = i;
            }
        }
        if !centers.contains(&points[max_idx]) {
            centers.push(points[max_idx]);
        } else if let Some(&point) = points.iter().find(|p| !centers.contains(p)) {
            centers.push(point);
        } else {
            break;
        }
    }
    centers
}

pub fn table_cells_to_html_structure(
    cell_bboxes: &[BoundingBox],
    tolerance: f32,
) -> Option<(Vec<String>, Vec<(usize, CellGridInfo)>)> {
    if cell_bboxes.is_empty() {
        return None;
    }
    let mut x_coords = Vec::with_capacity(cell_bboxes.len() * 2);
    let mut y_coords = Vec::with_capacity(cell_bboxes.len() * 2);
    for bbox in cell_bboxes {
        x_coords.push(bbox.x_min());
        x_coords.push(bbox.x_max());
        y_coords.push(bbox.y_min());
        y_coords.push(bbox.y_max());
    }
    let x_positions = cluster_positions(x_coords, tolerance);
    let y_positions = cluster_positions(y_coords, tolerance);
    if x_positions.len() < 2 || y_positions.len() < 2 {
        return None;
    }
    let num_rows = y_positions.len() - 1;
    let num_cols = x_positions.len() - 1;
    let mut entries = Vec::with_capacity(cell_bboxes.len());
    let mut cell_map = std::collections::HashMap::new();
    for bbox in cell_bboxes {
        let x1_idx = nearest_index(&x_positions, bbox.x_min());
        let x2_idx = nearest_index(&x_positions, bbox.x_max());
        let y1_idx = nearest_index(&y_positions, bbox.y_min());
        let y2_idx = nearest_index(&y_positions, bbox.y_max());
        let col_start = x1_idx.min(x2_idx).min(num_cols.saturating_sub(1));
        let col_end = x1_idx.max(x2_idx).min(num_cols);
        let row_start = y1_idx.min(y2_idx).min(num_rows.saturating_sub(1));
        let row_end = y1_idx.max(y2_idx).min(num_rows);
        let row_span = (row_end - row_start).max(1);
        let col_span = (col_end - col_start).max(1);
        let entry_idx = entries.len();
        entries.push(CellGridInfo {
            row: row_start,
            col: col_start,
            row_span,
            col_span,
        });
        let row_stop = (row_start + row_span).min(num_rows);
        let col_stop = (col_start + col_span).min(num_cols);
        for r in row_start..row_stop {
            for c in col_start..col_stop {
                cell_map.entry((r, c)).or_insert(entry_idx);
            }
        }
    }
    let mut structure_tokens = Vec::new();
    let mut cell_order = Vec::new();
    structure_tokens.push("<table>".to_string());
    structure_tokens.push("<tbody>".to_string());
    for r in 0..num_rows {
        structure_tokens.push("<tr>".to_string());
        let mut c = 0;
        while c < num_cols {
            if let Some(&entry_idx) = cell_map.get(&(r, c)) {
                let entry = &entries[entry_idx];
                if entry.row == r && entry.col == c {
                    let mut attrs = String::new();
                    if entry.row_span > 1 {
                        attrs.push_str(&format!(" rowspan=\"{}\"", entry.row_span));
                    }
                    if entry.col_span > 1 {
                        attrs.push_str(&format!(" colspan=\"{}\"", entry.col_span));
                    }
                    let token = if !attrs.is_empty() {
                        format!("<td{}></td>", attrs)
                    } else {
                        "<td></td>".to_string()
                    };
                    structure_tokens.push(token);
                    cell_order.push((entry_idx, entry.clone()));
                }
                c += entry.col_span.max(1);
            } else {
                structure_tokens.push("<td></td>".to_string());
                c += 1;
            }
        }
        structure_tokens.push("</tr>".to_string());
    }
    structure_tokens.push("</tbody>".to_string());
    structure_tokens.push("</table>".to_string());
    if cell_order.is_empty() {
        None
    } else {
        Some((structure_tokens, cell_order))
    }
}

fn cluster_positions(mut positions: Vec<f32>, tolerance: f32) -> Vec<f32> {
    if positions.is_empty() {
        return Vec::new();
    }
    positions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut clustered = Vec::new();
    let mut current_cluster = vec![positions[0]];
    for &pos in positions.iter().skip(1) {
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

fn nearest_index(positions: &[f32], value: f32) -> usize {
    positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (*a - value).abs();
            let db = (*b - value).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

pub fn 跨单元格_OCR_分裂(
    mapped_boxes: &[(BoundingBox, String)],
    _tolerance: f32,
) -> Vec<(BoundingBox, String)> {
    let mut results = Vec::new();
    for (bbox, text) in mapped_boxes {
        let parts: Vec<&str> = text.split("  ").filter(|s| !s.trim().is_empty()).collect();
        if parts.len() <= 1 {
            results.push((bbox.clone(), text.clone()));
        } else {
            let x_min = bbox.x_min();
            let x_max = bbox.x_max();
            let y_min = bbox.y_min();
            let y_max = bbox.y_max();
            let total_width = x_max - x_min;
            let total_chars: usize = parts.iter().map(|p| p.chars().count()).sum();
            if total_chars == 0 {
                results.push((bbox.clone(), text.clone()));
                continue;
            }
            let char_width = total_width / total_chars as f32;
            let mut current_x = x_min;
            for part in parts {
                let clean_part = part.trim().to_string();
                let part_chars_count = clean_part.chars().count();
                let part_width = part_chars_count as f32 * char_width;
                let part_box = BoundingBox::from_coords(
                    current_x,
                    y_min,
                    current_x + part_width,
                    y_max,
                );
                results.push((part_box, clean_part));
                current_x += part_width + char_width * 2.0;
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BoundingBox;

    #[test]
    fn test_cross_page_table_merging_state_machine() {
        let cols1 = vec![100.0, 200.0, 150.0];
        let cols2 = vec![105.0, 195.0, 150.0];

        assert!(can_merge_cross_page_tables(&cols1, &cols2, 1, 2));
        assert!(!can_merge_cross_page_tables(&cols1, &cols2, 1, 3));

        let html1 = "<table>\n<tbody>\n  <tr>\n    <td>Header</td>\n  </tr>\n  <tr>\n    <td>Data1</td>\n  </tr>\n</tbody>\n</table>";
        let html2 = "<table>\n<tbody>\n  <tr>\n    <td>Data2</td>\n  </tr>\n</tbody>\n</table>";

        let merged = merge_table_html_tokens(html1, html2);
        assert!(merged.contains("Data1") && merged.contains("Data2"));
    }

    #[test]
    fn test_cross_cell_ocr_split() {
        let bbox = BoundingBox::from_coords(0.0, 0.0, 100.0, 20.0);
        let mapped = vec![(bbox, "LeftColumn  RightColumn".to_string())];
        let split = 跨单元格_OCR_分裂(&mapped, 5.0);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0].1, "LeftColumn");
        assert_eq!(split[1].1, "RightColumn");
    }

    #[test]
    fn test_table_cells_to_html_structure() {
        let b1 = BoundingBox::from_coords(0.0, 0.0, 50.0, 20.0);
        let b2 = BoundingBox::from_coords(50.0, 0.0, 100.0, 20.0);
        let b3 = BoundingBox::from_coords(0.0, 20.0, 50.0, 40.0);
        let b4 = BoundingBox::from_coords(50.0, 20.0, 100.0, 40.0);
        let bboxes = vec![b1, b2, b3, b4];

        let res = table_cells_to_html_structure(&bboxes, 2.0);
        assert!(res.is_some());
        let (tokens, cell_order) = res.unwrap();
        assert!(tokens.contains(&"<table>".to_string()));
        assert_eq!(cell_order.len(), 4);
    }
}
