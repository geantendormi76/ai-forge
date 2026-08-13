use crate::types::{DocElement, LayoutType};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderLabel {
    Header,
    Footer,
    DocTitle,
    ParagraphTitle,
    Vision,
    NormalText,
    Unordered,
}

impl OrderLabel {
    fn from_layout_type(lt: LayoutType) -> Self {
        match lt {
            LayoutType::Header => OrderLabel::Header,
            LayoutType::Footer => OrderLabel::Footer,
            LayoutType::DocTitle => OrderLabel::DocTitle,
            LayoutType::ParagraphTitle => OrderLabel::ParagraphTitle,
            LayoutType::Image | LayoutType::Table => OrderLabel::Vision,
            LayoutType::Seal | LayoutType::AsideText => OrderLabel::Unordered,
            _ => OrderLabel::NormalText,
        }
    }
}

#[derive(Debug, Clone)]
struct SortableBlock {
    bbox: [f32; 4],
    original_index: usize,
    order_label: OrderLabel,
}

impl SortableBlock {
    fn new(doc_elem: &DocElement, index: usize) -> Self {
        Self {
            bbox: doc_elem.bbox,
            original_index: index,
            order_label: OrderLabel::from_layout_type(doc_elem.layout_type),
        }
    }

    fn x_min(&self) -> f32 { self.bbox[0] }
    fn y_min(&self) -> f32 { self.bbox[1] }
    fn x_max(&self) -> f32 { self.bbox[2] }
    fn width(&self) -> f32 { (self.bbox[2] - self.bbox[0]).max(0.0) }
}

pub fn sort_doc_elements(elements: &mut [DocElement], page_width: f32, page_height: f32) {
    if elements.len() <= 1 {
        for (i, elem) in elements.iter_mut().enumerate() {
            elem.order_index = (i + 1) as u32;
        }
        return;
    }

    let blocks: Vec<SortableBlock> = elements
        .iter()
        .enumerate()
        .map(|(i, elem)| SortableBlock::new(elem, i))
        .collect();

    let mut headers = Vec::new();
    let mut footers = Vec::new();
    let mut main_blocks = Vec::new();

    for block in blocks {
        match block.order_label {
            OrderLabel::Header => headers.push(block),
            OrderLabel::Footer => footers.push(block),
            _ => main_blocks.push(block),
        }
    }

    headers.sort_by(|a, b| a.y_min().partial_cmp(&b.y_min()).unwrap_or(Ordering::Equal));
    footers.sort_by(|a, b| a.y_min().partial_cmp(&b.y_min()).unwrap_or(Ordering::Equal));

    let sorted_main = sort_main_body(main_blocks, page_width, page_height);

    let mut final_blocks = Vec::with_capacity(elements.len());
    final_blocks.extend(headers);
    final_blocks.extend(sorted_main);
    final_blocks.extend(footers);

    let mut temp_elements = Vec::with_capacity(elements.len());
    for (order_idx, block) in final_blocks.into_iter().enumerate() {
        let mut elem = elements[block.original_index].clone();
        elem.order_index = (order_idx + 1) as u32;
        temp_elements.push(elem);
    }

    elements.clone_from_slice(&temp_elements);
}

fn sort_main_body(blocks: Vec<SortableBlock>, page_width: f32, _page_height: f32) -> Vec<SortableBlock> {
    if blocks.len() <= 1 {
        return blocks;
    }

    let mid_x = page_width * 0.5;
    let mut left_col = Vec::new();
    let mut right_col = Vec::new();
    let mut full_width = Vec::new();

    for block in blocks {
        let is_cross = block.width() > page_width * 0.65;
        if is_cross || block.order_label == OrderLabel::DocTitle {
            full_width.push(block);
        } else if block.x_max() <= mid_x + page_width * 0.1 {
            left_col.push(block);
        } else if block.x_min() >= mid_x - page_width * 0.1 {
            right_col.push(block);
        } else {
            full_width.push(block);
        }
    }

    left_col.sort_by(|a, b| a.y_min().partial_cmp(&b.y_min()).unwrap_or(Ordering::Equal));
    right_col.sort_by(|a, b| a.y_min().partial_cmp(&b.y_min()).unwrap_or(Ordering::Equal));
    full_width.sort_by(|a, b| a.y_min().partial_cmp(&b.y_min()).unwrap_or(Ordering::Equal));

    let mut result = Vec::with_capacity(left_col.len() + right_col.len() + full_width.len());
    let mut left_idx = 0;
    let mut right_idx = 0;

    for fw in full_width {
        let fw_y = fw.y_min();
        while left_idx < left_col.len() && left_col[left_idx].y_min() < fw_y {
            result.push(left_col[left_idx].clone());
            left_idx += 1;
        }
        while right_idx < right_col.len() && right_col[right_idx].y_min() < fw_y {
            result.push(right_col[right_idx].clone());
            right_idx += 1;
        }
        result.push(fw);
    }

    while left_idx < left_col.len() {
        result.push(left_col[left_idx].clone());
        left_idx += 1;
    }
    while right_idx < right_col.len() {
        result.push(right_col[right_idx].clone());
        right_idx += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting_simple_column() {
        let mut elements = vec![
            DocElement { bbox: [10.0, 100.0, 200.0, 150.0], layout_type: LayoutType::Text, order_index: 0, content: "Bottom".into(), score: 0.9 },
            DocElement { bbox: [10.0, 10.0, 200.0, 50.0], layout_type: LayoutType::DocTitle, order_index: 0, content: "Title".into(), score: 0.95 },
        ];

        sort_doc_elements(&mut elements, 400.0, 600.0);

        assert_eq!(elements[0].content, "Title");
        assert_eq!(elements[0].order_index, 1);
        assert_eq!(elements[1].content, "Bottom");
        assert_eq!(elements[1].order_index, 2);
    }
}
