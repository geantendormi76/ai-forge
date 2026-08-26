// 🛡️ 紫电 AI 桌面工坊 - 离线高精翻译专属强类型契约 (translation.ts)
// 100% 映射 Rust 后端 translation crate (支持文本与 Base64/物理图片多模态直推)

import { invoke } from "@tauri-apps/api/core";

export interface TranslationTask {
  texts: string[];
  source_lang?: string | null;
  target_lang: string;
  style?: string | null;
  output_file_path?: string | null;
}

export interface TranslationResult {
  success: boolean;
  translations: string[];
  output_file_path?: string | null;
  total_segments: number;
  elapsed_ms: number;
  error?: string | null;
}

export interface ImageTranslationTask {
  image_path?: string | null;
  image_base64?: string | null;
  target_lang: string;
}

export interface ImageTranslationResult {
  success: boolean;
  full_source_text: string;
  full_translated_text: string;
  ocr_elapsed_ms: number;
  trans_elapsed_ms: number;
  elapsed_ms: number;
  error?: string | null;
}

export async function runTranslation(task: TranslationTask): Promise<TranslationResult> {
  return await invoke<TranslationResult>("run_translation", { task });
}

export async function runImageTranslation(task: ImageTranslationTask): Promise<ImageTranslationResult> {
  return await invoke<ImageTranslationResult>("run_image_translation", { task });
}
