<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useUIStore } from '../store/uiStore';
import { useAgentSession } from '../composables/useAgentSession';
import { getLocalGgufModels, type GgufModelInfo } from '../bindings/agent';
import SessionSidebar, { type SessionItemData } from '../components/agent/SessionSidebar.vue';
import MessageItem from '../components/agent/MessageItem.vue';
import ChatComposer from '../components/agent/ChatComposer.vue';
import ArtifactCanvas from '../components/agent/ArtifactCanvas.vue';
import ModelHubModal from '../components/modals/ModelHubModal.vue';
import {
  Layers,
  ArrowDown,
  ArrowLeft,
  PanelLeftClose,
  PanelLeftOpen,
  Eye,
  Cpu
} from 'lucide-vue-next';

const ui = useUIStore();
const inputPrompt = ref('');
const showSessionSidebar = ref(true);
const showArtifactDrawer = ref(true);
const isArtifactMaximized = ref(false);
const thinkingEnabled = ref(true);
const currentToolPreset = ref('标准');

// 🛡️ 多模态图像附件管理
const attachedFiles = ref<string[]>([]);

const handleAddFile = (filePath: string) => {
  if (!attachedFiles.value.includes(filePath)) {
    attachedFiles.value.push(filePath);
  }
};

const handleRemoveFile = (index: number) => {
  attachedFiles.value.splice(index, 1);
};

// 🛡️ 模型热拔插中枢状态
const showModelModal = ref(false);
const currentActiveModel = ref<GgufModelInfo | null>(null);

const currentModelDisplayName = computed(() => {
  if (currentActiveModel.value) {
    const rawId = currentActiveModel.value.id.replace('abliterated_', '');
    const shortId = rawId.length > 20 ? rawId.slice(0, 18) + '...' : rawId;
    return currentActiveModel.value.supports_vision
      ? `${shortId} (VLM)`
      : `${shortId}`;
  }
  return 'Qwen3-VL-8B (VLM)';
});

const loadCurrentModel = async () => {
  try {
    const list = await getLocalGgufModels();
    const active = list.find(m => m.is_active) || list[0];
    if (active) {
      currentActiveModel.value = active;
    }
  } catch (err) {
    console.warn('获取当前活动模型失败:', err);
  }
};

const handleModelSwitched = (model: GgufModelInfo) => {
  currentActiveModel.value = model;
  ui.弹出提示(`主脑已热重载为: ${model.id}`, 'success');
};

// 本地会话管理
const sessionList = ref<SessionItemData[]>([
  {
    id: 'sess_default',
    name: '视觉多模态智能体会话',
    firstMessage: '端侧 Qwen3-VL-8B 视觉大模型就绪',
    cwd: 'C:\\dev\\ai-forge',
    modified: Date.now(),
    messageCount: 0,
  }
]);
const activeSessionId = ref<string>('sess_default');

// 初始化组合式状态机中枢
const {
  messages,
  isGenerating,
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
} = useAgentSession({
  onError: (errMsg) => {
    ui.弹出提示(errMsg, 'error');
  }
});

const currentSessionTitle = computed(() => {
  const current = sessionList.value.find(s => s.id === activeSessionId.value);
  return current?.name || current?.firstMessage || 'Pi Agent 桌面会话';
});

const handleNewSession = () => {
  const newId = `sess_${Date.now()}`;
  const newSess: SessionItemData = {
    id: newId,
    name: '新会话',
    firstMessage: '',
    cwd: 'C:\\dev\\ai-forge',
    modified: Date.now(),
    messageCount: 0,
  };
  sessionList.value.unshift(newSess);
  activeSessionId.value = newId;
  clearSession();
  attachedFiles.value = [];
};

const handleSelectSession = (id: string) => {
  activeSessionId.value = id;
};

const handleRenameSession = (id: string, newName: string) => {
  const target = sessionList.value.find(s => s.id === id);
  if (target) {
    target.name = newName;
  }
};

const handleDeleteSession = (id: string) => {
  sessionList.value = sessionList.value.filter(s => s.id !== id);
  if (activeSessionId.value === id) {
    if (sessionList.value.length > 0) {
      activeSessionId.value = sessionList.value[0].id;
    } else {
      handleNewSession();
    }
  }
};

// 核心下发：自动注入 @物理路径 视神经协议
const handleSend = async (text: string) => {
  const trimmed = text.trim();
  if (!trimmed && attachedFiles.value.length === 0) return;
  if (isGenerating.value) return;

  // 1. 组装原生视觉协议前缀
  let finalPrompt = trimmed;
  if (attachedFiles.value.length > 0) {
    const filePrefixes = attachedFiles.value.map(f => `@${f}`).join(' ');
    finalPrompt = trimmed ? `${filePrefixes} ${trimmed}` : `${filePrefixes} 请直接用你的眼睛阅读并详细分析这些图片。`;
  }

  inputPrompt.value = '';
  const sentFiles = [...attachedFiles.value];
  attachedFiles.value = []; // 清空附件输入栏

  const current = sessionList.value.find(s => s.id === activeSessionId.value);
  if (current) {
    if (!current.firstMessage) {
      current.firstMessage = trimmed || `[图片附件] ${sentFiles.map(f => f.split('\\').pop()).join(', ')}`;
    }
    current.modified = Date.now();
    current.messageCount += 2;
  }

  await sendMessage(finalPrompt);
};

const handleWheel = (e: WheelEvent) => {
  if (e.deltaY < 0) {
    markUserScrollIntent(true);
  }
};

onMounted(() => {
  loadCurrentModel();
});
</script>

<template>
  <div class="w-full h-full overflow-hidden bg-[#f7f6f3] dark:bg-[#141210] text-[#1c1a17] dark:text-[#faf9f7] select-none font-sans flex pt-10">
    <!-- ==================== 1. 最左栏：会话管理侧边栏 (260px) ==================== -->
    <div
      v-if="showSessionSidebar"
      class="h-full flex-shrink-0 transition-all duration-200 z-20"
    >
      <SessionSidebar
        :selected-session-id="activeSessionId"
        :sessions="sessionList"
        :is-running="isGenerating"
        @select-session="handleSelectSession"
        @new-session="handleNewSession"
        @rename-session="handleRenameSession"
        @delete-session="handleDeleteSession"
        @open-settings="showModelModal = true"
      />
    </div>

    <!-- ==================== 2. 中间栏：对话流与全能悬浮输入框 ==================== -->
    <div
      class="flex-1 flex flex-col h-full min-w-0 bg-[#f7f6f3] dark:bg-[#141210] relative overflow-hidden"
      :class="isArtifactMaximized ? 'hidden' : ''"
    >
      <!-- 点阵网格纸面背景与四角校准 Tick -->
      <div class="chat-grid-bg pointer-events-none absolute inset-0 z-0" aria-hidden="true" />
      <div class="chat-corner-tick chat-corner-tick-tl" aria-hidden="true" />
      <div class="chat-corner-tick chat-corner-tick-tr" aria-hidden="true" />
      <div class="chat-corner-tick chat-corner-tick-bl" aria-hidden="true" />
      <div class="chat-corner-tick chat-corner-tick-br" aria-hidden="true" />

      <!-- 中间栏顶栏：返回工坊、会话标题与状态指示条 -->
      <header class="h-11 border-b border-[#e4e1da] dark:border-[#33302a] px-4 flex items-center justify-between bg-[#fcfbf9]/90 dark:bg-[#1c1a17]/90 backdrop-blur-md shrink-0 z-10">
        <div class="flex items-center gap-2.5 min-w-0">
          <!-- 返回旧工坊主页胶囊 -->
          <button
            type="button"
            @click="ui.currentView = 'home'"
            class="h-7 px-2.5 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] flex items-center gap-1.5 text-xs font-mono font-bold transition-all cursor-pointer"
            title="返回紫电工坊主页"
          >
            <ArrowLeft :size="12" class="text-[#1764e8]" />
            <span>工坊主页</span>
          </button>

          <!-- 会话侧边栏显隐切换 -->
          <button
            type="button"
            @click="showSessionSidebar = !showSessionSidebar"
            class="w-7 h-7 rounded-lg border border-[#e4e1da] dark:border-[#33302a] text-[#746f66] hover:text-[#1c1a17] hover:bg-[#f0eeea] flex items-center justify-center cursor-pointer transition-colors"
            :title="showSessionSidebar ? '收起会话列表' : '展开会话列表'"
          >
            <PanelLeftClose v-if="showSessionSidebar" :size="13" />
            <PanelLeftOpen v-else :size="13" />
          </button>

          <span class="font-bold text-xs text-[#1c1a17] dark:text-[#faf9f7] truncate font-mono">
            {{ currentSessionTitle }}
          </span>

          <!-- 当前运行模型胶囊 (点击直达热拔插面板) -->
          <button
            type="button"
            @click="showModelModal = true"
            class="px-2 py-0.5 rounded-full bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-[10px] text-[#1764e8] dark:text-[#60a5fa] font-mono font-semibold flex items-center gap-1 cursor-pointer hover:bg-[#cce0ff] transition-all"
            title="点击切换端侧 GGUF 大模型"
          >
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            <Eye v-if="currentActiveModel?.supports_vision" :size="10" />
            <Cpu v-else :size="10" />
            <span>{{ currentModelDisplayName }}</span>
          </button>
        </div>

        <div class="flex items-center gap-2.5 shrink-0">
          <span class="text-[11px] font-mono text-[#746f66] dark:text-[#8f8a81]">
            12GB 显存专线
          </span>

          <!-- 画布开关 -->
          <button
            type="button"
            @click="showArtifactDrawer = !showArtifactDrawer"
            class="h-7 px-2.5 rounded-lg border text-xs font-mono font-bold flex items-center gap-1.5 transition-all cursor-pointer"
            :class="[
              showArtifactDrawer
                ? 'bg-[#1c1a17] dark:bg-[#faf9f7] text-white dark:text-[#1c1a17] border-transparent shadow-xs'
                : 'bg-white dark:bg-[#26231f] border-[#e4e1da] dark:border-[#33302a] text-[#746f66] hover:text-[#1c1a17]'
            ]"
          >
            <Layers :size="12" />
            <span>{{ showArtifactDrawer ? '画布开启' : '画布折叠' }}</span>
            <span v-if="allArtifacts.length > 0" class="px-1.5 py-0.2 rounded-full bg-emerald-500/20 text-emerald-600 text-[9.5px]">
              {{ allArtifacts.length }}
            </span>
          </button>
        </div>
      </header>

      <!-- 消息流主视窗滚动区 -->
      <main
        ref="scrollContainerRef"
        @scroll="updateScrollPresence"
        @wheel="handleWheel"
        class="flex-1 overflow-y-auto p-4 sm:p-6 space-y-6 custom-scrollbar z-10"
      >
        <div class="max-w-[760px] w-full mx-auto space-y-6">
          <MessageItem
            v-for="msg in messages"
            :key="msg.id"
            :message="msg"
            :is-generating="isGenerating"
            :active-artifact-id="activeArtifactId"
            :live-tokens-per-sec="currentTokensPerSec"
            :model-name="currentModelDisplayName"
            @select-artifact="(callId) => { selectArtifact(callId); showArtifactDrawer = true; }"
            @toggle-thinking="(msgId) => toggleThinking(msgId)"
          />
          <!-- 贴底定位锚点 -->
          <div ref="liveContentEndRef" class="h-1 w-full" />
        </div>
      </main>

      <!-- 悬浮磁吸贴底快捷按钮 -->
      <button
        v-if="isAwayFromBottom"
        type="button"
        @click="reattachAutoFollow"
        class="absolute bottom-24 right-8 h-8 px-3 rounded-full bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] text-[#1764e8] text-xs font-mono font-bold flex items-center gap-1.5 shadow-md hover:scale-105 transition-all cursor-pointer z-20"
        title="跳转至最新输出流"
      >
        <ArrowDown :size="12" />
        <span>回到底部</span>
      </button>

      <!-- 底部悬浮式白底 Composer 输入底座 (760px 居中对齐) -->
      <footer class="p-4 pt-1 z-20 shrink-0">
        <div class="max-w-[760px] w-full mx-auto">
          <ChatComposer
            v-model="inputPrompt"
            :is-generating="isGenerating"
            :model-name="currentModelDisplayName"
            :thinking-enabled="thinkingEnabled"
            :tool-preset="currentToolPreset"
            :attached-files="attachedFiles"
            @send="handleSend"
            @abort="abortCurrentTask"
            @toggle-thinking="thinkingEnabled = !thinkingEnabled"
            @toggle-model="showModelModal = true"
            @compact="clearSession"
            @add-file="handleAddFile"
            @remove-file="handleRemoveFile"
          />
        </div>
      </footer>
    </div>

    <!-- ==================== 3. 最右栏：全功能 Tab 产物/文件画布 ==================== -->
    <div
      v-if="showArtifactDrawer"
      class="h-full overflow-hidden transition-all duration-300 ease-in-out border-l border-[#e4e1da] dark:border-[#33302a] flex-shrink-0"
      :class="isArtifactMaximized ? 'w-full' : 'w-[480px] lg:w-[540px] xl:w-[600px]'"
    >
      <ArtifactCanvas
        :current-artifact="currentArtifact"
        :all-artifacts="allArtifacts"
        :is-maximized="isArtifactMaximized"
        @select-artifact="(callId) => selectArtifact(callId)"
        @toggle-maximize="isArtifactMaximized = !isArtifactMaximized"
        @close="showArtifactDrawer = false"
      />
    </div>

    <!-- ==================== 4. 纯血 GGUF 模型热拔插管理模态框 ==================== -->
    <ModelHubModal
      :is-open="showModelModal"
      @close="showModelModal = false"
      @switched="handleModelSwitched"
    />
  </div>
</template>
