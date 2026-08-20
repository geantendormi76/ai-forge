// 🛡️ 紫电 AI 桌面工坊 - 视频双语字幕专属强类型契约模块 (video-subtitle.ts)
// 100% 对齐 Rust 后端极简双轨架构 (video-subtitle / service-asr / service-translation)

import { invoke } from "@tauri-apps/api/core";

/** 视频字幕分类形态：纯生肉音频流 / 内嵌软字幕流 */
export type SubtitleSourceKind = "raw_audio" | "embedded_soft_stream";

/** 内嵌软字幕轨道元数据描述 */
export interface SubtitleTrackInfo {
  stream_index: number;
  codec_name: string;
  language?: string | null;
  title?: string | null;
  is_default: boolean;
}

/** 视频多模态探针诊断报告 */
export interface VideoProbeResult {
  video_path: string;
  duration_sec: number;
  width: number;
  height: number;
  has_audio: boolean;
  audio_streams_count: number;
  soft_tracks: SubtitleTrackInfo[];
  recommended_mode: SubtitleSourceKind;
  elapsed_ms: number;
}

/** 字幕显示模式：双语对照 / 仅目标译文 / 仅原始语言 */
export type SubtitleDisplayMode = "bilingual" | "target_only" | "source_only";

/** 封装输出模式：0.5s 无损软挂载 MKV / NVENC 显卡硬字幕压制 MP4 */
export type SubtitleOutputMode = "soft_mkv" | "hard_mp4_nvenc";

/** 领域专用术语干预表条目 */
export interface GlossaryTerm {
  source_term: string;
  target_term: string;
}

/** 单句识别与翻译结构切片 */
export interface SubtitleSegment {
  id: number;
  speaker: string;
  start_sec: number;
  end_sec: number;
  source_text: string;
  target_text: string;
}

/** 视频双语字幕转写与压制运行参数 */
export interface VideoSubtitleOptions {
  video_path: string;
  output_dir?: string | null;
  target_lang: string;
  display_mode: SubtitleDisplayMode;
  show_speaker: boolean;
  font_size_multiplier: number;
  output_mode: SubtitleOutputMode;
  hotwords?: string | null;
  glossary?: GlossaryTerm[] | null;
  source_kind?: SubtitleSourceKind | null;
  subtitle_stream_index?: number | null;
  /** 电影级柔和羽化半透明遮罩开关 (用于温润遮挡原片硬字幕) */
  mask_hardsub?: boolean;
}

/** 视频双语字幕流水线交付产物 */
export interface VideoSubtitleResult {
  success: boolean;
  output_video_path: string;
  srt_path: string;
  ass_path: string;
  json_path: string;
  total_segments: number;
  segments: SubtitleSegment[];
  elapsed_ms: number;
  error?: string | null;
}

export async function probeVideo(videoPath: string): Promise<VideoProbeResult> {
  return await invoke<VideoProbeResult>("probe_video", { videoPath });
}

export async function runVideoSubtitle(options: VideoSubtitleOptions): Promise<VideoSubtitleResult> {
  return await invoke<VideoSubtitleResult>("run_video_subtitle", { options });
}
