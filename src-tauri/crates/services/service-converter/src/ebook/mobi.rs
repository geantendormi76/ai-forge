use byteorder::{BigEndian, ByteOrder};
use flate2::read::{DeflateDecoder, ZlibDecoder};
use std::io::Read;

pub fn parse_mobi_text(buf: &[u8]) -> Result<String, String> {
    if buf.len() < 78 {
        return Err("MOBI 解析失败：文件头不完整".into());
    }

    let num_records = BigEndian::read_u16(&buf[76..78]) as usize;
    let record_list_offset = 78;
    if record_list_offset + (num_records + 1) * 8 > buf.len() || num_records == 0 || num_records > 20000 {
        return Err("MOBI 解析失败：记录数不合法".into());
    }

    let mut offsets = Vec::with_capacity(num_records + 1);
    for i in 0..=num_records {
        let at = record_list_offset + i * 8;
        offsets.push(BigEndian::read_u32(&buf[at..at + 4]) as usize);
    }

    let record0 = offsets[0];
    if record0 + 16 > buf.len() {
        return Err("MOBI 解析失败：PalmDOC 头部缺失".into());
    }

    let record_count = BigEndian::read_u16(&buf[record0 + 8..record0 + 10]) as usize;
    if record_count == 0 || record_count > num_records {
        return Err("MOBI 解析失败：文本记录数不合法".into());
    }

    let mut decompressed_chunks = Vec::new();

    for index in 1..=record_count {
        if index >= offsets.len() {
            break;
        }
        let start = offsets[index];
        let end = offsets.get(index + 1).copied().unwrap_or(buf.len()).min(buf.len());
        if start >= end || start >= buf.len() {
            break;
        }
        let chunk = &buf[start..end];

        // 4 级解密探测状态机 (Zlib / Raw Deflate / 偏移 Zlib / 明文回退)
        let mut decoded = Vec::new();
        let mut success = false;

        // 尝试 1: 标准 Zlib
        let mut z = ZlibDecoder::new(chunk);
        if z.read_to_end(&mut decoded).is_ok() && !decoded.is_empty() {
            success = true;
        }

        // 尝试 2: Raw Deflate
        if !success {
            decoded.clear();
            let mut d = DeflateDecoder::new(chunk);
            if d.read_to_end(&mut decoded).is_ok() && !decoded.is_empty() {
                success = true;
            }
        }

        // 尝试 3: 跳过 2 字节长度前缀的 Zlib
        if !success && chunk.len() > 2 {
            decoded.clear();
            let mut z = ZlibDecoder::new(&chunk[2..]);
            if z.read_to_end(&mut decoded).is_ok() && !decoded.is_empty() {
                success = true;
            }
        }

        if success {
            decompressed_chunks.extend_from_slice(&decoded);
        } else {
            decompressed_chunks.extend_from_slice(chunk);
        }
    }

    let raw_html = String::from_utf8_lossy(&decompressed_chunks);
    let cleaned = raw_html
        .replace("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "")
        .replace("<html>", "")
        .replace("</html>", "")
        .replace("<body>", "")
        .replace("</body>", "");

    if cleaned.trim().is_empty() {
        return Err("MOBI 解析失败：未提取到任何有效文本".into());
    }

    Ok(cleaned)
}
