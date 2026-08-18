use std::path::{Path, PathBuf};
use std::process::Stdio;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const COLOR_ZIDIAN_PRIMARY: &str = "&H00F8B4D8";      // 紫电主色 #D8B4F8 (上行译文)
pub const COLOR_QINGSHUANG_SECONDARY: &str = "&H00FCF3A5"; // 青霜副色 #A5F3FC (下行原文)
pub const BG_COLOR_SOFT_VIGNETTE: &str = "&H380A0A0A";

pub const SPEAKER_COLORS: &[&str] = &[
    "&H00F8B4D8", "&H00FCF3A5", "&H00FFB56B", "&H008FF286",
    "&H0000D7FF", "&H00DB8EFF", "&H00FFE75B", "&H00D8D8D8",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineSegment {
    pub id: usize,
    pub speaker: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub source_text: String,
    pub target_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedSrtSegment {
    pub id: usize,
    pub speaker: String,
    pub start_sec: f64,
    pub end_sec: f64,
    pub text: String,
}

pub fn resolve_ffmpeg_bin() -> PathBuf {
    let candidates = [
        PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\ffmpeg-x86_64-pc-windows-msvc.exe"),
        PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\ffmpeg.exe"),
        PathBuf::from(r"C:\dev\ai-toolkit\src-tauri\bin\ffmpeg-x86_64-pc-windows-msvc.exe"),
    ];
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    PathBuf::from("ffmpeg")
}

pub fn clean_acoustic_noise(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let re_square = Regex::new(r"(?i)\[(laughter|music|applause|singing|cough|snicker|sigh|groan)\]").unwrap();
    let re_round = Regex::new(r"(?i)\((laughter|music|applause|singing|cough|snicker|sigh|groan)\)").unwrap();
    let s1 = re_square.replace_all(text, "");
    let s2 = re_round.replace_all(&s1, "");
    s2.trim().to_string()
}

pub fn escape_ass_text(text: &str) -> String {
    text.replace('{', "(").replace('}', ")").replace('\n', "\\N")
}

/// 🛡️ 2026 SOTA 贪心自然标点折行算法：严格限制单行 CJK <= 18 汉字 / 英文 <= 38 字符
pub fn smart_wrap_line(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if text.is_empty() || char_count <= max_chars {
        return text.to_string();
    }

    let puncts = ['，', '；', '：', '。', '？', '！', ',', ';', ':', '?', '!'];
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    // 针对空格分词语言（如英文）
    if text.contains(' ') && !text.chars().any(|c| (c as u32) > 0x2E80) {
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            let candidate_len = if current_line.is_empty() {
                word.len()
            } else {
                current_line.len() + 1 + word.len()
            };

            if candidate_len <= max_chars {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        return lines.join("\\N");
    }

    // 针对 CJK 中文无空格语系：基于标点与字数的贪心切分
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let remaining = chars.len() - i;
        if remaining <= max_chars {
            let line: String = chars[i..].iter().collect();
            lines.push(line);
            break;
        }

        // 在 [max_chars - 6, max_chars] 区间内向前寻找最佳标点断点
        let window_end = (i + max_chars).min(chars.len());
        let window_start = (i + max_chars.saturating_sub(6)).max(i);
        let mut split_point = window_end;

        for p_idx in (window_start..window_end).rev() {
            if puncts.contains(&chars[p_idx]) {
                split_point = p_idx + 1; // 包含该标点
                break;
            }
        }

        let line: String = chars[i..split_point].iter().collect();
        lines.push(line);
        i = split_point;
    }

    lines.join("\\N")
}

/// 🛡️ 2026 SOTA 时域等比分句引擎：将超过 26 汉字且时长 > 2.5 秒的超长语流，沿标点等比切分为自然连贯的短句
pub fn split_long_segments_if_needed(segments: &[EngineSegment]) -> Vec<EngineSegment> {
    let mut new_segments = Vec::new();
    let mut current_id = 1;

    for seg in segments {
        let tgt_clean = clean_acoustic_noise(&seg.target_text);
        let src_clean = clean_acoustic_noise(&seg.source_text);
        let duration = seg.end_sec - seg.start_sec;
        let tgt_chars = tgt_clean.chars().count();

        // 当译文字数 > 26 且时长充足时触发等比拆分
        if tgt_chars > 26 && duration >= 2.5 {
            let sentence_puncts = ['。', '！', '？', '；', '!', '?', ';'];
            let mut clauses: Vec<String> = Vec::new();
            let mut buf = String::new();

            for ch in tgt_clean.chars() {
                buf.push(ch);
                if sentence_puncts.contains(&ch) {
                    if buf.chars().count() >= 12 {
                        clauses.push(buf.trim().to_string());
                        buf.clear();
                    }
                }
            }

            if !buf.trim().is_empty() {
                if let Some(last) = clauses.last_mut() {
                    if buf.chars().count() < 8 {
                        last.push_str(&buf);
                    } else {
                        clauses.push(buf.trim().to_string());
                    }
                } else {
                    clauses.push(buf.trim().to_string());
                }
            }

            // 若句号拆不出，退化为按逗号拆分
            if clauses.len() <= 1 && tgt_chars > 32 {
                clauses.clear();
                let sub_puncts = ['，', ','];
                let mut sub_buf = String::new();
                for ch in tgt_clean.chars() {
                    sub_buf.push(ch);
                    if sub_puncts.contains(&ch) && sub_buf.chars().count() >= 14 {
                        clauses.push(sub_buf.trim().to_string());
                        sub_buf.clear();
                    }
                }
                if !sub_buf.trim().is_empty() {
                    clauses.push(sub_buf.trim().to_string());
                }
            }

            if clauses.len() > 1 {
                let total_chars: usize = clauses.iter().map(|c| c.chars().count()).sum();
                let mut accumulated_t = seg.start_sec;

                for (c_idx, clause) in clauses.iter().enumerate() {
                    let c_len = clause.chars().count();
                    let sub_dur = duration * (c_len as f64 / total_chars as f64);
                    let sub_start = accumulated_t;
                    let sub_end = if c_idx == clauses.len() - 1 {
                        seg.end_sec
                    } else {
                        (accumulated_t + sub_dur).min(seg.end_sec)
                    };
                    accumulated_t = sub_end;

                    new_segments.push(EngineSegment {
                        id: current_id,
                        speaker: seg.speaker.clone(),
                        start_sec: (sub_start * 100.0).round() / 100.0,
                        end_sec: (sub_end * 100.0).round() / 100.0,
                        source_text: if c_idx == 0 { src_clean.clone() } else { String::new() },
                        target_text: clause.clone(),
                    });
                    current_id += 1;
                }
                continue;
            }
        }

        // 常规短句保留原样
        new_segments.push(EngineSegment {
            id: current_id,
            speaker: seg.speaker.clone(),
            start_sec: seg.start_sec,
            end_sec: seg.end_sec,
            source_text: src_clean,
            target_text: tgt_clean,
        });
        current_id += 1;
    }

    new_segments
}

pub fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let lines: Vec<&str> = text.split("\\N").collect();
    let mut max_w = 0.0f64;
    for line in lines {
        let mut w = 0.0f64;
        for ch in line.chars() {
            if (ch as u32) > 127 {
                w += font_size * 0.92;
            } else if ".,'!;: ".contains(ch) {
                w += font_size * 0.25;
            } else if ch.is_ascii_uppercase() || "MW@#%".contains(ch) {
                w += font_size * 0.62;
            } else {
                w += font_size * 0.46;
            }
        }
        if w > max_w {
            max_w = w;
        }
    }
    max_w
}

pub fn format_srt_time(seconds: f64) -> String {
    let ms = (seconds * 1000.0).round().max(0.0) as u64;
    let hrs = ms / 3_600_000;
    let rem1 = ms % 3_600_000;
    let mins = rem1 / 60_000;
    let rem2 = rem1 % 60_000;
    let secs = rem2 / 1_000;
    let millis = rem2 % 1_000;
    format!("{:02}:{:02}:{:02},{:03}", hrs, mins, secs, millis)
}

pub fn format_ass_time(seconds: f64) -> String {
    let cs = (seconds * 100.0).round().max(0.0) as u64;
    let hrs = cs / 360_000;
    let rem1 = cs % 360_000;
    let mins = rem1 / 6_000;
    let rem2 = rem1 % 6_000;
    let secs = rem2 / 100;
    let centis = rem2 % 100;
    format!("{}:{:02}:{:02}.{:02}", hrs, mins, secs, centis)
}

pub fn parse_srt_content(srt_text: &str) -> Vec<ParsedSrtSegment> {
    let mut segments = Vec::new();
    let normalized = srt_text.replace("\r\n", "\n");
    let blocks: Vec<&str> = normalized.split("\n\n").collect();
    let time_re = Regex::new(r"(\d{2}):(\d{2}):(\d{2})[,.](\d{3})\s*-->\s*(\d{2}):(\d{2}):(\d{2})[,.](\d{3})").unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();

    let mut seg_id = 1;
    for block in blocks {
        let lines: Vec<&str> = block.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        if lines.is_empty() {
            continue;
        }

        let mut time_line_idx: Option<usize> = None;
        for (idx, line) in lines.iter().enumerate() {
            if line.contains("-->") {
                time_line_idx = Some(idx);
                break;
            }
        }

        let time_line_idx = match time_line_idx {
            Some(idx) => idx,
            None => continue,
        };

        let caps = match time_re.captures(lines[time_line_idx]) {
            Some(c) => c,
            None => continue,
        };

        let h1: f64 = caps[1].parse().unwrap_or(0.0);
        let m1: f64 = caps[2].parse().unwrap_or(0.0);
        let s1: f64 = caps[3].parse().unwrap_or(0.0);
        let ms1: f64 = caps[4].parse().unwrap_or(0.0);

        let h2: f64 = caps[5].parse().unwrap_or(0.0);
        let m2: f64 = caps[6].parse().unwrap_or(0.0);
        let s2: f64 = caps[7].parse().unwrap_or(0.0);
        let ms2: f64 = caps[8].parse().unwrap_or(0.0);

        let start_sec = h1 * 3600.0 + m1 * 60.0 + s1 + ms1 / 1000.0;
        let end_sec = h2 * 3600.0 + m2 * 60.0 + s2 + ms2 / 1000.0;

        let text_lines = &lines[time_line_idx + 1..];
        let raw_text = text_lines.join(" ");
        let clean_text = tag_re.replace_all(&raw_text, "").trim().to_string();

        if !clean_text.is_empty() {
            segments.push(ParsedSrtSegment {
                id: seg_id,
                speaker: "S01".to_string(),
                start_sec: (start_sec * 1000.0).round() / 1000.0,
                end_sec: (end_sec * 1000.0).round() / 1000.0,
                text: clean_text,
            });
            seg_id += 1;
        }
    }
    segments
}

pub fn generate_srt(
    segments: &[EngineSegment],
    display_mode: &str,
    show_speaker: bool,
) -> String {
    let fine_segments = split_long_segments_if_needed(segments);
    let mut blocks = Vec::new();
    let mut valid_idx = 1;

    for seg in &fine_segments {
        let src = clean_acoustic_noise(&seg.source_text);
        let tgt = clean_acoustic_noise(&seg.target_text);
        let spk = if show_speaker && !seg.speaker.is_empty() {
            format!("[{}] ", seg.speaker)
        } else {
            String::new()
        };

        if src.is_empty() && tgt.is_empty() {
            continue;
        }

        let mut lines = Vec::new();
        match display_mode {
            "bilingual" => {
                if !tgt.is_empty() {
                    lines.push(format!("{}{}", spk, smart_wrap_line(&tgt, 18)));
                }
                if !src.is_empty() {
                    lines.push(smart_wrap_line(&src, 36));
                }
            }
            "target_only" => {
                if !tgt.is_empty() {
                    lines.push(format!("{}{}", spk, smart_wrap_line(&tgt, 18)));
                } else if !src.is_empty() {
                    lines.push(format!("{}{}", spk, smart_wrap_line(&src, 36)));
                }
            }
            "source_only" => {
                if !src.is_empty() {
                    lines.push(format!("{}{}", spk, smart_wrap_line(&src, 36)));
                }
            }
            _ => {
                if !tgt.is_empty() {
                    lines.push(format!("{}{}", spk, smart_wrap_line(&tgt, 18)));
                }
            }
        }

        if lines.is_empty() {
            continue;
        }

        let t_start = format_srt_time(seg.start_sec);
        let t_end = format_srt_time(seg.end_sec);
        let block_content = lines.join("\n");
        blocks.push(format!("{}\n{} --> {}\n{}\n", valid_idx, t_start, t_end, block_content));
        valid_idx += 1;
    }

    blocks.join("\n")
}

pub fn generate_ass(
    segments: &[EngineSegment],
    display_mode: &str,
    show_speaker: bool,
    font_size_multiplier: f32,
    video_width: u32,
    video_height: u32,
    mask_hardsub: bool,
) -> String {
    let fine_segments = split_long_segments_if_needed(segments);

    // 🛡️ 字号按 1080p 标准电影视距黄金配比校准
    let base_font_size = (video_height as f32 * 0.038 * font_size_multiplier).round().clamp(24.0, 62.0) as u32;
    let sec_font_size = (video_height as f32 * 0.024 * font_size_multiplier).round().clamp(16.0, 44.0) as u32;

    let margin_l = (video_width as f32 * 0.04).round().max(30.0) as u32;
    let margin_r = margin_l;
    let margin_v = (video_height as f32 * 0.035).round().max(20.0) as u32;

    let mut speakers = std::collections::BTreeSet::new();
    for seg in &fine_segments {
        if !seg.speaker.is_empty() {
            speakers.insert(seg.speaker.clone());
        }
    }
    if speakers.is_empty() {
        speakers.insert("S01".to_string());
    }

    let mut style_lines = vec![
        format!("Style: PrimaryStyle,Arial,{},{},&H000000FF,&H00000000,&H90000000,-1,0,0,0,100,100,0,0,1,2.0,1.0,2,{},{},{},1", base_font_size, COLOR_ZIDIAN_PRIMARY, margin_l, margin_r, margin_v),
        format!("Style: SecondaryStyle,Arial,{},{},&H000000FF,&H00000000,&H90000000,0,0,0,0,100,100,0,0,1,1.8,0.8,2,{},{},{},1", sec_font_size, COLOR_QINGSHUANG_SECONDARY, margin_l, margin_r, margin_v),
    ];

    for (idx, spk) in speakers.iter().enumerate() {
        let color = SPEAKER_COLORS[idx % SPEAKER_COLORS.len()];
        let spk_style_name = format!("Speaker_{}", spk);
        style_lines.push(
            format!("Style: {},Arial,{},{},&H000000FF,&H00000000,&H90000000,-1,0,0,0,100,100,0,0,1,2.0,1.0,2,{},{},{},1", spk_style_name, base_font_size, color, margin_l, margin_r, margin_v)
        );
    }

    style_lines.push(
        "Style: CapsuleMask,Arial,10,&H00000000,&H00000000,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1".to_string()
    );

    let mut dialogue_lines = Vec::new();

    for seg in &fine_segments {
        let raw_src = clean_acoustic_noise(&seg.source_text);
        let raw_tgt = clean_acoustic_noise(&seg.target_text);
        let spk = &seg.speaker;

        if raw_src.is_empty() && raw_tgt.is_empty() {
            continue;
        }

        // CJK 译文单行严格限制在 18 字内，原文单行 36 字符内
        let wrapped_src = smart_wrap_line(&raw_src, 36);
        let wrapped_tgt = smart_wrap_line(&raw_tgt, 18);

        let src = escape_ass_text(&wrapped_src);
        let tgt = escape_ass_text(&wrapped_tgt);

        let spk_prefix = if show_speaker && !spk.is_empty() {
            format!("[{}] ", spk)
        } else {
            String::new()
        };

        let t_start = format_ass_time(seg.start_sec);
        let t_end = format_ass_time(seg.end_sec);

        let style_name = if show_speaker && speakers.contains(spk) {
            format!("Speaker_{}", spk)
        } else {
            "PrimaryStyle".to_string()
        };

        if mask_hardsub {
            let w_tgt = if !tgt.is_empty() {
                estimate_text_width(&format!("{}{}", spk_prefix, tgt), base_font_size as f64)
            } else {
                0.0
            };
            let w_src = if !src.is_empty() {
                estimate_text_width(&src, sec_font_size as f64)
            } else {
                0.0
            };
            let max_text_w = w_tgt.max(w_src);

            let pad_x = (base_font_size as f64 * 0.55).round().max(18.0) as i64;
            let box_w = ((video_width as f64 * 0.85).round() as i64).min(100.max((max_text_w + (pad_x * 2) as f64).round() as i64));

            let has_two_lines = !tgt.is_empty() && !src.is_empty() && display_mode == "bilingual";
            let box_h = if has_two_lines {
                let tgt_lines = tgt.split("\\N").count() as f64;
                let src_lines = src.split("\\N").count() as f64;
                (base_font_size as f64 * 1.25 * tgt_lines + sec_font_size as f64 * 1.25 * src_lines + 10.0).round() as i64
            } else {
                let active_text = if display_mode != "source_only" && !tgt.is_empty() { &tgt } else { &src };
                let lines = active_text.split("\\N").count() as f64;
                (base_font_size as f64 * 1.3 * lines + 8.0).round() as i64
            };

            let box_x0 = 6.max((video_width as i64 - box_w) / 2);
            let box_y0 = 6.max(video_height as i64 - margin_v as i64 - box_h + 2);

            let mask_cmd = format!(
                r"{{\an7\pos({},{})\p1\c&H000000&\1a&H35&\bord0\shad0\blur6}}m 0 0 l {} 0 l {} {} l 0 {}{{\p0}}",
                box_x0, box_y0, box_w, box_w, box_h, box_h
            );
            dialogue_lines.push(format!("Dialogue: 0,{},{},CapsuleMask,,0,0,0,,{}", t_start, t_end, mask_cmd));
        }

        let text_content = match display_mode {
            "bilingual" => {
                if !tgt.is_empty() && !src.is_empty() {
                    format!("{}{}\\N{{\\rSecondaryStyle}}{}", spk_prefix, tgt, src)
                } else if !tgt.is_empty() {
                    format!("{}{}", spk_prefix, tgt)
                } else {
                    format!("{}{}", spk_prefix, src)
                }
            }
            "target_only" => {
                if !tgt.is_empty() {
                    format!("{}{}", spk_prefix, tgt)
                } else {
                    format!("{}{}", spk_prefix, src)
                }
            }
            "source_only" => {
                format!("{}{}", spk_prefix, src)
            }
            _ => {
                format!("{}{}", spk_prefix, tgt)
            }
        };

        dialogue_lines.push(format!("Dialogue: 1,{},{},{},,0,0,0,,{}", t_start, t_end, style_name, text_content));
    }

    let mut header = vec![
        "[Script Info]".to_string(),
        "Title: 紫电 AI 视频双语字幕工坊".to_string(),
        "ScriptType: v4.00+".to_string(),
        "WrapStyle: 0".to_string(),
        "ScaledBorderAndShadow: yes".to_string(),
        format!("PlayResX: {}", video_width),
        format!("PlayResY: {}", video_height),
        "".to_string(),
        "[V4+ Styles]".to_string(),
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding".to_string(),
    ];
    header.extend(style_lines);
    header.push("".to_string());
    header.push("[Events]".to_string());
    header.push("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text".to_string());
    header.extend(dialogue_lines);
    header.push("".to_string());

    header.join("\n")
}

pub async fn extract_audio_from_video(video_path: &Path, out_wav_path: &Path) -> Result<(), String> {
    let ffmpeg_bin = resolve_ffmpeg_bin();
    let status = tokio::process::Command::new(ffmpeg_bin)
        .arg("-y")
        .arg("-i").arg(video_path)
        .arg("-vn")
        .arg("-acodec").arg("pcm_s16le")
        .arg("-ar").arg("16000")
        .arg("-ac").arg("1")
        .arg(out_wav_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("调起 FFmpeg 提取音频失败: {e}"))?;

    if status.success() && out_wav_path.exists() {
        Ok(())
    } else {
        Err("FFmpeg 从视频提取音频 WAV 失败".to_string())
    }
}

pub async fn extract_subtitle_stream(video_path: &Path, stream_index: usize, out_sub_path: &Path) -> Result<Vec<ParsedSrtSegment>, String> {
    let ffmpeg_bin = resolve_ffmpeg_bin();
    let map_arg = format!("0:{}", stream_index);
    let status = tokio::process::Command::new(ffmpeg_bin)
        .arg("-y")
        .arg("-i").arg(video_path)
        .arg("-map").arg(&map_arg)
        .arg(out_sub_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("调起 FFmpeg 抽离字幕流失败: {e}"))?;

    if !status.success() || !out_sub_path.exists() {
        return Err(format!("FFmpeg 抽离字幕流 (index {}) 失败", stream_index));
    }

    let content = tokio::fs::read_to_string(out_sub_path).await
        .map_err(|e| format!("读取提取的字幕文件失败: {e}"))?;

    let segments = parse_srt_content(&content);
    Ok(segments)
}

pub async fn mux_soft_subtitles(video_path: &Path, ass_path: &Path, out_mkv_path: &Path) -> Result<(), String> {
    let ffmpeg_bin = resolve_ffmpeg_bin();
    let status = tokio::process::Command::new(ffmpeg_bin)
        .arg("-y")
        .arg("-i").arg(video_path)
        .arg("-i").arg(ass_path)
        .arg("-c:v").arg("copy")
        .arg("-c:a").arg("copy")
        .arg("-c:s").arg("copy")
        .arg("-metadata:s:0").arg("language=zho")
        .arg("-metadata:s:0").arg("title=紫电 AI 双语字幕")
        .arg("-disposition:s:0").arg("default")
        .arg(out_mkv_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("调起 FFmpeg 混流 MKV 失败: {e}"))?;

    if status.success() && out_mkv_path.exists() {
        Ok(())
    } else {
        Err("0.5s 无损软挂载 MKV 失败".to_string())
    }
}

pub async fn burn_hard_subtitles_nvenc(video_path: &Path, ass_path: &Path, out_mp4_path: &Path) -> Result<(), String> {
    let ffmpeg_bin = resolve_ffmpeg_bin();
    let escaped_ass = ass_path.to_string_lossy().replace('\\', "/").replace(':', "\\:");
    let vf_param = format!("subtitles='{}'", escaped_ass);

    let status_gpu = tokio::process::Command::new(&ffmpeg_bin)
        .arg("-y")
        .arg("-hwaccel").arg("cuda")
        .arg("-i").arg(video_path)
        .arg("-vf").arg(&vf_param)
        .arg("-c:v").arg("h264_nvenc")
        .arg("-preset").arg("p4")
        .arg("-cq").arg("23")
        .arg("-c:a").arg("copy")
        .arg(out_mp4_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    if let Ok(st) = status_gpu {
        if st.success() && out_mp4_path.exists() {
            return Ok(());
        }
    }

    let status_cpu = tokio::process::Command::new(&ffmpeg_bin)
        .arg("-y")
        .arg("-i").arg(video_path)
        .arg("-vf").arg(&vf_param)
        .arg("-c:v").arg("libx264")
        .arg("-crf").arg("23")
        .arg("-preset").arg("fast")
        .arg("-c:a").arg("copy")
        .arg(out_mp4_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("CPU 兜底压制启动失败: {e}"))?;

    if status_cpu.success() && out_mp4_path.exists() {
        Ok(())
    } else {
        Err("硬字幕压制 (NVENC & CPU 兜底) 均失败".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_acoustic_noise() {
        let input = "[laughter] Welcome to AI-Forge (music) project!";
        let cleaned = clean_acoustic_noise(input);
        assert_eq!(cleaned, "Welcome to AI-Forge  project!");
    }

    #[test]
    fn test_time_formatting() {
        assert_eq!(format_srt_time(65.123), "00:01:05,123");
        assert_eq!(format_ass_time(65.123), "0:01:05.12");
    }

    #[test]
    fn test_parse_srt_content() {
        let srt = "1\n00:00:01,000 --> 00:00:03,500\n<i>Hello world</i>\n\n2\n00:00:04,000 --> 00:00:06,000\nRust Native Engine\n";
        let parsed = parse_srt_content(srt);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].start_sec, 1.0);
        assert_eq!(parsed[0].end_sec, 3.5);
        assert_eq!(parsed[0].text, "Hello world");
        assert_eq!(parsed[1].text, "Rust Native Engine");
    }

    #[test]
    fn test_split_long_segments_and_greedy_wrap() {
        let long_seg = EngineSegment {
            id: 42,
            speaker: "S03".to_string(),
            start_sec: 163.35,
            end_sec: 177.31,
            source_text: "いや、俺は料理人じゃなくてただの係長なんだが。とはいえ、これをそのまま出すのはやっぱ気が引けるし、そもそも一人分しかないからな。".to_string(),
            target_text: "不，我并非厨师，只是个主管而已。不过，直接把这个拿出来还是觉得不太合适，而且本来也只够一个人吃而已。得加些东西来做出新的食物才行。".to_string(),
        };

        let splits = split_long_segments_if_needed(&[long_seg]);
        assert!(splits.len() >= 2, "超长 71 字台词应在时域上自然拆分为多句");
        println!("\n🎉 成功将 71 字超长句拆分为 {} 个时域自适应片段:", splits.len());
        for s in &splits {
            println!("  [{:.2}s -> {:.2}s] {}", s.start_sec, s.end_sec, s.target_text);
            assert!(s.target_text.chars().count() <= 38, "拆分后单句不可超长");
        }
    }

    #[test]
    fn test_generate_ass_with_mask() {
        let segs = vec![
            EngineSegment {
                id: 1,
                speaker: "S01".to_string(),
                start_sec: 1.0,
                end_sec: 3.0,
                source_text: "Hello world".to_string(),
                target_text: "你好世界".to_string(),
            }
        ];
        let ass = generate_ass(&segs, "bilingual", true, 1.35, 1920, 1080, true);
        assert!(ass.contains("Style: CapsuleMask"));
        assert!(ass.contains("Dialogue: 0,0:00:01.00,0:00:03.00,CapsuleMask"));
        assert!(ass.contains("Dialogue: 1,0:00:01.00,0:00:03.00,Speaker_S01"));
        assert!(ass.contains(r"[S01] 你好世界\N{\rSecondaryStyle}Hello world"));
    }
}
