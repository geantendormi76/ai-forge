<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { UploadCloud, X, Check, Loader2, FolderOpen, CheckCircle2 } from 'lucide-vue-next';
import DotField from '../effects/DotField.vue';
import type { ToolWorkflowInstance } from '../../composables/useToolWorkflow';

export interface MetaRowItem {
  label: string;
  value: string;
}

export interface FeatureCardItem {
  icon?: any;
  text: string;
}

withDefaults(
  defineProps<{
    kicker?: string;
    title: string;
    description: string;
    localNote?: string;
    badgeText?: string;
    vramCostMb?: number;
    steps?: string[];
    metaRows?: MetaRowItem[];
    featureCards?: FeatureCardItem[];
    workflow: ToolWorkflowInstance<any>;
    acceptedFormatsText?: string;
    dropzoneTitle?: string;
    dropzoneSubtitle?: string;
    uploadButtonText?: string;
    queueTitle?: string;
    queueHint?: string;
    actionButtonText?: string;
    bottomHintText?: string;
  }>(),
  {
    kicker: 'VIDEO PROCESSING TOOL',
    badgeText: 'SOTA 2026 MULTI-MODAL',
    vramCostMb: 1000,
    localNote: '全流程完全在本机离线运行，数据零上传，显存安全锁保护。',
    steps: () => ['01 添加视频', '02 目标定制', '03 批量处理', '04 打开结果'],
    acceptedFormatsText: '支持 MP4、MKV、MOV、WebM、AVI、FLV 等格式',
    dropzoneTitle: '把需要处理的视频放到这里',
    dropzoneSubtitle: '支持拖拽或点击上传，上传后进入队列，再选择目标参数统一处理。',
    uploadButtonText: '上传视频文件',
    queueTitle: '待处理视频',
    queueHint: '支持多文件排队，批量任务会按队列顺序处理。',
    actionButtonText: '开始处理',
    bottomHintText: '确认文件和目标格式后开始处理，完成后会自动弹出结果提示。',
  }
);

const emit = defineEmits<{
  (e: 'execute'): void;
}>();

const { locale } = useI18n();

const toggleLanguage = () => {
  locale.value = locale.value === 'zh-CN' ? 'en-US' : 'zh-CN';
};
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#060607] text-[#f5f5f3] select-none font-sans flex flex-col">
    <!-- 原生隐藏文件选择器 (支持多选) -->
    <input
      type="file"
      multiple
      :ref="(el) => (workflow.fileInputRef.value = el as HTMLInputElement)"
      class="hidden"
      @change="workflow.handleFileChange"
    />

    <!-- 背景点阵力场微光 -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden opacity-35">
      <DotField
        :dot-radius="1.2"
        :dot-spacing="16"
        :bulge-strength="45"
        :glow-radius="160"
        gradient-from="rgba(255, 255, 255, 0.18)"
        gradient-to="rgba(255, 255, 255, 0.03)"
        glow-color="#ffffff"
      />
    </div>

    <!-- 主工作区滚动容器 -->
    <div
      class="relative z-10 flex-1 w-full overflow-y-auto custom-scrollbar px-6 sm:px-10 pb-16"
      @dragover="workflow.handleDragOver"
      @dragleave="workflow.handleDragLeave"
      @drop="workflow.handleDrop"
    >
      <div class="max-w-[1480px] mx-auto space-y-6 pt-2">
        <!-- 1. 顶栏控制器与状态标杆 -->
        <header class="flex items-center justify-between border-b border-white/[0.08] pb-4">
          <div class="flex items-center gap-2 text-xs font-mono text-[#8b8b87]">
            <span class="px-2 py-0.5 rounded-full bg-white/10 text-white font-bold text-[10px] border border-white/20">
              {{ badgeText }}
            </span>
            <span>/</span>
            <span class="text-white font-bold uppercase tracking-wider">{{ kicker }}</span>
          </div>

          <div class="flex items-center gap-3">
            <div class="bg-black/50 border border-white/10 px-3 py-1 rounded-xl shadow-lg backdrop-blur-md text-xs flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_8px_#34d399]"></span>
              <span class="text-[#8b8b87] text-[11px] font-mono font-medium">
                VRAM: <strong class="text-white">{{ vramCostMb.toLocaleString() }} MB</strong> POOL
              </span>
            </div>
            <button
              type="button"
              @click="toggleLanguage"
              class="bg-black/40 border border-white/10 hover:border-white/20 hover:bg-white/[0.06] text-[#f2f2ef] px-3 py-1 rounded-xl shadow-sm text-xs font-mono font-bold transition-all cursor-pointer flex items-center gap-1.5"
            >
              <span>🌐</span>
              <span>{{ locale === 'zh-CN' ? 'English' : '简体中文' }}</span>
            </button>
          </div>
        </header>

        <!-- 2. 原生桌面端左右双翼拓扑 (左侧海报栏 340px + 右侧主工作台) -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-7 items-start">
          
          <!-- 左侧海报栏 (Poster Column: 4 栏 / ~340px) -->
          <aside class="lg:col-span-4 bg-[#0a0a0b]/80 border border-white/10 rounded-3xl p-7 space-y-6 shadow-2xl backdrop-blur-xl sticky top-2">
            <div>
              <span class="text-[11px] font-mono tracking-widest uppercase text-[#8b8b87] block mb-2">{{ kicker }}</span>
              <h1 class="text-3xl sm:text-4xl font-black text-white tracking-tight leading-tight">
                {{ title }}
              </h1>
              <p class="text-xs text-[#a0a09b] mt-3 leading-relaxed">
                {{ description }}
              </p>
            </div>

            <!-- LOCAL ONLY 离线安全卡片 -->
            <div class="p-4 rounded-2xl bg-white/[0.03] border border-white/10 space-y-1">
              <span class="text-[10px] font-mono font-bold tracking-wider text-white uppercase block">LOCAL ONLY</span>
              <p class="text-xs text-[#8b8b87] leading-relaxed">
                {{ localNote }}
              </p>
            </div>

            <!-- 元数据行 -->
            <div v-if="metaRows && metaRows.length > 0" class="border-t border-white/[0.08] pt-4 space-y-2 text-xs font-mono">
              <div v-for="item in metaRows" :key="item.label" class="flex justify-between items-center text-[#8b8b87]">
                <span>{{ item.label }}</span>
                <strong class="text-white font-bold">{{ item.value }}</strong>
              </div>
            </div>

            <!-- 垂直 01~04 步骤流 -->
            <div v-if="steps && steps.length > 0" class="border-t border-white/[0.08] pt-4 space-y-3">
              <div
                v-for="(step, idx) in steps"
                :key="idx"
                class="flex items-center gap-3 text-xs font-medium text-[#8b8b87]"
              >
                <span class="w-6 h-6 rounded-full bg-white/10 border border-white/15 text-white font-mono text-[10px] flex items-center justify-center font-bold shrink-0">
                  {{ idx + 1 }}
                </span>
                <span class="text-[#f2f2ef] tracking-wide">{{ step }}</span>
              </div>
            </div>
          </aside>

          <!-- 右侧多任务工作台 (Workspace Column: 8 栏) -->
          <main class="lg:col-span-8 space-y-6">
            
            <!-- 卡片 1：上传控制条 (Drop or Select Bar) -->
            <div
              @click="workflow.triggerFileSelect"
              class="border border-white/10 rounded-3xl p-6 sm:p-8 bg-[#0a0a0b]/80 hover:bg-white/[0.02] hover:border-white/20 transition-all cursor-pointer flex flex-col sm:flex-row sm:items-center justify-between gap-5 shadow-xl backdrop-blur-xl group"
              :class="workflow.isDragging.value ? 'border-white/60 bg-white/[0.05]' : ''"
            >
              <div class="space-y-1 min-w-0">
                <span class="text-[10px] font-mono tracking-widest uppercase text-[#8b8b87] block">DROP OR SELECT</span>
                <h3 class="text-lg sm:text-xl font-bold text-white tracking-tight group-hover:text-white">
                  {{ dropzoneTitle }}
                </h3>
                <p class="text-xs text-[#8b8b87]">
                  {{ dropzoneSubtitle }}
                </p>
              </div>

              <button
                type="button"
                class="px-6 py-3 rounded-full bg-white hover:bg-[#f2f2ef] text-black font-bold text-xs sm:text-sm shadow-md transition-all active:scale-95 cursor-pointer shrink-0 flex items-center justify-center gap-2 pointer-events-none"
              >
                <UploadCloud :size="16" class="stroke-[2.5]" />
                <span>{{ uploadButtonText }}</span>
              </button>
            </div>

            <!-- 卡片 2：参数选项插槽 (如目标格式 / 目标语言胶囊排) -->
            <div v-if="$slots.options" class="bg-[#0a0a0b]/80 border border-white/10 rounded-3xl p-6 shadow-xl backdrop-blur-xl space-y-4">
              <slot name="options" />
            </div>

            <!-- 卡片 3：待处理视频多任务队列 (Convert Queue List) -->
            <div class="bg-[#0a0a0b]/80 border border-white/10 rounded-3xl p-6 shadow-xl backdrop-blur-xl space-y-4">
              <div class="flex items-center justify-between border-b border-white/[0.06] pb-3">
                <div>
                  <span class="text-[10px] font-mono tracking-widest uppercase text-[#8b8b87] block">CONVERT QUEUE</span>
                  <h4 class="text-base font-bold text-white mt-0.5 flex items-center gap-2">
                    <span>{{ queueTitle }}</span>
                    <span class="text-xs font-mono font-normal text-[#8b8b87]">({{ workflow.queue.value.length }})</span>
                  </h4>
                </div>
                <div class="flex items-center gap-3">
                  <span class="text-[11px] text-[#5b5b58] font-mono hidden sm:inline">{{ queueHint }}</span>
                  <button
                    v-if="workflow.queue.value.length > 0"
                    type="button"
                    @click.stop="workflow.clearQueue"
                    class="text-[11px] font-mono text-[#8b8b87] hover:text-rose-400 cursor-pointer"
                  >
                    清空队列
                  </button>
                </div>
              </div>

              <!-- 队列列表项 -->
              <div v-if="workflow.queue.value.length > 0" class="space-y-2 max-h-[320px] overflow-y-auto custom-scrollbar pr-1">
                <div
                  v-for="(item, idx) in workflow.queue.value"
                  :key="item.id"
                  class="p-3.5 bg-black/40 border border-white/[0.06] hover:border-white/15 rounded-2xl flex items-center justify-between gap-3 transition-all"
                >
                  <div class="flex items-center gap-3 min-w-0">
                    <span class="w-6 h-6 rounded-lg bg-white/5 border border-white/10 text-white/70 font-mono text-xs flex items-center justify-center font-bold shrink-0">
                      {{ idx + 1 }}
                    </span>
                    <span class="text-xs font-bold text-white truncate" :title="item.path">
                      {{ item.name }}
                    </span>
                    <span class="text-[11px] font-mono text-[#8b8b87] shrink-0 hidden sm:inline">
                      {{ item.durationFormatted ? `${item.durationFormatted} — ` : '' }}{{ item.sizeFormatted }}
                    </span>
                  </div>

                  <button
                    type="button"
                    @click.stop="workflow.removeFile(idx)"
                    class="w-7 h-7 rounded-lg text-[#5b5b58] hover:text-white hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer shrink-0"
                    title="移出队列"
                  >
                    <X :size="14" class="stroke-[2]" />
                  </button>
                </div>
              </div>

              <!-- 队列为空状态 -->
              <div v-else class="py-10 text-center text-[#5b5b58] space-y-2">
                <p class="text-xs">暂无待处理视频，请点击上方按钮或直接拖入视频添加到队列</p>
              </div>
            </div>

            <!-- 卡片 4：底层 4 栏特性与格式说明卡片行 -->
            <div v-if="featureCards && featureCards.length > 0" class="space-y-2">
              <span class="text-[10px] font-mono tracking-widest uppercase text-[#8b8b87] block px-1">支持特性</span>
              <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-3">
                <div
                  v-for="(feat, idx) in featureCards"
                  :key="idx"
                  class="p-4 rounded-2xl bg-[#0a0a0b]/80 border border-white/[0.06] hover:border-white/15 transition-all text-xs text-[#8b8b87] leading-relaxed flex items-start gap-2.5 shadow-md"
                >
                  <component v-if="feat.icon" :is="feat.icon" :size="15" class="text-white/80 shrink-0 mt-0.5" />
                  <span>{{ feat.text }}</span>
                </div>
              </div>
            </div>

            <!-- 卡片 5：底部行动栏 (Bottom Action Bar) -->
            <div class="p-6 rounded-3xl bg-[#0a0a0b]/80 border border-white/10 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-xl backdrop-blur-xl">
              <p class="text-xs text-[#8b8b87]">
                {{ bottomHintText }}
              </p>

              <button
                type="button"
                :disabled="workflow.queue.value.length === 0 || workflow.state.value === 'running'"
                @click="emit('execute')"
                class="px-9 py-3.5 rounded-full font-bold text-xs sm:text-sm transition-all duration-200 flex items-center justify-center gap-2 cursor-pointer shrink-0"
                :class="[
                  workflow.queue.value.length === 0 || workflow.state.value === 'running'
                    ? 'bg-white/10 text-white/30 cursor-not-allowed border border-white/[0.04]'
                    : 'bg-white hover:bg-[#f2f2ef] text-black shadow-xl hover:scale-[1.02] active:scale-95'
                ]"
              >
                <Loader2 v-if="workflow.state.value === 'running'" :size="15" class="animate-spin" />
                <span>{{ workflow.state.value === 'running' ? '正在处理...' : actionButtonText }}</span>
              </button>
            </div>

          </main>
        </div>

        <!-- 3. 统一底部落款 -->
        <footer class="flex justify-between items-center text-[11px] font-mono text-[#5b5b58] border-t border-white/[0.06] pt-8 pb-4">
          <span>ZIDIAN AI DESKTOP 2026 / HIGH PERFORMANCE WORKBENCH</span>
          <span>ALL COMPUTATION 100% STAYS ON YOUR DEVICE</span>
        </footer>
      </div>
    </div>

    <!-- ======================================================== -->
    <!-- 模态遮罩 1：处理中流光遮罩 (Processing Modal Overlay)     -->
    <!-- ======================================================== -->
    <div
      v-if="workflow.isProcessingModalOpen.value"
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/80 backdrop-blur-md animate-in fade-in duration-200 select-none"
    >
      <div class="max-w-[460px] w-full p-8 rounded-3xl bg-[#0e0e10] border border-white/15 shadow-2xl text-center space-y-6 flex flex-col items-center">
        <div class="w-14 h-14 rounded-2xl bg-white/[0.06] border border-white/10 text-white flex items-center justify-center shadow-lg animate-pulse">
          <Loader2 :size="26" class="animate-spin text-white" />
        </div>

        <div class="space-y-1">
          <h3 class="text-lg font-bold text-white">
            {{ workflow.statusText.value || '正在处理中...' }}
          </h3>
          <p class="text-xs text-[#8b8b87]">
            全流程本地 GPU 运算，请勿关闭窗口
          </p>
        </div>

        <div class="w-full space-y-1.5">
          <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
            <div
              class="h-full bg-white transition-all duration-300 rounded-full shadow-[0_0_12px_#ffffff]"
              :style="{ width: `${workflow.progress.value}%` }"
            ></div>
          </div>
          <div class="flex justify-between text-[10px] font-mono text-[#5b5b58]">
            <span>GPU NATIVE PIPELINE</span>
            <span>{{ workflow.progress.value }}%</span>
          </div>
        </div>

        <button
          type="button"
          @click="workflow.cancelProcessing"
          class="px-6 py-2 rounded-full border border-white/15 hover:border-white/30 text-xs text-[#8b8b87] hover:text-white transition-colors cursor-pointer"
        >
          取消
        </button>
      </div>
    </div>

    <!-- ======================================================== -->
    <!-- 模态遮罩 2：处理成功交付对话框 (Success Dialog)          -->
    <!-- ======================================================== -->
    <div
      v-if="workflow.isSuccessModalOpen.value && workflow.successInfo.value"
      class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/80 backdrop-blur-md animate-in fade-in duration-200 select-none"
    >
      <div class="max-w-[480px] w-full p-8 rounded-3xl bg-white text-black shadow-2xl text-center space-y-6 flex flex-col items-center">
        <div class="w-14 h-14 rounded-full bg-emerald-500 text-white flex items-center justify-center shadow-lg shadow-emerald-500/30">
          <Check :size="28" class="stroke-[3]" />
        </div>

        <div class="space-y-1.5">
          <h3 class="text-xl font-extrabold text-black tracking-tight">
            处理成功
          </h3>
          <p class="text-xs text-slate-600">
            {{ workflow.successInfo.value.title }}
          </p>
        </div>

        <!-- 详细信息清单 -->
        <div class="w-full bg-slate-100 rounded-2xl p-4 text-xs font-mono text-left space-y-2 border border-slate-200">
          <div class="flex justify-between text-slate-500">
            <span>目标封装:</span>
            <strong class="text-black font-bold">{{ workflow.successInfo.value.targetFormat }}</strong>
          </div>
          <div class="flex justify-between text-slate-500">
            <span>处理文件:</span>
            <strong class="text-black font-bold">{{ workflow.successInfo.value.totalProcessed }} 个</strong>
          </div>
          <div class="flex justify-between items-center text-slate-500 gap-2">
            <span class="shrink-0">保存路径:</span>
            <span class="text-[11px] text-slate-700 truncate font-semibold" :title="workflow.successInfo.value.outputDir">
              {{ workflow.successInfo.value.outputDir }}
            </span>
          </div>
        </div>

        <!-- 动作操作组 -->
        <div class="w-full flex items-center justify-center gap-3 pt-1">
          <button
            type="button"
            @click="workflow.openFolder()"
            class="flex-1 py-3 px-5 rounded-full bg-slate-200 hover:bg-slate-300 text-slate-900 font-bold text-xs shadow-sm transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-2"
          >
            <FolderOpen :size="15" />
            <span>打开文件夹</span>
          </button>
          <button
            type="button"
            @click="workflow.closeSuccessModal"
            class="flex-1 py-3 px-5 rounded-full bg-black hover:bg-slate-800 text-white font-bold text-xs shadow-lg transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1.5"
          >
            <CheckCircle2 :size="15" />
            <span>确定</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
