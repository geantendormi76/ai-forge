use service_converter::archive::zip::extract_archive_to_memory;
use service_converter::audio::av3a::extract_av3a_stream;
use service_converter::audio::kgm::convert_kgma_bytes;
use service_converter::audio::kwm::convert_kwm_bytes;
use service_converter::audio::ncm::convert_ncm_bytes;
use service_converter::audio::qmc::convert_qmc_bytes;
use service_converter::docx::build_docx_bytes;
use service_converter::ebook::epub::{build_epub_bytes, split_chapters};
use service_converter::ebook::mobi::parse_mobi_text;
use service_converter::image::bmp::decode_bmp_to_raw;
use service_converter::image::ico::{encode_ico, extract_best_frame, PngFrameInput};
use service_converter::image::pdf::{build_images_pdf, ImagePageInput};
use service_converter::text::csv::{csv_to_json_objects, csv_to_markdown, json_to_csv, normalize_tsv_to_csv};
use service_converter::text::xml::xml_to_json_value;

use std::path::{Path, PathBuf};

use crate::{FormatConvertResult, FormatConvertTask};

pub struct FormatConvertService;

impl FormatConvertService {
    pub fn convert(task: &FormatConvertTask) -> FormatConvertResult {
        let input_path = PathBuf::from(&task.input_path);
        if !input_path.exists() {
            return FormatConvertResult {
                success: false,
                input_path: task.input_path.clone(),
                output_path: None,
                detected_format: "unknown".to_string(),
                message: Some("输入文件不存在".into()),
            };
        }

        let ext = input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("converted");

        let out_dir = task
            .output_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| input_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf());

        let raw_bytes = match std::fs::read(&input_path) {
            Ok(b) => b,
            Err(e) => {
                return FormatConvertResult {
                    success: false,
                    input_path: task.input_path.clone(),
                    output_path: None,
                    detected_format: ext,
                    message: Some(format!("读取输入文件失败: {}", e)),
                };
            }
        };

        // 1. 音频解密域路由
        if ext == "ncm" {
            match convert_ncm_bytes(&raw_bytes) {
                Ok(res) => {
                    let out_path = out_dir.join(format!("{}.{}", stem, res.format));
                    if let Err(e) = std::fs::write(&out_path, &res.audio_data) {
                        return FormatConvertResult::error(&task.input_path, &res.format, format!("写入文件失败: {}", e));
                    }
                    return FormatConvertResult::ok(&task.input_path, &out_path, &res.format);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "ncm", e),
            }
        }

        if matches!(ext.as_str(), "mflac" | "mgg" | "qmc0" | "qmc3" | "qmcflac" | "qmcogg") {
            match convert_qmc_bytes(&raw_bytes) {
                Ok((audio, fmt)) => {
                    let out_path = out_dir.join(format!("{}.{}", stem, fmt));
                    if let Err(e) = std::fs::write(&out_path, &audio) {
                        return FormatConvertResult::error(&task.input_path, &fmt, format!("写入文件失败: {}", e));
                    }
                    return FormatConvertResult::ok(&task.input_path, &out_path, &fmt);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, &ext, e),
            }
        }

        if ext == "kgma" || ext == "vpr" {
            match convert_kgma_bytes(&raw_bytes) {
                Ok((audio, fmt)) => {
                    let out_path = out_dir.join(format!("{}.{}", stem, fmt));
                    if let Err(e) = std::fs::write(&out_path, &audio) {
                        return FormatConvertResult::error(&task.input_path, &fmt, format!("写入文件失败: {}", e));
                    }
                    return FormatConvertResult::ok(&task.input_path, &out_path, &fmt);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "kgma", e),
            }
        }

        if ext == "kwm" {
            match convert_kwm_bytes(&raw_bytes) {
                Ok((audio, fmt)) => {
                    let out_path = out_dir.join(format!("{}.{}", stem, fmt));
                    if let Err(e) = std::fs::write(&out_path, &audio) {
                        return FormatConvertResult::error(&task.input_path, &fmt, format!("写入文件失败: {}", e));
                    }
                    return FormatConvertResult::ok(&task.input_path, &out_path, &fmt);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "kwm", e),
            }
        }

        if ext == "av3a" {
            match extract_av3a_stream(&raw_bytes) {
                Ok(stream) => {
                    let out_path = out_dir.join(format!("{}.av3a", stem));
                    if let Err(e) = std::fs::write(&out_path, &stream) {
                        return FormatConvertResult::error(&task.input_path, "av3a", format!("写入文件失败: {}", e));
                    }
                    return FormatConvertResult::ok(&task.input_path, &out_path, "av3a");
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "av3a", e),
            }
        }

        // 2. 文本数据中台路由
        if ext == "csv" && task.target_format == "json" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                match csv_to_json_objects(text) {
                    Ok(json_objs) => {
                        let out_path = out_dir.join(format!("{}.json", stem));
                        let formatted = serde_json::to_string_pretty(&json_objs).unwrap_or_default();
                        let _ = std::fs::write(&out_path, formatted);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "json");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, "csv", e),
                }
            }
        }

        if ext == "csv" && (task.target_format == "md" || task.target_format == "markdown") {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                match csv_to_markdown(text) {
                    Ok(md) => {
                        let out_path = out_dir.join(format!("{}.md", stem));
                        let _ = std::fs::write(&out_path, md);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "markdown");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, "csv", e),
                }
            }
        }

        if ext == "tsv" && task.target_format == "csv" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                match normalize_tsv_to_csv(text) {
                    Ok(csv_out) => {
                        let out_path = out_dir.join(format!("{}.csv", stem));
                        let _ = std::fs::write(&out_path, csv_out);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "csv");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, "tsv", e),
                }
            }
        }

        if ext == "json" && task.target_format == "csv" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                match json_to_csv(text) {
                    Ok(csv_out) => {
                        let out_path = out_dir.join(format!("{}.csv", stem));
                        let _ = std::fs::write(&out_path, csv_out);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "csv");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, "json", e),
                }
            }
        }

        if ext == "xml" && task.target_format == "json" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                match xml_to_json_value(text) {
                    Ok(json_val) => {
                        let out_path = out_dir.join(format!("{}.json", stem));
                        let formatted = serde_json::to_string_pretty(&json_val).unwrap_or_default();
                        let _ = std::fs::write(&out_path, formatted);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "json");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, "xml", e),
                }
            }
        }

        // 3. 电子书与排版路由
        if (ext == "md" || ext == "txt") && task.target_format == "epub" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                let is_md = ext == "md";
                let chapters = split_chapters(text, is_md);
                match build_epub_bytes(stem, &chapters) {
                    Ok(epub_bytes) => {
                        let out_path = out_dir.join(format!("{}.epub", stem));
                        let _ = std::fs::write(&out_path, epub_bytes);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "epub");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, &ext, e),
                }
            }
        }

        if (ext == "md" || ext == "txt" || ext == "html") && task.target_format == "docx" {
            if let Ok(text) = std::str::from_utf8(&raw_bytes) {
                let is_md = ext == "md";
                match build_docx_bytes(text, is_md) {
                    Ok(docx_bytes) => {
                        let out_path = out_dir.join(format!("{}.docx", stem));
                        let _ = std::fs::write(&out_path, docx_bytes);
                        return FormatConvertResult::ok(&task.input_path, &out_path, "docx");
                    }
                    Err(e) => return FormatConvertResult::error(&task.input_path, &ext, e),
                }
            }
        }

        if ext == "mobi" && (task.target_format == "txt" || task.target_format == "html") {
            match parse_mobi_text(&raw_bytes) {
                Ok(html_text) => {
                    let out_path = out_dir.join(format!("{}.{}", stem, task.target_format));
                    let _ = std::fs::write(&out_path, html_text);
                    return FormatConvertResult::ok(&task.input_path, &out_path, &task.target_format);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "mobi", e),
            }
        }

        // 4. 原生图像与 ICO
        if ext == "ico" {
            match extract_best_frame(&raw_bytes) {
                Ok(best) => {
                    let out_ext = if best.is_png { "png" } else { "bmp" };
                    let out_path = out_dir.join(format!("{}.{}", stem, out_ext));
                    let _ = std::fs::write(&out_path, &best.data);
                    return FormatConvertResult::ok(&task.input_path, &out_path, out_ext);
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "ico", e),
            }
        }

        if (ext == "png" || ext == "jpg") && task.target_format == "ico" {
            let frame = PngFrameInput {
                size: 256,
                data: &raw_bytes,
            };
            match encode_ico(&[frame]) {
                Ok(ico_bytes) => {
                    let out_path = out_dir.join(format!("{}.ico", stem));
                    let _ = std::fs::write(&out_path, ico_bytes);
                    return FormatConvertResult::ok(&task.input_path, &out_path, "ico");
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, &ext, e),
            }
        }

        if ext == "bmp" {
            match decode_bmp_to_raw(&raw_bytes) {
                Ok(raw_bmp) => {
                    if task.target_format == "pdf" {
                        let page = ImagePageInput {
                            width: raw_bmp.width,
                            height: raw_bmp.height,
                            rgb_data: &raw_bmp.data,
                        };
                        match build_images_pdf(&[page]) {
                            Ok(pdf_bytes) => {
                                let out_path = out_dir.join(format!("{}.pdf", stem));
                                let _ = std::fs::write(&out_path, pdf_bytes);
                                return FormatConvertResult::ok(&task.input_path, &out_path, "pdf");
                            }
                            Err(e) => return FormatConvertResult::error(&task.input_path, "bmp", e),
                        }
                    }
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "bmp", e),
            }
        }

        // 5. ZIP 归档解压
        if ext == "zip" && task.target_format == "extract" {
            match extract_archive_to_memory(&raw_bytes) {
                Ok(items) => {
                    let target_extract_dir = out_dir.join(stem);
                    let _ = std::fs::create_dir_all(&target_extract_dir);
                    for (rel_path, content) in items {
                        let file_dest = target_extract_dir.join(rel_path);
                        if let Some(parent) = file_dest.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(file_dest, content);
                    }
                    return FormatConvertResult::ok(&task.input_path, &target_extract_dir, "directory");
                }
                Err(e) => return FormatConvertResult::error(&task.input_path, "zip", e),
            }
        }

        FormatConvertResult {
            success: false,
            input_path: task.input_path.clone(),
            output_path: None,
            detected_format: ext,
            message: Some(format!("暂不支持将当前格式转换为 {}", task.target_format)),
        }
    }
}
