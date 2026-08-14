use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub const PNG_MAGIC: u32 = 0x89504e47;

#[derive(Debug, Clone)]
pub struct IcoEntry {
    pub width: u32,
    pub height: u32,
    pub bytes: usize,
    pub offset: usize,
    pub is_png: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IcoBestFrame {
    pub is_png: bool,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn is_ico_buffer(buf: &[u8]) -> bool {
    buf.len() >= 6 && LittleEndian::read_u16(&buf[0..2]) == 0 && LittleEndian::read_u16(&buf[2..4]) == 1
}

pub fn parse_ico(buf: &[u8]) -> Result<Vec<IcoEntry>, String> {
    if !is_ico_buffer(buf) {
        return Err("不是有效的 ICO 文件".into());
    }

    let count = LittleEndian::read_u16(&buf[4..6]) as usize;
    if count == 0 || count > 512 {
        return Err("ICO 图像帧数量异常".into());
    }

    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let base = 6 + i * 16;
        if base + 16 > buf.len() {
            return Err("ICO 目录项不完整".into());
        }

        let w_raw = buf[base] as u32;
        let h_raw = buf[base + 1] as u32;
        let width = if w_raw == 0 { 256 } else { w_raw };
        let height = if h_raw == 0 { 256 } else { h_raw };

        let bytes = LittleEndian::read_u32(&buf[base + 8..base + 12]) as usize;
        let offset = LittleEndian::read_u32(&buf[base + 12..base + 16]) as usize;

        if offset + bytes > buf.len() {
            return Err("ICO 图像数据物理越界".into());
        }

        let data = buf[offset..offset + bytes].to_vec();
        let is_png = data.len() >= 4 && BigEndian::read_u32(&data[0..4]) == PNG_MAGIC;

        entries.push(IcoEntry {
            width,
            height,
            bytes,
            offset,
            is_png,
            data,
        });
    }

    Ok(entries)
}

pub fn dib_to_bmp(dib: &[u8]) -> Result<Vec<u8>, String> {
    if dib.len() < 4 {
        return Err("DIB 数据过短".into());
    }
    let dib_size = LittleEndian::read_u32(&dib[0..4]) as usize;
    let mut body = dib.to_vec();

    if dib_size >= 40 && body.len() >= 12 {
        let bi_height = LittleEndian::read_i32(&body[8..12]);
        if bi_height.abs() >= 2 {
            LittleEndian::write_i32(&mut body[8..12], bi_height / 2);
        }
    } else if dib_size == 12 && body.len() >= 8 {
        let bi_height = LittleEndian::read_u16(&body[6..8]);
        if bi_height >= 2 {
            LittleEndian::write_u16(&mut body[6..8], bi_height / 2);
        }
    }

    let bit_count = if dib_size == 12 && body.len() >= 12 {
        LittleEndian::read_u16(&body[10..12])
    } else if body.len() >= 16 {
        LittleEndian::read_u16(&body[14..16])
    } else {
        24
    };

    let palette_bytes = if bit_count <= 8 {
        let count = match bit_count {
            1 => 2,
            4 => 16,
            8 => 256,
            _ => 0,
        };
        count * 4
    } else {
        0
    };

    let pixel_offset = 14 + dib_size + palette_bytes;
    let mut header = vec![0u8; 14];
    header[0] = b'B';
    header[1] = b'M';
    LittleEndian::write_u32(&mut header[2..6], (14 + body.len()) as u32);
    LittleEndian::write_u32(&mut header[6..10], 0);
    LittleEndian::write_u32(&mut header[10..14], pixel_offset as u32);

    let mut full_bmp = header;
    full_bmp.extend_from_slice(&body);
    Ok(full_bmp)
}

pub fn extract_best_frame(buf: &[u8]) -> Result<IcoBestFrame, String> {
    let entries = parse_ico(buf)?;
    if entries.is_empty() {
        return Err("ICO 文件不包含任何有效图像帧".into());
    }

    let mut ranked = entries;
    ranked.sort_by(|a, b| {
        if a.is_png != b.is_png {
            if a.is_png {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            (b.width * b.height).cmp(&(a.width * a.height))
        }
    });

    let best = ranked.into_iter().next().unwrap();
    if best.is_png {
        Ok(IcoBestFrame {
            is_png: true,
            width: best.width,
            height: best.height,
            data: best.data,
        })
    } else {
        let fixed_bmp = dib_to_bmp(&best.data)?;
        Ok(IcoBestFrame {
            is_png: false,
            width: best.width,
            height: best.height,
            data: fixed_bmp,
        })
    }
}

pub struct PngFrameInput<'a> {
    pub size: u16,
    pub data: &'a [u8],
}

pub fn encode_ico(frames: &[PngFrameInput]) -> Result<Vec<u8>, String> {
    let valid_frames: Vec<&PngFrameInput> = frames.iter().filter(|f| !f.data.is_empty()).collect();
    if valid_frames.is_empty() {
        return Err("没有可写入 ICO 的 PNG 图像帧".into());
    }

    let count = valid_frames.len();
    let dir_len = 6 + count * 16;
    let mut dir = vec![0u8; dir_len];

    LittleEndian::write_u16(&mut dir[0..2], 0);
    LittleEndian::write_u16(&mut dir[2..4], 1);
    LittleEndian::write_u16(&mut dir[4..6], count as u16);

    let mut offset = dir_len;
    let mut image_payloads = Vec::new();

    for (i, frame) in valid_frames.iter().enumerate() {
        let base = 6 + i * 16;
        let w = if frame.size >= 256 { 0 } else { frame.size as u8 };
        dir[base] = w;
        dir[base + 1] = w;
        dir[base + 2] = 0;
        dir[base + 3] = 0;
        LittleEndian::write_u16(&mut dir[base + 4..base + 6], 1);
        LittleEndian::write_u16(&mut dir[base + 6..base + 8], 32);
        LittleEndian::write_u32(&mut dir[base + 8..base + 12], frame.data.len() as u32);
        LittleEndian::write_u32(&mut dir[base + 12..base + 16], offset as u32);

        image_payloads.extend_from_slice(frame.data);
        offset += frame.data.len();
    }

    let mut final_ico = dir;
    final_ico.extend_from_slice(&image_payloads);
    Ok(final_ico)
}
