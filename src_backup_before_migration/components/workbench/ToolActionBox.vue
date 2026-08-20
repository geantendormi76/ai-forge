<script setup lang="ts">
import { Check, Loader2 } from 'lucide-vue-next';
import type { WorkflowState } from '../../composables/useToolWorkflow';

defineProps<{
  state: WorkflowState;
  title?: string;
  description?: string;
  progress?: number;
  statusText?: string;
  actionButtonText?: string;
  successButtonText?: string;
  icon?: any;
}>();

const emit = defineEmits<{
  (e: 'execute'): void;
  (e: 'successAction'): void;
}>();
</script>

<template>
  <div class="w-full bg-[#0e0e10]/90 border border-white/10 rounded-2xl p-6 sm:p-7 space-y-6 shadow-xl backdrop-blur-xl select-none">
    <div class="space-y-1">
      <h4 class="text-sm sm:text-base font-bold text-white flex items-center gap-2">
        <component v-if="icon" :is="icon" :size="16" class="text-white/80 shrink-0" />
        <span>{{ title || '执行处理' }}</span>
      </h4>
      <p class="text-xs text-[#8b8b87]">
        {{ description || '全流程完全在本机离线运行，数据零上传。' }}
      </p>
    </div>

    <div v-if="$slots.options" class="pt-1 border-t border-white/[0.06]">
      <slot name="options" />
    </div>

    <div v-if="state === 'running' || state === 'success'" class="space-y-2 pt-2">
      <div class="flex justify-between items-center text-xs font-mono">
        <span class="flex items-center gap-1.5 font-bold" :class="state === 'success' ? 'text-white' : 'text-[#8b8b87]'">
          <Check v-if="state === 'success'" :size="13" class="text-emerald-400 stroke-[3]" />
          <Loader2 v-else :size="13" class="animate-spin text-white/70" />
          <span>{{ statusText || (state === 'success' ? '处理完成！' : '正在处理中...') }}</span>
        </span>
        <span class="text-[11px] text-[#5b5b58]">{{ progress || 0 }}%</span>
      </div>
      <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
        <div
          class="h-full bg-white transition-all duration-300 rounded-full"
          :style="{ width: `${progress || 0}%` }"
        ></div>
      </div>
    </div>

    <div class="pt-2">
      <button
        v-if="state === 'success'"
        type="button"
        @click="emit('successAction')"
        class="w-full sm:w-auto px-7 py-3 rounded-full bg-white hover:bg-[#f2f2ef] text-black font-bold text-xs sm:text-sm shadow-lg transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-2"
      >
        <slot name="success-icon" />
        <span>{{ successButtonText || '查看产物' }}</span>
      </button>

      <button
        v-else
        type="button"
        :disabled="state === 'running'"
        @click="emit('execute')"
        class="w-full sm:w-auto px-8 py-3 rounded-full font-bold text-xs sm:text-sm transition-all duration-200 flex items-center justify-center gap-2 cursor-pointer"
        :class="[
          state === 'running'
            ? 'bg-white/10 text-white/30 cursor-not-allowed'
            : 'bg-white hover:bg-[#f2f2ef] text-black shadow-lg hover:scale-[1.02] active:scale-95'
        ]"
      >
        <Loader2 v-if="state === 'running'" :size="15" class="animate-spin" />
        <span>{{ state === 'running' ? '处理中...' : (actionButtonText || '开始处理') }}</span>
      </button>
    </div>
  </div>
</template>
