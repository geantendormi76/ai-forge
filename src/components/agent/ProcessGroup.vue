<script setup lang="ts">
import { ref, computed } from 'vue';
import type { ToolTrace } from '../../composables/useAgentSession';
import {
  Workflow,
  ChevronDown,
  ChevronRight,
  Loader2,
  CheckCircle2,
  AlertCircle,
  Clock,
  ArrowUpRight
} from 'lucide-vue-next';

const props = withDefaults(
  defineProps<{
    tools: ToolTrace[];
    activeArtifactId?: string | null;
    isGenerating?: boolean;
  }>(),
  {
    tools: () => [],
    activeArtifactId: null,
    isGenerating: false,
  }
);

const emit = defineEmits<{
  (e: 'selectArtifact', callId: string): void;
}>();

// 默认在生成过程中自动展开，生成完毕后允许用户随时折叠
const isExpanded = ref(true);

// 深度计算已完成算子数，避免响应式穿透失效
const completedCount = computed(() => {
  return props.tools.filter(t => t.status === 'success').length;
});

// 仅当明确存在 running 状态时才标记执行中
const isRunning = computed(() => {
  return props.tools.some(t => t.status === 'running');
});

const isAllFinished = computed(() => {
  return props.tools.length > 0 && props.tools.every(t => t.status === 'success' || t.status === 'error');
});

const totalDurationSec = computed(() => {
  const sumMs = props.tools.reduce((acc, t) => acc + (t.durationMs || 0), 0);
  return (sumMs / 1000).toFixed(2);
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
  };
  return map[name] || name;
};

const handleStepClick = (callId: string) => {
  emit('selectArtifact', callId);
};
</script>

<template>
  <div
    v-if="tools && tools.length > 0"
    class="w-full rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] overflow-hidden transition-all duration-200 shadow-xs"
  >
    <!-- 1. 聚合看板顶栏控制器 -->
    <button
      type="button"
      @click="isExpanded = !isExpanded"
      class="w-full px-3.5 py-2 flex items-center justify-between bg-[#f0eeea]/60 dark:bg-[#26231f]/60 hover:bg-[#f0eeea] dark:hover:bg-[#26231f] border-b border-[#e4e1da] dark:border-[#33302a] text-xs font-mono transition-colors cursor-pointer select-none"
    >
      <div class="flex items-center gap-2 min-w-0">
        <component :is="isExpanded ? ChevronDown : ChevronRight" :size="13" class="text-[#1764e8] shrink-0" />
        <div class="w-4 h-4 rounded bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 flex items-center justify-center text-[#1764e8] shrink-0">
          <Workflow :size="10" />
        </div>
        <span class="font-bold text-[#1c1a17] dark:text-[#faf9f7] tracking-tight truncate">
          端侧算子流水线 ({{ completedCount }}/{{ tools.length }})
        </span>

        <!-- 状态胶囊：优先判断全部就绪 -->
        <span
          v-if="isAllFinished && completedCount === tools.length"
          class="px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/40 text-[10px] text-emerald-700 dark:text-emerald-400 font-bold flex items-center gap-1"
        >
          <CheckCircle2 :size="10" /> 全部就绪
        </span>
        <span
          v-else-if="isRunning"
          class="px-2 py-0.5 rounded-full bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-[10px] text-[#1764e8] font-bold flex items-center gap-1 animate-pulse"
        >
          <Loader2 :size="10" class="animate-spin" /> 执行中
        </span>
        <span
          v-else
          class="px-2 py-0.5 rounded-full bg-[#f0eeea] dark:bg-[#26231f] text-[10px] text-[#746f66] dark:text-[#8f8a81]"
        >
          待命
        </span>
      </div>

      <div class="flex items-center gap-2.5 text-[11px] text-[#746f66] dark:text-[#8f8a81] shrink-0">
        <span v-if="parseFloat(totalDurationSec) > 0" class="flex items-center gap-1">
          <Clock :size="10" />
          <span>{{ totalDurationSec }}s</span>
        </span>
        <span class="text-[10px] hidden sm:inline">
          {{ isExpanded ? '收起' : '展开' }}
        </span>
      </div>
    </button>

    <!-- 2. 展开后的各个原子算子流水线步骤列表 -->
    <div v-if="isExpanded" class="p-2 space-y-1 bg-[#f7f6f3]/40 dark:bg-[#141210]/40">
      <div
        v-for="(tool, sIdx) in tools"
        :key="tool.callId"
        @click="handleStepClick(tool.callId)"
        class="w-full px-3 py-1.5 rounded-lg border flex items-center justify-between text-xs font-mono transition-all duration-150 cursor-pointer select-none group"
        :class="[
          activeArtifactId === tool.callId
            ? 'bg-[#edf4ff] dark:bg-[#1e293b] border-[#cce0ff] dark:border-[#3b82f6]/60 text-[#1764e8] dark:text-[#60a5fa] shadow-xs'
            : 'bg-white dark:bg-[#1c1a17] hover:bg-[#f0eeea] dark:hover:bg-[#26231f] border-[#e4e1da]/60 dark:border-[#33302a] text-[#57534a] dark:text-[#a19d92]'
        ]"
      >
        <!-- 左侧序号与算子名称 -->
        <div class="flex items-center gap-2 min-w-0">
          <span
            class="w-4 h-4 rounded flex items-center justify-center text-[9.5px] font-bold shrink-0 transition-colors"
            :class="[
              activeArtifactId === tool.callId
                ? 'bg-[#1764e8] text-white'
                : tool.status === 'success'
                  ? 'bg-[#f0eeea] dark:bg-[#26231f] text-[#1c1a17] dark:text-[#faf9f7] group-hover:bg-[#edf4ff] group-hover:text-[#1764e8]'
                  : 'bg-[#f0eeea] text-[#746f66]'
            ]"
          >
            {{ sIdx + 1 }}
          </span>
          <span class="font-medium truncate max-w-[220px] sm:max-w-[280px]">
            {{ getToolDisplayLabel(tool.toolName) }}
          </span>
        </div>

        <!-- 右侧状态标识与耗时 -->
        <div class="flex items-center gap-2 shrink-0">
          <span v-if="tool.durationMs" class="text-[10px] text-[#746f66] dark:text-[#8f8a81]">
            {{ (tool.durationMs / 1000).toFixed(2) }}s
          </span>
          <span
            v-if="tool.status === 'running'"
            class="text-[#1764e8] flex items-center gap-1 text-[10.5px] font-bold"
          >
            <Loader2 :size="10" class="animate-spin" /> 计算中
          </span>
          <span
            v-else-if="tool.status === 'success'"
            class="flex items-center gap-1 text-[10.5px] font-medium transition-colors"
            :class="activeArtifactId === tool.callId ? 'text-[#1764e8] dark:text-[#60a5fa] font-bold' : 'text-emerald-600 dark:text-emerald-400'"
          >
            <CheckCircle2 :size="11" />
            <span class="hidden sm:inline">交付产物</span>
            <ArrowUpRight :size="10" class="opacity-70 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-transform" />
          </span>
          <span
            v-else
            class="text-rose-600 dark:text-rose-400 flex items-center gap-1 text-[10.5px] font-bold"
          >
            <AlertCircle :size="11" /> 失败
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
