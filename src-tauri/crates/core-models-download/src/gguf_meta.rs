//! 🛡️ 紫电 AI - 纯血 GGUF 64KB 极速元数据嗅探引擎 (gguf_meta.rs)
//! 100% 零外部依赖，仅解析 GGUF 头部 Key-Value 元数据，在海量 Tensor 权重前毫秒级截停。

use std::collections::HashMap;
use std::path::Path;

/// GGUF 固定 4 字节魔数 "GGUF" (小端序 u32: 0x46554747)
const GGUF_MAGIC: u32 = 0x4655_4747;

// GGUF 元数据类型标记定义
const T_UINT8: u32 = 0;
const T_INT8: u32 = 1;
const T_UINT16: u32 = 2;
const T_INT16: u32 = 3;
const T_UINT32: u32 = 4;
const T_INT32: u32 = 5;
const T_FLOAT32: u32 = 6;
const T_BOOL: u32 = 7;
const T_STRING: u32 = 8;
const T_ARRAY: u32 = 9;
const T_UINT64: u32 = 10;
const T_INT64: u32 = 11;
const T_FLOAT64: u32 = 12;

const MAX_STRING_LEN: usize = 64 * 1024 * 1024;
const MAX_ARRAY_LEN: u64 = 16 * 1024 * 1024;
const MAX_STORED_ARRAY_LEN: u64 = 4096;
const MAX_KV_COUNT: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum GgufValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
}

impl GgufValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            GgufValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            GgufValue::Bool(b) => Some(*b),
            GgufValue::U8(v) => Some(*v != 0),
            GgufValue::I8(v) => Some(*v != 0),
            GgufValue::U32(v) => Some(*v != 0),
            GgufValue::I32(v) => Some(*v != 0),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            GgufValue::U64(v) => Some(*v),
            GgufValue::U32(v) => Some(*v as u64),
            GgufValue::U16(v) => Some(*v as u64),
            GgufValue::U8(v) => Some(*v as u64),
            GgufValue::I64(v) if *v >= 0 => Some(*v as u64),
            GgufValue::I32(v) if *v >= 0 => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_string_array(&self) -> Option<Vec<String>> {
        match self {
            GgufValue::Array(items) => items.iter().map(|v| v.as_str().map(str::to_string)).collect(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GgufMetadata {
    pub kv: HashMap<String, GgufValue>,
}

impl GgufMetadata {
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.kv.get(key).and_then(GgufValue::as_str)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.kv.get(key).and_then(GgufValue::as_bool)
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.kv.get(key).and_then(GgufValue::as_u64)
    }

    pub fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        self.kv.get(key).and_then(GgufValue::as_string_array)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GgufError {
    NotGguf,
    UnsupportedVersion(u32),
    Truncated { needed: usize },
    Malformed(&'static str),
}

impl std::fmt::Display for GgufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GgufError::NotGguf => write!(f, "非合规 GGUF 文件 (魔数不匹配)"),
            GgufError::UnsupportedVersion(v) => write!(f, "不支持的 GGUF 版本号: {v}"),
            GgufError::Truncated { needed } => write!(f, "缓冲区截断，至少需要 {needed} 字节"),
            GgufError::Malformed(why) => write!(f, "GGUF 头部格式异常: {why}"),
        }
    }
}
impl std::error::Error for GgufError {}

struct ByteCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        ByteCursor { buf, pos: 0 }
    }

    fn truncated(&self, more: usize) -> GgufError {
        GgufError::Truncated {
            needed: self.pos.saturating_add(more),
        }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], GgufError> {
        let end = self.pos.checked_add(n).ok_or(GgufError::Malformed("指针溢出"))?;
        if end > self.buf.len() {
            return Err(self.truncated(n));
        }
        let slice = &self.buf[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32, GgufError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, GgufError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }

    fn string_len(&mut self) -> Result<usize, GgufError> {
        let len = usize::try_from(self.u64()?).map_err(|_| GgufError::Malformed("字符串长度溢出"))?;
        if len > MAX_STRING_LEN {
            return Err(GgufError::Malformed("字符串超出安全上限"));
        }
        Ok(len)
    }

    fn string(&mut self) -> Result<String, GgufError> {
        let len = self.string_len()?;
        let bytes = self.take(len)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    fn skip_string(&mut self) -> Result<(), GgufError> {
        let len = self.string_len()?;
        self.take(len)?;
        Ok(())
    }
}

fn read_value(cur: &mut ByteCursor, value_type: u32) -> Result<GgufValue, GgufError> {
    Ok(match value_type {
        T_UINT8 => GgufValue::U8(cur.take(1)?[0]),
        T_INT8 => GgufValue::I8(cur.take(1)?[0] as i8),
        T_UINT16 => {
            let b = cur.take(2)?;
            GgufValue::U16(u16::from_le_bytes([b[0], b[1]]))
        }
        T_INT16 => {
            let b = cur.take(2)?;
            GgufValue::I16(i16::from_le_bytes([b[0], b[1]]))
        }
        T_UINT32 => GgufValue::U32(cur.u32()?),
        T_INT32 => GgufValue::I32(cur.u32()? as i32),
        T_FLOAT32 => GgufValue::F32(f32::from_bits(cur.u32()?)),
        T_BOOL => GgufValue::Bool(cur.take(1)?[0] != 0),
        T_STRING => GgufValue::String(cur.string()?),
        T_UINT64 => GgufValue::U64(cur.u64()?),
        T_INT64 => GgufValue::I64(cur.u64()? as i64),
        T_FLOAT64 => GgufValue::F64(f64::from_bits(cur.u64()?)),
        T_ARRAY => {
            let elem_type = cur.u32()?;
            if elem_type == T_ARRAY {
                return Err(GgufError::Malformed("禁止多维嵌套数组"));
            }
            let len = cur.u64()?;
            if len > MAX_ARRAY_LEN || len > MAX_STORED_ARRAY_LEN {
                return Err(GgufError::Malformed("数组超长"));
            }
            let mut items = Vec::with_capacity(len.min(512) as usize);
            for _ in 0..len {
                items.push(read_value(cur, elem_type)?);
            }
            GgufValue::Array(items)
        }
        _ => return Err(GgufError::Malformed("未知元数据值类型")),
    })
}

fn scalar_size(value_type: u32) -> Option<usize> {
    match value_type {
        T_UINT8 | T_INT8 | T_BOOL => Some(1),
        T_UINT16 | T_INT16 => Some(2),
        T_UINT32 | T_INT32 | T_FLOAT32 => Some(4),
        T_UINT64 | T_INT64 | T_FLOAT64 => Some(8),
        _ => None,
    }
}

fn skip_value(cur: &mut ByteCursor, value_type: u32) -> Result<(), GgufError> {
    if let Some(size) = scalar_size(value_type) {
        cur.take(size)?;
        return Ok(());
    }

    match value_type {
        T_STRING => cur.skip_string(),
        T_ARRAY => {
            let elem_type = cur.u32()?;
            let len = cur.u64()?;
            if let Some(size) = scalar_size(elem_type) {
                let bytes = usize::try_from(len)
                    .ok()
                    .and_then(|l| l.checked_mul(size))
                    .ok_or(GgufError::Malformed("数组计算溢出"))?;
                cur.take(bytes)?;
            } else if elem_type == T_STRING {
                for _ in 0..len {
                    cur.skip_string()?;
                }
            }
            Ok(())
        }
        _ => Err(GgufError::Malformed("未知跳过类型")),
    }
}

/// 核心函数：仅从内存切片中解析出所关注的 key 列表
pub fn parse_header(bytes: &[u8], wanted_keys: &[&str]) -> Result<GgufMetadata, GgufError> {
    let mut cur = ByteCursor::new(bytes);

    let magic = cur.u32()?;
    if magic != GGUF_MAGIC {
        return Err(GgufError::NotGguf);
    }
    let version = cur.u32()?;
    if version != 2 && version != 3 {
        return Err(GgufError::UnsupportedVersion(version));
    }
    // 跳过 tensor_count 与 kv_count
    cur.u64()?;
    let kv_count = cur.u64()?;
    if kv_count > MAX_KV_COUNT {
        return Err(GgufError::Malformed("异常 KV 数量"));
    }

    let mut kv = HashMap::with_capacity(wanted_keys.len());
    for _ in 0..kv_count {
        let key = cur.string()?;
        let value_type = cur.u32()?;
        if wanted_keys.contains(&key.as_str()) {
            let value = read_value(&mut cur, value_type)?;
            kv.insert(key, value);
            if kv.len() == wanted_keys.len() {
                break;
            }
        } else {
            skip_value(&mut cur, value_type)?;
        }
    }

    Ok(GgufMetadata { kv })
}

/// 🛡️ 极速物理文件嗅探器：默认仅读取头部 64 KB（按需几何级倍增至 16MB）
pub fn probe_file_header(path: &Path, wanted_keys: &[&str]) -> Result<GgufMetadata, GgufError> {
    use std::io::Read;
    const INITIAL_PREFIX: usize = 64 * 1024; // 64 KiB
    const MAX_PREFIX: usize = 16 * 1024 * 1024; // 16 MiB

    let mut size = INITIAL_PREFIX;
    loop {
        let mut file = std::fs::File::open(path).map_err(|_| GgufError::Malformed("无法打开目标模型文件"))?;
        let mut buf = vec![0u8; size];
        let mut filled = 0;
        while filled < buf.len() {
            match file.read(&mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(GgufError::Malformed("读取流中断")),
            }
        }
        buf.truncate(filled);

        match parse_header(&buf, wanted_keys) {
            Ok(meta) => return Ok(meta),
            Err(GgufError::Truncated { needed }) => {
                if buf.len() < size {
                    return Err(GgufError::Malformed("文件体积小于头部声明尺寸"));
                }
                let next = needed.max(size.saturating_mul(2)).min(MAX_PREFIX);
                if next <= size {
                    return Err(GgufError::Truncated { needed });
                }
                size = next;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_synthetic_gguf_parsing() {
        let mut data = Vec::new();
        data.extend_from_slice(&GGUF_MAGIC.to_le_bytes());
        data.extend_from_slice(&3u32.to_le_bytes()); // version 3
        data.extend_from_slice(&0u64.to_le_bytes()); // tensor_count
        data.extend_from_slice(&2u64.to_le_bytes()); // kv_count

        // Key 1: "general.architecture" -> "moss"
        let k1 = "general.architecture";
        data.extend_from_slice(&(k1.len() as u64).to_le_bytes());
        data.extend_from_slice(k1.as_bytes());
        data.extend_from_slice(&T_STRING.to_le_bytes());
        let v1 = "moss";
        data.extend_from_slice(&(v1.len() as u64).to_le_bytes());
        data.extend_from_slice(v1.as_bytes());

        // Key 2: "llama.context_length" -> 4096 (u64)
        let k2 = "llama.context_length";
        data.extend_from_slice(&(k2.len() as u64).to_le_bytes());
        data.extend_from_slice(k2.as_bytes());
        data.extend_from_slice(&T_UINT64.to_le_bytes());
        data.extend_from_slice(&4096u64.to_le_bytes());

        let meta = parse_header(&data, &["general.architecture", "llama.context_length"]).unwrap();
        assert_eq!(meta.get_str("general.architecture"), Some("moss"));
        assert_eq!(meta.get_u64("llama.context_length"), Some(4096));
    }

    #[test]
    fn test_invalid_magic_rejected() {
        let bad_data = b"NOT_A_GGUF_FILE_DATA";
        let res = parse_header(bad_data, &["general.architecture"]);
        assert_eq!(res, Err(GgufError::NotGguf));
    }

    #[test]
    fn test_real_local_models_probe() {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\models\service-translation\Hy-MT2-1.8B-Q4.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\service-asr\MOSS-Transcribe-Diarize-Q5_K_M.gguf"),
        ];

        for path in &candidates {
            if path.exists() {
                let t0 = std::time::Instant::now();
                let meta = probe_file_header(
                    path,
                    &[
                        "general.architecture",
                        "general.name",
                        "llama.context_length",
                        "general.size_label",
                    ],
                )
                .expect("解析本地 GGUF 头部失败");
                let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

                println!("\n🔍 [GGUF 真实大模型 64KB 极速嗅探结果]");
                println!("   • 模型文件:       {:?}", path.file_name().unwrap());
                println!("   • 架构识别:       {:?}", meta.get_str("general.architecture"));
                println!("   • 模型名称:       {:?}", meta.get_str("general.name"));
                println!("   • 上下文限制:     {:?}", meta.get_u64("llama.context_length"));
                println!("   ⏱️ 纯物理嗅探耗时: {:.3} ms (0.001 秒级极速！)", elapsed_ms);

                assert!(elapsed_ms < 300.0, "头部嗅探耗时必须在毫秒级内完成");
            }
        }
    }
}
