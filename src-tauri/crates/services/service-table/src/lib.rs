pub mod decoder;
pub mod engine;
pub mod types;

pub use decoder::{parse_cell_grid_info, wrap_table_html_with_content, TableError, TableResult, TableStructureDecode};
pub use engine::{preprocess_image, TableEngine};
pub use types::{BBox8, CellGridInfo, TableCellPredict, TableStructureResult};

use std::path::Path;
use std::sync::{Mutex, OnceLock};

static ENGINE_INSTANCE: OnceLock<Mutex<TableEngine>> = OnceLock::new();

pub fn recognize_table_crop(
    image: &image::RgbImage,
    model_path: &Path,
    dict_path: &Path,
    device_override: Option<&str>,
) -> TableResult<TableStructureResult> {
    let mutex = ENGINE_INSTANCE.get_or_init(|| {
        tracing::info!("🚀 初始化并常驻 service-table 表格结构解析引擎...");
        let engine = TableEngine::new(model_path, dict_path, device_override)
            .expect("🚨 初始化 TableEngine 失败，请检查模型与字典路径");
        Mutex::new(engine)
    });

    let mut engine = mutex
        .lock()
        .map_err(|e| TableError::Infer(format!("锁抢占失败: {e}")))?;

    engine.recognize(image)
}
