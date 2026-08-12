use crate::models::text_region::{BoundingBox, Point};
use crate::processors::geometry::{MinAreaRect, ScanlineBuffer};
use crate::processors::types::{BoxType, ImageScaleInfo, ScoreMode};
use clipper2_rust::{
    clipper::inflate_paths_d,
    core::{area, PathD, PathsD, PointD},
    offset::{EndType, JoinType},
};
use image::GrayImage;
use imageproc::contours::{find_contours, Contour};
use ndarray::Axis;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct DBPostProcessConfig {
    pub thresh: f32,
    pub box_thresh: f32,
    pub unclip_ratio: f32,
}

impl DBPostProcessConfig {
    pub fn new(thresh: f32, box_thresh: f32, unclip_ratio: f32) -> Self {
        Self {
            thresh,
            box_thresh,
            unclip_ratio,
        }
    }
}

#[derive(Debug)]
pub struct DBPostProcess {
    pub thresh: f32,
    pub box_thresh: f32,
    pub max_candidates: usize,
    pub unclip_ratio: f32,
    pub min_size: f32,
    pub score_mode: ScoreMode,
    pub box_type: BoxType,
    pub use_dilation: bool,
}

impl DBPostProcess {
    pub fn new(
        thresh: Option<f32>,
        box_thresh: Option<f32>,
        max_candidates: Option<usize>,
        unclip_ratio: Option<f32>,
        use_dilation: Option<bool>,
        score_mode: Option<ScoreMode>,
        box_type: Option<BoxType>,
    ) -> Self {
        Self {
            thresh: thresh.unwrap_or(0.3),
            box_thresh: box_thresh.unwrap_or(0.6),
            max_candidates: max_candidates.unwrap_or(1000),
            unclip_ratio: unclip_ratio.unwrap_or(1.5),
            min_size: 3.0,
            score_mode: score_mode.unwrap_or(ScoreMode::Fast),
            box_type: box_type.unwrap_or(BoxType::Quad),
            use_dilation: use_dilation.unwrap_or(false),
        }
    }

    pub fn apply(
        &self,
        preds: &ndarray::Array4<f32>,
        img_shapes: Vec<ImageScaleInfo>,
        config: Option<&DBPostProcessConfig>,
    ) -> (Vec<Vec<BoundingBox>>, Vec<Vec<f32>>) {
        let thresh = config.map(|c| c.thresh).unwrap_or(self.thresh);
        let box_thresh = config.map(|c| c.box_thresh).unwrap_or(self.box_thresh);
        let unclip_ratio = config.map(|c| c.unclip_ratio).unwrap_or(self.unclip_ratio);

        let mut all_boxes = Vec::with_capacity(img_shapes.len());
        let mut all_scores = Vec::with_capacity(img_shapes.len());

        for (batch_idx, shape_batch) in img_shapes.iter().enumerate() {
            let pred_slice = preds.index_axis(Axis(0), batch_idx);
            let pred_channel = pred_slice.index_axis(Axis(0), 0);

            let (boxes, scores) =
                self.process(&pred_channel, shape_batch, thresh, box_thresh, unclip_ratio);
            all_boxes.push(boxes);
            all_scores.push(scores);
        }

        (all_boxes, all_scores)
    }

    fn process(
        &self,
        pred: &ndarray::ArrayView2<f32>,
        img_shape: &ImageScaleInfo,
        thresh: f32,
        box_thresh: f32,
        unclip_ratio: f32,
    ) -> (Vec<BoundingBox>, Vec<f32>) {
        let src_h = img_shape.src_h as u32;
        let src_w = img_shape.src_w as u32;

        let mask_img = self.threshold_to_mask(pred, thresh);
        let mask_img = if self.use_dilation {
            imageproc::morphology::dilate(&mask_img, imageproc::distance_transform::Norm::LInf, 1)
        } else {
            mask_img
        };

        match self.box_type {
            BoxType::Poly => {
                self.polygons_from_bitmap(pred, &mask_img, src_w, src_h, box_thresh, unclip_ratio)
            }
            BoxType::Quad => {
                self.boxes_from_bitmap(pred, &mask_img, src_w, src_h, box_thresh, unclip_ratio)
            }
        }
    }

    fn threshold_to_mask(&self, pred: &ndarray::ArrayView2<f32>, thresh: f32) -> GrayImage {
        let height = pred.shape()[0] as u32;
        let width = pred.shape()[1] as u32;
        let mut mask_img = GrayImage::new(width, height);
        let buf: &mut [u8] = mask_img.as_mut();
        let row_bytes = width as usize;

        let fill_row = |y: usize, row: &mut [u8]| {
            let pred_row = pred.row(y);
            if let Some(slice) = pred_row.as_slice() {
                for (dst, &val) in row.iter_mut().zip(slice) {
                    *dst = if val > thresh { 255 } else { 0 };
                }
            } else {
                for (x, dst) in row.iter_mut().enumerate() {
                    *dst = if pred_row[x] > thresh { 255 } else { 0 };
                }
            }
        };

        if height >= 64 {
            buf.par_chunks_mut(row_bytes)
                .enumerate()
                .for_each(|(y, row)| fill_row(y, row));
        } else {
            buf.chunks_mut(row_bytes)
                .enumerate()
                .for_each(|(y, row)| fill_row(y, row));
        }

        mask_img
    }

    pub fn polygons_from_bitmap(
        &self,
        pred: &ndarray::ArrayView2<f32>,
        bitmap: &GrayImage,
        dest_width: u32,
        dest_height: u32,
        box_thresh: f32,
        unclip_ratio: f32,
    ) -> (Vec<BoundingBox>, Vec<f32>) {
        let height = bitmap.height() as usize;
        let width = bitmap.width() as usize;
        let width_scale = dest_width as f32 / width as f32;
        let height_scale = dest_height as f32 / height as f32;
        let dest_w_f = dest_width as f32;
        let dest_h_f = dest_height as f32;

        let contours = find_contours::<u32>(bitmap);
        let max_candidates = self.max_candidates;
        let mut boxes: Vec<BoundingBox> = Vec::with_capacity(contours.len().min(max_candidates));
        let mut scores: Vec<f32> = Vec::with_capacity(boxes.capacity());

        for contour in contours.into_iter().take(max_candidates) {
            if contour.points.len() < 4 {
                continue;
            }

            let bbox = BoundingBox::from_contour(&contour);
            let epsilon = 0.002 * bbox.perimeter();
            let approx = bbox.approx_poly_dp(epsilon);

            if approx.points.len() < 4 {
                continue;
            }

            let score = self.box_score_fast(pred, &approx);
            if score < box_thresh {
                continue;
            }

            let unclipped = self.unclip(&approx, unclip_ratio);
            if unclipped.points.is_empty() {
                continue;
            }

            let Some((_, sside)) = self.get_mini_boxes_from_points(&unclipped.points) else {
                continue;
            };
            if sside < self.min_size + 2.0 {
                continue;
            }

            let n = unclipped.points.len();
            let mut scaled_points: Vec<Point> = Vec::with_capacity(n);
            for point in &unclipped.points {
                let x = (point.x * width_scale).round().clamp(0.0, dest_w_f);
                let y = (point.y * height_scale).round().clamp(0.0, dest_h_f);
                scaled_points.push(Point::new(x, y));
            }

            boxes.push(BoundingBox::new(scaled_points));
            scores.push(score);
        }

        (boxes, scores)
    }

    pub fn boxes_from_bitmap(
        &self,
        pred: &ndarray::ArrayView2<f32>,
        bitmap: &GrayImage,
        dest_width: u32,
        dest_height: u32,
        box_thresh: f32,
        unclip_ratio: f32,
    ) -> (Vec<BoundingBox>, Vec<f32>) {
        let height = bitmap.height() as usize;
        let width = bitmap.width() as usize;
        let width_scale = dest_width as f32 / width as f32;
        let height_scale = dest_height as f32 / height as f32;
        let dest_w_f = dest_width as f32;
        let dest_h_f = dest_height as f32;

        let contours = find_contours::<u32>(bitmap);
        let max_candidates = self.max_candidates;
        let mut boxes: Vec<BoundingBox> = Vec::with_capacity(contours.len().min(max_candidates));
        let mut scores: Vec<f32> = Vec::with_capacity(boxes.capacity());

        for contour in contours.into_iter().take(max_candidates) {
            let Some((mini_box_points, min_side)) = self.get_mini_boxes_from_contour(&contour)
            else {
                continue;
            };
            if min_side < self.min_size {
                continue;
            }
            let mini_box = BoundingBox::new(mini_box_points);

            let score = match self.score_mode {
                ScoreMode::Fast => self.box_score_fast(pred, &mini_box),
                ScoreMode::Slow => self.box_score_slow(pred, &contour),
            };

            if score < box_thresh {
                continue;
            }

            let unclipped = self.unclip(&mini_box, unclip_ratio);
            if unclipped.points.is_empty() {
                continue;
            }

            let Some((box_points, sside)) = self.get_mini_boxes_from_points(&unclipped.points)
            else {
                continue;
            };
            if sside < self.min_size + 2.0 {
                continue;
            }

            let n = box_points.len();
            let mut scaled_points: Vec<Point> = Vec::with_capacity(n);
            for point in &box_points {
                let x = (point.x * width_scale).round().clamp(0.0, dest_w_f);
                let y = (point.y * height_scale).round().clamp(0.0, dest_h_f);
                scaled_points.push(Point::new(x, y));
            }

            boxes.push(BoundingBox::new(scaled_points));
            scores.push(score);
        }

        (boxes, scores)
    }

    fn get_mini_boxes_from_contour(&self, contour: &Contour<u32>) -> Option<(Vec<Point>, f32)> {
        let points = contour
            .points
            .iter()
            .map(|p| Point::new(p.x as f32, p.y as f32))
            .collect::<Vec<_>>();
        let simplified = Self::simplify_chain_points(&points);
        if simplified.len() >= 3 {
            self.get_mini_boxes_from_points(&simplified)
        } else {
            self.get_mini_boxes_from_points(&points)
        }
    }

    fn get_mini_boxes_from_points(&self, points: &[Point]) -> Option<(Vec<Point>, f32)> {
        if points.len() < 3 {
            return None;
        }

        let min_rect = BoundingBox::get_min_area_rect_from_points(points);
        let min_side = min_rect.min_side();
        if !min_side.is_finite() || min_side <= 0.0 {
            return None;
        }

        let raw_points = Self::box_points_without_reorder(&min_rect);
        if raw_points.len() != 4 {
            return None;
        }

        Some((Self::paddlex_order_mini_box_points(raw_points), min_side))
    }

    fn box_points_without_reorder(rect: &MinAreaRect) -> Vec<Point> {
        let cos_a = (rect.angle * PI / 180.0).cos();
        let sin_a = (rect.angle * PI / 180.0).sin();
        let w_2 = rect.width / 2.0;
        let h_2 = rect.height / 2.0;
        let corners = [(-w_2, -h_2), (w_2, -h_2), (w_2, h_2), (-w_2, h_2)];

        corners
            .iter()
            .map(|(x, y)| {
                let rotated_x = x * cos_a - y * sin_a + rect.center.x;
                let rotated_y = x * sin_a + y * cos_a + rect.center.y;
                Point::new(rotated_x, rotated_y)
            })
            .collect()
    }

    fn simplify_chain_points(points: &[Point]) -> Vec<Point> {
        if points.len() <= 2 {
            return points.to_vec();
        }

        let mut simplified = Vec::with_capacity(points.len());
        let n = points.len();

        for i in 0..n {
            let prev = points[(i + n - 1) % n];
            let curr = points[i];
            let next = points[(i + 1) % n];

            let dir_prev = (
                Self::sign_step(curr.x - prev.x),
                Self::sign_step(curr.y - prev.y),
            );
            let dir_next = (
                Self::sign_step(next.x - curr.x),
                Self::sign_step(next.y - curr.y),
            );

            if dir_prev != dir_next {
                simplified.push(curr);
            }
        }

        if simplified.len() < 3 {
            points.to_vec()
        } else {
            simplified
        }
    }

    fn sign_step(v: f32) -> i8 {
        if v > 0.0 {
            1
        } else if v < 0.0 {
            -1
        } else {
            0
        }
    }

    fn paddlex_order_mini_box_points(mut points: Vec<Point>) -> Vec<Point> {
        if points.len() != 4 {
            return points;
        }

        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal));

        let (index_1, index_4) = if points[1].y > points[0].y {
            (0usize, 1usize)
        } else {
            (1usize, 0usize)
        };
        let (index_2, index_3) = if points[3].y > points[2].y {
            (2usize, 3usize)
        } else {
            (3usize, 2usize)
        };

        vec![
            points[index_1],
            points[index_2],
            points[index_3],
            points[index_4],
        ]
    }

    fn unclip(&self, bbox: &BoundingBox, unclip_ratio: f32) -> BoundingBox {
        if bbox.points.len() < 3 {
            return bbox.clone();
        }

        let clipper_path: PathD = bbox
            .points
            .iter()
            .map(|point| PointD {
                x: point.x as f64,
                y: point.y as f64,
            })
            .collect();

        if clipper_path.len() < 3 {
            return BoundingBox::new(Vec::new());
        }

        let polygon_area = area(&clipper_path).abs();
        if polygon_area <= f64::EPSILON {
            return BoundingBox::new(Vec::new());
        }

        let mut perimeter = 0.0f64;
        let n = clipper_path.len();
        if n >= 2 {
            let mut p1 = &clipper_path[0];
            for p2 in &clipper_path[1..] {
                perimeter += (p2.x - p1.x).hypot(p2.y - p1.y);
                p1 = p2;
            }
            let first = &clipper_path[0];
            perimeter += (first.x - p1.x).hypot(first.y - p1.y);
        }

        if perimeter <= f64::EPSILON {
            return BoundingBox::new(Vec::new());
        }

        let delta = polygon_area * unclip_ratio as f64 / perimeter;
        if delta.abs() <= f64::EPSILON {
            return BoundingBox::new(Vec::new());
        }

        let paths: PathsD = vec![clipper_path];
        let offset_paths = inflate_paths_d(
            &paths,
            delta,
            JoinType::Round,
            EndType::Polygon,
            2.0,
            2,
            0.0,
        );

        if offset_paths.len() != 1 {
            return BoundingBox::new(Vec::new());
        }

        let path = offset_paths.into_iter().next().unwrap();

        let mut points: Vec<Point> = path
            .iter()
            .map(|pt| Point::new(pt.x as f32, pt.y as f32))
            .collect();

        if points.len() > 1
            && let (Some(first), Some(last)) = (points.first(), points.last())
            && (first.x - last.x).abs() < f32::EPSILON
            && (first.y - last.y).abs() < f32::EPSILON
        {
            points.pop();
        }

        if points.len() < 3 {
            return BoundingBox::new(Vec::new());
        }

        BoundingBox::new(points)
    }

    pub fn box_score_fast(&self, pred: &ndarray::ArrayView2<f32>, bbox: &BoundingBox) -> f32 {
        let height = pred.shape()[0];
        let width = pred.shape()[1];

        let (min_x, min_y, max_x, max_y) = bbox.aabb();

        let min_x = min_x.floor().max(0.0).min(width as f32 - 1.0);
        let max_x = max_x.ceil().max(0.0).min(width as f32 - 1.0);
        let min_y = min_y.floor().max(0.0).min(height as f32 - 1.0);
        let max_y = max_y.ceil().max(0.0).min(height as f32 - 1.0);

        let start_y = min_y as usize;
        let end_y = max_y as usize + 1;
        let start_x = min_x as usize;
        let end_x = max_x as usize + 1;

        self.box_score_fast_contour(pred, bbox, start_y, end_y, start_x, end_x)
    }

    fn box_score_fast_contour(
        &self,
        pred: &ndarray::ArrayView2<f32>,
        bbox: &BoundingBox,
        start_y: usize,
        end_y: usize,
        start_x: usize,
        end_x: usize,
    ) -> f32 {
        let max_polygon_points = bbox.points.len();
        let mut scanline_buffer = ScanlineBuffer::new(max_polygon_points);

        let mut total_score = 0.0;
        let mut total_pixels = 0;

        for y in start_y..end_y {
            let scanline_y = y as f32 + 0.5;
            let (line_score, line_pixels) =
                scanline_buffer.process_scanline(scanline_y, bbox, start_x, end_x, pred);
            total_score += line_score;
            total_pixels += line_pixels;
        }

        if total_pixels > 0 {
            total_score / total_pixels as f32
        } else {
            0.0
        }
    }

    pub fn box_score_slow(
        &self,
        pred: &ndarray::ArrayView2<f32>,
        contour: &Contour<u32>,
    ) -> f32 {
        let height = pred.shape()[0];
        let width = pred.shape()[1];

        if height == 0 || width == 0 || contour.points.is_empty() {
            return 0.0;
        }

        let bbox = BoundingBox::from_contour(contour);
        let (min_x, min_y, max_x, max_y) = bbox.aabb();

        let start_y = min_y.floor().max(0.0).min(height as f32 - 1.0) as usize;
        let end_y = max_y.ceil().max(0.0).min(height as f32 - 1.0) as usize + 1;
        let start_x = min_x.floor().max(0.0).min(width as f32 - 1.0) as usize;
        let end_x = max_x.ceil().max(0.0).min(width as f32 - 1.0) as usize + 1;

        self.box_score_fast_contour(pred, &bbox, start_y, end_y, start_x, end_x)
    }
}
