use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug, Clone)]
pub struct RawBitmapBuffer {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<u8>,
}

pub fn is_bmp_buffer(buf: &[u8]) -> bool {
    buf.len() >= 2 && &buf[0..2] == b"BM"
}

pub fn decode_bmp_to_raw(buf: &[u8]) -> Result<RawBitmapBuffer, String> {
    if !is_bmp_buffer(buf) {
        return Err("不是有效的 BMP 文件（缺少 BM 魔数）".into());
    }
    if buf.len() < 54 {
        return Err("BMP 文件头不完整".into());
    }

    let pixel_offset = LittleEndian::read_u32(&buf[10..14]) as usize;
    let dib_size = LittleEndian::read_u32(&buf[14..18]) as usize;

    let (width, height_raw) = if dib_size == 12 {
        (
            LittleEndian::read_u16(&buf[18..20]) as i32,
            LittleEndian::read_u16(&buf[22..24]) as i32,
        )
    } else {
        (
            LittleEndian::read_i32(&buf[18..22]),
            LittleEndian::read_i32(&buf[22..26]),
        )
    };

    let bit_count = if dib_size == 12 {
        LittleEndian::read_u16(&buf[24..26])
    } else {
        LittleEndian::read_u16(&buf[28..30])
    };

    let compression = if dib_size == 12 {
        0
    } else {
        LittleEndian::read_u32(&buf[30..34])
    };

    if width <= 0 || height_raw == 0 || width > 65535 || height_raw.abs() > 65535 {
        return Err(format!("BMP 图像物理尺寸不合法: {}x{}", width, height_raw));
    }
    if compression != 0 {
        return Err("暂不支持压缩型 BMP（仅支持未压缩的标准 1/4/8/24/32 位及调色板 BMP）".into());
    }
    if !matches!(bit_count, 1 | 4 | 8 | 24 | 32) {
        return Err(format!("暂不支持 {} 位 BMP", bit_count));
    }

    let height = height_raw.abs() as usize;
    let width_u = width as usize;
    let top_down = height_raw < 0;
    let row_bytes = ((width_u * bit_count as usize + 31) / 32) * 4;

    let mut palette: Vec<[u8; 3]> = Vec::new();
    if bit_count <= 8 {
        let max_entries = match bit_count {
            1 => 2,
            4 => 16,
            8 => 256,
            _ => 0,
        };
        let declared = if dib_size == 12 {
            0
        } else if buf.len() >= 50 {
            LittleEndian::read_u32(&buf[46..50]) as usize
        } else {
            0
        };

        let palette_entries = if declared > 0 {
            declared.min(max_entries)
        } else {
            max_entries
        };

        let palette_start = 14 + dib_size;
        for i in 0..palette_entries {
            let offset = palette_start + i * 4;
            if offset + 3 <= buf.len() {
                // BGR -> RGB
                palette.push([buf[offset + 2], buf[offset + 1], buf[offset]]);
            } else {
                palette.push([0, 0, 0]);
            }
        }
    }

    let required = pixel_offset + row_bytes * height;
    if buf.len() < required {
        return Err("BMP 像素数据不完整或文件被截断".into());
    }

    let mut output = vec![0u8; width_u * height * 3];

    for row in 0..height {
        let source_row = if top_down { row } else { height - 1 - row };
        let src_start = pixel_offset + source_row * row_bytes;
        let dst_start = row * width_u * 3;

        let mut src_index = src_start;
        let mut dst_index = dst_start;

        match bit_count {
            24 => {
                for _ in 0..width_u {
                    output[dst_index] = buf[src_index + 2];
                    output[dst_index + 1] = buf[src_index + 1];
                    output[dst_index + 2] = buf[src_index];
                    src_index += 3;
                    dst_index += 3;
                }
            }
            32 => {
                for _ in 0..width_u {
                    output[dst_index] = buf[src_index + 2];
                    output[dst_index + 1] = buf[src_index + 1];
                    output[dst_index + 2] = buf[src_index];
                    src_index += 4;
                    dst_index += 3;
                }
            }
            8 => {
                for _ in 0..width_u {
                    let idx = buf[src_index] as usize;
                    let color = palette.get(idx).copied().unwrap_or([0, 0, 0]);
                    output[dst_index] = color[0];
                    output[dst_index + 1] = color[1];
                    output[dst_index + 2] = color[2];
                    src_index += 1;
                    dst_index += 3;
                }
            }
            4 => {
                for col in 0..width_u {
                    let byte = buf[src_index + col / 2];
                    let idx = if col % 2 == 0 { (byte >> 4) & 0x0f } else { byte & 0x0f } as usize;
                    let color = palette.get(idx).copied().unwrap_or([0, 0, 0]);
                    output[dst_index] = color[0];
                    output[dst_index + 1] = color[1];
                    output[dst_index + 2] = color[2];
                    dst_index += 3;
                }
            }
            1 => {
                for col in 0..width_u {
                    let byte = buf[src_index + col / 8];
                    let bit = ((byte >> (7 - (col % 8))) & 0x01) as usize;
                    let color = palette.get(bit).copied().unwrap_or([0, 0, 0]);
                    output[dst_index] = color[0];
                    output[dst_index + 1] = color[1];
                    output[dst_index + 2] = color[2];
                    dst_index += 3;
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(RawBitmapBuffer {
        width: width_u as u32,
        height: height as u32,
        channels: 3,
        data: output,
    })
}
