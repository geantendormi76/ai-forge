use image::{imageops, RgbImage};
use ndarray::Array4;

pub struct LayoutPreprocess;

impl LayoutPreprocess {
    /// 🛡️ 1:1 对齐官方 inference.yml: Resize(800, 800, keep_ratio=false) + Normalize(mean=0, std=1 => [0.0, 1.0])
    pub fn preprocess_image(
        img: &RgbImage,
        target_size: (u32, u32),
    ) -> (Array4<f32>, f32, f32) {
        let (orig_w, orig_h) = img.dimensions();
        let (target_w, target_h) = target_size;

        let resized = imageops::resize(
            img,
            target_w,
            target_h,
            imageops::FilterType::Triangle,
        );

        let mut tensor = Array4::<f32>::zeros((1, 3, target_h as usize, target_w as usize));

        for y in 0..target_h as usize {
            for x in 0..target_w as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                tensor[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
                tensor[[0, 1, y, x]] = pixel[1] as f32 / 255.0;
                tensor[[0, 2, y, x]] = pixel[2] as f32 / 255.0;
            }
        }

        let scale_w = target_w as f32 / orig_w as f32;
        let scale_h = target_h as f32 / orig_h as f32;

        (tensor, scale_w, scale_h)
    }
}
