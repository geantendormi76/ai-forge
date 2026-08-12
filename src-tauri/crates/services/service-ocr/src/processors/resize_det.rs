use crate::core::constants::{DEFAULT_LIMIT_SIDE_LEN, DEFAULT_MAX_SIDE_LIMIT};
use crate::processors::types::{ImageScaleInfo, LimitType, ResizeType};
use image::{DynamicImage, GenericImageView};
use rayon::prelude::*;
use tracing::{error, warn};

#[derive(Debug)]
pub struct DetResizeForTest {
    pub resize_type: ResizeType,
    pub limit_side_len: Option<u32>,
    pub limit_type: Option<LimitType>,
    pub max_side_limit: u32,
    pub filter: image::imageops::FilterType,
}

impl DetResizeForTest {
    pub fn new(
        input_shape: Option<(u32, u32, u32)>,
        image_shape: Option<(u32, u32)>,
        keep_ratio: Option<bool>,
        limit_side_len: Option<u32>,
        limit_type: Option<LimitType>,
        resize_long: Option<u32>,
        max_side_limit: Option<u32>,
    ) -> Self {
        let resize_type = if let Some(shape) = input_shape {
            ResizeType::Type3 { input_shape: shape }
        } else if let Some(shape) = image_shape {
            ResizeType::Type1 {
                image_shape: shape,
                keep_ratio: keep_ratio.unwrap_or(false),
            }
        } else if let Some(long) = resize_long {
            ResizeType::Type2 { resize_long: long }
        } else {
            ResizeType::Type0
        };

        Self {
            resize_type,
            limit_side_len: limit_side_len.or(Some(DEFAULT_LIMIT_SIDE_LEN)),
            limit_type: limit_type.or(Some(LimitType::Max)),
            max_side_limit: max_side_limit.unwrap_or(DEFAULT_MAX_SIDE_LIMIT),
            filter: image::imageops::FilterType::Triangle,
        }
    }

    pub fn apply(
        &self,
        imgs: Vec<DynamicImage>,
        limit_side_len: Option<u32>,
        limit_type: Option<LimitType>,
        max_side_limit: Option<u32>,
    ) -> (Vec<DynamicImage>, Vec<ImageScaleInfo>) {
        let resized: Vec<(DynamicImage, ImageScaleInfo)> = imgs
            .into_par_iter()
            .map(|img| self.resize(img, limit_side_len, limit_type.as_ref(), max_side_limit))
            .collect();

        let mut resize_imgs = Vec::with_capacity(resized.len());
        let mut img_shapes = Vec::with_capacity(resized.len());
        for (resized_img, shape) in resized {
            resize_imgs.push(resized_img);
            img_shapes.push(shape);
        }

        (resize_imgs, img_shapes)
    }

    fn resize(
        &self,
        mut img: DynamicImage,
        limit_side_len: Option<u32>,
        limit_type: Option<&LimitType>,
        max_side_limit: Option<u32>,
    ) -> (DynamicImage, ImageScaleInfo) {
        let (src_w, src_h) = img.dimensions();

        if (src_h + src_w) < 64 {
            img = self.image_padding(img);
        }

        let (resized_img, ratios) = match &self.resize_type {
            ResizeType::Type0 => {
                self.resize_image_type0(img, limit_side_len, limit_type, max_side_limit)
            }
            ResizeType::Type1 {
                image_shape,
                keep_ratio,
            } => self.resize_image_type1(img, *image_shape, *keep_ratio),
            ResizeType::Type2 { resize_long } => self.resize_image_type2(img, *resize_long),
            ResizeType::Type3 { input_shape } => self.resize_image_type3(img, *input_shape),
        };

        let scale_info = ImageScaleInfo::new(src_h as f32, src_w as f32, ratios[0], ratios[1]);
        (resized_img, scale_info)
    }

    fn image_padding(&self, img: DynamicImage) -> DynamicImage {
        let (w, h) = img.dimensions();
        let new_w = w.max(32);
        let new_h = h.max(32);

        if new_w == w && new_h == h {
            return img;
        }

        let mut padded = DynamicImage::new_rgb8(new_w, new_h);
        image::imageops::overlay(&mut padded, &img, 0, 0);
        padded
    }

    fn resize_image_type0(
        &self,
        img: DynamicImage,
        limit_side_len: Option<u32>,
        limit_type: Option<&LimitType>,
        max_side_limit: Option<u32>,
    ) -> (DynamicImage, [f32; 2]) {
        let (w, h) = img.dimensions();
        let limit_side_len = limit_side_len
            .or(self.limit_side_len)
            .unwrap_or(DEFAULT_LIMIT_SIDE_LEN);
        let limit_type = limit_type
            .or(self.limit_type.as_ref())
            .unwrap_or(&LimitType::Min);
        let max_side_limit = max_side_limit.unwrap_or(self.max_side_limit);

        let ratio = match limit_type {
            LimitType::Max => {
                if h.max(w) > limit_side_len {
                    limit_side_len as f32 / h.max(w) as f32
                } else {
                    1.0
                }
            }
            LimitType::Min => {
                if h.min(w) < limit_side_len {
                    limit_side_len as f32 / h.min(w) as f32
                } else {
                    1.0
                }
            }
            LimitType::ResizeLong => limit_side_len as f32 / h.max(w) as f32,
        };

        let mut resize_h = (h as f32 * ratio) as u32;
        let mut resize_w = (w as f32 * ratio) as u32;

        if resize_h.max(resize_w) > max_side_limit {
            warn!(
                "Resized image size ({}x{}) exceeds max_side_limit of {}. Resizing to fit within limit.",
                resize_h, resize_w, max_side_limit
            );
            let limit_ratio = max_side_limit as f32 / resize_h.max(resize_w) as f32;
            resize_h = (resize_h as f32 * limit_ratio) as u32;
            resize_w = (resize_w as f32 * limit_ratio) as u32;
        }

        resize_h = ((resize_h + 16) / 32 * 32).max(32);
        resize_w = ((resize_w + 16) / 32 * 32).max(32);

        if resize_h == h && resize_w == w {
            return (img, [1.0, 1.0]);
        }

        if resize_w == 0 || resize_h == 0 {
            error!("Invalid resize dimensions: {}x{}", resize_w, resize_h);
            return (img, [1.0, 1.0]);
        }

        let resized_img = img.resize_exact(resize_w, resize_h, self.filter);
        let ratio_h = resize_h as f32 / h as f32;
        let ratio_w = resize_w as f32 / w as f32;

        (resized_img, [ratio_h, ratio_w])
    }

    fn resize_image_type1(
        &self,
        img: DynamicImage,
        image_shape: (u32, u32),
        keep_ratio: bool,
    ) -> (DynamicImage, [f32; 2]) {
        let (ori_w, ori_h) = img.dimensions();
        let (resize_h, mut resize_w) = image_shape;

        if keep_ratio {
            resize_w = (ori_w * resize_h) / ori_h;
            let n = resize_w.div_ceil(32);
            resize_w = n * 32;
        }

        if resize_h == ori_h && resize_w == ori_w {
            return (img, [1.0, 1.0]);
        }

        let ratio_h = resize_h as f32 / ori_h as f32;
        let ratio_w = resize_w as f32 / ori_w as f32;
        let resized_img = img.resize_exact(resize_w, resize_h, self.filter);

        (resized_img, [ratio_h, ratio_w])
    }

    fn resize_image_type2(&self, img: DynamicImage, resize_long: u32) -> (DynamicImage, [f32; 2]) {
        let (w, h) = img.dimensions();

        let ratio = if h > w {
            resize_long as f32 / h as f32
        } else {
            resize_long as f32 / w as f32
        };

        let mut resize_h = (h as f32 * ratio) as u32;
        let mut resize_w = (w as f32 * ratio) as u32;

        let max_stride = 128;
        resize_h = resize_h.div_ceil(max_stride) * max_stride;
        resize_w = resize_w.div_ceil(max_stride) * max_stride;

        if resize_h == h && resize_w == w {
            return (img, [1.0, 1.0]);
        }

        let resized_img = img.resize_exact(resize_w, resize_h, self.filter);
        let ratio_h = resize_h as f32 / h as f32;
        let ratio_w = resize_w as f32 / w as f32;

        (resized_img, [ratio_h, ratio_w])
    }

    fn resize_image_type3(
        &self,
        img: DynamicImage,
        input_shape: (u32, u32, u32),
    ) -> (DynamicImage, [f32; 2]) {
        let (ori_w, ori_h) = img.dimensions();
        let (_, resize_h, resize_w) = input_shape;

        if resize_h == ori_h && resize_w == ori_w {
            return (img, [1.0, 1.0]);
        }

        let ratio_h = resize_h as f32 / ori_h as f32;
        let ratio_w = resize_w as f32 / ori_w as f32;
        let resized_img = img.resize_exact(resize_w, resize_h, self.filter);

        (resized_img, [ratio_h, ratio_w])
    }
}
