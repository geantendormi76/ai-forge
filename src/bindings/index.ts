// 🛡️ 紫电 AI 桌面工坊 - 端到端强类型契约门面中枢 (bindings/index.ts)

import { invoke } from "@tauri-apps/api/core";
import { runFormatConvert } from "./tools/format-converter";
import { parsePdf } from "./tools/pdf-parse";
import { runVideoSubtitle, probeVideo } from "./tools/video-subtitle";

// 1. 导出各工具领域专有强类型契约与独立调用方法
export * from "./tools/format-converter";
export * from "./tools/pdf-parse";
export * from "./tools/video-subtitle";

// 2. 导出全局公共底座命令
export async function getHardwareFingerprint(): Promise<string> {
  return await invoke<string>("get_hardware_fingerprint");
}

// 3. 统一门面 RPC 命名空间 (Commands Facade)
export const commands = {
  runFormatConvert,
  parsePdf,
  probeVideo,
  runVideoSubtitle,
  getHardwareFingerprint,
};
