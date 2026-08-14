use std::path::{Path, PathBuf};
use std::process::Command;

pub fn resolve_ffmpeg_path() -> PathBuf {
    if let Ok(env_p) = std::env::var("FFMPEG_PATH") {
        let p = PathBuf::from(env_p);
        if p.exists() {
            return p;
        }
    }

    let candidates = [
        PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\ffmpeg.exe"),
        PathBuf::from(r"src-tauri\bin\ffmpeg.exe"),
        PathBuf::from(r"bin\ffmpeg.exe"),
        PathBuf::from("ffmpeg.exe"),
        PathBuf::from("ffmpeg"),
    ];

    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }

    PathBuf::from("ffmpeg.exe")
}

pub fn is_heic_buffer(header_12b: &[u8]) -> bool {
    if header_12b.len() < 12 {
        return false;
    }
    let box_type = &header_12b[4..8];
    if box_type != b"ftyp" {
        return false;
    }
    let brand = &header_12b[8..12];
    matches!(
        brand,
        b"heic" | b"heif" | b"mif1" | b"heix" | b"heim" | b"HEIC" | b"HEIF"
    )
}

pub fn build_image_to_video_args<'a>(
    input_path: &'a str,
    output_path: &'a str,
    is_gif: bool,
    target_format: &'a str,
) -> Vec<&'a str> {
    let mut args = vec!["-hide_banner", "-y"];
    if is_gif {
        args.extend_from_slice(&["-i", input_path]);
    } else {
        args.extend_from_slice(&["-loop", "1", "-i", input_path, "-t", "3"]);
    }

    args.extend_from_slice(&["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", "-an"]);

    if target_format == "mp4" {
        args.extend_from_slice(&[
            "-codec:v", "libx264", "-preset", "medium", "-crf", "23",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart",
        ]);
    } else {
        args.extend_from_slice(&[
            "-codec:v", "libvpx-vp9", "-crf", "30", "-b:v", "0",
            "-pix_fmt", "yuv420p",
        ]);
    }

    args.push(output_path);
    args
}

pub fn execute_ffmpeg_command(ffmpeg_bin: &Path, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new(ffmpeg_bin);
    cmd.args(args);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = cmd.output().map_err(|e| format!("启动 FFmpeg 进程失败: {}", e))?;
    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg 执行失败: {}", err_msg));
    }

    Ok(())
}
