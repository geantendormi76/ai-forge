#![allow(non_snake_case, dead_code, non_camel_case_types)]

use crate::config::BoundingBox;
use std::cmp::Ordering;

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
    let mut result = latex.to_string();

    // 1. 优先前置处理 cases 环境换行与末尾闭合
    if result.contains(r"\begin{cases}") || result.contains(r"\begin {cases}") {
        result = result.replace(r"\\end{cases}", r"\end{cases}");
        result = result.replace(r"\ \end{cases}", r"\end{cases}");
        result = result.replace(r"\ \end {cases}", r"\end{cases}");
        result = result.replace(r"\end {cases}", r"\end{cases}");
        result = result.replace(r"} { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} { 0 ,}", r"} \\ { 0 ,}");
        result = result.replace(r"}{ 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} \ { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"}\ { 0 , }", r"} \\ { 0 , }");
        result = result.replace(r"} \ {", r"} \\ {");
        result = result.replace(r"}\ {", r"} \\ {");
        result = result.replace(r"} \ \ {", r"} \\ {");
    }

    // 2. 清洗 BPE 专用标记
    result = result.replace('Ġ', " ").replace(' ', " ");

    // 3. 基础符号清洗与转义修复
    result = result.replace("\\_", "_");
    result = result.replace("\\^", "^");
    while result.contains("^^") {
        result = result.replace("^^", "^");
    }
    result = result.replace("{{\\}}", "").replace("{\\}}", "");
    result = result.replace("^{\\star}", "*");
    result = result.replace("^{\\ast}", "*");

    // 4. 规整 PP-FormulaNet 固有 Token 碎片与 argmax 变体
    result = result.replace(r"\ { tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\{ tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c } ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{tau __{ c c } ^{ * }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_ c ^{ *}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_ c ^{*}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_c^{ *}}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\tau{{} }{_c^{* }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c } ^{*} }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{ tau __{ c c }", r"{\boldsymbol{\tau}_c");
    result = result.replace(r"tau __{ c c }", r"\boldsymbol{\tau}_c");
    result = result.replace(r"\mathrm{Disill}", r"\mathrm{Distill}");
    result = result.replace("Disill", "Distill");

    // 5. 符号与括号闭合规整
    result = result.replace(r"{\boldsymbol{\tau}_c^{* } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{* } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{* }}", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"{\boldsymbol{\tau}_c ^{ * } }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\boldsymbol{\tau}_c ^{* }", r"\boldsymbol{\tau}_c^*");
    result = result.replace(r"\ \boldsymbol{\tau}_c^*", r"\boldsymbol{\tau}_c^*");

    // 6. 统一精准规整 argmax 宏
    while result.contains("operatornamearg") {
        result = result.replace(r"\mathop{{ operatornamearg r a } * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
        result = result.replace(r"\mathop{{operatornamearg r a } * * m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
        result = result.replace(r"\mathop{{ operatornamearg r a } ** m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
        result = result.replace(r"\mathop{{operatornamearg r a } ** m{ m x } }", r"\mathop{\operatorname{arg}\operatorname*{max}}");
        result = result.replace(r"{ operatornamearg r a } * * m{ m x }", r"\operatorname{arg}\operatorname*{max}");
        result = result.replace(r"{ operatornamearg r a } ** m{ m x }", r"\operatorname{arg}\operatorname*{max}");
        result = result.replace(r"operatornamearg r a", r"\operatorname{arg}");
        result = result.replace(r"operatornamearg", r"\operatorname{arg}");
        break;
    }
    result = result.replace(r"\operatorname{arg}\operatorname{max}", r"\operatorname{arg}\operatorname*{max}");
    result = result.replace(r"\operatorname*{arg}\operatorname*{max}", r"\operatorname{arg}\operatorname*{max}");

    // 7. 清理非法单字母反斜杠命令 (\A -> A)
    let invalid_single_cmds = [
        "\\A ", "\\B ", "\\C ", "\\D ", "\\F ", "\\G ", "\\H ", "\\I ", "\\J ", "\\K ", "\\L ",
        "\\M ", "\\N ", "\\O ", "\\P ", "\\Q ", "\\R ", "\\S ", "\\T ", "\\U ", "\\V ", "\\W ",
        "\\X ", "\\Y ", "\\Z ",
    ];
    for invalid_cmd in &invalid_single_cmds {
        result = result.replace(invalid_cmd, &invalid_cmd[1..]);
    }

    // 8. 清除空边界符
    result = result.replace("\\left.", "").replace("\\right.", "");

    // 9. 收拢连续空格 (保留 \\ 换行)
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
