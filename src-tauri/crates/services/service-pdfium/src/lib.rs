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

/// 字符粒度精准边界框契约（为未来的 PDF 编辑与定位服务）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfCharInfo {
    pub unicode_char: char,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// PDF 页面物理元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageMeta {
    pub page_index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub is_landscape: bool,
}

/// 常驻单例 Pdfium 引擎包装器
static PDFIUM_INSTANCE: OnceLock<Mutex<PdfiumEngine>> = OnceLock::new();

pub struct PdfiumEngine {
    pdfium: Pdfium,
}

impl PdfiumEngine {
    /// 自动寻找并绑定 pdfium.dll
    pub fn resolve_dll_path() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\pdfium.dll"),
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\pdfium.dll"),
            PathBuf::from(r"C:\dev\rpa\pdfium.dll"),
            PathBuf::from("pdfium.dll"),
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    pub fn init() -> PdfiumEngineResult<Self> {
        let bindings = if let Some(dll_path) = Self::resolve_dll_path() {
            tracing::info!("🌐 [service-pdfium] 绑定指定路径 pdfium.dll: {:?}", dll_path);
            Pdfium::bind_to_library(&dll_path)
                .map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?
        } else {
            tracing::info!("🌐 [service-pdfium] 尝试绑定系统默认/环境变量 pdfium.dll");
            Pdfium::bind_to_system_library()
                .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")))
                .map_err(|e| PdfiumEngineError::InitFailed(format!("未找到 pdfium.dll，请确保 pdfium.dll 存在于 C:\\dev\\ai-forge\\models\\pdfium.dll: {e}")))?
        };

        let pdfium = Pdfium::new(bindings);
        Ok(Self { pdfium })
    }

    pub fn global() -> PdfiumEngineResult<&'static Mutex<PdfiumEngine>> {
        let mutex = PDFIUM_INSTANCE.get_or_init(|| {
            let engine = Self::init().expect("🚨 [service-pdfium] 绑定初始化失败，请确保 pdfium.dll 存在于 C:\\dev\\ai-forge\\models\\pdfium.dll");
            Mutex::new(engine)
        });
        Ok(mutex)
    }

    /// 1. 获取 PDF 总页数
    pub fn get_page_count(path: &Path) -> PdfiumEngineResult<usize> {
        let lock = Self::global()?;
        let engine = lock.lock().map_err(|e| PdfiumEngineError::InitFailed(e.to_string()))?;
        let doc = engine.pdfium.load_pdf_from_file(path, None)
            .map_err(|e| PdfiumEngineError::LoadPdfFailed { path: path.to_path_buf(), cause: e.to_string() })?;
        Ok(doc.pages().len() as usize)
    }

    /// 2. 高保真渲染页面至 DynamicImage (默认 300 DPI 高清图像推导)
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

        // 300DPI 缩放比例因子 (1 point = 1/72 inch)
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

    /// 3. 原生提取指定页面的纯文本
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

    /// 4. 提取带物理边界框的字符数组（为未来 PDF 编辑与定位打底）
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

        let text_page = page.text()
            .map_err(|e| PdfiumEngineError::TextExtractFailed(e.to_string()))?;

        let mut char_infos = Vec::new();
        for ch in text_page.chars().iter() {
            if let (Some(unicode), Ok(rect)) = (ch.unicode_char(), ch.loose_bounds()) {
                char_infos.push(PdfCharInfo {
                    unicode_char: unicode,
                    x1: rect.left().value,
                    y1: rect.bottom().value,
                    x2: rect.right().value,
                    y2: rect.top().value,
                });
            }
        }

        Ok(char_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Instant;

    fn resolve_pdf_fixture() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\service-pdfium\1.pdf"),
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\pdf-parse-fast.pdf"),
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\tool-pdf-parse-fast.pdf"),
        ];
        candidates.into_iter().find(|p| p.exists()).unwrap_or_else(|| {
            PathBuf::from(r"C:\dev\ai-forge\test\fixtures\service-pdfium\1.pdf")
        })
    }

    #[test]
    fn test_service_pdfium_pipeline_benchmark() {
        let pdf_path = resolve_pdf_fixture();
        let out_dir = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-pdfium");
        let _ = fs::create_dir_all(&out_dir);

        println!("\n🚀 ===== [service-pdfium 原生底座 TDD 物理打靶启动] =====");
        println!("  物理 PDF 文件: {:?}", pdf_path);
        println!("  产物保存路径: {:?}", out_dir);

        if !pdf_path.exists() {
            println!("⚠️ [TDD 拦截] 物理 PDF 测试文件不存在，请检查测试物理路径");
            return;
        }

        if PdfiumEngine::resolve_dll_path().is_none() {
            println!("⚠️ [TDD 拦截] 本地未找到 pdfium.dll！请确保文件保存在 C:\\dev\\ai-forge\\models\\pdfium.dll");
            return;
        }

        let t0 = Instant::now();

        // 1. 获取 PDF 页数断言
        let page_count = PdfiumEngine::get_page_count(&pdf_path).expect("获取 PDF 页数失败");
        assert!(page_count > 0, "PDF 总页数不可为 0");
        println!("  📄 [打靶 1 - 元数据] PDF 总页数: {} 页", page_count);

        // 2. 高保真 300DPI 渲染断言
        let img = PdfiumEngine::render_page_to_image(&pdf_path, 0, 300).expect("渲染 300DPI 图像失败");
        assert!(img.width() > 0 && img.height() > 0, "渲染图像尺寸不可为 0");
        let out_img_path = out_dir.join("page_1_300dpi.png");
        img.save(&out_img_path).expect("保存 300DPI PNG 图像失败");
        println!("  🖼️ [打靶 2 - 300DPI 渲染] 尺寸: {}x{} px ➔ 落盘: {:?}", img.width(), img.height(), out_img_path);

        // 3. 原生文本提取断言
        let text = PdfiumEngine::extract_page_text(&pdf_path, 0).expect("提炼页面纯文本失败");
        assert!(!text.trim().is_empty(), "提炼的纯文本不可为空");
        let out_text_path = out_dir.join("page_1_text.txt");
        fs::write(&out_text_path, &text).expect("落盘纯文本失败");
        println!("  📝 [打靶 3 - 纯文本提取] 字符数: {} 字 ➔ 落盘: {:?}", text.len(), out_text_path);

        // 4. 字符粒度物理 BBox 坐标数组提取断言
        let chars = PdfiumEngine::extract_page_chars(&pdf_path, 0).expect("提取字符级 BBox 失败");
        assert!(!chars.is_empty(), "字符粒度 BBox 数组不可为空");
        let chars_json = serde_json::to_string_pretty(&chars).expect("序列化 BBox JSON 失败");
        let out_chars_path = out_dir.join("page_1_chars.json");
        fs::write(&out_chars_path, chars_json).expect("落盘 BBox JSON 失败");
        println!("  📐 [打靶 4 - 字符级 BBox] 字符对象数: {} 个 ➔ 落盘: {:?}", chars.len(), out_chars_path);

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("\n🎉 ===== [service-pdfium TDD 端到端 4 重打靶全量成功] =====");
        println!("  ⏱️ 物理总耗时: {:.2} ms (极致 C-FFI 推导速度！)", elapsed_ms);
        println!("  💾 4 组全量物理产物落盘目录: {:?}", out_dir);
        println!("===========================================================\n");
    }
}
