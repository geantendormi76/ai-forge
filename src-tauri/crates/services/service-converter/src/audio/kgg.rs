use crate::audio::ncm::detect_audio_format;
use crate::audio::qmc::create_qmc2_cipher;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128;
use byteorder::{ByteOrder, LittleEndian};
use md5::{Digest, Md5};
use std::path::{Path, PathBuf};

pub const DB_PAGE_SIZE: usize = 0x400; // 1024 字节
pub const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
pub const MASTER_KEY: [u8; 16] = [
    0x1d, 0x61, 0x31, 0x45, 0xb2, 0x47, 0xbf, 0x7f,
    0x3d, 0x18, 0x96, 0x72, 0x14, 0x4f, 0xe4, 0xbf,
];

pub fn next_page_iv(seed: u32) -> u32 {
    let left = seed.wrapping_mul(0x9ef4);
    let right = (seed / 0xce26).wrapping_mul(0x7fffff07);
    let value = left.wrapping_sub(right);
    if (value & 0x80000000) == 0 {
        value
    } else {
        value.wrapping_add(0x7fffff07)
    }
}

pub fn derive_page_aes_key(seed: u32, master: &[u8; 16]) -> [u8; 16] {
    let mut buf = [0u8; 0x18];
    buf[0..0x10].copy_from_slice(master);
    LittleEndian::write_u32(&mut buf[0x10..0x14], seed);
    LittleEndian::write_u32(&mut buf[0x14..0x18], 0x546c4173);

    let mut hasher = Md5::new();
    hasher.update(&buf);
    let digest = hasher.finalize();
    let mut key = [0u8; 16];
    key.copy_from_slice(&digest);
    key
}

pub fn derive_page_aes_iv(mut seed: u32) -> [u8; 16] {
    let mut iv_buf = [0u8; 0x10];
    seed = seed.wrapping_add(1);
    for i in (0..0x10).step_by(4) {
        seed = next_page_iv(seed);
        LittleEndian::write_u32(&mut iv_buf[i..i + 4], seed);
    }

    let mut hasher = Md5::new();
    hasher.update(&iv_buf);
    let digest = hasher.finalize();
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&digest);
    iv
}

pub fn aes_128_cbc_decrypt(data: &mut [u8], key: &[u8; 16], iv: &[u8; 16]) {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut prev_block = *iv;
    for chunk in data.chunks_exact_mut(16) {
        let mut current_cipher = [0u8; 16];
        current_cipher.copy_from_slice(chunk);
        let block = GenericArray::from_mut_slice(chunk);
        cipher.decrypt_block(block);
        for i in 0..16 {
            chunk[i] ^= prev_block[i];
        }
        prev_block = current_cipher;
    }
}

pub fn validate_page1_header(header: &[u8]) -> bool {
    if header.len() < 0x18 {
        return false;
    }
    let o10 = LittleEndian::read_u32(&header[0x10..0x14]);
    let o14 = LittleEndian::read_u32(&header[0x14..0x18]);
    let v6 = ((o10 & 0xff) << 8) | ((o10 & 0xff00) << 16);
    o14 == 0x20204000 && v6.wrapping_sub(0x200) <= 0xfe00 && (v6 & (v6.wrapping_sub(1))) == 0
}

pub fn decrypt_pc_database(buffer: &mut [u8]) -> Result<(), String> {
    if buffer.len() >= SQLITE_HEADER.len() && &buffer[0..SQLITE_HEADER.len()] == SQLITE_HEADER {
        return Ok(()); // 已经是未加密明文
    }
    if buffer.is_empty() || buffer.len() % DB_PAGE_SIZE != 0 {
        return Err("酷狗数据库大小不合法（非 1024 字节整数倍）".into());
    }

    // 1. 处理第 1 页
    let mut first_page = [0u8; DB_PAGE_SIZE];
    first_page.copy_from_slice(&buffer[0..DB_PAGE_SIZE]);

    if !validate_page1_header(&first_page) {
        return Err("酷狗数据库第 1 页校验头不合法".into());
    }

    let mut expected_hdr = [0u8; 8];
    expected_hdr.copy_from_slice(&first_page[0x10..0x18]);

    let mut hdr = [0u8; 16];
    hdr.copy_from_slice(&first_page[0..0x10]);
    first_page[0x10..0x18].copy_from_slice(&hdr[0x08..0x10]);

    let key1 = derive_page_aes_key(1, &MASTER_KEY);
    let iv1 = derive_page_aes_iv(1);
    aes_128_cbc_decrypt(&mut first_page[0x10..], &key1, &iv1);

    if &first_page[0x10..0x18] != &expected_hdr {
        return Err("酷狗数据库第 1 页完整性校验失败".into());
    }

    buffer[0..16].copy_from_slice(SQLITE_HEADER);
    buffer[0x10..DB_PAGE_SIZE].copy_from_slice(&first_page[0x10..DB_PAGE_SIZE]);

    // 2. 解密后续所有页面
    let total_pages = buffer.len() / DB_PAGE_SIZE;
    for page_no in 2..=total_pages {
        let start = (page_no - 1) * DB_PAGE_SIZE;
        let end = start + DB_PAGE_SIZE;
        let key = derive_page_aes_key(page_no as u32, &MASTER_KEY);
        let iv = derive_page_aes_iv(page_no as u32);
        aes_128_cbc_decrypt(&mut buffer[start..end], &key, &iv);
    }

    Ok(())
}

pub fn candidate_kugou_db_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(app_data) = std::env::var("APPDATA") {
        candidates.push(Path::new(&app_data).join("KuGou8").join("KGMusicV3.db"));
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(Path::new(&local_app_data).join("KuGou8").join("KGMusicV3.db"));
    }
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        candidates.push(Path::new(&user_profile).join("AppData").join("Roaming").join("KuGou8").join("KGMusicV3.db"));
    }
    candidates
}

pub fn convert_kgg_bytes_with_ekey(buf: &[u8], ekey: &[u8]) -> Result<(Vec<u8>, String), String> {
    if buf.len() < 76 {
        return Err("KGG 文件不完整".into());
    }

    let header_len = LittleEndian::read_u32(&buf[16..20]) as usize;
    let mode = LittleEndian::read_u32(&buf[20..24]);

    if mode != 5 {
        return Err(format!("暂不支持这个 KGG 版本 (mode={})，仅支持 v5", mode));
    }
    if header_len >= buf.len() {
        return Err("KGG 头部长度越界".into());
    }

    let cipher = create_qmc2_cipher(ekey);
    let mut audio = buf[header_len..].to_vec();
    cipher.decrypt(&mut audio, 0);

    let format = detect_audio_format(&audio);
    if format == "unknown" {
        return Err("KGG 解密结果不是可识别的音频格式".into());
    }

    Ok((audio, format.to_string()))
}
