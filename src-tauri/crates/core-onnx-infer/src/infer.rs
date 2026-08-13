use crate::config::{parse_device_config, OrtSessionConfig};
use crate::errors::OrtInferError;
use ndarray::{Array2, Array3, Array4};
use ort::session::Session;
use std::path::Path;

pub struct OrtInfer {
    session: Session,
    input_name: String,
}

impl OrtInfer {
    pub fn new(model_path: &Path, device_override: Option<&str>) -> Result<Self, OrtInferError> {
        let config = if let Some(dev) = device_override {
            parse_device_config(dev)?
        } else {
            OrtSessionConfig::default()
        };

        Self::new_with_config(model_path, config)
    }

    pub fn new_with_config(
        model_path: &Path,
        config: OrtSessionConfig,
    ) -> Result<Self, OrtInferError> {
        let mut builder = config.build_session_builder()?;
        let session = builder
            .commit_from_file(model_path)
            .map_err(|e| OrtInferError::ModelLoad {
                path: model_path.to_string_lossy().to_string(),
                context: format!("Failed to commit session from ONNX file: {e}"),
            })?;

        let input_name = if let Some(input) = session.inputs().first() {
            input.name().to_string()
        } else {
            "x".to_string()
        };

        Ok(Self { session, input_name })
    }

    pub fn input_names(&self) -> Vec<String> {
        self.session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect()
    }

    pub fn infer_array4(&mut self, input: &Array4<f32>) -> Result<Array4<f32>, OrtInferError> {
        let shape = input.shape().to_vec();
        let data = if let Some(slice) = input.as_slice() {
            slice.to_vec()
        } else {
            input.iter().copied().collect()
        };

        let value = ort::value::Tensor::from_array((shape, data)).map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to create input tensor value: {e}"),
        })?;

        let outputs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => &value])
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Inference run failed: {e}"),
            })?;

        let output_val = outputs
            .into_iter()
            .next()
            .ok_or_else(|| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: "No output tensor returned from ONNX session".to_string(),
            })?
            .1;

        let (shape, data) = output_val.try_extract_tensor::<f32>().map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to extract f32 tensor output: {e}"),
        })?;

        if shape.len() == 4 {
            let batch = shape[0] as usize;
            let channel = shape[1] as usize;
            let height = shape[2] as usize;
            let width = shape[3] as usize;
            Array4::from_shape_vec((batch, channel, height, width), data.to_vec())
                .map_err(|e| OrtInferError::InvalidInput(format!("Output 4D shape mismatch: {e}")))
        } else {
            Err(OrtInferError::InvalidInput(format!(
                "Expected 4D output from ONNX model, got shape {:?}",
                shape
            )))
        }
    }

    pub fn infer_array3(&mut self, input: &Array4<f32>) -> Result<Array3<f32>, OrtInferError> {
        let shape = input.shape().to_vec();
        let data = if let Some(slice) = input.as_slice() {
            slice.to_vec()
        } else {
            input.iter().copied().collect()
        };

        let value = ort::value::Tensor::from_array((shape, data)).map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to create input tensor value: {e}"),
        })?;

        let outputs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => &value])
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Inference run failed: {e}"),
            })?;

        let output_val = outputs
            .into_iter()
            .next()
            .ok_or_else(|| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: "No output tensor returned from ONNX session".to_string(),
            })?
            .1;

        let (shape, data) = output_val.try_extract_tensor::<f32>().map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to extract f32 tensor output: {e}"),
        })?;

        if shape.len() == 3 {
            let batch = shape[0] as usize;
            let time_steps = shape[1] as usize;
            let vocab_size = shape[2] as usize;
            Array3::from_shape_vec((batch, time_steps, vocab_size), data.to_vec())
                .map_err(|e| OrtInferError::InvalidInput(format!("Output 3D shape mismatch: {e}")))
        } else {
            Err(OrtInferError::InvalidInput(format!(
                "Expected 3D output from ONNX model, got shape {:?}",
                shape
            )))
        }
    }

    /// 🛡️ 解构 2D int64 标志位矩阵：专门适配 PP-FormulaNet 等公式 Token 序列输出
    pub fn infer_array2_i64(&mut self, input: &Array4<f32>) -> Result<Array2<i64>, OrtInferError> {
        let shape = input.shape().to_vec();
        let data = if let Some(slice) = input.as_slice() {
            slice.to_vec()
        } else {
            input.iter().copied().collect()
        };

        let value = ort::value::Tensor::from_array((shape, data)).map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to create input tensor value: {e}"),
        })?;

        let outputs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => &value])
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Inference run failed: {e}"),
            })?;

        let output_val = outputs
            .into_iter()
            .next()
            .ok_or_else(|| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: "No output tensor returned from ONNX session".to_string(),
            })?
            .1;

        let (shape, data) = output_val.try_extract_tensor::<i64>().map_err(|e| OrtInferError::Inference {
            model_name: "ONNX".to_string(),
            context: format!("Failed to extract i64 tensor output: {e}"),
        })?;

        if shape.len() == 2 {
            let batch = shape[0] as usize;
            let seq_len = shape[1] as usize;
            Array2::from_shape_vec((batch, seq_len), data.to_vec())
                .map_err(|e| OrtInferError::InvalidInput(format!("Output 2D i64 shape mismatch: {e}")))
        } else {
            Err(OrtInferError::InvalidInput(format!(
                "Expected 2D i64 output from ONNX model, got shape {:?}",
                shape
            )))
        }
    }

    /// 🛡️ 多输入 2D 矩阵解构推导：专门适配 PP-DocLayoutV3 的 [300, 7] 格式
    pub fn infer_scale_aware_array2(
        &mut self,
        image_input: &Array4<f32>,
        orig_shape: (f32, f32), // (orig_w, orig_h)
    ) -> Result<ndarray::Array2<f32>, OrtInferError> {
        let input_names = self.input_names();
        let img_shape = image_input.shape();
        let target_h = img_shape[2] as f32;
        let target_w = img_shape[3] as f32;
        let (orig_w, orig_h) = orig_shape;

        let scale_h = target_h / orig_h;
        let scale_w = target_w / orig_w;

        let image_data = if let Some(slice) = image_input.as_slice() {
            slice.to_vec()
        } else {
            image_input.iter().copied().collect()
        };

        let image_val = ort::value::Tensor::from_array((img_shape.to_vec(), image_data))
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Failed to create image input tensor: {e}"),
            })?;

        let im_shape_val = ort::value::Tensor::from_array((vec![1usize, 2], vec![target_h, target_w]))
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Failed to create im_shape input tensor: {e}"),
            })?;

        let scale_val = ort::value::Tensor::from_array((vec![1usize, 2], vec![scale_h, scale_w]))
            .map_err(|e| OrtInferError::Inference {
                model_name: "ONNX".to_string(),
                context: format!("Failed to create scale_factor input tensor: {e}"),
            })?;

        let image_node_name = input_names.iter().find(|n| n.contains("image") || *n == "x").cloned()
            .unwrap_or_else(|| self.input_name.clone());

        let im_shape_node_name = input_names.iter().find(|n| n.contains("im_shape")).cloned();
        let scale_factor_node_name = input_names.iter().find(|n| n.contains("scale")).cloned();

        let outputs = match (im_shape_node_name, scale_factor_node_name) {
            (Some(im_name), Some(scale_name)) => {
                self.session.run(ort::inputs![
                    image_node_name.as_str() => &image_val,
                    im_name.as_str() => &im_shape_val,
                    scale_name.as_str() => &scale_val
                ])
            }
            (Some(im_name), None) => {
                self.session.run(ort::inputs![
                    image_node_name.as_str() => &image_val,
                    im_name.as_str() => &im_shape_val
                ])
            }
            (None, Some(scale_name)) => {
                self.session.run(ort::inputs![
                    image_node_name.as_str() => &image_val,
                    scale_name.as_str() => &scale_val
                ])
            }
            (None, None) => {
                self.session.run(ort::inputs![
                    image_node_name.as_str() => &image_val
                ])
            }
        }.map_err(|e| OrtInferError::Inference {
            model_name: "PP-DocLayout".to_string(),
            context: format!("Scale-aware inference run failed: {e}"),
        })?;

        let output_val = outputs
            .into_iter()
            .next()
            .ok_or_else(|| OrtInferError::Inference {
                model_name: "PP-DocLayout".to_string(),
                context: "No output tensor returned from ONNX session".to_string(),
            })?
            .1;

        let (shape, data) = output_val.try_extract_tensor::<f32>().map_err(|e| OrtInferError::Inference {
            model_name: "PP-DocLayout".to_string(),
            context: format!("Failed to extract f32 tensor output: {e}"),
        })?;

        if shape.len() == 2 {
            let rows = shape[0] as usize;
            let cols = shape[1] as usize;
            ndarray::Array2::from_shape_vec((rows, cols), data.to_vec())
                .map_err(|e| OrtInferError::InvalidInput(format!("Output 2D shape mismatch: {e}")))
        } else {
            Err(OrtInferError::InvalidInput(format!(
                "Expected 2D output from ONNX model, got shape {:?}",
                shape
            )))
        }
    }
}
