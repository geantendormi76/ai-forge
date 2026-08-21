// 🛡️ 紫电 AI 桌面工坊 - 算子依赖感知与断点续传强类型契约 (dependencies.ts)
import { invoke } from "@tauri-apps/api/core";

export interface DependencyItem {
  id: string;
  name: string;
  relative_path: string;
  size_bytes: number;
  size_formatted: string;
  sha256: string;
  download_urls: string[];
  is_ready: boolean;
}

export interface DependencyProgressPayload {
  tool_id: string;
  item_id: string;
  item_name: string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  phase: "waiting" | "downloading" | "verifying" | "complete";
}

/** 检查指定算子所需依赖在本地的就绪状态 */
export async function checkToolDependencies(toolId: string): Promise<DependencyItem[]> {
  return await invoke<DependencyItem[]>("check_tool_dependencies", { toolId });
}

/** 触发工具依赖批量流式下载 (支持断点续传) */
export async function downloadToolDependencies(toolId: string): Promise<boolean> {
  return await invoke<boolean>("download_tool_dependencies", { toolId });
}

/** 紧急取消当前正在执行的依赖下载 */
export async function cancelDependencyDownloads(): Promise<boolean> {
  return await invoke<boolean>("cancel_dependency_downloads");
}
