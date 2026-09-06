use image::DynamicImage;
use pdfium_render::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfiumEngineError {
    #[error("Pdfium 动态库绑定/初始化失败: {0}")]
    InitFailed(String),
    #[error("加载 PDF 物理文件失败 ({path:?}): {cause}")]
    LoadPdfFailed { path: PathBuf, cause: String },
    #[error("访问 PDF 页码 {page_index} 超出范围 (总页数: {total_pages})")]
    PageOutOfBounds { page_index: usize, total_pages: usize },
    #[error("渲染 PDF 页面位图失败: {0}")]
    RenderFailed(String),
    #[error("提炼 PDF 页面文本失败: {0}")]
    TextExtractFailed(String),
}

pub type PdfiumEngineResult<T> = Result<T, PdfiumEngineError>;

/// 字符粒度精准边界框契约 (保留历史 x1..y2 兼容字段，并提供 Web/视觉归一化 top_left 坐标)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfCharInfo {
    pub unicode_char: char,
    /// 历史向下兼容原始坐标
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    /// 现代视觉/Web 标准坐标 (原点在左上角 [0,0]，Y 轴向下增长)
    pub top_left_x: f32,
    pub top_left_y: f32,
    pub width_pt: f32,
    pub height_pt: f32,
}

/// PDF 页面物理元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageMeta {
    pub page_index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub is_landscape: bool,
}

/// PDF 完整文档元数据摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocumentMeta {
    pub file_path: String,
    pub total_pages: usize,
    pub pages: Vec<PdfPageMeta>,
}

/// 常驻单例 Pdfium 引擎包装器
static PDFIUM_INSTANCE: OnceLock<Mutex<PdfiumEngine>> = OnceLock::new();

pub struct PdfiumEngine {
    pdfium: Pdfium,
}

impl PdfiumEngine {
    /// 自动寻找并绑定 pdfium.dll (优先全局共享 C:\dev\bin 与安装目录)
    pub fn resolve_dll_path() -> Option<PathBuf> {
        let mut candidates = vec![
            PathBuf::from(r"C:\dev\bin\pdfium.dll"),
            PathBuf::from(r"C:\dev\ai-forge\bin\pdfium.dll"),
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\pdfium.dll"),
            PathBuf::from(r"C:\dev\ai-forge\models\pdfium.dll"),
            PathBuf::from("bin/pdfium.dll"),
            PathBuf::from("../bin/pdfium.dll"),
            PathBuf::from("pdfium.dll"),
        ];

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(exe_dir.join("pdfium.dll"));
                candidates.push(exe_dir.join("bin").join("pdfium.dll"));
                candidates.push(exe_dir.join("Data").join("bin").join("pdfium.dll"));
                candidates.push(exe_dir.join("resources").join("bin").join("pdfium.dll"));
            }
        }

        candidates.into_iter().find(|p| p.exists())
    }

    pub fn init() -> PdfiumEngineResult<Self> {
        let bindings = if let Some(dll_path) = Self::resolve_dll_path() {
            tracing::info!("🌐 [service-pdfium] 成功绑定物理路径 pdfium.dll: {:?}", dll_path);
            Pdfium::bind_to_library(&dll_path)
                .map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?
        } else {
            tracing::info!("🌐 [service-pdfium] 尝试绑定系统默认/环境变量 pdfium.dll");
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
                .map_err(|e| PdfiumEngineError::InitFailed(format!("未找到 pdfium.dll: {e}")))?
        };

        let pdfium = Pdfium::new(bindings);
        Ok(Self { pdfium })
    }

    pub fn global() -> PdfiumEngineResult<&'static Mutex<PdfiumEngine>> {
        let mutex = PDFIUM_INSTANCE.get_or_init(|| {
            let engine = Self::init().expect("🚨 [service-pdfium] 绑定初始化失败，请确保 pdfium.dll 存在于 C:\\dev\\bin\\pdfium.dll 或安装目录");
            Mutex::new(engine)
        });
        Ok(mutex)
    }

    /// 1. 获取 PDF 完整文档元数据 (包含所有页面的 Point 尺寸与横竖版型)
    pub fn get_document_meta(path: &Path) -> PdfiumEngineResult<PdfDocumentMeta> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;

        let pages = doc.pages();
        let total_pages = pages.len() as usize;
        let mut page_metas = Vec::with_capacity(total_pages);

        for (idx, page) in pages.iter().enumerate() {
            let w = page.width().value;
            let h = page.height().value;
            page_metas.push(PdfPageMeta {
                page_index: idx,
                width_pt: w,
                height_pt: h,
                is_landscape: w > h,
            });
        }

        Ok(PdfDocumentMeta {
            file_path: path.to_string_lossy().to_string(),
            total_pages,
            pages: page_metas,
        })
    }

    /// 2. 获取 PDF 总页数 (极速轻量通道)
    pub fn get_page_count(path: &Path) -> PdfiumEngineResult<usize> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;
        Ok(doc.pages().len() as usize)
    }

    /// 3. 获取指定页面的 Point 尺寸 (保留历史兼容接口)
    pub fn get_page_dimensions(path: &Path, page_index: usize) -> PdfiumEngineResult<(f32, f32)> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;
        let pages = doc.pages();
        let total = pages.len() as usize;
        if page_index >= total {
            return Err(PdfiumEngineError::PageOutOfBounds { page_index, total_pages: total });
        }
        let page = pages.get(page_index as i32)
            .map_err(|e| PdfiumEngineError::RenderFailed(e.to_string()))?;
        Ok((page.width().value, page.height().value))
    }

    /// 4. 高保真渲染页面至 DynamicImage (默认 300 DPI 高清图像推导)
    pub fn render_page_to_image(path: &Path, page_index: usize, target_dpi: u32) -> PdfiumEngineResult<DynamicImage> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;

        let pages = doc.pages();
        let total = pages.len() as usize;
        if page_index >= total {
            return Err(PdfiumEngineError::PageOutOfBounds { page_index, total_pages: total });
        }

        let page = pages.get(page_index as i32)
            .map_err(|e| PdfiumEngineError::RenderFailed(e.to_string()))?;

        let scale_factor = target_dpi as f32 / 72.0;
        let render_config = PdfRenderConfig::new()
            .scale_page_by_factor(scale_factor)
            .render_annotations(true)
            .render_form_data(true);

        let image = page.render_with_config(&render_config)
            .map_err(|e| PdfiumEngineError::RenderFailed(e.to_string()))?
            .as_image()
            .map_err(|e| PdfiumEngineError::RenderFailed(e.to_string()))?;

        Ok(image)
    }

    /// 5. 原生提取指定页面的纯文本流
    pub fn extract_page_text(path: &Path, page_index: usize) -> PdfiumEngineResult<String> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;

        let pages = doc.pages();
        let total = pages.len() as usize;
        if page_index >= total {
            return Err(PdfiumEngineError::PageOutOfBounds { page_index, total_pages: total });
        }

        let page = pages.get(page_index as i32)
            .map_err(|e| PdfiumEngineError::TextExtractFailed(e.to_string()))?;
        let text_page = page.text()
            .map_err(|e| PdfiumEngineError::TextExtractFailed(e.to_string()))?;

        Ok(text_page.all())
    }

    /// 6. 提取带物理边界框的字符数组 (已自动校准为左上角 Web/视觉标准坐标系，并保留 x1..y2 兼容)
    pub fn extract_page_chars(path: &Path, page_index: usize) -> PdfiumEngineResult<Vec<PdfCharInfo>> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;

        let pages = doc.pages();
        let total = pages.len() as usize;
        if page_index >= total {
            return Err(PdfiumEngineError::PageOutOfBounds { page_index, total_pages: total });
        }

        let page = pages.get(page_index as i32)
            .map_err(|e| PdfiumEngineError::TextExtractFailed(e.to_string()))?;
        let page_height = page.height().value;
        let text_page = page.text()
            .map_err(|e| PdfiumEngineError::TextExtractFailed(e.to_string()))?;

        let mut char_infos = Vec::new();
        for ch in text_page.chars().iter() {
            if let (Some(unicode), Ok(rect)) = (ch.unicode_char(), ch.loose_bounds()) {
                let px1 = rect.left().value;
                let py1 = rect.bottom().value;
                let px2 = rect.right().value;
                let py2 = rect.top().value;

                let w = (px2 - px1).abs();
                let h = (py2 - py1).abs();

                // 🛡️ 核心校准：将 PDF 左下角坐标系精准映射为 Web/OCR 左上角坐标系
                let top_left_x = px1.min(px2);
                let top_left_y = (page_height - py2).max(0.0);

                char_infos.push(PdfCharInfo {
                    unicode_char: unicode,
                    x1: px1,
                    y1: py1,
                    x2: px2,
                    y2: py2,
                    top_left_x,
                    top_left_y,
                    width_pt: w,
                    height_pt: h,
                });
            }
        }
        Ok(char_infos)
    }
}
