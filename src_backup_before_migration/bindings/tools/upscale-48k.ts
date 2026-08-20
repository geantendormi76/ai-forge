// 🛡️ 紫电 AI 桌面工坊 - 4K/8K 视觉超分专属强类型契约 (upscale-48k.ts)

import { invoke } from "@tauri-apps/api/core";

export interface UpscaleTask {
  /** 待超分输入图像物理绝对路径 */
  input_path: string;
  /** 超分结果输出物理绝对路径 */
  output_path: string;
  /** 可选指定的 ONNX 模型物理路径 */
  model_path?: string | null;
  /** 目标放大倍率 (如 2.0, 4.0, 8.0) */
  target_scale?: number | null;
  /** 智能封顶最大物理边长 (4K 极速: 3840, 8K 旗舰: 8192) */
  max_output_side?: number | null;
  /** 切块网格尺寸 (默认 256 或 512) */
  tile_size?: number | null;
  /** 切块重叠 Padding 消除接缝 (默认 10) */
  tile_pad?: number | null;
}

export interface UpscaleResult {
  /** 是否处理成功 */
  success: boolean;
  /** 源输入图像路径 */
  input_path: string;
  /** 产物输出路径 */
  output_path: string;
  /** 原始物理分辨率 (宽, 高) */
  original_size: [number, number];
  /** 最终超分物理分辨率 (宽, 高) */
  output_size: [number, number];
  /** 实际收敛生效的放大倍率 */
  actual_scale: number;
  /** 端到端物理耗时 (毫秒) */
  elapsed_ms: number;
  /** 切块总数 */
  total_tiles: number;
}

export async function runUpscale48k(task: UpscaleTask): Promise<UpscaleResult> {
  return await invoke<UpscaleResult>("run_upscale_48k", { task });
}
