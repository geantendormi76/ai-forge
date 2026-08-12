use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrtInferError {
    #[error("Failed to load ONNX model at '{path}': {context}")]
    ModelLoad { path: String, context: String },

    #[error("Inference execution failed for model '{model_name}': {context}")]
    Inference { model_name: String, context: String },

    #[error("Invalid tensor shape or data: {0}")]
    InvalidInput(String),

    #[error("Hardware execution provider error: {0}")]
    ProviderError(String),
}

pub type OrtInferResult<T> = Result<T, OrtInferError>;
