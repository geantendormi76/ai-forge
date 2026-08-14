use crate::audio::ncm::detect_audio_format;
use base64::prelude::*;
use byteorder::{BigEndian, ByteOrder, LittleEndian};

const TEA_DELTA: u32 = 0x9e3779b9;
const MIX_KEY1: [u8; 16] = [
    0x33, 0x38, 0x36, 0x5a, 0x4a, 0x59, 0x21, 0x40,
    0x23, 0x2a, 0x24, 0x25, 0x5e, 0x26, 0x29, 0x28,
];
const MIX_KEY2: [u8; 16] = [
    0x2a, 0x2a, 0x23, 0x21, 0x28, 0x23, 0x24, 0x25,
    0x26, 0x5e, 0x61, 0x31, 0x63, 0x5a, 0x2c, 0x54,
];

const QMC2_ENCV2_PREFIX: &[u8] = b"QQMusic EncV2,Key:";
const QMC_MAP_BOUNDARY: usize = 0x7fff;
const QMC_MAP_INDEX_OFFSET: usize = 71214;
const QMC_MAP_KEY_SIZE: usize = 128;
const QMC_FIRST_SEGMENT: usize = 0x80;
const QMC_OTHER_SEGMENT: usize = 0x1400;
const QMC_RC4_STREAM_SIZE: usize = QMC_OTHER_SEGMENT + 512;

pub struct TeaCipher {
    k0: u32,
    k1: u32,
    k2: u32,
    k3: u32,
    rounds: u32,
}

impl TeaCipher {
    pub fn new(key: &[u8; 16], rounds: u32) -> Self {
        Self {
            k0: BigEndian::read_u32(&key[0..4]),
            k1: BigEndian::read_u32(&key[4..8]),
            k2: BigEndian::read_u32(&key[8..12]),
            k3: BigEndian::read_u32(&key[12..16]),
            rounds,
        }
    }

    pub fn decrypt_block(&self, src: &[u8], dst: &mut [u8]) {
        let mut v0 = BigEndian::read_u32(&src[0..4]);
        let mut v1 = BigEndian::read_u32(&src[4..8]);
        let mut sum = TEA_DELTA.wrapping_mul(self.rounds / 2);

        for _ in 0..(self.rounds / 2) {
            v1 = v1.wrapping_sub(
                ((v0 << 4).wrapping_add(self.k2))
                    ^ (v0.wrapping_add(sum))
                    ^ ((v0 >> 5).wrapping_add(self.k3)),
            );
            v0 = v0.wrapping_sub(
                ((v1 << 4).wrapping_add(self.k0))
                    ^ (v1.wrapping_add(sum))
                    ^ ((v1 >> 5).wrapping_add(self.k1)),
            );
            sum = sum.wrapping_sub(TEA_DELTA);
        }

        BigEndian::write_u32(&mut dst[0..4], v0);
        BigEndian::write_u32(&mut dst[4..8], v1);
    }
}

pub fn decrypt_tencent_tea(in_buf: &[u8], key: &[u8; 16]) -> Option<Vec<u8>> {
    if in_buf.len() % 8 != 0 || in_buf.len() < 16 {
        return None;
    }

    let cipher = TeaCipher::new(key, 32);
    let mut tmp_buf = [0u8; 8];
    cipher.decrypt_block(&in_buf[0..8], &mut tmp_buf);

    let pad_len = (tmp_buf[0] & 0x07) as usize;
    if in_buf.len() < 1 + pad_len + 2 + 7 {
        return None;
    }
    let out_len = in_buf.len() - 1 - pad_len - 2 - 7;
    let mut out_buf = vec![0u8; out_len];

    let mut iv_prev = [0u8; 8];
    let mut iv_cur = [0u8; 8];
    iv_cur.copy_from_slice(&in_buf[0..8]);

    let mut in_buf_pos = 8;
    let mut tmp_idx = 1 + pad_len;

    let mut i = 1;
    while i <= 2 {
        if tmp_idx < 8 {
            tmp_idx += 1;
            i += 1;
        } else {
            iv_prev = iv_cur;
            iv_cur.copy_from_slice(&in_buf[in_buf_pos..in_buf_pos + 8]);
            for j in 0..8 {
                tmp_buf[j] ^= iv_cur[j];
            }
            let prev_tmp = tmp_buf;
            cipher.decrypt_block(&prev_tmp, &mut tmp_buf);
            in_buf_pos += 8;
            tmp_idx = 0;
        }
    }

    let mut out_buf_pos = 0;
    while out_buf_pos < out_len {
        if tmp_idx < 8 {
            out_buf[out_buf_pos] = tmp_buf[tmp_idx] ^ iv_prev[tmp_idx];
            out_buf_pos += 1;
            tmp_idx += 1;
        } else {
            iv_prev = iv_cur;
            iv_cur.copy_from_slice(&in_buf[in_buf_pos..in_buf_pos + 8]);
            for j in 0..8 {
                tmp_buf[j] ^= iv_cur[j];
            }
            let prev_tmp = tmp_buf;
            cipher.decrypt_block(&prev_tmp, &mut tmp_buf);
            in_buf_pos += 8;
            tmp_idx = 0;
        }
    }

    Some(out_buf)
}

pub struct Qmc2Map {
    key_map: [u8; QMC_MAP_KEY_SIZE],
}

impl Qmc2Map {
    pub fn new(key: &[u8]) -> Self {
        let n = key.len();
        let mut key_map = [0u8; QMC_MAP_KEY_SIZE];
        for i in 0..QMC_MAP_KEY_SIZE {
            let j = (i * i + QMC_MAP_INDEX_OFFSET) % n;
            let shift = (j + 4) % 8;
            let b = key[j];
            key_map[i] = ((b << shift) | (b >> shift)) & 0xff;
        }
        Self { key_map }
    }

    pub fn decrypt(&self, data: &mut [u8], mut offset: usize) {
        for byte in data.iter_mut() {
            let idx = if offset <= QMC_MAP_BOUNDARY {
                offset
            } else {
                offset % QMC_MAP_BOUNDARY
            };
            *byte ^= self.key_map[idx % self.key_map.len()];
            offset += 1;
        }
    }
}

fn qmc2_hash(key: &[u8]) -> u32 {
    let mut hash: u32 = 1;
    for &b in key {
        if b == 0 {
            continue;
        }
        let next = hash.wrapping_mul(b as u32);
        if next <= hash {
            break;
        }
        hash = next;
    }
    hash
}

fn qmc2_segment_key(key_hash: u32, segment_id: usize, seed: u8) -> usize {
    if seed == 0 {
        return 0;
    }
    let denominator = (seed as usize).wrapping_mul(segment_id + 1);
    if denominator == 0 {
        return 0;
    }
    ((key_hash as usize) / denominator) * 100
}

fn rc4_keystream(key: &[u8], len: usize) -> Vec<u8> {
    let n = key.len();
    let mut s = vec![0u8; n];
    for i in 0..n {
        s[i] = i as u8;
    }
    let mut j = 0;
    for i in 0..n {
        j = (j + s[i] as usize + key[i] as usize) % n;
        s.swap(i, j);
    }
    let mut out = vec![0u8; len];
    let mut a = 0;
    let mut b = 0;
    for k in 0..len {
        a = (a + 1) % n;
        b = (b + s[a] as usize) % n;
        s.swap(a, b);
        out[k] = s[(s[a] as usize + s[b] as usize) % n];
    }
    out
}

pub struct Qmc2Rc4 {
    key: Vec<u8>,
    hash: u32,
    stream: Vec<u8>,
}

impl Qmc2Rc4 {
    pub fn new(key: &[u8]) -> Self {
        let hash = qmc2_hash(key);
        let stream = rc4_keystream(key, QMC_RC4_STREAM_SIZE);
        Self {
            key: key.to_vec(),
            hash,
            stream,
        }
    }

    pub fn decrypt(&self, data: &mut [u8], mut offset: usize) {
        let n = self.key.len();
        let mut pos = 0;

        if offset < QMC_FIRST_SEGMENT {
            let process_len = (data.len()).min(QMC_FIRST_SEGMENT - offset);
            for i in 0..process_len {
                let idx = qmc2_segment_key(self.hash, offset, self.key[offset % n]) % n;
                data[i] ^= self.key[idx];
                offset += 1;
            }
            pos = process_len;
        }

        while pos < data.len() {
            let segment_idx = offset / QMC_OTHER_SEGMENT;
            let segment_offset = offset % QMC_OTHER_SEGMENT;
            let skip_len = qmc2_segment_key(self.hash, segment_idx, self.key[segment_idx % n]) & 0x1ff;
            let process_len = (data.len() - pos).min(QMC_OTHER_SEGMENT - segment_offset);
            for i in 0..process_len {
                data[pos + i] ^= self.stream[skip_len + segment_offset + i];
            }
            offset += process_len;
            pos += process_len;
        }
    }
}

pub enum Qmc2Cipher {
    Map(Qmc2Map),
    Rc4(Qmc2Rc4),
}

impl Qmc2Cipher {
    pub fn decrypt(&self, data: &mut [u8], offset: usize) {
        match self {
            Qmc2Cipher::Map(m) => m.decrypt(data, offset),
            Qmc2Cipher::Rc4(r) => r.decrypt(data, offset),
        }
    }
}

pub fn create_qmc2_cipher(key: &[u8]) -> Qmc2Cipher {
    if key.len() < 300 {
        Qmc2Cipher::Map(Qmc2Map::new(key))
    } else {
        Qmc2Cipher::Rc4(Qmc2Rc4::new(key))
    }
}

fn simple_make_key(salt: f64, length: usize) -> Vec<u8> {
    let mut key_buf = Vec::with_capacity(length);
    for i in 0..length {
        let tmp = (salt + (i as f64) * 0.1).tan();
        let val = (tmp.abs() * 100.0) as u32;
        key_buf.push((val & 0xff) as u8);
    }
    key_buf
}

pub fn derive_qmc_key(ekey_binary: &[u8]) -> Option<Vec<u8>> {
    if ekey_binary.len() < 8 {
        return None;
    }
    let simple_key = simple_make_key(106.0, 8);
    let mut tea_key = [0u8; 16];
    for i in 0..8 {
        tea_key[i << 1] = simple_key[i];
        tea_key[(i << 1) + 1] = ekey_binary[i];
    }
    let sub = decrypt_tencent_tea(&ekey_binary[8..], &tea_key)?;
    let mut result = Vec::with_capacity(8 + sub.len());
    result.extend_from_slice(&ekey_binary[..8]);
    result.extend_from_slice(&sub);
    Some(result)
}

pub fn convert_qmc_bytes(buf: &[u8]) -> Result<(Vec<u8>, String), String> {
    if buf.len() < 16 {
        return Err("QMC/MFLAC 文件过小或损坏".into());
    }

    let key_size = LittleEndian::read_u32(&buf[buf.len() - 4..]) as usize;
    if key_size > 0 && key_size <= 0x400 && key_size < buf.len() - 8 {
        let key_start = buf.len() - 4 - key_size;
        let key_region = &buf[key_start..buf.len() - 4];
        
        let mut audio_data = buf[..key_start].to_vec();

        if let Ok(decoded) = BASE64_STANDARD.decode(key_region) {
            if decoded.len() >= QMC2_ENCV2_PREFIX.len() && &decoded[..QMC2_ENCV2_PREFIX.len()] == QMC2_ENCV2_PREFIX {
                if let Some(dec1) = decrypt_tencent_tea(&decoded[QMC2_ENCV2_PREFIX.len()..], &MIX_KEY1) {
                    if let Some(dec2) = decrypt_tencent_tea(&dec1, &MIX_KEY2) {
                        if let Ok(text) = String::from_utf8(dec2) {
                            let nums: Vec<u8> = text.split(',')
                                .filter_map(|s| s.trim().parse::<u8>().ok())
                                .collect();
                            if let Some(final_key) = derive_qmc_key(&nums) {
                                let cipher = create_qmc2_cipher(&final_key);
                                cipher.decrypt(&mut audio_data, 0);
                                let fmt = detect_audio_format(&audio_data);
                                if fmt != "unknown" {
                                    return Ok((audio_data, fmt.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err("无法识别或解密此 QMC/MFLAC 文件".into())
}
