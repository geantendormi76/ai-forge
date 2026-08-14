use byteorder::{BigEndian, ByteOrder};

#[derive(Debug, Clone)]
pub struct Mp4Box {
    pub box_type: String,
    pub start: usize,
    pub body: usize,
    pub end: usize,
}

pub fn parse_boxes(data: &[u8], start: usize, end: usize) -> Result<Vec<Mp4Box>, String> {
    let mut result = Vec::new();
    let mut offset = start;

    while offset + 8 <= end {
        let size_raw = BigEndian::read_u32(&data[offset..offset + 4]) as usize;
        let type_str = match std::str::from_utf8(&data[offset + 4..offset + 8]) {
            Ok(s) => s.to_string(),
            Err(_) => return Err("无效的 MP4 Box 类型编码".into()),
        };

        let (size, header_size) = if size_raw == 1 {
            if offset + 16 > end {
                return Err(format!("扩展 MP4 Box 越界: {}", type_str));
            }
            let ext_size = BigEndian::read_u64(&data[offset + 8..offset + 16]) as usize;
            (ext_size, 16)
        } else if size_raw == 0 {
            (end - offset, 8)
        } else {
            (size_raw, 8)
        };

        if size < header_size || offset + size > end {
            return Err(format!("MP4 Box 尺寸异常: {}", type_str));
        }

        result.push(Mp4Box {
            box_type: type_str,
            start: offset,
            body: offset + header_size,
            end: offset + size,
        });

        offset += size;
    }

    Ok(result)
}

pub fn find_child(data: &[u8], parent: &Mp4Box, target_type: &str) -> Option<Mp4Box> {
    if let Ok(boxes) = parse_boxes(data, parent.body, parent.end) {
        boxes.into_iter().find(|b| b.box_type == target_type)
    } else {
        None
    }
}

pub fn find_path(data: &[u8], parent: &Mp4Box, path_types: &[&str]) -> Option<Mp4Box> {
    let mut current = parent.clone();
    for &target in path_types {
        current = find_child(data, &current, target)?;
    }
    Some(current)
}

fn read_track_codec(data: &[u8], stbl: &Mp4Box) -> Option<String> {
    let stsd = find_child(data, stbl, "stsd")?;
    if stsd.body + 8 > stsd.end {
        return None;
    }
    let count = BigEndian::read_u32(&data[stsd.body + 4..stsd.body + 8]);
    if count == 0 {
        return None;
    }
    let entries = parse_boxes(data, stsd.body + 8, stsd.end).ok()?;
    entries.first().map(|e| e.box_type.clone())
}

#[derive(Debug, Clone)]
pub struct AudioTrackInfo {
    pub codec: String,
    pub stbl: Mp4Box,
}

pub fn find_audio_track(data: &[u8]) -> Result<AudioTrackInfo, String> {
    let root_boxes = parse_boxes(data, 0, data.len())?;
    let moov = root_boxes
        .into_iter()
        .find(|b| b.box_type == "moov")
        .ok_or_else(|| "M4A 文件缺少 moov 容器".to_string())?;

    let traks = parse_boxes(data, moov.body, moov.end)?
        .into_iter()
        .filter(|b| b.box_type == "trak")
        .collect::<Vec<_>>();

    for trak in traks {
        if let Some(stbl) = find_path(data, &trak, &["mdia", "minf", "stbl"]) {
            if let Some(codec) = read_track_codec(data, &stbl) {
                return Ok(AudioTrackInfo { codec, stbl });
            }
        }
    }

    Err("M4A 文件中没有可识别的音频轨道".to_string())
}

pub fn inspect_mp4_audio_codec(data: &[u8]) -> Result<String, String> {
    Ok(find_audio_track(data)?.codec)
}

struct SampleToChunk {
    first_chunk: usize,
    samples_per_chunk: usize,
}

pub fn extract_av3a_stream(data: &[u8]) -> Result<Vec<u8>, String> {
    let track = find_audio_track(data)?;
    if track.codec != "av3a" {
        return Err(format!("M4A 音频编码不是 AV3A (检测到 {})", track.codec));
    }

    let stsz = find_child(data, &track.stbl, "stsz").ok_or("缺少 stsz 样本大小表")?;
    let stsc = find_child(data, &track.stbl, "stsc").ok_or("缺少 stsc 样本分块表")?;
    let chunk_box = find_child(data, &track.stbl, "stco")
        .or_else(|| find_child(data, &track.stbl, "co64"))
        .ok_or("缺少 stco/co64 分块偏移表")?;

    // 1. 解析 Sample Sizes
    if stsz.body + 12 > stsz.end {
        return Err("AV3A stsz 样本大小表不完整".into());
    }
    let fixed_size = BigEndian::read_u32(&data[stsz.body + 4..stsz.body + 8]) as usize;
    let sample_count = BigEndian::read_u32(&data[stsz.body + 8..stsz.body + 12]) as usize;
    let sample_sizes: Vec<usize> = if fixed_size > 0 {
        vec![fixed_size; sample_count]
    } else {
        if stsz.body + 12 + sample_count * 4 > stsz.end {
            return Err("AV3A stsz 样本大小表越界".into());
        }
        (0..sample_count)
            .map(|i| BigEndian::read_u32(&data[stsz.body + 12 + i * 4..stsz.body + 16 + i * 4]) as usize)
            .collect()
    };

    // 2. 解析 Chunk Offsets
    let chunk_count = BigEndian::read_u32(&data[chunk_box.body + 4..chunk_box.body + 8]) as usize;
    let is_64bit = chunk_box.box_type == "co64";
    let entry_width = if is_64bit { 8 } else { 4 };
    if chunk_box.body + 8 + chunk_count * entry_width > chunk_box.end {
        return Err("AV3A 分块偏移表越界".into());
    }
    let chunk_offsets: Vec<usize> = (0..chunk_count)
        .map(|i| {
            let at = chunk_box.body + 8 + i * entry_width;
            if is_64bit {
                BigEndian::read_u64(&data[at..at + 8]) as usize
            } else {
                BigEndian::read_u32(&data[at..at + 4]) as usize
            }
        })
        .collect();

    // 3. 解析 Sample to Chunk
    let mapping_count = BigEndian::read_u32(&data[stsc.body + 4..stsc.body + 8]) as usize;
    if stsc.body + 8 + mapping_count * 12 > stsc.end {
        return Err("AV3A 样本分块表越界".into());
    }
    let mappings: Vec<SampleToChunk> = (0..mapping_count)
        .map(|i| {
            let at = stsc.body + 8 + i * 12;
            SampleToChunk {
                first_chunk: BigEndian::read_u32(&data[at..at + 4]) as usize,
                samples_per_chunk: BigEndian::read_u32(&data[at + 4..at + 8]) as usize,
            }
        })
        .collect();

    if mappings.is_empty() {
        return Err("AV3A 样本分块表为空".into());
    }

    // 4. 时序拼装裸流
    let mut samples_out = Vec::new();
    let mut sample_index = 0;
    let mut mapping_index = 0;

    for chunk_index in 1..=chunk_offsets.len() {
        if mapping_index + 1 < mappings.len() && chunk_index >= mappings[mapping_index + 1].first_chunk {
            mapping_index += 1;
        }

        let mut offset = chunk_offsets[chunk_index - 1];
        let count_in_chunk = mappings[mapping_index].samples_per_chunk;

        for _ in 0..count_in_chunk {
            if sample_index >= sample_sizes.len() {
                break;
            }
            let size = sample_sizes[sample_index];
            if offset + size > data.len() {
                return Err("AV3A 样本数据物理偏移越界".into());
            }
            samples_out.extend_from_slice(&data[offset..offset + size]);
            sample_index += 1;
            offset += size;
        }
    }

    if sample_index != sample_sizes.len() {
        return Err(format!("AV3A 采样数不一致: {}/{}", sample_index, sample_sizes.len()));
    }

    Ok(samples_out)
}
