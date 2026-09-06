// 🛡️ 紫电 AI 桌面工坊 - 纯血 Vue 3 智能体会话与状态机中枢 (useAgentSession.ts)
// 100% 萃取自 pi-desktop 核心状态机架构，坚守端侧离线与数据隐私防线
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import {
  sendAgentPrompt,
  abortAgentTask,
  listenAgentEvents,
  type AgentEvent
} from '../bindings/agent';

export type AgentPhaseKind = 'idle' | 'waiting_model' | 'running_tools' | 'running_command';

export interface RunningToolInfo {
  callId: string;
  toolName: string;
}

export type AgentPhase =
  | { kind: 'idle' }
  | { kind: 'waiting_model' }
  | { kind: 'running_command' }
  | { kind: 'running_tools'; tools: RunningToolInfo[] };

export interface ToolTrace {
  toolName: string;
  callId: string;
  input: any;
  result?: any;
  status: 'running' | 'success' | 'error';
  timestamp: number;
  durationMs?: number;
}

export interface MessageItem {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  thinking?: string;
  showThinking?: boolean;
  tools: ToolTrace[];
  elapsedMs?: number;
  tokensPerSec?: number;
  timestamp: number;
}

export function useAgentSession(options?: {
  onAgentEnd?: () => void;
  onError?: (err: string) => void;
}) {
  // 1. 核心状态
  const messages = ref<MessageItem[]>([
    {
      id: 'msg_welcome',
      role: 'assistant',
      content: '会话已重置。我是**紫电全血端侧智能体**，所有算力在本地独享，随时待命！',
      showThinking: false,
      tools: [],
      timestamp: Date.now(),
    }
  ]);

  const isGenerating = ref(false);
  const agentPhase = ref<AgentPhase>({ kind: 'idle' });
  const activeArtifactId = ref<string | null>(null);

  // 2. 流速测速与性能统计 (Tokens/s Gauge)
  const streamStartTime = ref<number | null>(null);
  const streamTokenEstimate = ref(0);
  const currentTokensPerSec = ref<number | null>(null);
  let tpsTimer: number | null = null;

  // 3. 智能磁吸贴底滚轮控制器 (Scroll Magnet from pi-desktop)
  const scrollContainerRef = ref<HTMLDivElement | null>(null);
  const liveContentEndRef = ref<HTMLDivElement | null>(null);
  const isAwayFromBottom = ref(false);
  const autoFollowMagnet = ref(true);
  let userScrollIntentUntil = 0;

  let unlistenEvents: (() => void) | null = null;
  let currentAssistantMsg: MessageItem | null = null;

  // 4. 产物提取与注册
  const allArtifacts = computed(() => {
    const list: { msgId: string; tool: ToolTrace }[] = [];
    for (const msg of messages.value) {
      if (msg.tools && msg.tools.length > 0) {
        for (const t of msg.tools) {
          list.push({ msgId: msg.id, tool: t });
        }
      }
    }
    return list;
  });

  const currentArtifact = computed(() => {
    if (allArtifacts.value.length === 0) return null;
    if (activeArtifactId.value) {
      const found = allArtifacts.value.find(a => a.tool.callId === activeArtifactId.value);
      if (found) return found.tool;
    }
    return allArtifacts.value[allArtifacts.value.length - 1].tool;
  });

  const selectArtifact = (callId: string) => {
    activeArtifactId.value = callId;
  };

  // 5. 磁吸滚动算法实现
  const isNearBottom = (container: HTMLElement): boolean => {
    const threshold = 72;
    return container.scrollHeight - container.scrollTop - container.clientHeight <= threshold;
  };

  const updateScrollPresence = () => {
    if (!scrollContainerRef.value) return;
    const near = isNearBottom(scrollContainerRef.value);
    isAwayFromBottom.value = !near;
    if (near && Date.now() > userScrollIntentUntil) {
      autoFollowMagnet.value = true;
    }
  };

  const markUserScrollIntent = (isUpward = false) => {
    userScrollIntentUntil = Date.now() + 1200;
    if (isUpward) {
      autoFollowMagnet.value = false;
    }
  };

  const scrollLiveContentToBottom = () => {
    if (!autoFollowMagnet.value) return;
    liveContentEndRef.value?.scrollIntoView({ behavior: 'auto', block: 'end' });
  };

  const reattachAutoFollow = () => {
    autoFollowMagnet.value = true;
    userScrollIntentUntil = 0;
    isAwayFromBottom.value = false;
    scrollLiveContentToBottom();
  };

  // 6. 流速测速器循环
  const startTpsTracker = () => {
    streamStartTime.value = Date.now();
    streamTokenEstimate.value = 0;
    currentTokensPerSec.value = null;
    if (tpsTimer) clearInterval(tpsTimer);
    tpsTimer = window.setInterval(() => {
      if (!isGenerating.value || !streamStartTime.value) return;
      const elapsedSec = (Date.now() - streamStartTime.value) / 1000;
      if (elapsedSec > 0.4 && streamTokenEstimate.value > 0) {
        currentTokensPerSec.value = Math.round((streamTokenEstimate.value / elapsedSec) * 10) / 10;
      }
    }, 250);
  };

  const stopTpsTracker = (msg?: MessageItem | null) => {
    if (tpsTimer) {
      clearInterval(tpsTimer);
      tpsTimer = null;
    }
    if (msg && currentTokensPerSec.value) {
      msg.tokensPerSec = currentTokensPerSec.value;
    }
    currentTokensPerSec.value = null;
    streamStartTime.value = null;
  };

  // 清洗模型裸露吐出的 <tool_call> 文本标签
  const cleanRawToolCallText = (text: string): string => {
    return text.replace(/<tool_call>[\s\S]*?<\/tool_call>/g, '').trim();
  };

  // 7. 事件泵精准分流
  const handleAgentEvent = (event: AgentEvent) => {
    if (!currentAssistantMsg) return;

    switch (event.type) {
      case 'ThinkingDelta':
        if (!currentAssistantMsg.thinking) {
          currentAssistantMsg.thinking = '';
          currentAssistantMsg.showThinking = true;
        }
        currentAssistantMsg.thinking += event.payload;
        streamTokenEstimate.value += Math.max(1, Math.round(event.payload.length / 3));
        agentPhase.value = { kind: 'waiting_model' };
        scrollLiveContentToBottom();
        break;

      case 'ContentDelta':
        currentAssistantMsg.content += event.payload;
        streamTokenEstimate.value += Math.max(1, Math.round(event.payload.length / 3));
        agentPhase.value = { kind: 'waiting_model' };
        scrollLiveContentToBottom();
        break;

      case 'ToolCallStarted': {
        const newTool: ToolTrace = {
          toolName: event.payload.tool_name,
          callId: event.payload.call_id || `call_${Date.now()}`,
          input: event.payload.input,
          status: 'running',
          timestamp: Date.now(),
        };

        // 避免重复推入同一 callId
        const existingIdx = currentAssistantMsg.tools.findIndex(t => t.callId === newTool.callId);
        if (existingIdx >= 0) {
          currentAssistantMsg.tools[existingIdx] = newTool;
        } else {
          currentAssistantMsg.tools.push(newTool);
        }

        activeArtifactId.value = newTool.callId;
        agentPhase.value = {
          kind: 'running_tools',
          tools: [{ callId: newTool.callId, toolName: newTool.toolName }]
        };
        // 强制触发响应式刷新
        messages.value = [...messages.value];
        scrollLiveContentToBottom();
        break;
      }

      case 'ToolCallFinished': {
        const payloadCallId = event.payload.call_id;
        let target = currentAssistantMsg.tools.find(t => t.callId === payloadCallId);

        // 宽容匹配：若 callId 缺省或匹配不上，直接匹配最新一个 running 状态的工具
        if (!target) {
          target = currentAssistantMsg.tools.slice().reverse().find(t => t.status === 'running');
        }

        if (target) {
          target.result = event.payload.result;
          target.durationMs = Date.now() - target.timestamp;
          target.status =
            event.payload.result?.success === false || event.payload.result?.status === 'error'
              ? 'error'
              : 'success';
          activeArtifactId.value = target.callId;
        }

        agentPhase.value = { kind: 'waiting_model' };
        // 强制触发响应式刷新
        messages.value = [...messages.value];
        scrollLiveContentToBottom();
        break;
      }

      case 'TurnFinished':
        currentAssistantMsg.elapsedMs = event.payload.elapsed_ms;
        // 自动清洗思维链与正文中的裸露 tool_call 脏标签
        if (currentAssistantMsg.thinking) {
          currentAssistantMsg.thinking = cleanRawToolCallText(currentAssistantMsg.thinking);
        }
        if (currentAssistantMsg.content) {
          currentAssistantMsg.content = cleanRawToolCallText(currentAssistantMsg.content);
        }
        stopTpsTracker(currentAssistantMsg);
        agentPhase.value = { kind: 'idle' };
        isGenerating.value = false;
        options?.onAgentEnd?.();
        currentAssistantMsg = null;
        // 强制触发深度响应
        messages.value = [...messages.value];
        scrollLiveContentToBottom();
        break;

      case 'Error':
        options?.onError?.(event.payload);
        stopTpsTracker(currentAssistantMsg);
        agentPhase.value = { kind: 'idle' };
        isGenerating.value = false;
        currentAssistantMsg = null;
        messages.value = [...messages.value];
        break;
    }
  };

  // 8. 用户交互 Action
  const sendMessage = async (promptText: string) => {
    const text = promptText.trim();
    if (!text || isGenerating.value) return;

    // 推入用户提问气泡
    messages.value.push({
      id: `user_${Date.now()}`,
      role: 'user',
      content: text,
      tools: [],
      timestamp: Date.now(),
    });

    isGenerating.value = true;
    agentPhase.value = { kind: 'waiting_model' };
    autoFollowMagnet.value = true;

    // 构建初始助手气泡
    const assistantMsg: MessageItem = {
      id: `assistant_${Date.now()}`,
      role: 'assistant',
      content: '',
      thinking: '',
      showThinking: true,
      tools: [],
      timestamp: Date.now(),
    };

    messages.value.push(assistantMsg);
    currentAssistantMsg = assistantMsg;
    startTpsTracker();

    await nextTick();
    scrollLiveContentToBottom();

    try {
      await sendAgentPrompt(text);
    } catch (err: any) {
      stopTpsTracker(assistantMsg);
      isGenerating.value = false;
      agentPhase.value = { kind: 'idle' };
      currentAssistantMsg = null;
      options?.onError?.(`下发任务失败: ${err}`);
    }
  };

  const abortCurrentTask = async () => {
    try {
      await abortAgentTask();
    } catch (e) {
      console.warn('截停智能体异常:', e);
    } finally {
      stopTpsTracker(currentAssistantMsg);
      isGenerating.value = false;
      agentPhase.value = { kind: 'idle' };
      currentAssistantMsg = null;
      messages.value = [...messages.value];
    }
  };

  const clearSession = () => {
    messages.value = [
      {
        id: 'msg_welcome',
        role: 'assistant',
        content: '会话已重置。我是**紫电全血端侧智能体**，所有算力在本地独享，随时待命！',
        showThinking: false,
        tools: [],
        timestamp: Date.now(),
      }
    ];
    activeArtifactId.value = null;
    agentPhase.value = { kind: 'idle' };
    isGenerating.value = false;
  };

  const toggleThinking = (msgId: string) => {
    const target = messages.value.find(m => m.id === msgId);
    if (target) {
      target.showThinking = !target.showThinking;
    }
  };

  // 9. 生命周期管理
  onMounted(async () => {
    try {
      unlistenEvents = await listenAgentEvents(handleAgentEvent);
    } catch (e) {
      console.warn('注册智能体事件泵异常:', e);
    }
  });

  onUnmounted(() => {
    if (tpsTimer) clearInterval(tpsTimer);
    if (unlistenEvents) {
      unlistenEvents();
      unlistenEvents = null;
    }
  });

  return {
    messages,
    isGenerating,
    agentPhase,
    allArtifacts,
    currentArtifact,
    activeArtifactId,
    currentTokensPerSec,
    isAwayFromBottom,
    scrollContainerRef,
    liveContentEndRef,
    selectArtifact,
    sendMessage,
    abortCurrentTask,
    clearSession,
    toggleThinking,
    updateScrollPresence,
    markUserScrollIntent,
    reattachAutoFollow,
  };
}
