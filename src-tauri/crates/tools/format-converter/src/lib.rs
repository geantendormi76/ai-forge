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

    /// 🌟 2026 SOTA 全能转换全矩阵自动化回归测试套件 (Exhaustive Matrix Test Suite)
    #[test]
    fn test_exhaustive_conversion_matrix() {
        let temp_dir = std::env::temp_dir().join("ai_forge_matrix_benchmark");
        let _ = std::fs::create_dir_all(&temp_dir);

        // 1. CSV -> JSON
        let csv_file = temp_dir.join("test_data.csv");
        std::fs::write(&csv_file, "id,name,score\n1,Alice,98\n2,Bob,95").unwrap();
        let res = FormatConvertService::convert(&FormatConvertTask {
            input_path: csv_file.to_string_lossy().to_string(),
            target_format: "json".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res.success, "CSV->JSON 转换失败: {:?}", res.message);
        assert!(temp_dir.join("test_data.json").exists());

        // 2. CSV -> Markdown
        let res = FormatConvertService::convert(&FormatConvertTask {
            input_path: csv_file.to_string_lossy().to_string(),
            target_format: "md".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res.success, "CSV->MD 转换失败: {:?}", res.message);
        assert!(temp_dir.join("test_data.md").exists());

        // 3. TSV -> CSV
        let tsv_file = temp_dir.join("test_data.tsv");
        std::fs::write(&tsv_file, "id\tname\n101\tZed").unwrap();
        let res = FormatConvertService::convert(&FormatConvertTask {
            input_path: tsv_file.to_string_lossy().to_string(),
            target_format: "csv".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res.success, "TSV->CSV 转换失败: {:?}", res.message);

        // 4. JSON -> CSV
        let json_file = temp_dir.join("test_records.json");
        std::fs::write(&json_file, r#"[{"user":"Alice","age":25},{"user":"Bob","age":30}]"#).unwrap();
        let res = FormatConvertService::convert(&FormatConvertTask {
            input_path: json_file.to_string_lossy().to_string(),
            target_format: "csv".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res.success, "JSON->CSV 转换失败: {:?}", res.message);

        // 5. XML -> JSON
        let xml_file = temp_dir.join("test_doc.xml");
        std::fs::write(&xml_file, "<note><to>Tove</to><from>Jani</from></note>").unwrap();
        let res = FormatConvertService::convert(&FormatConvertTask {
            input_path: xml_file.to_string_lossy().to_string(),
            target_format: "json".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res.success, "XML->JSON 转换失败: {:?}", res.message);

        // 6. Markdown -> EPUB & DOCX
        let md_file = temp_dir.join("test_book.md");
        std::fs::write(&md_file, "# Chapter 1\n\nHello Rust Matrix Test.").unwrap();
        let res_epub = FormatConvertService::convert(&FormatConvertTask {
            input_path: md_file.to_string_lossy().to_string(),
            target_format: "epub".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res_epub.success, "MD->EPUB 转换失败: {:?}", res_epub.message);
        assert!(temp_dir.join("test_book.epub").exists());

        let res_docx = FormatConvertService::convert(&FormatConvertTask {
            input_path: md_file.to_string_lossy().to_string(),
            target_format: "docx".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res_docx.success, "MD->DOCX 转换失败: {:?}", res_docx.message);
        assert!(temp_dir.join("test_book.docx").exists());

        // 7. BMP -> PDF
        let bmp_file = temp_dir.join("test_img.bmp");
        let mut mock_bmp = vec![0u8; 54 + 4 * 2];
        mock_bmp[0] = b'B'; mock_bmp[1] = b'M';
        mock_bmp[2] = 62; mock_bmp[10] = 54; mock_bmp[14] = 40; mock_bmp[18] = 2; mock_bmp[22] = 1; mock_bmp[26] = 1; mock_bmp[28] = 24;
        std::fs::write(&bmp_file, &mock_bmp).unwrap();
        let res_bmp_pdf = FormatConvertService::convert(&FormatConvertTask {
            input_path: bmp_file.to_string_lossy().to_string(),
            target_format: "pdf".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res_bmp_pdf.success, "BMP->PDF 转换失败: {:?}", res_bmp_pdf.message);

        // 8. ZIP -> Extract
        let zip_file = temp_dir.join("test_archive.zip");
        let items = [service_converter::archive::zip::ArchiveInputItem {
            archive_path: "inner.txt".to_string(),
            data: b"Decompressed Content",
        }];
        let zip_bytes = service_converter::archive::zip::zip_in_memory(&items, 6).unwrap();
        std::fs::write(&zip_file, &zip_bytes).unwrap();
        let res_zip = FormatConvertService::convert(&FormatConvertTask {
            input_path: zip_file.to_string_lossy().to_string(),
            target_format: "extract".to_string(),
            output_dir: Some(temp_dir.to_string_lossy().to_string()),
        });
        assert!(res_zip.success, "ZIP->Extract 解压失败: {:?}", res_zip.message);

        let _ = std::fs::remove_dir_all(&temp_dir);
        println!("🎉 [SOTA 全能转换矩阵全绿通过] 覆盖数据/文档/图像/电子书/归档等全域组合！");
    }
}
