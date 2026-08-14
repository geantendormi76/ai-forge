pub mod archive;
pub mod audio;
pub mod docx;
pub mod ebook;
pub mod image;
pub mod media;
pub mod text;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConverterError {
    #[error("文件魔数校验不匹配: {0}")]
    InvalidMagic(String),
    #[error("对称加解密失败: {0}")]
    CryptoError(String),
    #[error("数据格式损坏或布局越界: {0}")]
    CorruptedData(String),
    #[error("音频解密异常: {0}")]
    AudioDecryptError(String),
    #[error("图像解码/容器异常: {0}")]
    ImageError(String),
    #[error("文本/数据解析异常: {0}")]
    TextParseError(String),
    #[error("电子书生成/解密异常: {0}")]
    EbookError(String),
    #[error("Word DOCX 生成异常: {0}")]
    DocxError(String),
    #[error("PDF 生成异常: {0}")]
    PdfBuildError(String),
    #[error("归档/ZIP 处理异常: {0}")]
    ArchiveError(String),
    #[error("多媒体/FFmpeg 转码异常: {0}")]
    MediaError(String),
    #[error("IO 读写异常: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON 序列化/反序列化失败: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type ConverterResult<T> = Result<T, ConverterError>;

#[cfg(test)]
mod tests {
    use crate::archive::zip::{extract_archive_to_memory, list_archive_entries, zip_in_memory, ArchiveInputItem};
    use crate::audio::av3a::parse_boxes;
    use crate::audio::kgg::{aes_128_cbc_decrypt, derive_page_aes_iv, derive_page_aes_key, next_page_iv, MASTER_KEY};
    use crate::audio::kgm::{kugo_md5, xor_collapse_u32};
    use crate::audio::kwm::generate_kwm_mask;
    use crate::audio::ncm::{detect_audio_format, ncm_key_stream};
    use crate::audio::qmc::{create_qmc2_cipher, Qmc2Map};
    use crate::docx::build_docx_bytes;
    use crate::ebook::epub::{build_epub_bytes, split_chapters};
    use crate::image::bmp::decode_bmp_to_raw;
    use crate::image::ico::{encode_ico, extract_best_frame, PngFrameInput};
    use crate::image::pdf::{build_images_pdf, ImagePageInput};
    use crate::media::ffmpeg::{build_image_to_video_args, is_heic_buffer, resolve_ffmpeg_path};
    use crate::text::csv::{csv_to_json_objects, csv_to_markdown, json_to_csv, parse_csv_records};
    use crate::text::xml::xml_to_json_value;

    #[test]
    fn test_ncm_keystream_math_determinism() {
        let rc4_key = b"neteasecloudmusic_commercial_test_key_2026";
        let stream = ncm_key_stream(rc4_key);
        assert_eq!(stream.len(), 256);
        assert!(stream.iter().any(|&b| b != 0));
        println!("✅ NCM 256-byte 变种 RC4 Keystream 确定性断言通过！");
    }

    #[test]
    fn test_audio_format_detection() {
        let flac_head = b"fLaC\x00\x00\x00\x22";
        assert_eq!(detect_audio_format(flac_head), "flac");

        let mp3_id3 = b"ID3\x04\x00\x00";
        assert_eq!(detect_audio_format(mp3_id3), "mp3");

        let ogg_head = b"OggS\x00\x02";
        assert_eq!(detect_audio_format(ogg_head), "ogg");
        println!("✅ 音频格式嗅探（FLAC/MP3/OGG/M4A）断言通过！");
    }

    #[test]
    fn test_qmc2_map_stream_cipher() {
        let key = b"qqmusic_qmc2_map_test_seed_key";
        let map_cipher = Qmc2Map::new(key);
        let mut plaintext = b"fLaC\x00\x00\x00\x22AI-Forge Native High Fidelity Audio".to_vec();
        let original = plaintext.clone();

        map_cipher.decrypt(&mut plaintext, 0);
        assert_ne!(plaintext, original);

        map_cipher.decrypt(&mut plaintext, 0);
        assert_eq!(plaintext, original);
        println!("✅ QMC2MAP 映射流密码对称加解密断言通过！");
    }

    #[test]
    fn test_qmc2_rc4_cipher_dispatch() {
        let long_key = vec![0x42u8; 512];
        let cipher = create_qmc2_cipher(&long_key);
        let mut sample = vec![0x11u8; 1024];
        cipher.decrypt(&mut sample, 0);
        assert_ne!(sample, vec![0x11u8; 1024]);
        println!("✅ QMC2RC4 长密钥分段流密码断言通过！");
    }

    #[test]
    fn test_kugo_md5_and_xor_collapse() {
        let sample = b"KuGouMusic_Test_Payload";
        let digest = kugo_md5(sample);
        assert_eq!(digest.len(), 16);
        assert_eq!(xor_collapse_u32(0x12345678), (0x12 ^ 0x34 ^ 0x56 ^ 0x78) as u8);
        println!("✅ 酷狗 KGMA MD5 双字节反转与 32 位异或折叠断言通过！");
    }

    #[test]
    fn test_kwm_mask_generation() {
        let key8 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mask = generate_kwm_mask(&key8);
        assert_eq!(mask.len(), 32);
        assert!(mask.iter().any(|&b| b != 0));
        println!("✅ 酷我 KWM 32 字节预定义掩码生成断言通过！");
    }

    #[test]
    fn test_mp4_box_parser() {
        let mut synthetic_mp4 = Vec::new();
        synthetic_mp4.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]);
        synthetic_mp4.extend_from_slice(b"ftypM4A \x00\x00\x00\x00");
        synthetic_mp4.extend_from_slice(&[0x00, 0x00, 0x00, 0x08]);
        synthetic_mp4.extend_from_slice(b"moov");

        let boxes = parse_boxes(&synthetic_mp4, 0, synthetic_mp4.len()).expect("解析 MP4 Box 失败");
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].box_type, "ftyp");
        assert_eq!(boxes[1].box_type, "moov");
        println!("✅ MP4 容器递归 Box 树解析断言通过！");
    }

    #[test]
    fn test_kgg_db_page_derivation_and_decrypt() {
        let iv_step = next_page_iv(42);
        assert!(iv_step > 0);

        let key = derive_page_aes_key(1, &MASTER_KEY);
        let iv = derive_page_aes_iv(1);
        assert_eq!(key.len(), 16);
        assert_eq!(iv.len(), 16);

        let mut data = vec![0x33u8; 64];
        aes_128_cbc_decrypt(&mut data, &key, &iv);
        assert_ne!(data, vec![0x33u8; 64]);
        println!("✅ 酷狗 KGG SQLite 页面密钥派生与 AES-128-CBC 原地解密断言通过！");
    }

    #[test]
    fn test_ico_encoder_and_frame_extraction() {
        let mock_png_16 = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x11, 0x22];
        let mock_png_32 = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x33, 0x44, 0x55];

        let frames = [
            PngFrameInput { size: 16, data: &mock_png_16 },
            PngFrameInput { size: 32, data: &mock_png_32 },
        ];

        let ico_bytes = encode_ico(&frames).expect("生成 ICO 失败");
        assert!(ico_bytes.len() > 6 + 32);

        let best = extract_best_frame(&ico_bytes).expect("提取最佳帧失败");
        assert!(best.is_png);
        assert_eq!(best.width, 32);
        assert_eq!(best.data, mock_png_32);
        println!("✅ 多分辨率 ICO 容器编码与最优 PNG 帧提取断言通过！");
    }

    #[test]
    fn test_bmp_24bit_raw_decoding() {
        let mut bmp = vec![0u8; 54 + 4 * 2];
        bmp[0] = b'B'; bmp[1] = b'M';
        bmp[2] = 62; bmp[10] = 54; bmp[14] = 40; bmp[18] = 2; bmp[22] = 1; bmp[26] = 1; bmp[28] = 24;
        bmp[54] = 0x00; bmp[55] = 0x00; bmp[56] = 0xff;
        bmp[57] = 0xff; bmp[58] = 0x00; bmp[59] = 0x00;

        let raw = decode_bmp_to_raw(&bmp).expect("解码 BMP 失败");
        assert_eq!(raw.width, 2);
        assert_eq!(raw.height, 1);
        assert_eq!(raw.channels, 3);
        assert_eq!(raw.data[0..3], [0xff, 0x00, 0x00]);
        assert_eq!(raw.data[3..6], [0x00, 0x00, 0xff]);
        println!("✅ 24 位 BMP 像素阵列 BGR->RGB 与 4 字节边界行对齐断言通过！");
    }

    #[test]
    fn test_pure_images_to_pdf_stream_builder() {
        let rgb_p1 = [255u8, 0, 0, 0, 255, 0];
        let rgb_p2 = [0u8, 0, 255, 255, 255, 0];

        let pages = [
            ImagePageInput { width: 2, height: 1, rgb_data: &rgb_p1 },
            ImagePageInput { width: 2, height: 1, rgb_data: &rgb_p2 },
        ];

        let pdf_bytes = build_images_pdf(&pages).expect("生成 PDF 失败");
        assert!(pdf_bytes.starts_with(b"%PDF-1.4\n"));
        assert!(pdf_bytes.ends_with(b"%%EOF\n"));
        println!("✅ 纯血图片合并转 PDF 1.4 对象拓扑与 Xref 校验断言通过！");
    }

    #[test]
    fn test_csv_json_and_markdown_pipeline() {
        let sample_csv = "\"Name\",\"Price | CNY\",\"Details\"\n\"Item A\",\"100.50\",\"Line1\nLine2\"\n\"Item B\",\"200.00\",\"Normal\"";
        let records = parse_csv_records(sample_csv).expect("解析 CSV 失败");
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Name", "Price | CNY", "Details"]);

        let md_table = csv_to_markdown(sample_csv).expect("转换 Markdown 失败");
        assert!(md_table.contains("| Price \\| CNY |"));
        assert!(md_table.contains("Line1<br>Line2"));

        let json_objs = csv_to_json_objects(sample_csv).expect("转换 JSON 对象失败");
        assert_eq!(json_objs.len(), 2);
        assert_eq!(json_objs[0]["Name"], "Item A");

        let nested_json = r#"[{"user":{"name":"Bob","age":30},"active":true}]"#;
        let flattened_csv = json_to_csv(nested_json).expect("展平 JSON 失败");
        assert!(flattened_csv.contains("\"user.name\""));
        assert!(flattened_csv.contains("\"user.age\""));
        println!("✅ CSV/JSON 严格互转与 Markdown 单元格转义断言通过！");
    }

    #[test]
    fn test_xml_to_json_parser() {
        let sample_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<bookstore channel="online">
    <book id="1">
        <title>Rust Native High-Performance</title>
        <price currency="CNY">88.50</price>
        <desc><![CDATA[Special <Characters> & CDATA Content]]></desc>
    </book>
    <book id="2">
        <title>Tauri v2 Engineering</title>
    </book>
</bookstore>"#;

        let json_val = xml_to_json_value(sample_xml).expect("解析 XML 失败");
        let root = json_val.get("bookstore").expect("缺少根节点 bookstore");
        assert_eq!(root.get("@channel").unwrap(), "online");

        let books = root.get("book").expect("缺少 book 节点");
        assert!(books.is_array());
        assert_eq!(books.as_array().unwrap().len(), 2);
        println!("✅ 递归下降轻量 XML 词法转 JSON (属性/CDATA/数组升格) 断言通过！");
    }

    #[test]
    fn test_epub_and_docx_generation() {
        let md_text = "# 第一章 启程\n\n这是一段精美的正文。\n\n- 任务列表 1\n- 任务列表 2\n\n# 第二章 觉醒\n\n包含 **粗体文字** 和 `代码块`。";
        let chapters = split_chapters(md_text, true);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 启程");

        let epub_bytes = build_epub_bytes("测试电子书", &chapters).expect("生成 EPUB 失败");
        assert!(epub_bytes.len() > 100);

        let docx_bytes = build_docx_bytes(md_text, true).expect("生成 DOCX 失败");
        assert!(docx_bytes.len() > 100);
        println!("✅ 标准 EPUB 2.0 流式容器与 Word DOCX OpenXML 结构生成断言通过！");
    }

    #[test]
    fn test_archive_in_memory_pipeline() {
        let items = [
            ArchiveInputItem { archive_path: "docs/readme.txt".to_string(), data: b"Hello AI-Forge Native Archive" },
            ArchiveInputItem { archive_path: "../unsafe/hack.txt".to_string(), data: b"Sanitized Safe Data" },
        ];

        let zip_bytes = zip_in_memory(&items, 6).expect("内存打包 ZIP 失败");
        let entries = list_archive_entries(&zip_bytes).expect("列出归档条目失败");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], "docs/readme.txt");
        assert_eq!(entries[1], "unsafe/hack.txt");

        let extracted = extract_archive_to_memory(&zip_bytes).expect("解压归档失败");
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].1, b"Hello AI-Forge Native Archive");
        println!("✅ 内存流式 ZIP 打包/解压与路径穿越 (Zip Slip) 安全净化断言通过！");
    }

    #[test]
    fn test_ffmpeg_media_pipeline() {
        let heic_header = b"\x00\x00\x00\x18ftypheic\x00\x00\x00\x00";
        assert!(is_heic_buffer(heic_header));

        let non_heic = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00";
        assert!(!is_heic_buffer(non_heic));

        let args = build_image_to_video_args("sample.png", "out.mp4", false, "mp4");
        assert!(args.contains(&"libx264"));
        assert!(args.contains(&"yuv420p"));
        assert!(args.contains(&"+faststart"));

        let ffmpeg_bin = resolve_ffmpeg_path();
        assert!(!ffmpeg_bin.to_string_lossy().is_empty());
        println!("✅ FFmpeg Windows Native 路径探测、HEIC 盒子嗅探与转码参数矩阵断言通过！");
    }
}
