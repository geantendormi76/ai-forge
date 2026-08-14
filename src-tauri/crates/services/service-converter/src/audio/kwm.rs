use crate::audio::ncm::detect_audio_format;
use byteorder::{ByteOrder, LittleEndian};

pub const KWM_HEADER_SIZE: usize = 0x400;
const KWM_MAGIC_1: &[u8; 16] = b"yeelion-kuwo-tme";
const KWM_MAGIC_2: &[u8; 16] = b"yeelion-kuwo\0\0\0\0";
const KWM_PREDEFINED_KEY: &[u8; 32] = b"MoOtOiTvINGwd2E6n0E1i7L5t2IoOoNk";

pub fn pad_or_truncate(raw: &[u8], length: usize) -> Vec<u8> {
    if raw.is_empty() {
        return vec![0u8; length];
    }
    let mut out = vec![0u8; length];
    if raw.len() >= length {
        out.copy_from_slice(&raw[..length]);
    } else {
        for i in 0..length {
            out[i] = raw[i % raw.len()];
        }
    }
    out
}

pub fn generate_kwm_mask(key8: &[u8; 8]) -> [u8; 32] {
    let key_int = LittleEndian::read_u64(key8);
    let key_str = key_int.to_string();
    let key_bytes = pad_or_truncate(key_str.as_bytes(), 32);
    let mut mask = [0u8; 32];
    for i in 0..32 {
        mask[i] = KWM_PREDEFINED_KEY[i] ^ key_bytes[i];
    }
    mask
}

pub fn convert_kwm_bytes(buf: &[u8]) -> Result<(Vec<u8>, String), String> {
    if buf.len() < KWM_HEADER_SIZE {
        return Err("KWM 文件过小或损坏".into());
    }
    let magic = &buf[0..0x10];
    if magic != KWM_MAGIC_1 && magic != KWM_MAGIC_2 {
        return Err("不是合法的 KWM 加密音频文件".into());
    }

    let mut key8 = [0u8; 8];
    key8.copy_from_slice(&buf[0x18..0x20]);
    let mask = generate_kwm_mask(&key8);

    let mut audio = buf[KWM_HEADER_SIZE..].to_vec();
    for (i, b) in audio.iter_mut().enumerate() {
        *b ^= mask[i & 0x1f];
    }

    let format = detect_audio_format(&audio);
    if format == "unknown" {
        return Err("KWM 解密结果不是可识别的音频格式".into());
    }

    Ok((audio, format.to_string()))
}
