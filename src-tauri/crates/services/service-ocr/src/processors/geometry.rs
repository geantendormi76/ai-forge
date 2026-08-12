use crate::models::text_region::{BoundingBox, Point};
use imageproc::contours::Contour;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinAreaRect {
    pub center: Point,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
}

impl MinAreaRect {
    pub fn min_side(&self) -> f32 {
        self.width.min(self.height)
    }
}

pub fn get_min_area_rect(points: &[Point]) -> MinAreaRect {
    if points.is_empty() {
        return MinAreaRect {
            center: Point::new(0.0, 0.0),
            width: 0.0,
            height: 0.0,
            angle: 0.0,
        };
    }

    let mut best_area = f32::INFINITY;
    let mut best_rect = MinAreaRect {
        center: Point::new(0.0, 0.0),
        width: 0.0,
        height: 0.0,
        angle: 0.0,
    };

    for deg in 0..90 {
        let rad = (deg as f32).to_radians();
        let cos_a = rad.cos();
        let sin_a = rad.sin();

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in points {
            let rx = p.x * cos_a + p.y * sin_a;
            let ry = -p.x * sin_a + p.y * cos_a;
            min_x = min_x.min(rx);
            max_x = max_x.max(rx);
            min_y = min_y.min(ry);
            max_y = max_y.max(ry);
        }

        let w = max_x - min_x;
        let h = max_y - min_y;
        let area = w * h;

        if area < best_area {
            best_area = area;
            let cx_rot = (min_x + max_x) * 0.5;
            let cy_rot = (min_y + max_y) * 0.5;

            let cx = cx_rot * cos_a - cy_rot * sin_a;
            let cy = cx_rot * sin_a + cy_rot * cos_a;

            best_rect = MinAreaRect {
                center: Point::new(cx, cy),
                width: w,
                height: h,
                angle: deg as f32,
            };
        }
    }

    best_rect
}

impl BoundingBox {
    pub fn from_contour(contour: &Contour<u32>) -> Self {
        let points = contour
            .points
            .iter()
            .map(|p| Point::new(p.x as f32, p.y as f32))
            .collect();
        Self { points }
    }

    pub fn perimeter(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut perim = 0.0f32;
        let n = self.points.len();
        for i in 0..n {
            let p1 = &self.points[i];
            let p2 = &self.points[(i + 1) % n];
            let dx = p1.x - p2.x;
            let dy = p1.y - p2.y;
            perim += (dx * dx + dy * dy).sqrt();
        }
        perim
    }

    pub fn get_min_area_rect_from_points(points: &[Point]) -> MinAreaRect {
        get_min_area_rect(points)
    }

    pub fn approx_poly_dp(&self, epsilon: f32) -> Self {
        if self.points.len() <= 2 {
            return self.clone();
        }
        let simplified = rdp_simplify(&self.points, epsilon);
        Self { points: simplified }
    }
}

fn rdp_simplify(points: &[Point], epsilon: f32) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    fn perpendicular_distance(p: &Point, line_start: &Point, line_end: &Point) -> f32 {
        let dx = line_end.x - line_start.x;
        let dy = line_end.y - line_start.y;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            let px = p.x - line_start.x;
            let py = p.y - line_start.y;
            return (px * px + py * py).sqrt();
        }
        let num = ((line_end.y - line_start.y) * p.x - (line_end.x - line_start.x) * p.y
            + line_end.x * line_start.y
            - line_end.y * line_start.x)
            .abs();
        num / len_sq.sqrt()
    }

    let mut dmax = 0.0f32;
    let mut index = 0;
    let end = points.len() - 1;

    for i in 1..end {
        let d = perpendicular_distance(&points[i], &points[0], &points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon {
        let mut rec_results1 = rdp_simplify(&points[..=index], epsilon);
        let mut rec_results2 = rdp_simplify(&points[index..], epsilon);
        rec_results1.pop();
        rec_results1.append(&mut rec_results2);
        rec_results1
    } else {
        vec![points[0], points[end]]
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScanlineBuffer {
    intersections: Vec<f32>,
}

impl ScanlineBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            intersections: Vec::with_capacity(capacity),
        }
    }

    pub fn process_scanline(
        &mut self,
        scanline_y: f32,
        bbox: &BoundingBox,
        start_x: usize,
        end_x: usize,
        pred: &ndarray::ArrayView2<f32>,
    ) -> (f32, usize) {
        self.intersections.clear();
        let pts = &bbox.points;
        let n = pts.len();
        if n < 3 {
            return (0.0, 0);
        }

        for i in 0..n {
            let p1 = &pts[i];
            let p2 = &pts[(i + 1) % n];

            if (p1.y <= scanline_y && p2.y > scanline_y) || (p2.y <= scanline_y && p1.y > scanline_y) {
                let dy = p2.y - p1.y;
                if dy.abs() > 1e-6 {
                    let t = (scanline_y - p1.y) / dy;
                    let x_inter = p1.x + t * (p2.x - p1.x);
                    self.intersections.push(x_inter);
                }
            }
        }

        self.intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut total_score = 0.0f32;
        let mut total_pixels = 0usize;

        let y_idx = (scanline_y.floor() as usize).min(pred.shape()[0].saturating_sub(1));

        let mut i = 0;
        while i + 1 < self.intersections.len() {
            let x_start = (self.intersections[i].ceil() as usize).max(start_x);
            let x_end = (self.intersections[i + 1].floor() as usize + 1).min(end_x);

            if x_start < x_end {
                let max_w = pred.shape()[1];
                let row = pred.row(y_idx);
                for x in x_start..x_end.min(max_w) {
                    total_score += row[x];
                    total_pixels += 1;
                }
            }
            i += 2;
        }

        (total_score, total_pixels)
    }
}
