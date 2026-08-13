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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ioa_calculation() {
        let inner = [10.0, 10.0, 50.0, 50.0];
        let container = [0.0, 0.0, 100.0, 100.0];
        assert_eq!(calculate_ioa(&inner, &container), 1.0);

        let outside = [150.0, 150.0, 200.0, 200.0];
        assert_eq!(calculate_ioa(&outside, &container), 0.0);
    }
}
