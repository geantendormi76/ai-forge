// 🛡️ 紫电 AI 桌面工坊 - 端到端强类型契约门面中枢 (bindings/index.ts)
import { invoke } from "@tauri-apps/api/core";
import { runFormatConvert } from "./tools/format-converter";
import { parsePdf } from "./tools/pdf-parse";
import { runVideoSubtitle, probeVideo } from "./tools/video-subtitle";
import { runUpscale48k } from "./tools/upscale-48k";
import {
  checkToolDependencies,
  downloadToolDependencies,
  cancelDependencyDownloads,
} from "./dependencies";

export * from "./tools/format-converter";
export * from "./tools/pdf-parse";
export * from "./tools/video-subtitle";
export * from "./tools/upscale-48k";
export * from "./dependencies";

export interface QuotaLogRecord {
  id: number;
  tool_name: string;
  points_deducted: number;
  created_at: string;
}

export interface QuotaStatus {
  success: boolean;
  device_fingerprint: string;
  stage: string;
  daily_limit: number;
  used_today: number;
  bonus_points: number;
  remaining_points: number;
  status: string;
  is_offline_pro: boolean;
  recent_logs?: QuotaLogRecord[];
}

export async function isPortable(): Promise<boolean> {
  return await invoke<boolean>("is_portable");
}

export async function getQuotaStatus(): Promise<QuotaStatus> {
  return await invoke<QuotaStatus>("get_quota_status");
}

export async function getHardwareFingerprint(): Promise<string> {
  return await invoke<string>("get_hardware_fingerprint");
}

export async function cancelCurrentTask(): Promise<boolean> {
  return await invoke<boolean>("cancel_current_task");
}

export const commands = {
  isPortable,
  getQuotaStatus,
  runFormatConvert,
  parsePdf,
  probeVideo,
  runVideoSubtitle,
  runUpscale48k,
  cancelCurrentTask,
  getHardwareFingerprint,
  checkToolDependencies,
  downloadToolDependencies,
  cancelDependencyDownloads,
};
