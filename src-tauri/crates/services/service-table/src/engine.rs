use crate::decoder::{TableError, TableResult, TableStructureDecode};
use crate::types::TableStructureResult;
use core_onnx_infer::OrtInfer;
use std::path::Path;

pub fn preprocess_image(img: &image::RgbImage, target_size: u32) -> (ndarray::Array4<f32>, (f32, f32)) {
    let (orig_w, orig_h) = (img.width() as f32, img.height() as f32);
    let scale = (target_size as f32) / orig_h.max(orig_w);
    let resized_w = (orig_w * scale).round() as u32;
    let resized_h = (orig_h * scale).round() as u32;

    let resized = image::imageops::resize(
        img,
        resized_w,
        resized_h,
        image::imageops::FilterType::Triangle,
    );

    let mean = [0.485f32, 0.456f32, 0.406f32];
    let std = [0.229f32, 0.224f32, 0.225f32];

    let target_usize = target_size as usize;
    let mut tensor = ndarray::Array4::<f32>::zeros((1, 3, target_usize, target_usize));

    for y in 0..resized_h {
        for x in 0..resized_w {
            let pixel = resized.get_pixel(x, y);
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;

            tensor[[0, 0, y as usize, x as usize]] = (b - mean[0]) / std[0];
            tensor[[0, 1, y as usize, x as usize]] = (g - mean[1]) / std[1];
            tensor[[0, 2, y as usize, x as usize]] = (r - mean[2]) / std[2];
        }
    }

    (tensor, (orig_h, orig_w))
}

pub struct TableEngine {
    infer: OrtInfer,
    decoder: TableStructureDecode,
    target_size: u32,
}

impl TableEngine {
    pub fn new(model_path: &Path, dict_path: &Path, device_override: Option<&str>) -> TableResult<Self> {
        let infer = OrtInfer::new(model_path, device_override)
            .map_err(|e| TableError::Infer(e.to_string()))?;
        let decoder = TableStructureDecode::from_dict_file(dict_path)?;
        Ok(Self {
            infer,
            decoder,
            target_size: 488,
        })
    }

    pub fn recognize(&mut self, image: &image::RgbImage) -> TableResult<TableStructureResult> {
        let (tensor, orig_shape) = preprocess_image(image, self.target_size);
        let (out0, out1) = self
            .infer
            .infer_dual_array3(&tensor)
            .map_err(|e| TableError::Infer(e.to_string()))?;

        let shape0 = out0.shape().to_vec();
        let shape1 = out1.shape().to_vec();

        let (bbox_preds, structure_logits) = if shape0[2] == 8 {
            (out0, out1)
        } else if shape1[2] == 8 {
            (out1, out0)
        } else {
            return Err(TableError::InvalidTensor(format!(
                "无法匹配 SLANet 8 坐标通道, shape0: {:?}, shape1: {:?}",
                shape0, shape1
            )));
        };

        let bbox_slice = bbox_preds.slice(ndarray::s![0, .., ..]).to_owned();
        let logits_slice = structure_logits.slice(ndarray::s![0, .., ..]).to_owned();

        self.decoder.decode_single(&logits_slice, &bbox_slice, orig_shape, self.target_size as f32)
    }
}
