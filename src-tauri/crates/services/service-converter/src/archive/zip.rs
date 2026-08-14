use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub struct ArchiveInputItem<'a> {
    pub archive_path: String,
    pub data: &'a [u8],
}

pub fn sanitize_archive_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches('/')
        .replace("../", "")
        .replace("..", "")
}

pub fn zip_in_memory(items: &[ArchiveInputItem], compression_level: u8) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    let method = if compression_level == 0 {
        CompressionMethod::Stored
    } else {
        CompressionMethod::Deflated
    };

    let options = SimpleFileOptions::default().compression_method(method);

    for item in items {
        let sanitized = sanitize_archive_path(&item.archive_path);
        if sanitized.is_empty() {
            continue;
        }
        zip.start_file(sanitized, options).map_err(|e| format!("创建归档条目失败: {}", e))?;
        zip.write_all(item.data).map_err(|e| format!("写入归档数据失败: {}", e))?;
    }

    zip.finish().map_err(|e| format!("收尾 ZIP 失败: {}", e))?;
    Ok(buf.into_inner())
}

pub fn list_archive_entries(zip_bytes: &[u8]) -> Result<Vec<String>, String> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开 ZIP 归档失败: {}", e))?;
    let mut names = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| format!("读取条目索引失败: {}", e))?;
        names.push(file.name().to_string());
    }

    Ok(names)
}

pub fn extract_archive_to_memory(zip_bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, String> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开 ZIP 归档失败: {}", e))?;
    let mut extracted = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("解压条目失败: {}", e))?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_string();
        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|e| format!("读取条目内容失败: {}", e))?;
        extracted.push((name, content));
    }

    Ok(extracted)
}
