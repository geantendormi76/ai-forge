use crate::models::text_region::BoundingBox;
use image::{imageops, RgbImage};

pub fn crop_text_region(img: &RgbImage, bbox: &BoundingBox) -> RgbImage {
    let (img_w, img_h) = img.dimensions();
    if img_w == 0 || img_h == 0 || bbox.points.is_empty() {
        return RgbImage::new(1, 1);
    }

    let (min_x, min_y, max_x, max_y) = bbox.aabb();

    let pad = 2.0;
    let x1 = (min_x - pad).floor().max(0.0) as u32;
    let y1 = (min_y - pad).floor().max(0.0) as u32;
    let x2 = (max_x + pad).ceil().min(img_w as f32) as u32;
    let y2 = (max_y + pad).ceil().min(img_h as f32) as u32;

    let crop_w = (x2.saturating_sub(x1)).max(1);
    let crop_h = (y2.saturating_sub(y1)).max(1);

    imageops::crop_imm(img, x1, y1, crop_w, crop_h).to_image()
}
