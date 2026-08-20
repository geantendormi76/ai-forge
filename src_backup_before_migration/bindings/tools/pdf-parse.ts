// 🛡️ 紫电 AI 桌面工坊 - PDF 智能解析专属强类型契约 (pdf-parse.ts)
// 100% 对齐 Rust 后端 Native 本地直出结构 (零 Zip / 零 Web 污染)

import { invoke } from "@tauri-apps/api/core";

export interface PdfParseResult {
  /** 是否解析成功 */
  success: boolean;
  /** 高保真 GFM Markdown 完整文本内容 */
  markdown: String;
  /** 本地生成的 Markdown 物理绝对路径 */
  output_md_path: string;
  /** 任务专属产物根目录物理路径 */
  task_out_dir: string;
  /** 图片资源物理目录 (task_out_dir/images) */
  images_dir: string;
  /** 智能分流路由标识 (例如: "🧠 智能分流: 矢量轨 (4页) + 扫描轨 (0页)") */
  route_label: string;
  /** 端到端解析总耗时 (毫秒) */
  elapsed_ms: number;
  /** 错误信息 */
  error?: string | null;
}

export async function parsePdf(filePath: string): Promise<PdfParseResult> {
  return await invoke<PdfParseResult>("parse_pdf", { filePath });
}
