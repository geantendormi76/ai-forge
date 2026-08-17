// 🛡️ 紫电 AI 桌面工坊 - PDF 智能解析专属强类型契约 (pdf-parse.ts)

import { invoke } from "@tauri-apps/api/core";

export interface PdfParseResult {
  file_path: string;
  total_pages: number;
  markdown: string;
  route_label: string;
  elapsed_ms: number;
  download_zip_url?: string | null;
}

export async function parsePdf(filePath: string): Promise<PdfParseResult> {
  return await invoke<PdfParseResult>("parse_pdf", { filePath });
}
