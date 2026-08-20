// 🛡️ 紫电 AI 桌面工坊 - 全能格式转换专属强类型契约 (format-converter.ts)

import { invoke } from "@tauri-apps/api/core";

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

export async function runFormatConvert(task: FormatConvertTask): Promise<FormatConvertResult> {
  return await invoke<FormatConvertResult>("run_format_convert", { task });
}
