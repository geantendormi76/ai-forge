// 🛡️ 紫电 AI 桌面工坊 - 全血自主智能体强类型契约 (src/bindings/agent.ts)
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ToolCallStartedPayload {
  tool_name: string;
  call_id: string;
  input: any;
}

export interface ToolCallFinishedPayload {
  tool_name: string;
  call_id: string;
  result: any;
}

export interface TurnFinishedPayload {
  total_tokens?: number | null;
  elapsed_ms: number;
}

export type AgentEvent =
  | { type: "ThinkingDelta"; payload: string }
  | { type: "ContentDelta"; payload: string }
  | { type: "ToolCallStarted"; payload: ToolCallStartedPayload }
  | { type: "ToolCallFinished"; payload: ToolCallFinishedPayload }
  | { type: "TurnFinished"; payload: TurnFinishedPayload }
  | { type: "Error"; payload: string };

export interface GgufModelInfo {
  id: string;
  name: string;
  file_name: string;
  file_path: string;
  size_bytes: number;
  size_formatted: string;
  supports_vision: boolean;
  mmproj_path: string | null;
  is_active: boolean;
}

/** 获取本地所有可切换的 GGUF 大模型资产列表 */
export async function getLocalGgufModels(): Promise<GgufModelInfo[]> {
  return await invoke<GgufModelInfo[]>("get_local_gguf_models");
}

/** 一键热拔插切换主脑模型 */
export async function switchGgufModel(modelId: string): Promise<GgufModelInfo> {
  return await invoke<GgufModelInfo>("switch_gguf_model", { modelId });
}

/** 下发智能体对话或自动化任务 */
export async function sendAgentPrompt(prompt: string): Promise<boolean> {
  return await invoke<boolean>("send_agent_prompt", { prompt });
}

/** 紧急截停当前运行中的智能体 */
export async function abortAgentTask(): Promise<boolean> {
  return await invoke<boolean>("abort_agent_task");
}

/** 注册全局智能体流式事件监听器 */
export async function listenAgentEvents(
  callback: (event: AgentEvent) => void
): Promise<UnlistenFn> {
  return await listen<AgentEvent>("agent-event", (e) => {
    callback(e.payload);
  });
}
