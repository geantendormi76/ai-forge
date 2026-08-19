//! 🛡️ AI-Forge 纯血 Native PDF 解析组装引擎 (pdf-parse)
//! 100% 零 Python 沙箱、零 IPC 网络通信、纯 C-FFI 硬件直推

pub mod deep_track;
pub mod exporter;
pub mod fast_track;
pub mod hybrid;
pub mod pipeline;
pub mod probe;
pub mod service;

pub use deep_track::{DeepTrackEngine, DeepTrackResult};
pub use exporter::ZipContainerExporter;
pub use fast_track::{FastTrackEngine, FastTrackResult};
pub use hybrid::{HybridEngine, HybridResult};
pub use pipeline::{
    apply_class_margin, clean_katex_markdown, extract_vector_text_in_bbox, PipelineResult,
    RawCellBox, RawLayoutElement, TextExtractMode, UnifiedPipeline,
};
pub use probe::{PageMetrics, PdfRouteProbe, RouteDecision};
pub use service::{PdfParseResult, PdfParseService};
