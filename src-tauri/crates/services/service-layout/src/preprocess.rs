use image::{imageops, RgbImage};
use ndarray::Array4;

pub struct LayoutPreprocess;

impl LayoutPreprocess {
    /// 🛡️ 工业级 Letterbox 等比例缩放 + 填充预处理（保真宽高比，防图像畸变）
    pub fn preprocess_image(
        img: &RgbImage,
        target_size: (u32, u32),
    ) -> (Array4<f32>, f32, f32) {
        let (orig_w, orig_h) = img.dimensions();
        let (target_w, target_h) = target_size;

        // 1. 计算保持原始宽高比的统一缩放系数 scale
        let scale = (target_w as f32 / orig_w as f32).min(target_h as f32 / orig_h as f32);
        let new_w = ((orig_w as f32 * scale).round() as u32).clamp(1, target_w);
        let new_h = ((orig_h as f32 * scale).round() as u32).clamp(1, target_h);

        // 2. 无畸变调整等比尺寸
        let resized = imageops::resize(
            img,
            new_w,
            new_h,
            imageops::FilterType::Triangle,
        );

        // 3. 将缩放后的图像贴到 target_w x target_h 画布上（居中或靠左上）
        let mean = [0.485f32, 0.456, 0.406];
        let std = [0.229f32, 0.224, 0.225];

        let mut tensor = Array4::<f32>::zeros((1, 3, target_h as usize, target_w as usize));

        for y in 0..new_h as usize {
            for x in 0..new_w as usize {
                let pixel = resized.get_pixel(x as u32, y as u32);
                let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
                let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
                let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];

                tensor[[0, 0, y, x]] = r;
                tensor[[0, 1, y, x]] = g;
                tensor[[0, 2, y, x]] = b;
            }
        }

        // scale_w 与 scale_h 为统一的保真缩放比 scale
        (tensor, scale, scale)
    }
}
