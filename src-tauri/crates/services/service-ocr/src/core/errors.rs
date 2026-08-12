use core_onnx_infer::OrtInferError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OCRError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Model load failed: {path} ({context})")]
    ModelLoad {
        path: String,
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Inference failed for {model_name}: {context}")]
    Inference {
        model_name: String,
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Image load failed: {0}")]
    ImageLoad(#[from] image::ImageError),

    #[error("ONNX inference engine error: {0}")]
    OrtInfer(#[from] OrtInferError),
}

pub type OcrResult<T> = Result<T, OCRError>;
