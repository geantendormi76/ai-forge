// 🛡️ 紫电 AI 桌面工坊 - 端到端强类型契约中枢 (bindings.ts)
// 由 Rust 后端 Schema 严格对齐映射，严禁在前端随意篡改字段命名！

import { invoke } from "@tauri-apps/api/core";

// ==========================================
// 1. 全能格式转换契约 (Format Converter)
// ==========================================

export interface FormatConvertTask {
  /** 待转换源文件物理绝对路径 */
  input_path: string;
  /** 目标扩展名 (如 mp3, flac, json, md, docx, epub, ico, pdf, extract 等) */
  target_format: string;
  /** 可选输出目录，未指定时默认输出到源文件同级目录 */
  output_dir?: string | null;
}

export interface FormatConvertResult {
  /** 是否转换成功 */
  success: boolean;
  /** 源文件路径 */
  input_path: string;
  /** 转换产物输出物理路径 */
  output_path?: string | null;
  /** 实际嗅探/转换的格式 */
  detected_format: string;
  /** 异常或成功描述信息 */
  message?: string | null;
}

// ==========================================
// 2. PDF 智能解析契约 (PDF Parse)
// ==========================================

export interface PdfParseResult {
  file_path: string;
  total_pages: number;
  markdown: string;
  route_label: string;
  elapsed_ms: number;
  download_zip_url?: string | null;
}

// ==========================================
// 3. 视频双语字幕工坊契约 (Video Subtitle)
// ==========================================

export interface SubtitleSegment {
  id: number;
  start_sec: number;
  end_sec: number;
  speaker: string;
  source_text: string;
  target_text?: string | null;
}

export interface VideoSubtitleOptions {
  video_path: string;
  output_dir?: string | null;
  target_lang: string;
  display_mode: string;
  show_speaker: boolean;
  font_size_multiplier: number;
  output_mode: string;
  hotwords?: string | null;
  glossary?: Record<string, string> | null;
}

export interface VideoSubtitleResult {
  success: boolean;
  total_segments: number;
  elapsed_ms: number;
  output_video_path: string;
  srt_path: string;
  ass_path: string;
  segments: SubtitleSegment[];
}

// ==========================================
// 4. 强类型 RPC 命令分发中枢 (Typed Commands)
// ==========================================

export const commands = {
  /**
   * 调起全能格式转换引擎 (纯 CPU 内存直推 / 零显存占用)
   */
  async runFormatConvert(task: FormatConvertTask): Promise<FormatConvertResult> {
    return await invoke<FormatConvertResult>("run_format_convert", { task });
  },

  /**
   * 调起 PDF 混合智能解析流水线
   */
  async parsePdf(filePath: string): Promise<PdfParseResult> {
    return await invoke<PdfParseResult>("parse_pdf", { filePath });
  },

  /**
   * 调起视频双语字幕转写与压制流水线
   */
  async runVideoSubtitle(options: VideoSubtitleOptions): Promise<VideoSubtitleResult> {
    return await invoke<VideoSubtitleResult>("run_video_subtitle", { options });
  },

  /**
   * 获取当前主机的硬件指纹哈希
   */
  async getHardwareFingerprint(): Promise<string> {
    return await invoke<string>("get_hardware_fingerprint");
  },
};
