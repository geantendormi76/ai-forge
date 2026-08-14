pub mod service;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConvertTask {
    pub input_path: String,
    pub target_format: String,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConvertResult {
    pub success: bool,
    pub input_path: String,
    pub output_path: Option<String>,
    pub detected_format: String,
    pub message: Option<String>,
}

impl FormatConvertResult {
    pub fn ok(input_path: &str, out_path: &Path, detected_format: &str) -> Self {
        Self {
            success: true,
            input_path: input_path.to_string(),
            output_path: Some(out_path.to_string_lossy().to_string()),
            detected_format: detected_format.to_string(),
            message: None,
        }
    }

    pub fn error(input_path: &str, detected_format: &str, msg: String) -> Self {
        Self {
            success: false,
            input_path: input_path.to_string(),
            output_path: None,
            detected_format: detected_format.to_string(),
            message: Some(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::service::FormatConvertService;
    use super::*;

    #[test]
    fn test_format_converter_dispatcher_csv_to_md() {
        let temp_dir = std::env::temp_dir().join("ai_forge_test_fc");
        let _ = std::fs::create_dir_all(&temp_dir);
        let sample_csv = temp_dir.join("sample.csv");
        std::fs::write(&sample_csv, "Name,Score\nAlice,100\nBob,95").unwrap();

        let task = FormatConvertTask {
            input_path: sample_csv.to_string_lossy().to_string(),
            target_format: "md".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        };

        let res = FormatConvertService::convert(&task);
        assert!(res.success);
        assert_eq!(res.detected_format, "markdown");
        assert!(res.output_path.is_some());

        let out_md = res.output_path.unwrap();
        let md_content = std::fs::read_to_string(&out_md).unwrap();
        assert!(md_content.contains("| Alice | 100 |"));

        let _ = std::fs::remove_dir_all(temp_dir);
        println!("✅ format-converter 桌面端工具中枢调度与分流交付断言通过！");
    }
}
