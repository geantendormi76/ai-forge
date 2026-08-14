#![allow(non_snake_case, dead_code, non_camel_case_types)]

use crate::config::{BoundingBox, SortDirection, 增强版面元素, 排序标签, 排序数据块};
use std::cmp::Ordering;

const 跨栏参考文本块行数阈值: f32 = 10.0;
const GUTTER_WIDTH_THRESHOLD: f32 = 12.0;

pub fn detect_central_gutter(
    blocks: &[排序数据块],
    page_width: f32,
) -> Option<(f32, f32)> {
    if blocks.is_empty() || page_width <= 0.0 {
        return None;
    }

    let search_start = page_width * 0.25;
    let search_end = page_width * 0.75;

    let mut left_col_max_x = f32::MIN;
    let mut right_col_min_x = f32::MAX;

    let mut multi_col_candidates_count = 0;

    for block in blocks {
        let x1 = block.物理边界.x_min();
        let x2 = block.物理边界.x_max();
        let width = x2 - x1;

        if width >= page_width * 0.75 || block.标签 == 排序标签::文档标题 {
            continue;
        }

        if x1 < page_width * 0.45 && x2 > page_width * 0.55 {
            continue;
        }

        multi_col_candidates_count += 1;
        let cx = (x1 + x2) / 2.0;

        if cx < page_width * 0.5 {
            left_col_max_x = left_col_max_x.max(x2);
        } else {
            right_col_min_x = right_col_min_x.min(x1);
        }
    }

    if multi_col_candidates_count < 2 {
        return None;
    }

    if left_col_max_x < right_col_min_x && left_col_max_x > search_start && right_col_min_x < search_end {
        let gutter_width = right_col_min_x - left_col_max_x;
        if gutter_width >= GUTTER_WIDTH_THRESHOLD {
            return Some((left_col_max_x, right_col_min_x));
        }
    }

    None
}

pub fn sort_boxes_xycut(boxes: &[BoundingBox], direction: SortDirection) -> Vec<BoundingBox> {
    let indices = sort_by_xycut(boxes, direction, 1);
    indices.into_iter().map(|i| boxes[i].clone()).collect()
}

pub fn sort_by_xycut(boxes: &[BoundingBox], direction: SortDirection, min_gap: i32) -> Vec<usize> {
    if boxes.is_empty() {
        return Vec::new();
    }
    let 所有框: Vec<[i32; 4]> = boxes
        .iter()
        .map(|b| {
            [
                b.x_min() as i32,
                b.y_min() as i32,
                b.x_max() as i32,
                b.y_max() as i32,
            ]
        })
        .collect();
    let mut 结果集 = Vec::with_capacity(boxes.len());
    let 初始索引集: Vec<usize> = (0..boxes.len()).collect();
    let (主轴, 次轴) = match direction {
        SortDirection::Vertical => (1, 0),
        SortDirection::Horizontal => (0, 1),
    };
    递归执行_切割(&所有框, &初始索引集, &mut 结果集, 主轴, 次轴, min_gap);
    结果集
}

fn 计算投影(所有框: &[[i32; 4]], 当前索引集: &[usize], 轴: usize) -> (Vec<i32>, i32) {
    if 当前索引集.is_empty() {
        return (Vec::new(), 0);
    }
    let mut 最小值 = i32::MAX;
    let mut 最大值 = i32::MIN;
    for &idx in 当前索引集 {
        let 框 = &所有框[idx];
        最小值 = 最小值.min(框[轴]);
        最大值 = 最大值.max(框[轴 + 2]);
    }
    if 最小值 >= 最大值 {
        return (vec![1], 最小值);
    }
    let 跨度 = (最大值 - 最小值) as usize + 1;
    let mut 投影数组 = vec![0i32; 跨度];
    for &idx in 当前索引集 {
        let 框 = &所有框[idx];
        let 起点 = (框[轴] - 最小值).max(0) as usize;
        let 终点 = (框[轴 + 2] - 最小值).max(0) as usize;
        for i in 起点..终点.min(跨度) {
            投影数组[i] += 1;
        }
    }
    (投影数组, 最小值)
}

fn 寻找切割区间(投影数组: &[i32], 最小间距: i32) -> Option<(Vec<usize>, Vec<usize>)> {
    let 有效索引集: Vec<usize> = 投影数组
        .iter()
        .enumerate()
        .filter(|&(_, &值)| 值 > 0)
        .map(|(索引, _)| 索引)
        .collect();
    if 有效索引集.is_empty() {
        return None;
    }
    let mut 区间起点集 = Vec::new();
    let mut 区间终点集 = Vec::new();
    区间起点集.push(有效索引集[0]);
    for i in 1..有效索引集.len() {
        let 间距 = (有效索引集[i] - 有效索引集[i - 1]) as i32;
        if 间距 > 最小间距 {
            区间终点集.push(有效索引集[i - 1] + 1);
            区间起点集.push(有效索引集[i]);
        }
    }
    区间终点集.push(有效索引集.last().unwrap() + 1);
    if 区间起点集.len() > 1 {
        Some((区间起点集, 区间终点集))
    } else {
        None
    }
}

fn 递归执行_切割(
    所有框: &[[i32; 4]],
    当前索引集: &[usize],
    结果集: &mut Vec<usize>,
    主轴: usize,
    次轴: usize,
    最小间距: i32,
) {
    if 当前索引集.is_empty() {
        return;
    }
    if 当前索引集.len() == 1 {
        结果集.push(当前索引集[0]);
        return;
    }
    let mut 主轴排序索引 = 当前索引集.to_vec();
    主轴排序索引.sort_by_key(|&idx| 所有框[idx][主轴]);
    let (主轴投影, 主轴偏移) = 计算投影(所有框, &主轴排序索引, 主轴);
    if let Some((起点集, 终点集)) = 寻找切割区间(&主轴投影, 最小间距) {
        for (起点, 终点) in 起点集.into_iter().zip(终点集.into_iter()) {
            let 物理起点 = 起点 as i32 + 主轴偏移;
            let 物理终点 = 终点 as i32 + 主轴偏移;
            let mut 子块索引集 = Vec::new();
            for &idx in &主轴排序索引 {
                let 框 = &所有框[idx];
                if 框[主轴] >= 物理起点 && 框[主轴] < 物理终点 {
                    子块索引集.push(idx);
                }
            }
            递归执行_切割(所有框, &子块索引集, 结果集, 主轴, 次轴, 最小间距);
        }
        return;
    }
    let mut 次轴排序索引 = 当前索引集.to_vec();
    次轴排序索引.sort_by_key(|&idx| 所有框[idx][次轴]);
    let (次轴投影, 次轴偏移) = 计算投影(所有框, &次轴排序索引, 次轴);
    if let Some((起点集, 终点集)) = 寻找切割区间(&次轴投影, 最小间距) {
        for (起点, 终点) in 起点集.into_iter().zip(终点集.into_iter()) {
            let 物理起点 = 起点 as i32 + 次轴偏移;
            let 物理终点 = 终点 as i32 + 次轴偏移;
            let mut 子块索引集 = Vec::new();
            for &idx in &次轴排序索引 {
                let 框 = &所有框[idx];
                if 框[次轴] >= 物理起点 && 框[次轴] < 物理终点 {
                    子块索引集.push(idx);
                }
            }
            递归执行_切割(所有框, &子块索引集, 结果集, 主轴, 次轴, 最小间距);
        }
        return;
    }
    结果集.extend(次轴排序索引);
}

pub fn 获取权重(label: &排序标签, direction: SortDirection) -> [f32; 4] {
    match label {
        排序标签::文档标题 => {
            if matches!(direction, SortDirection::Horizontal) {
                [1.0, 0.1, 0.1, 1.0]
            } else {
                [0.2, 0.1, 1.0, 1.0]
            }
        }
        排序标签::段落标题 | 排序标签::视觉实体 | 排序标签::视觉标题 | 排序标签::跨栏元素 => {
            [1.0, 1.0, 0.1, 1.0]
        }
        _ => [1.0, 1.0, 1.0, 0.1],
    }
}

pub fn 获取加权边缘距离(b1: &BoundingBox, b2: &BoundingBox, weights: &[f32; 4]) -> f32 {
    let h_overlap = 计算一维投影重叠率(b1, b2, SortDirection::Horizontal);
    let v_overlap = 计算一维投影重叠率(b1, b2, SortDirection::Vertical);
    if h_overlap > 0.0 && v_overlap > 0.0 {
        return 0.0;
    }
    let mut min_x = 0.0;
    let mut min_y = 0.0;
    if h_overlap == 0.0 {
        let w = if b1.x_max() < b2.x_min() {
            weights[0]
        } else {
            weights[1]
        };
        min_x = (b1.x_min() - b2.x_max()).abs().min((b1.x_max() - b2.x_min()).abs()) * w;
    }
    if v_overlap == 0.0 {
        let w = if b1.y_max() < b2.y_min() {
            weights[2]
        } else {
            weights[3]
        };
        min_y = (b1.y_min() - b2.y_max()).abs().min((b1.y_max() - b2.y_max()).abs()) * w;
    }
    min_x + min_y
}

pub fn 计算一维投影重叠率(b1: &BoundingBox, b2: &BoundingBox, direction: SortDirection) -> f32 {
    let (min1, max1, min2, max2) = match direction {
        SortDirection::Horizontal => (b1.x_min(), b1.x_max(), b2.x_min(), b2.x_max()),
        SortDirection::Vertical => (b1.y_min(), b1.y_max(), b2.y_min(), b2.y_max()),
    };
    let intersection = (max1.min(max2) - min1.max(min2)).max(0.0);
    let union = max1.max(max2) - min1.min(min2);
    if union > 0.0 {
        intersection / union
    } else {
        0.0
    }
}

pub fn 计算二维重叠率(b1: &BoundingBox, b2: &BoundingBox) -> f32 {
    let inter = b1.intersection_area(b2);
    if inter <= 0.0 {
        return 0.0;
    }
    let union_area = b1.area() + b2.area() - inter;
    if union_area <= 0.0 {
        0.0
    } else {
        inter / union_area
    }
}

pub fn 加权距离插补(
    block: 排序数据块,
    sorted_blocks: &mut Vec<排序数据块>,
    region_direction: SortDirection,
) {
    if sorted_blocks.is_empty() {
        sorted_blocks.push(block);
        return;
    }
    let 边缘距离比较容差: f32 = 2.0;
    let 边缘权重基数: f32 = 10000.0;
    let (_x1, y1, x2, _y2) = (
        block.物理边界.x_min(),
        block.物理边界.y_min(),
        block.物理边界.x_max(),
        block.物理边界.y_max(),
    );
    let mut min_weighted_distance = f32::INFINITY;
    let mut min_up_edge_distance = f32::INFINITY;
    let mut nearest_index = 0;
    for (idx, sorted_block) in sorted_blocks.iter().enumerate() {
        let (x1_p, y1_p, x2_p, y2_p) = (
            sorted_block.物理边界.x_min(),
            sorted_block.物理边界.y_min(),
            sorted_block.物理边界.x_max(),
            sorted_block.物理边界.y_max(),
        );
        let weight = 获取权重(&block.标签, block.主方向);
        let edge_distance =
            (获取加权边缘距离(&block.物理边界, &sorted_block.物理边界, &weight) / 50.0).floor()
                * 50.0;
        let (mut up_dist, mut left_dist) = match region_direction {
            SortDirection::Horizontal => (y1_p, x1_p),
            SortDirection::Vertical => (-x2_p, y1_p),
        };
        let is_below = match region_direction {
            SortDirection::Horizontal => y2_p < y1,
            SortDirection::Vertical => x1_p > x2,
        };
        if is_below {
            up_dist = -up_dist;
            left_dist = -left_dist;
        }
        if (min_up_edge_distance - up_dist).abs() <= 边缘距离比较容差 {
            up_dist = min_up_edge_distance;
        }
        let weighted_dist = edge_distance * 边缘权重基数 + up_dist * 1.0 + left_dist * 2.0;
        min_up_edge_distance = min_up_edge_distance.min(up_dist);
        if weighted_dist < min_weighted_distance {
            min_weighted_distance = weighted_dist;
            nearest_index = if y1 > y1_p { idx + 1 } else { idx };
        }
    }
    sorted_blocks.insert(nearest_index.min(sorted_blocks.len()), block);
}

pub fn 曼哈顿距离插补(block: 排序数据块, sorted_blocks: &mut Vec<排序数据块>) {
    if sorted_blocks.is_empty() {
        sorted_blocks.push(block);
        return;
    }
    let mut min_distance = f32::INFINITY;
    let mut nearest_index = 0;
    for (idx, sorted_block) in sorted_blocks.iter().enumerate() {
        let distance = (block.物理边界.x_min() - sorted_block.物理边界.x_min()).abs()
            + (block.物理边界.y_min() - sorted_block.物理边界.y_min()).abs();
        if distance < min_distance {
            min_distance = distance;
            nearest_index = idx;
        }
    }
    sorted_blocks.insert((nearest_index + 1).min(sorted_blocks.len()), block);
}

pub fn sort_by_xycut_enhanced(
    elements: &[增强版面元素],
    page_width: f32,
    _page_height: f32,
) -> Vec<usize> {
    if elements.is_empty() {
        return Vec::new();
    }
    let blocks: Vec<排序数据块> = elements
        .iter()
        .enumerate()
        .map(|(i, e)| 排序数据块::new(e.物理边界.clone(), i, e.元素类型, e.估算行数))
        .collect();
    let (mut headers, mut footers, mut main_q) = (Vec::new(), Vec::new(), Vec::new());
    for block in blocks {
        match block.标签 {
            排序标签::页眉 => headers.push(block),
            排序标签::页脚 => footers.push(block),
            _ => main_q.push(block),
        }
    }
    headers.sort_by(|a, b| {
        a.物理边界
            .y_min()
            .partial_cmp(&b.物理边界.y_min())
            .unwrap_or(Ordering::Equal)
    });
    footers.sort_by(|a, b| {
        a.物理边界
            .y_min()
            .partial_cmp(&b.物理边界.y_min())
            .unwrap_or(Ordering::Equal)
    });

    let sorted_main = sort_main_blocks(main_q, page_width);
    let mut result = Vec::with_capacity(elements.len());
    result.extend(headers.into_iter().map(|b| b.原始索引));
    result.extend(sorted_main.into_iter().map(|b| b.原始索引));
    result.extend(footers.into_iter().map(|b| b.原始索引));
    result
}

fn sort_main_blocks(mut blocks: Vec<排序数据块>, page_width: f32) -> Vec<排序数据块> {
    if blocks.is_empty() {
        return blocks;
    }
    检测跨栏元素(&mut blocks, page_width);
    let (mut xy, mut title, mut insert, mut unordered) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for block in blocks {
        match block.标签 {
            排序标签::跨栏元素 | 排序标签::跨栏引用 => insert.push(block),
            排序标签::视觉实体 | 排序标签::视觉标题 => {
                if block.宽度() >= page_width * 0.60 {
                    insert.push(block);
                } else {
                    xy.push(block);
                }
            }
            排序标签::文档标题 => title.push(block),
            排序标签::无序占位 => unordered.push(block),
            _ => xy.push(block),
        }
    }
    let mut sorted_blocks = if !xy.is_empty() {
        if let Some((gutter_left, gutter_right)) = detect_central_gutter(&xy, page_width) {
            let mut left_col = Vec::new();
            let mut right_col = Vec::new();
            let mut span_blocks = Vec::new();

            for b in xy {
                let x1 = b.物理边界.x_min();
                let x2 = b.物理边界.x_max();

                if (x2 - x1) >= page_width * 0.75 {
                    span_blocks.push(b);
                } else if x2 <= gutter_right {
                    left_col.push(b);
                } else if x1 >= gutter_left {
                    right_col.push(b);
                } else {
                    left_col.push(b);
                }
            }

            let mut sorted = Vec::new();
            if !span_blocks.is_empty() {
                sorted.extend(direction_aware_xycut_sort(&mut span_blocks));
            }
            if !left_col.is_empty() {
                sorted.extend(direction_aware_xycut_sort(&mut left_col));
            }
            if !right_col.is_empty() {
                sorted.extend(direction_aware_xycut_sort(&mut right_col));
            }
            sorted
        } else {
            direction_aware_xycut_sort(&mut xy)
        }
    } else {
        Vec::new()
    };

    title.sort_by(|a, b| {
        a.物理边界
            .y_min()
            .partial_cmp(&b.物理边界.y_min())
            .unwrap_or(Ordering::Equal)
    });

    for block in title.into_iter().rev() {
        sorted_blocks.insert(0, block);
    }

    insert.sort_by(|a, b| {
        a.物理边界
            .y_min()
            .partial_cmp(&b.物理边界.y_min())
            .unwrap_or(Ordering::Equal)
    });
    for block in insert {
        加权距离插补(block, &mut sorted_blocks, SortDirection::Horizontal);
    }
    unordered.sort_by(|a, b| {
        a.物理边界
            .y_min()
            .partial_cmp(&b.物理边界.y_min())
            .unwrap_or(Ordering::Equal)
    });
    for block in unordered {
        曼哈顿距离插补(block, &mut sorted_blocks);
    }
    关联子数据块(&mut sorted_blocks);
    sorted_blocks
}

fn direction_aware_xycut_sort(blocks: &mut [排序数据块]) -> Vec<排序数据块> {
    收缩重叠边界(blocks, SortDirection::Vertical);
    let shrunk_bboxes: Vec<BoundingBox> = blocks.iter().map(|b| b.物理边界.clone()).collect();
    let sorted_indices = sort_by_xycut(&shrunk_bboxes, SortDirection::Vertical, 1);
    sorted_indices.into_iter().map(|i| blocks[i].clone()).collect()
}

pub fn 检测跨栏元素(blocks: &mut [排序数据块], page_width: f32) {
    if blocks.len() < 2 {
        return;
    }

    blocks.sort_by(|a, b| {
        a.物理边界
            .x_min()
            .partial_cmp(&b.物理边界.x_min())
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                b.宽度()
                    .partial_cmp(&a.宽度())
                    .unwrap_or(Ordering::Equal)
            })
    });

    let 屏蔽标签 = [
        排序标签::文档标题,
        排序标签::跨栏元素,
        排序标签::跨栏引用,
    ];

    let n = blocks.len();
    let mut block_data: Vec<(BoundingBox, 排序标签, f32, f32)> = blocks
        .iter()
        .map(|b| (b.物理边界.clone(), b.标签, b.面积(), b.长边长度()))
        .collect();

    let 行高集合: Vec<f32> = blocks.iter().map(|b| b.行高).collect();

    let mut h_proj = vec![vec![0.0f32; n]; n];
    for i in 0..n {
        for j in 0..n {
            h_proj[i][j] =
                计算一维投影重叠率(&block_data[i].0, &block_data[j].0, SortDirection::Horizontal);
        }
    }

    let h_neighbors: Vec<Vec<usize>> = (0..n)
        .map(|i| (0..n).filter(|&j| j != i && h_proj[i][j] > 0.0).collect())
        .collect();

    for block_idx in 0..n {
        if 屏蔽标签.contains(&block_data[block_idx].1) {
            continue;
        }

        let mut 确认跨栏 = false;
        if blocks[block_idx].宽度() < page_width * 0.60 {
            continue;
        }

        for &ref_idx in &h_neighbors[block_idx] {
            if 屏蔽标签.contains(&block_data[ref_idx].1) {
                continue;
            }
            if blocks[ref_idx].标签 == 排序标签::跨栏元素 {
                continue;
            }
            if blocks[block_idx].标签 == 排序标签::跨栏元素 {
                break;
            }

            let bbox_overlap = 计算二维重叠率(&block_data[block_idx].0, &block_data[ref_idx].0);
            if bbox_overlap > 0.0 {
                if block_data[ref_idx].1 == 排序标签::视觉实体 {
                    blocks[ref_idx].标签 = 排序标签::跨栏元素;
                    block_data[ref_idx].1 = 排序标签::跨栏元素;
                    continue;
                }
                if bbox_overlap > 0.1 && block_data[block_idx].2 < block_data[ref_idx].2 {
                    确认跨栏 = true;
                    break;
                }
            }

            for &second_ref_idx in &h_neighbors[block_idx] {
                if second_ref_idx == ref_idx || 屏蔽标签.contains(&block_data[second_ref_idx].1) {
                    continue;
                }
                if blocks[second_ref_idx].标签 == 排序标签::跨栏元素 {
                    continue;
                }

                let bbox_overlap2 =
                    计算二维重叠率(&block_data[block_idx].0, &block_data[second_ref_idx].0);
                if bbox_overlap2 > 0.1 {
                    if block_data[second_ref_idx].1 == 排序标签::视觉实体 {
                        blocks[second_ref_idx].标签 = 排序标签::跨栏元素;
                        block_data[second_ref_idx].1 = 排序标签::跨栏元素;
                        continue;
                    }
                    if block_data[block_idx].1 == 排序标签::视觉实体
                        || block_data[block_idx].2 < block_data[second_ref_idx].2
                    {
                        确认跨栏 = true;
                        break;
                    }
                }

                let ref_match_proj = h_proj[ref_idx][second_ref_idx];
                let secondary_ref_match = 计算一维投影重叠率(
                    &block_data[ref_idx].0,
                    &block_data[second_ref_idx].0,
                    SortDirection::Vertical,
                );

                if ref_match_proj == 0.0 && secondary_ref_match > 0.0 {
                    if block_data[block_idx].1 == 排序标签::视觉实体 {
                        确认跨栏 = true;
                        break;
                    }
                    if block_data[ref_idx].1 == 排序标签::普通文本
                        && block_data[second_ref_idx].1 == 排序标签::普通文本
                        && block_data[ref_idx].3 > 行高集合[ref_idx] * 跨栏参考文本块行数阈值
                        && block_data[second_ref_idx].3
                            > 行高集合[second_ref_idx] * 跨栏参考文本块行数阈值
                    {
                        确认跨栏 = true;
                        break;
                    }
                }
            }

            if 确认跨栏 {
                break;
            }
        }

        if 确认跨栏 {
            if block_data[block_idx].1 == 排序标签::文献引用 {
                blocks[block_idx].标签 = 排序标签::跨栏引用;
            } else {
                blocks[block_idx].标签 = 排序标签::跨栏元素;
            }
            block_data[block_idx].1 = blocks[block_idx].标签;
        }
    }
}

pub fn 收缩重叠边界(blocks: &mut [排序数据块], direction: SortDirection) {
    if blocks.len() < 2 {
        return;
    }
    match direction {
        SortDirection::Vertical => {
            blocks.sort_by(|a, b| {
                a.物理边界
                    .y_max()
                    .partial_cmp(&b.物理边界.y_max())
                    .unwrap_or(Ordering::Equal)
            });
        }
        SortDirection::Horizontal => {
            blocks.sort_by(|a, b| {
                a.物理边界
                    .x_max()
                    .partial_cmp(&b.物理边界.x_max())
                    .unwrap_or(Ordering::Equal)
            });
        }
    }

    for i in 0..blocks.len() - 1 {
        let perp_direction = match direction {
            SortDirection::Vertical => SortDirection::Horizontal,
            SortDirection::Horizontal => SortDirection::Vertical,
        };

        let cut_iou = 计算一维投影重叠率(&blocks[i].物理边界, &blocks[i + 1].物理边界, direction);
        let match_iou = 计算一维投影重叠率(&blocks[i].物理边界, &blocks[i + 1].物理边界, perp_direction);

        match direction {
            SortDirection::Vertical => {
                let y2 = blocks[i].物理边界.y_max();
                let y1_prime = blocks[i + 1].物理边界.y_min();
                if (match_iou > 0.0 && cut_iou > 0.0 && cut_iou < 0.1)
                    || y2 == y1_prime
                    || (y2 - y1_prime).abs() <= 3.0
                {
                    let overlap_y_min =
                        blocks[i].物理边界.y_min().max(blocks[i + 1].物理边界.y_min());
                    let overlap_y_max =
                        blocks[i].物理边界.y_max().min(blocks[i + 1].物理边界.y_max());
                    let split_y = ((overlap_y_min + overlap_y_max) / 2.0).floor();

                    if blocks[i].物理边界.y_min() < blocks[i + 1].物理边界.y_min() {
                        blocks[i].物理边界 = BoundingBox::from_coords(
                            blocks[i].物理边界.x_min(),
                            blocks[i].物理边界.y_min(),
                            blocks[i].物理边界.x_max(),
                            split_y - 1.0,
                        );
                        blocks[i + 1].物理边界 = BoundingBox::from_coords(
                            blocks[i + 1].物理边界.x_min(),
                            split_y + 1.0,
                            blocks[i + 1].物理边界.x_max(),
                            blocks[i + 1].物理边界.y_max(),
                        );
                    } else {
                        blocks[i].物理边界 = BoundingBox::from_coords(
                            blocks[i].物理边界.x_min(),
                            split_y - 1.0,
                            blocks[i].物理边界.x_max(),
                            blocks[i].物理边界.y_max(),
                        );
                        blocks[i + 1].物理边界 = BoundingBox::from_coords(
                            blocks[i + 1].物理边界.x_min(),
                            blocks[i + 1].物理边界.y_min(),
                            blocks[i + 1].物理边界.x_max(),
                            split_y + 1.0,
                        );
                    }
                }
            }
            SortDirection::Horizontal => {
                let x2 = blocks[i].物理边界.x_max();
                let x1_prime = blocks[i + 1].物理边界.x_min();
                if (match_iou > 0.0 && cut_iou > 0.0 && cut_iou < 0.1)
                    || x2 == x1_prime
                    || (x2 - x1_prime).abs() <= 3.0
                {
                    let overlap_x_min =
                        blocks[i].物理边界.x_min().max(blocks[i + 1].物理边界.x_min());
                    let overlap_x_max =
                        blocks[i].物理边界.x_max().min(blocks[i + 1].物理边界.x_max());
                    let split_x = ((overlap_x_min + overlap_x_max) / 2.0).floor();

                    if blocks[i].物理边界.x_min() < blocks[i + 1].物理边界.x_min() {
                        blocks[i].物理边界 = BoundingBox::from_coords(
                            blocks[i].物理边界.x_min(),
                            blocks[i].物理边界.y_min(),
                            split_x - 1.0,
                            blocks[i].物理边界.y_max(),
                        );
                        blocks[i + 1].物理边界 = BoundingBox::from_coords(
                            split_x + 1.0,
                            blocks[i + 1].物理边界.y_min(),
                            blocks[i + 1].物理边界.x_max(),
                            blocks[i + 1].物理边界.y_max(),
                        );
                    } else {
                        blocks[i].物理边界 = BoundingBox::from_coords(
                            split_x - 1.0,
                            blocks[i].物理边界.y_min(),
                            blocks[i].物理边界.x_max(),
                            blocks[i].物理边界.y_max(),
                        );
                        blocks[i + 1].物理边界 = BoundingBox::from_coords(
                            blocks[i + 1].物理边界.x_min(),
                            blocks[i + 1].物理边界.y_min(),
                            split_x + 1.0,
                            blocks[i + 1].物理边界.y_max(),
                        );
                    }
                }
            }
        }
    }
}

pub fn 关联子数据块(sorted_blocks: &mut Vec<排序数据块>) {
    if sorted_blocks.len() < 2 {
        return;
    }
    let mut moves: Vec<(usize, usize)> = Vec::new();
    for (i, block) in sorted_blocks.iter().enumerate() {
        if block.标签 != 排序标签::视觉标题 {
            continue;
        }
        let mut best_vision_idx = None;
        let mut best_distance = f32::INFINITY;

        for (j, other) in sorted_blocks.iter().enumerate() {
            if other.标签 != 排序标签::视觉实体 {
                continue;
            }
            let dist = 获取加权边缘距离(&block.物理边界, &other.物理边界, &[1.0, 1.0, 1.0, 1.0]);
            if dist < best_distance {
                best_distance = dist;
                best_vision_idx = Some(j);
            }
        }

        if let Some(vision_idx) = best_vision_idx {
            let threshold = sorted_blocks[vision_idx].行高 * 3.0;
            if best_distance < threshold {
                if block.物理边界.y_min() < sorted_blocks[vision_idx].物理边界.y_min() {
                    moves.push((i, vision_idx));
                } else {
                    moves.push((i, vision_idx + 1));
                }
            }
        }
    }

    for (from_idx, target_idx) in moves.into_iter().rev() {
        if from_idx == target_idx || from_idx + 1 == target_idx {
            continue;
        }
        let block = sorted_blocks.remove(from_idx);
        let adjusted_target = if from_idx < target_idx {
            target_idx - 1
        } else {
            target_idx
        };
        let insert_pos = adjusted_target.min(sorted_blocks.len());
        sorted_blocks.insert(insert_pos, block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BoundingBox, SortDirection, 增强版面元素, 排序标签};

    #[test]
    fn test_rxyc_gutter_detection_and_doctitle_escalation() {
        let title_elem = 增强版面元素 {
            物理边界: BoundingBox::from_coords(100.0, 150.0, 900.0, 200.0),
            元素类型: 排序标签::文档标题,
            估算行数: Some(1),
        };

        let left_col_elem = 增强版面元素 {
            物理边界: BoundingBox::from_coords(100.0, 250.0, 450.0, 400.0),
            元素类型: 排序标签::普通文本,
            估算行数: Some(10),
        };

        let right_col_elem = 增强版面元素 {
            物理边界: BoundingBox::from_coords(550.0, 220.0, 900.0, 400.0),
            元素类型: 排序标签::普通文本,
            估算行数: Some(10),
        };

        let elements = vec![right_col_elem, left_col_elem, title_elem];

        let sorted = sort_by_xycut_enhanced(&elements, 1000.0, 1400.0);

        assert_eq!(sorted[0], 2, "DocTitle 提权机制生效：大标题永远位于 Page 1 第一位！");
        assert_eq!(sorted[1], 1, "RXYC++ Gutter 切割生效：双栏排版中左栏段落优先于右栏段落！");
        assert_eq!(sorted[2], 0, "RXYC++ Gutter 切割生效：右栏段落排列在左栏之后！");
    }

    #[test]
    fn test_xycut_basic_sort() {
        let b1 = BoundingBox::from_coords(0.0, 100.0, 100.0, 150.0);
        let b2 = BoundingBox::from_coords(0.0, 0.0, 100.0, 50.0);
        let b3 = BoundingBox::from_coords(0.0, 50.0, 100.0, 90.0);
        let boxes = vec![b1, b2, b3];

        let indices = sort_by_xycut(&boxes, SortDirection::Vertical, 1);
        assert_eq!(indices, vec![1, 2, 0], "XY-Cut 纵向物理切割顺序与阅读序不符！");
    }
}
