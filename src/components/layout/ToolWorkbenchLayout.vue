<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useUIStore } from '../../store/uiStore';
import {
  UploadCloud,
  X,
  Check,
  Loader2,
  FolderOpen,
  CheckCircle2,
  ArrowRight,
  ArrowLeft,
} from 'lucide-vue-next';
import DotField from '../effects/DotField.vue';
import GradientText from '../effects/GradientText.vue';
import type { ToolWorkflowInstance } from '../../composables/useToolWorkflow';

export interface GuideItem {
  title: string;
  tag?: string;
  desc: string;
}

withDefaults(
  defineProps<{
    title: string;
    steps?: string[];
    guides?: GuideItem[];
    workflow: ToolWorkflowInstance<any>;
    uploadButtonText?: string;
    queueTitle?: string;
    actionButtonText?: string;
    dropzoneTitle?: string;
    dropzoneSubtitle?: string;
  }>(),
  {
    steps: () => ['添加文件', '定制选项', '完成交付'],
    uploadButtonText: '选择文件',
    queueTitle: '待处理文件队列',
    actionButtonText: '开始处理',
    dropzoneTitle: '把需要处理的文件拖放到这里',
    dropzoneSubtitle: '支持多选批量排队，本地离线极速处理',
  }
);

const emit = defineEmits<{
  (e: 'execute'): void;
}>();

const ui = useUIStore();

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && !ui.toast显示) {
    ui.currentView = 'home';
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans flex flex-col">
    <input
      type="file"
      multiple
      :ref="(el) => (workflow.fileInputRef.value = el as HTMLInputElement)"
      class="hidden"
      @change="workflow.handleFileChange"
    />

    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden opacity-30">
      <DotField
        :dot-radius="1.2"
        :dot-spacing="16"
        :bulge-strength="45"
        :glow-radius="180"
        gradient-from="rgba(2, 195, 180, 0.35)"
        gradient-to="rgba(0, 210, 255, 0.12)"
        glow-color="#02c3b4"
      />
    </div>

    <div
      class="relative z-10 flex-1 w-full overflow-y-auto custom-scrollbar pt-12 pb-10 transition-all duration-300 ease-in-out"
      :class="ui.侧边栏收起 ? 'px-6 sm:px-10 lg:px-14' : 'pl-[256px] pr-6 sm:pr-10 lg:pr-14'"
      @dragover="workflow.handleDragOver"
      @dragleave="workflow.handleDragLeave"
      @drop="workflow.handleDrop"
    >
      <div class="max-w-[1480px] w-full mx-auto space-y-5 pt-1">

        <header class="relative flex items-center justify-between border-b border-white/[0.06] pb-3.5 pt-1">
          <button
            type="button"
            @click="ui.currentView = 'home'"
            class="h-8 px-3.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.08] border border-white/10 hover:border-[#02c3b4]/40 text-[#8b999b] hover:text-white transition-all duration-200 flex items-center gap-2 text-xs font-bold shadow-md active:scale-95 cursor-pointer group"
            title="返回工坊主页 (Esc)"
          >
            <ArrowLeft :size="14" class="group-hover:-translate-x-0.5 transition-transform text-[#02c3b4]" />
            <span>返回主页</span>
          </button>

          <div class="absolute left-1/2 -translate-x-1/2 pointer-events-none">
            <GradientText
              :text="title"
              :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
              :animation-speed="6"
              class="text-2xl sm:text-3xl font-black tracking-tight text-center"
            />
          </div>

          <div
            @click="ui.currentView = 'home'"
            class="flex items-center gap-1.5 text-[11px] font-mono text-[#5b696b] hover:text-[#8b999b] cursor-pointer transition-colors select-none"
            title="点击或按 ESC 返回主页"
          >
            <kbd class="px-1.5 py-0.5 rounded-md bg-white/[0.04] border border-white/10 text-[#8b999b] text-[10px] font-bold">ESC</kbd>
            <span class="hidden sm:inline">快捷返回</span>
          </div>
        </header>

        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch w-full">

          <aside class="lg:col-span-5 w-full min-w-0 bg-white/[0.02] border border-white/[0.06] rounded-3xl p-6 space-y-5 shadow-xl backdrop-blur-xl flex flex-col justify-between">
            <div class="space-y-4">
              <div class="flex items-center justify-between pb-2.5 border-b border-white/[0.04]">
                <span class="text-sm font-bold text-[#02c3b4] font-mono tracking-wider flex items-center gap-1.5">
                  <span>⚙️ 选项配置指南</span>
                </span>
                <span class="text-[#5b696b] text-xs font-mono">直观选型参考</span>
              </div>

              <div v-if="guides && guides.length > 0" class="space-y-3">
                <div
                  v-for="(g, idx) in guides"
                  :key="idx"
                  class="p-3.5 rounded-2xl bg-white/[0.02] hover:bg-white/[0.04] border border-white/[0.03] hover:border-white/[0.08] transition-all space-y-1"
                >
                  <div class="flex items-center justify-between">
                    <span class="text-sm sm:text-base font-bold text-white tracking-wide">{{ g.title }}</span>
                    <span v-if="g.tag" class="text-xs sm:text-sm text-[#02c3b4] font-bold font-mono tracking-wide">
                      {{ g.tag }}
                    </span>
                  </div>
                  <p class="text-xs sm:text-[13px] text-[#a0b0b2] leading-relaxed">
                    {{ g.desc }}
                  </p>
                </div>
              </div>
            </div>

            <div v-if="steps && steps.length > 0" class="pt-2">
              <div class="grid grid-cols-3 gap-2.5">
                <div
                  v-for="(step, idx) in steps"
                  :key="idx"
                  class="p-3 rounded-xl bg-white/[0.02] border border-white/[0.04] flex items-center justify-center gap-2 shadow-sm"
                >
                  <span class="w-6 h-6 rounded-lg bg-[#02c3b4]/15 text-[#02c3b4] font-mono text-xs font-bold flex items-center justify-center shrink-0">
                    {{ idx + 1 }}
                  </span>
                  <span class="text-xs sm:text-sm font-bold text-white/90 whitespace-nowrap">{{ step.replace(/^\d+\s*/, '') }}</span>
                </div>
              </div>
            </div>
          </aside>

          <main class="lg:col-span-7 w-full min-w-0 flex flex-col justify-between space-y-4 min-h-full">

            <div
              class="flex-1 flex flex-col justify-between bg-white/[0.02] border border-white/[0.06] rounded-3xl p-6 shadow-lg backdrop-blur-xl space-y-4 min-h-[220px] transition-all"
              :class="workflow.isDragging.value ? 'border-[#02c3b4]/60 bg-[#02c3b4]/5' : ''"
              @dragover.stop="workflow.handleDragOver"
              @dragleave.stop="workflow.handleDragLeave"
              @drop.stop="workflow.handleDrop"
            >
              <div class="flex items-center justify-between border-b border-white/[0.06] pb-3">
                <h4 class="text-sm font-bold text-white flex items-center gap-2">
                  <span>{{ queueTitle }}</span>
                  <span class="text-xs font-mono text-[#02c3b4]">({{ workflow.queue.value.length }})</span>
                </h4>
                <div class="flex items-center gap-3">
                  <button
                    v-if="workflow.queue.value.length > 0"
                    type="button"
                    @click.stop="workflow.triggerFileSelect"
                    class="text-xs font-mono text-[#02c3b4] hover:underline cursor-pointer flex items-center gap-1 font-bold"
                  >
                    <span>+ 继续添加</span>
                  </button>
                  <button
                    v-if="workflow.queue.value.length > 0"
                    type="button"
                    @click.stop="workflow.clearQueue"
                    class="text-xs font-mono text-[#8b999b] hover:text-rose-400 cursor-pointer"
                  >
                    清空
                  </button>
                </div>
              </div>

              <div v-if="workflow.queue.value.length > 0" class="space-y-2 max-h-[240px] overflow-y-auto custom-scrollbar pr-1 flex-1">
                <div
                  v-for="(item, idx) in workflow.queue.value"
                  :key="item.id"
                  class="p-3 bg-white/[0.02] hover:bg-white/[0.05] rounded-2xl flex items-center justify-between gap-3 transition-all"
                >
                  <div class="flex items-center gap-3 min-w-0">
                    <span class="w-6 h-6 rounded-lg bg-[#02c3b4]/15 text-[#02c3b4] font-mono text-xs flex items-center justify-center font-bold shrink-0">
                      {{ idx + 1 }}
                    </span>
                    <span class="text-sm font-medium text-white truncate" :title="item.path">
                      {{ item.name }}
                    </span>
                    <span class="text-xs font-mono text-[#5b696b] shrink-0 hidden sm:inline">
                      {{ item.sizeFormatted }}
                    </span>
                  </div>

                  <button
                    type="button"
                    @click.stop="workflow.removeFile(idx)"
                    class="w-7 h-7 rounded-lg text-[#5b696b] hover:text-white hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer shrink-0"
                  >
                    <X :size="14" />
                  </button>
                </div>
              </div>

              <div
                v-else
                @click="workflow.triggerFileSelect"
                class="flex-1 flex flex-col items-center justify-center py-8 space-y-4 cursor-pointer select-none group"
              >
                <div class="space-y-1 text-center">
                  <h3 class="text-base font-bold text-white tracking-tight group-hover:text-[#02c3b4] transition-colors">
                    {{ dropzoneTitle }}
                  </h3>
                  <p class="text-xs text-[#8b999b]">
                    {{ dropzoneSubtitle }}
                  </p>
                </div>

                <button
                  type="button"
                  class="px-8 py-3.5 rounded-2xl bg-[#02c3b4]/15 hover:bg-[#02c3b4]/25 border border-[#02c3b4]/50 text-[#02c3b4] font-bold text-sm shadow-[0_0_24px_rgba(2,195,180,0.2)] transition-all active:scale-95 flex items-center gap-2 cursor-pointer pointer-events-none group-hover:scale-105"
                >
                  <UploadCloud :size="18" class="stroke-[2.5]" />
                  <span>{{ uploadButtonText }}</span>
                </button>
              </div>
            </div>

            <div v-if="$slots.options" class="bg-white/[0.02] border border-white/[0.06] rounded-3xl p-6 shadow-lg backdrop-blur-xl space-y-4 shrink-0">
              <slot name="options" />
            </div>

            <div class="p-5 rounded-3xl bg-white/[0.02] border border-white/[0.06] flex items-center justify-between gap-4 shadow-xl backdrop-blur-xl shrink-0">
              <span class="text-xs text-[#8b999b] font-medium">
                纯血 Rust 端侧离线推理 · 零数据上传 · 隐私安全
              </span>

              <button
                type="button"
                :disabled="workflow.queue.value.length === 0 || workflow.state.value === 'running'"
                @click="emit('execute')"
                class="px-9 py-3.5 rounded-2xl font-bold text-sm transition-all duration-200 flex items-center justify-center gap-2 cursor-pointer shrink-0 shadow-xl active:scale-95"
                :class="[
                  workflow.queue.value.length === 0 || workflow.state.value === 'running'
                    ? 'bg-white/[0.03] text-[#5b696b] cursor-not-allowed border border-white/[0.04]'
                    : 'bg-[#02c3b4]/15 hover:bg-[#02c3b4]/25 text-[#02c3b4] border border-[#02c3b4]/50 shadow-[0_0_24px_rgba(2,195,180,0.25)] hover:scale-[1.02]'
                ]"
              >
                <Loader2 v-if="workflow.state.value === 'running'" :size="16" class="animate-spin" />
                <span>{{ workflow.state.value === 'running' ? '正在处理...' : actionButtonText }}</span>
                <ArrowRight v-if="workflow.state.value !== 'running'" :size="16" />
              </button>
            </div>

          </main>
        </div>

      </div>
    </div>

    <div
      v-if="workflow.isProcessingModalOpen.value"
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none"
    >
      <div class="max-w-[420px] w-full p-7 rounded-3xl bg-[#1c2426] border border-white/10 shadow-2xl text-center space-y-5 flex flex-col items-center">
        <div class="w-14 h-14 rounded-2xl bg-[#02c3b4]/15 border border-[#02c3b4]/30 text-[#02c3b4] flex items-center justify-center shadow-lg animate-pulse">
          <Loader2 :size="26" class="animate-spin" />
        </div>

        <div class="space-y-1">
          <h3 class="text-base font-bold text-white">
            {{ workflow.statusText.value || '正在处理中...' }}
          </h3>
          <p class="text-xs text-[#8b999b]">
            端侧 GPU/CPU 运算中，请勿关闭窗口
          </p>
        </div>

        <div class="w-full space-y-1.5">
          <div class="w-full h-1.5 bg-white/[0.06] rounded-full overflow-hidden">
            <div
              class="h-full bg-[#02c3b4] transition-all duration-300 rounded-full shadow-[0_0_12px_#02c3b4]"
              :style="{ width: `${workflow.progress.value}%` }"
            ></div>
          </div>
          <div class="flex justify-between text-xs font-mono text-[#5b696b]">
            <span>PURE RUST NATIVE</span>
            <span>{{ workflow.progress.value }}%</span>
          </div>
        </div>

        <button
          type="button"
          @click="workflow.cancelProcessing"
          class="px-6 py-2 rounded-full border border-white/10 hover:border-white/20 text-xs text-[#8b999b] hover:text-white transition-colors cursor-pointer"
        >
          取消
        </button>
      </div>
    </div>

    <div
      v-if="workflow.isSuccessModalOpen.value && workflow.successInfo.value"
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none"
    >
      <div class="max-w-[440px] w-full p-7 rounded-3xl bg-[#1c2426] border border-white/10 shadow-2xl text-center space-y-5 flex flex-col items-center">
        <div class="w-14 h-14 rounded-full bg-[#02c3b4]/20 text-[#02c3b4] border border-[#02c3b4]/40 flex items-center justify-center shadow-lg shadow-teal-500/20">
          <Check :size="26" class="stroke-[3]" />
        </div>

        <div class="space-y-1">
          <h3 class="text-lg font-bold text-white">
            解析完成
          </h3>
          <p class="text-xs text-[#8b999b]">
            {{ workflow.successInfo.value.title }}
          </p>
        </div>

        <div class="w-full bg-black/30 rounded-2xl p-4 text-xs font-mono text-left space-y-2 border border-white/[0.06]">
          <div class="flex justify-between text-[#8b999b]">
            <span>导出格式:</span>
            <strong class="text-[#02c3b4] font-bold">{{ workflow.successInfo.value.targetFormat }}</strong>
          </div>
          <div class="flex justify-between text-[#8b999b]">
            <span>处理数量:</span>
            <strong class="text-white font-bold">{{ workflow.successInfo.value.totalProcessed }} 个</strong>
          </div>
          <div class="flex justify-between items-center text-[#8b999b] gap-2">
            <span class="shrink-0">存储路径:</span>
            <span class="text-xs text-white/90 truncate font-medium" :title="workflow.successInfo.value.outputDir">
              {{ workflow.successInfo.value.outputDir }}
            </span>
          </div>
        </div>

        <div class="w-full flex items-center justify-center gap-3 pt-1">
          <button
            type="button"
            @click="workflow.openFolder()"
            class="flex-1 py-2.5 px-4 rounded-xl bg-white/[0.05] hover:bg-white/[0.1] border border-white/10 text-white font-bold text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1.5"
          >
            <FolderOpen :size="15" />
            <span>打开目录</span>
          </button>
          <button
            type="button"
            @click="workflow.closeSuccessModal"
            class="flex-1 py-2.5 px-4 rounded-xl bg-[#02c3b4]/20 hover:bg-[#02c3b4]/30 border border-[#02c3b4]/50 text-[#02c3b4] font-bold text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1"
          >
            <CheckCircle2 :size="15" />
            <span>确定</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
