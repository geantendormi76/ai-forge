use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128;
use base64::prelude::*;
use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

const CORE_KEY: [u8; 16] = [
    0x68, 0x7a, 0x48, 0x52, 0x41, 0x6d, 0x73, 0x6f,
    0x35, 0x6b, 0x49, 0x6e, 0x62, 0x61, 0x78, 0x57,
];

const META_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6c, 0x6a, 0x6b, 0x5f, 0x21,
    0x5c, 0x5d, 0x26, 0x30, 0x55, 0x3c, 0x27, 0x28,
];

pub const NCM_MAGIC: &[u8] = b"CTENFDAM";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NcmMetadata {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub format: String,
    pub bitrate: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct NcmDecryptedResult {
    pub audio_data: Vec<u8>,
    pub format: String,
    pub metadata: Option<NcmMetadata>,
    pub cover_data: Option<Vec<u8>>,
    pub cover_format: Option<String>,
}

fn aes_128_ecb_decrypt(data: &[u8], key: &[u8; 16]) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut out = data.to_vec();
    for chunk in out.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }
    out
}

fn pkcs7_unpad(data: &[u8]) -> &[u8] {
    if let Some(&pad_len) = data.last() {
        let pad = pad_len as usize;
        if pad > 0 && pad <= 16 && pad <= data.len() {
            if data[data.len() - pad..].iter().all(|&b| b == pad_len) {
                return &data[..data.len() - pad];
            }
        }
    }
    data
}

pub fn ncm_key_stream(rc4key: &[u8]) -> [u8; 256] {
    let mut s = [0u8; 256];
    for i in 0..256 {
        s[i] = i as u8;
    }
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(rc4key[i % rc4key.len()]);
        s.swap(i, j as usize);
    }
    let mut k = [0u8; 256];
    for i in 0..256 {
        let a = (i + 1) & 255;
        let sa = s[a] as usize;
        let b = s[(a + sa) & 255] as usize;
        k[i] = s[(sa + b) & 255];
    }
    k
}

pub fn ncm_decrypt_audio(rc4key: &[u8], data: &mut [u8]) {
    let k = ncm_key_stream(rc4key);
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= k[i & 255];
    }
}

pub fn detect_audio_format(buf: &[u8]) -> &'static str {
    if buf.len() > 3 && &buf[..4] == b"fLaC" {
        return "flac";
    }
    if buf.len() > 2 && &buf[..3] == b"ID3" {
        return "mp3";
    }
    if buf.len() > 3 && &buf[..4] == b"OggS" {
        return "ogg";
    }
    if buf.len() > 11 && &buf[4..8] == b"ftyp" {
        return "m4a";
    }
    if buf.len() > 1 && buf[0] == 0xff && (buf[1] & 0xe0) == 0xe0 {
        return "mp3";
    }
    "unknown"
}

pub fn detect_cover_format(buf: &[u8]) -> Option<String> {
    if buf.len() >= 8 && &buf[..8] == &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a] {
        return Some("png".to_string());
    }
    if buf.len() >= 3 && buf[0] == 0xff && buf[1] == 0xd8 && buf[2] == 0xff {
        return Some("jpg".to_string());
    }
    None
}

fn decode_ncm_meta(meta_data: &[u8]) -> Option<NcmMetadata> {
    let mut xored = meta_data.to_vec();
    for b in &mut xored {
        *b ^= 0x63;
    }
    let xor_str = String::from_utf8(xored).ok()?;
    if xor_str.len() < 22 {
        return None;
    }
    let b64_part = &xor_str[22..];
    let decoded_b64 = BASE64_STANDARD.decode(b64_part).ok()?;
    let decrypted = aes_128_ecb_decrypt(&decoded_b64, &META_KEY);
    let unpadded = pkcs7_unpad(&decrypted);
    if unpadded.len() < 6 {
        return None;
    }
    let json_slice = &unpadded[6..];
    
    #[derive(Deserialize)]
    struct RawMeta {
        #[serde(alias = "musicName", alias = "title")]
        name: Option<String>,
        #[serde(alias = "artist")]
        artists: Option<serde_json::Value>,
        album: Option<String>,
        format: Option<String>,
        bitrate: Option<u32>,
    }

    let raw: RawMeta = serde_json::from_slice(json_slice).ok()?;
    
    let mut artist_list = Vec::new();
    if let Some(artists_val) = raw.artists {
        if let Some(arr) = artists_val.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    artist_list.push(s.to_string());
                } else if let Some(sub_arr) = item.as_array() {
                    if let Some(first_str) = sub_arr.first().and_then(|v| v.as_str()) {
                        artist_list.push(first_str.to_string());
                    }
                } else if let Some(obj) = item.as_object() {
                    if let Some(name) = obj.get("name").or_else(|| obj.get("artistName")).and_then(|v| v.as_str()) {
                        artist_list.push(name.to_string());
                    }
                }
            }
        } else if let Some(s) = artists_val.as_str() {
            artist_list.push(s.to_string());
        }
    }

    Some(NcmMetadata {
        title: raw.name.unwrap_or_default(),
        artists: artist_list,
        album: raw.album.unwrap_or_default(),
        format: raw.format.unwrap_or_else(|| "flac".to_string()),
        bitrate: raw.bitrate,
    })
}

pub fn try_decrypt_ncm(buf: &[u8], key_len_off: usize, key_start: usize) -> Option<NcmDecryptedResult> {
    if key_len_off + 4 > buf.len() {
        return None;
    }
    let key_len = LittleEndian::read_u32(&buf[key_len_off..key_len_off + 4]) as usize;
    if key_len == 0 || key_len > 8192 || key_start + key_len + 4 > buf.len() {
        return None;
    }

    let mut enc = buf[key_start..key_start + key_len].to_vec();
    for b in &mut enc {
        *b ^= 0x64;
    }
    let key_box = aes_128_ecb_decrypt(&enc, &CORE_KEY);
    let unpadded_key_box = pkcs7_unpad(&key_box);
    if unpadded_key_box.len() < 17 {
        return None;
    }
    let rc4key = &unpadded_key_box[17..];
    if rc4key.len() < 4 {
        return None;
    }

    let meta_off = key_start + key_len;
    if meta_off + 4 > buf.len() {
        return None;
    }
    let meta_len = LittleEndian::read_u32(&buf[meta_off..meta_off + 4]) as usize;
    let meta = if meta_len > 0 && meta_off + 4 + meta_len <= buf.len() {
        decode_ncm_meta(&buf[meta_off + 4..meta_off + 4 + meta_len])
    } else {
        None
    };

    let mut audio_off = meta_off + 4 + meta_len + 4 + 5;
    let mut cover_data = None;
    let mut cover_format = None;

    if audio_off + 4 <= buf.len() {
        let cover_len = LittleEndian::read_u32(&buf[audio_off..audio_off + 4]) as usize;
        audio_off += 4;
        if cover_len > 0 && audio_off + cover_len <= buf.len() {
            let cover_slice = &buf[audio_off..audio_off + cover_len];
            if let Some(fmt) = detect_cover_format(cover_slice) {
                cover_data = Some(cover_slice.to_vec());
                cover_format = Some(fmt);
            }
            audio_off += cover_len;
        }
    }

    if audio_off >= buf.len() {
        return None;
    }

    let mut audio_data = buf[audio_off..].to_vec();
    ncm_decrypt_audio(rc4key, &mut audio_data);

    let format = detect_audio_format(&audio_data);
    if format == "unknown" {
        return None;
    }

    Some(NcmDecryptedResult {
        audio_data,
        format: format.to_string(),
        metadata: meta,
        cover_data,
        cover_format,
    })
}

pub fn convert_ncm_bytes(buf: &[u8]) -> Result<NcmDecryptedResult, String> {
    if buf.len() < 8 || &buf[..8] != NCM_MAGIC {
        return Err("不是有效的网易云 NCM 文件（缺少 CTENFDAM 文件头）".to_string());
    }

    let layouts: [(usize, usize); 7] = [
        (10, 14),
        (9, 13),
        (8, 12),
        (8, 10),
        (10, 12),
        (12, 14),
        (12, 16),
    ];

    for (key_len_off, key_start) in layouts {
        if let Some(result) = try_decrypt_ncm(buf, key_len_off, key_start) {
            return Ok(result);
        }
    }

    Err("NCM 解密失败：文件可能已损坏或非标准客户端生成".to_string())
}
