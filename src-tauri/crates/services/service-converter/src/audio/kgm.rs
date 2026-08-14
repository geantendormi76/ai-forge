use crate::audio::ncm::detect_audio_format;
use byteorder::{ByteOrder, LittleEndian};
use md5::{Digest, Md5};

pub const KGM_HEADER: [u8; 16] = [
    0x7c, 0xd5, 0x32, 0xeb, 0x86, 0x02, 0x7f, 0x4b,
    0xa8, 0xaf, 0xa6, 0x8e, 0x0f, 0xff, 0x99, 0x14,
];
const KGM_V3_SLOT2_KEY: [u8; 4] = [0x6c, 0x2c, 0x2f, 0x27];
const KGM_V3_FILE_BOX_SUFFIX: u8 = 0x6b;

pub fn kugo_md5(buffer: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(buffer);
    let digest = hasher.finalize();
    let mut ret = [0u8; 16];
    for i in (0..16).step_by(2) {
        ret[i] = digest[14 - i];
        ret[i + 1] = digest[15 - i];
    }
    ret
}

pub fn xor_collapse_u32(i: u32) -> u8 {
    ((i & 0xff) ^ ((i >> 8) & 0xff) ^ ((i >> 16) & 0xff) ^ ((i >> 24) & 0xff)) as u8
}

pub fn convert_kgma_bytes(buf: &[u8]) -> Result<(Vec<u8>, String), String> {
    if buf.len() < 0x3c {
        return Err("KGMA 文件过小或不完整".into());
    }
    if &buf[0..16] != KGM_HEADER {
        return Err("不是合法的 KGM/KGMA 加密音频文件".into());
    }

    let audio_offset = LittleEndian::read_u32(&buf[0x10..0x14]) as usize;
    let crypto_version = LittleEndian::read_u32(&buf[0x14..0x18]);
    let crypto_slot = LittleEndian::read_u32(&buf[0x18..0x1c]);

    if crypto_version != 3 {
        return Err(format!("暂不支持这个 KGM 版本 (version={})，仅支持 KGMA/v3", crypto_version));
    }
    if crypto_slot != 1 {
        return Err(format!("不支持的加密槽位 (slot={})，仅支持 1", crypto_slot));
    }
    if audio_offset >= buf.len() {
        return Err("KGMA 音频数据偏移越界".into());
    }

    let crypto_key = &buf[0x2c..0x3c];
    let slot_box = kugo_md5(&KGM_V3_SLOT2_KEY);

    let mut file_box = [0u8; 17];
    file_box[0..16].copy_from_slice(&kugo_md5(crypto_key));
    file_box[16] = KGM_V3_FILE_BOX_SUFFIX;

    let mut audio = buf[audio_offset..].to_vec();
    for (i, b) in audio.iter_mut().enumerate() {
        let mut val = *b;
        val ^= file_box[i % 17];
        val ^= (val << 4) & 0xff;
        val ^= slot_box[i % 16];
        val ^= xor_collapse_u32(i as u32);
        *b = val;
    }

    let format = detect_audio_format(&audio);
    if format == "unknown" {
        return Err("KGMA 解密结果不是可识别的音频格式".into());
    }

    Ok((audio, format.to_string()))
}
