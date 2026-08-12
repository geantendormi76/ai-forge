use crate::core::errors::OCRError;
use crate::engine::ocr_engine::OcrEngine;
use crate::models::text_region::TextRegion;
use image::DynamicImage;
use std::path::{Path, PathBuf};

pub struct OcrService;

impl OcrService {
    pub fn resolve_default_det_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_det.onnx"),
            PathBuf::from(r"C:\dev\rpa\models\pp-ocrv6_small\pp-ocrv6_small_det.onnx"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\PP-OCRv6_medium_det\model.onnx"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_det.onnx")
        })
    }

    pub fn resolve_default_rec_model_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_rec.onnx"),
            PathBuf::from(r"C:\dev\rpa\models\pp-ocrv6_small\pp-ocrv6_small_rec.onnx"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\PP-OCRv6_medium_rec\model.onnx"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\pp-ocrv6_small_rec.onnx")
        })
    }

    pub fn resolve_default_dict_path() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\ppocrv6_dict.txt"),
            PathBuf::from(r"C:\dev\rpa\models\pp-ocrv6_small\ppocrv6_dict.txt"),
            PathBuf::from(r"C:\dev\ai-forge\models\tool-pdf-parse\PP-OCRv6_medium_rec\ppocrv6_dict.txt"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            PathBuf::from(r"C:\dev\ai-forge\models\service-ocr\PP-OCRv6_small\ppocrv6_dict.txt")
        })
    }

    pub fn default_engine() -> Result<OcrEngine, OCRError> {
        let det_path = Self::resolve_default_det_model_path();
        let rec_path = Self::resolve_default_rec_model_path();
        let dict_path = Self::resolve_default_dict_path();
        OcrEngine::new(&det_path, &rec_path, &dict_path)
    }

    pub fn run_ocr_default(img: DynamicImage) -> Result<Vec<TextRegion>, OCRError> {
        let mut engine = Self::default_engine()?;
        engine.process_image(img)
    }

    pub fn run_ocr(
        det_model_path: &Path,
        rec_model_path: &Path,
        dict_path: &Path,
        img: DynamicImage,
    ) -> Result<Vec<TextRegion>, OCRError> {
        let mut engine = OcrEngine::new(det_model_path, rec_model_path, dict_path)?;
        engine.process_image(img)
    }
}
