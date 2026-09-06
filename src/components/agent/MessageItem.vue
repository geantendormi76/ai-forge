<script setup lang="ts">
import { ref, computed } from 'vue';
import type { MessageItem } from '../../composables/useAgentSession';
import ProcessGroup from './ProcessGroup.vue';
import { marked } from 'marked';
import {
  Bot,
  User,
  Sparkles,
  ChevronDown,
  ChevronRight,
  Copy,
  Check,
  Clock,
  Zap,
  Activity,
  Loader2
} from 'lucide-vue-next';

const props = withDefaults(
  defineProps<{
    message: MessageItem;
    isGenerating?: boolean;
    activeArtifactId?: string | null;
    liveTokensPerSec?: number | null;
    modelName?: string;
  }>(),
  {
    isGenerating: false,
    activeArtifactId: null,
    liveTokensPerSec: null,
    modelName: 'Qwen3-VL-8B',
  }
);

const emit = defineEmits<{
  (e: 'selectArtifact', callId: string): void;
  (e: 'toggleThinking', messageId: string): void;
}>();

const copiedContent = ref(false);
const copiedThinking = ref(false);

const isUser = computed(() => props.message.role === 'user');
const isAssistant = computed(() => props.message.role === 'assistant');

// 强制解构 tools 列表，阻断响应式死区
const normalizedTools = computed(() => {
  return (props.message.tools || []).map(t => ({ ...t }));
});

// 是否处于纯等待初期（无内容、无思维链）
const isInitialLoading = computed(() => {
  return (
    isAssistant.value &&
    props.isGenerating &&
    !props.message.content &&
    !props.message.thinking &&
    normalizedTools.value.length === 0
  );
});

// 是否正在运行工具
const runningToolName = computed(() => {
  const running = normalizedTools.value.find(t => t.status === 'running');
  return running ? running.toolName : null;
});

const getToolDisplayLabel = (name: string): string => {
  const map: Record<string, string> = {
    native_upscale_image: '4K/8K 视觉超分重构',
    ocr_recognize_image: 'PP-OCRv6 视觉文字提取',
    native_neural_translate: 'Hy-MT2 神经机器翻译',
    pdf_get_meta: 'PDF 物理元数据探测',
    pdf_render_page_image: 'PDF 页面高清渲染',
    pdf_extract_raw_text: 'PDF 原生纯文本提取',
    formula_recognize_latex: 'PP-Formula 数学公式转写',
    table_recognize_structure: 'SLANet 复杂表格重构',
    native_format_convert: '全能格式解密转换',
    native_doc_parse: '多模态文档深度重构',
    execute_python_sandbox: 'Python 动态数据沙箱',
    write: '本地文件持久化写入',
    read: '本地文件探针读取',
  };
  return map[name] || name;
};

// 实时或历史 TPS 吐字速率
const effectiveTps = computed(() => {
  if (props.isGenerating && props.liveTokensPerSec) {
    return props.liveTokensPerSec;
  }
  return props.message.tokensPerSec || null;
});

// TPS 速率动态色彩药丸
const tpsBadgeClass = computed(() => {
  const tps = effectiveTps.value;
  if (!tps) return 'bg-[#f0eeea] dark:bg-[#26231f] text-[#746f66] dark:text-[#8f8a81] border-[#e4e1da] dark:border-[#33302a]';
  if (tps >= 40) return 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/40';
  if (tps >= 20) return 'bg-blue-50 dark:bg-blue-950/40 text-[#1764e8] dark:text-blue-400 border-blue-200 dark:border-blue-800/40';
  return 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-400 border-amber-200 dark:border-amber-800/40';
});

const formattedTime = computed(() => {
  if (!props.message.timestamp) return '';
  const d = new Date(props.message.timestamp);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
});

const renderMarkdown = (text: string): string => {
  try {
    return marked.parse(text || '') as string;
  } catch {
    return text;
  }
};

const copyText = async (text: string, type: 'content' | 'thinking') => {
  if (!text) return;
  await navigator.clipboard.writeText(text);
  if (type === 'content') {
    copiedContent.value = true;
    setTimeout(() => { copiedContent.value = false; }, 2000);
  } else {
    copiedThinking.value = true;
    setTimeout(() => { copiedThinking.value = false; }, 2000);
  }
};
</script>

<template>
  <div
    class="w-full flex gap-3 transition-all duration-200"
    :class="isUser ? 'justify-end' : 'justify-start'"
  >
    <!-- 助手头像 (1:1 对齐 pi-desktop 极简质感黑标) -->
    <div
      v-if="isAssistant"
      class="w-7 h-7 rounded-lg bg-[#1c1a17] dark:bg-[#faf9f7] text-white dark:text-[#1c1a17] flex items-center justify-center shrink-0 shadow-xs mt-0.5 select-none"
    >
      <Bot :size="15" />
    </div>

    <!-- 消息内容主体容器 (最大宽度 90%) -->
    <div
      class="space-y-3 max-w-[92%] sm:max-w-[88%]"
      :class="isUser ? 'items-end ml-auto' : 'items-start'"
    >
      <!-- ==================== 助手消息专属：模型标牌与 TPS 测速栏 ==================== -->
      <div
        v-if="isAssistant"
        class="flex items-center gap-2 text-xs font-mono text-[#746f66] dark:text-[#8f8a81] pl-1 select-none"
      >
        <span class="font-bold text-[#1c1a17] dark:text-[#faf9f7] tracking-tight flex items-center gap-1.5">
          <Zap :size="12" class="text-[#1764e8]" />
          <span>{{ modelName }}</span>
        </span>
        <!-- 12GB 独显就绪标识 -->
        <span class="text-[10px] px-2 py-0.5 rounded-full bg-[#f0eeea] dark:bg-[#26231f] border border-[#e4e1da] dark:border-[#33302a] text-[#57534a] dark:text-[#a19d92]">
          12GB 独显直出
        </span>
        <!-- 动态 TPS 测速药丸 -->
        <span
          v-if="effectiveTps"
          class="text-[10px] px-2 py-0.5 rounded-full border font-bold flex items-center gap-1 transition-all"
          :class="tpsBadgeClass"
        >
          <Activity :size="10" />
          <span>{{ effectiveTps }} t/s</span>
        </span>
      </div>

      <!-- ==================== 0. 即时调度中的动态反馈指示器 ==================== -->
      <div
        v-if="isInitialLoading"
        class="p-3.5 rounded-2xl bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] shadow-xs flex items-center gap-2.5 text-xs font-mono text-[#1764e8] animate-pulse"
      >
        <Loader2 :size="14" class="animate-spin text-[#1764e8]" />
        <span class="font-medium">端侧神经模型正在分析长流程任务...</span>
      </div>

      <!-- ==================== 1. 极简纸面思维链盒 (<think>) ==================== -->
      <div
        v-if="isAssistant && message.thinking"
        class="w-full rounded-xl border border-[#d6d3cd] dark:border-[#4a453c] bg-[#fbfaf8] dark:bg-[#1a1815] overflow-hidden transition-all duration-200 shadow-xs"
      >
        <div class="px-3.5 py-1.5 flex items-center justify-between border-b border-[#ece9e2] dark:border-[#33302a] bg-[#f7f6f3]/60 dark:bg-[#1c1a17]/60">
          <button
            type="button"
            @click="emit('toggleThinking', message.id)"
            class="flex items-center gap-1.5 text-xs font-mono font-semibold text-[#1764e8] dark:text-[#60a5fa] hover:opacity-80 cursor-pointer select-none transition-colors"
          >
            <component :is="message.showThinking ? ChevronDown : ChevronRight" :size="13" />
            <Sparkles :size="12" />
            <span>思维链深度推理 (Reasoning Stream)</span>
          </button>
          <div class="flex items-center gap-2">
            <span v-if="isGenerating && !message.content" class="text-[10px] text-[#1764e8] font-mono flex items-center gap-1">
              <Loader2 :size="10" class="animate-spin" />
              <span>思考中</span>
            </span>
            <button
              type="button"
              @click="copyText(message.thinking || '', 'thinking')"
              class="px-2 py-0.5 rounded-md border border-[#e4e1da] dark:border-[#33302a] bg-white dark:bg-[#26231f] hover:bg-[#f0eeea] text-[10px] text-[#746f66] dark:text-[#8f8a81] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] flex items-center gap-1 transition-all cursor-pointer select-none"
              title="复制思维链"
            >
              <Check v-if="copiedThinking" :size="10" class="text-emerald-600 dark:text-emerald-400" />
              <Copy v-else :size="10" />
              <span>{{ copiedThinking ? '已复制' : '复制思考' }}</span>
            </button>
          </div>
        </div>
        <div
          v-if="message.showThinking"
          class="p-3 pl-4 border-l-2 border-[#1764e8]/60 font-mono text-[11.5px] leading-relaxed whitespace-pre-wrap text-[#57534a] dark:text-[#a19d92] select-text italic max-h-[320px] overflow-y-auto custom-scrollbar"
        >
          {{ message.thinking }}
        </div>
      </div>

      <!-- ==================== 2. 算子调用流水线看板 (ProcessGroup) ==================== -->
      <div v-if="normalizedTools.length > 0" class="w-full">
        <ProcessGroup
          :tools="normalizedTools"
          :active-artifact-id="activeArtifactId"
          :is-generating="isGenerating"
          @select-artifact="(callId) => emit('selectArtifact', callId)"
        />
      </div>

      <!-- ==================== 3. 消息正文主体气泡 ==================== -->
      <div
        v-if="message.content || (isAssistant && isGenerating && runningToolName)"
        class="p-4 rounded-2xl leading-relaxed transition-all duration-200 select-text font-sans"
        :class="[
          isUser
            ? 'bg-[#1c1a17] dark:bg-[#2a2620] text-[#faf9f7] shadow-sm ml-auto text-[13.5px]'
            : 'bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] text-[#1c1a17] dark:text-[#faf9f7] shadow-xs prose dark:prose-invert max-w-none text-[13.5px]'
        ]"
      >
        <div
          v-if="isAssistant && message.content"
          class="overflow-x-auto text-[13.5px] leading-relaxed"
          v-html="renderMarkdown(message.content)"
        />
        <div
          v-else-if="isAssistant && isGenerating && runningToolName"
          class="flex items-center gap-2.5 text-xs font-mono text-[#57534a] dark:text-[#a19d92]"
        >
          <Loader2 :size="13" class="animate-spin text-[#1764e8]" />
          <span>正在调用端侧算子: <strong class="text-[#1764e8]">{{ getToolDisplayLabel(runningToolName) }}</strong>，请稍候...</span>
        </div>
        <div
          v-else-if="isUser"
          class="whitespace-pre-wrap font-sans text-[13.5px] font-medium leading-relaxed"
        >
          {{ message.content }}
        </div>
      </div>

      <!-- ==================== 4. 底部状态与操作栏 ==================== -->
      <div
        class="flex items-center gap-3 text-[11px] font-mono text-[#746f66] dark:text-[#8f8a81] px-1 select-none"
        :class="isUser ? 'justify-end' : 'justify-start'"
      >
        <span v-if="formattedTime">{{ formattedTime }}</span>
        <span v-if="message.elapsedMs" class="flex items-center gap-1">
          <Clock :size="11" />
          <span>耗时: {{ (message.elapsedMs / 1000).toFixed(2) }}s</span>
        </span>
        <button
          v-if="isAssistant && message.content && !isGenerating"
          type="button"
          @click="copyText(message.content, 'content')"
          class="flex items-center gap-1 text-[#746f66] dark:text-[#8f8a81] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] cursor-pointer transition-colors"
          title="复制回答正文"
        >
          <Check v-if="copiedContent" :size="11" class="text-emerald-600 dark:text-emerald-400" />
          <Copy v-else :size="11" />
          <span>{{ copiedContent ? '已复制正文' : '复制回答' }}</span>
        </button>
      </div>
    </div>

    <!-- 用户头像 -->
    <div
      v-if="isUser"
      class="w-7 h-7 rounded-lg bg-[#e4e1da] dark:bg-[#33302a] text-[#1c1a17] dark:text-[#faf9f7] flex items-center justify-center shrink-0 shadow-xs mt-0.5 select-none"
    >
      <User :size="15" />
    </div>
  </div>
</template>
