<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useUIStore } from '../../store/uiStore';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  downloadToolDependencies,
  cancelDependencyDownloads,
  type DependencyItem,
  type DependencyProgressPayload,
} from '../../bindings';
import { Download, Loader2 } from 'lucide-vue-next';

const ui = useUIStore();
const isDownloading = ref(false);
const progressMap = ref<Record<string, DependencyProgressPayload>>({});
let unlistenProgress: UnlistenFn | null = null;

// 1. 统一极简标题
const modalTitle = computed(() => '需要模型依赖');

// 2. 统一各工具极简说明文案（100% 统一工具命名）
const modalDesc = computed(() => {
  switch (ui.dependencyTargetTool) {
    case 'asr':
    case 'video-subtitle':
      return '首次使用「视频字幕生成」需下载本地离线模型，完成后自动进入。';
    case 'upscale':
    case 'upscale-48k':
      return '首次使用「4K/8K 图像超分」需下载本地离线模型，完成后自动进入。';
    case 'pdf':
    case 'pdf-parse':
      return '首次使用「PDF 智能解析」需下载本地离线模型，完成后自动进入。';
    default:
      return '当前功能需要下载本地离线模型，下载完成后自动进入。';
  }
});

// 计算总体下载进度 (0 - 100%)
const overallPercent = computed(() => {
  const items = ui.dependencyItems;
  if (!items || items.length === 0) return 100;
  let totalPercent = 0;
  for (const item of items) {
    if (item.is_ready) {
      totalPercent += 100;
      continue;
    }
    const prog = progressMap.value[item.id];
    if (prog) {
      if (prog.phase === 'complete') totalPercent += 100;
      else if (prog.phase === 'verifying') totalPercent += 98;
      else totalPercent += prog.percent || 0;
    }
  }
  return Math.min(100, Math.round(totalPercent / items.length));
});

const getItemStatusText = (item: DependencyItem) => {
  if (item.is_ready) return '已就绪';
  const prog = progressMap.value[item.id];
  if (!prog) return '等待下载';
  if (prog.phase === 'verifying') return '正在校验...';
  if (prog.phase === 'complete') return '已就绪';
  return `下载中 ${prog.percent}%`;
};

const getItemStatusClass = (item: DependencyItem) => {
  if (item.is_ready) return 'text-emerald-600 font-bold';
  const prog = progressMap.value[item.id];
  if (!prog) return 'text-slate-400 font-bold';
  if (prog.phase === 'verifying') return 'text-amber-500 font-bold';
  if (prog.phase === 'complete') return 'text-emerald-600 font-bold';
  return 'text-[#02c3b4] font-bold';
};

const handleStartDownload = async () => {
  if (isDownloading.value || !ui.dependencyTargetTool) return;
  isDownloading.value = true;
  try {
    const success = await downloadToolDependencies(ui.dependencyTargetTool);
    if (success) {
      ui.弹出提示('🎉 依赖模型已全部就绪！', 'success');
      isDownloading.value = false;
      const targetTool = ui.dependencyTargetTool;
      const cb = ui.dependencyCallback;
      ui.closeDependencyModal();
      if (cb) {
        cb();
      } else if (targetTool) {
        ui.currentView = targetTool as any;
      }
    }
  } catch (err: any) {
    isDownloading.value = false;
    ui.弹出提示(`🚨 依赖下载异常: ${err}`, 'error');
  }
};

const handleCancel = async () => {
  if (isDownloading.value) {
    try {
      await cancelDependencyDownloads();
      ui.弹出提示('已取消依赖下载', 'info');
    } catch {}
  }
  isDownloading.value = false;
  ui.closeDependencyModal();
};

onMounted(async () => {
  try {
    unlistenProgress = await listen<DependencyProgressPayload>(
      'dependency-download-progress',
      (event) => {
        const payload = event.payload;
        if (!payload?.item_id) return;
        progressMap.value[payload.item_id] = payload;
        if (payload.phase === 'complete') {
          const matched = ui.dependencyItems.find((i) => i.id === payload.item_id);
          if (matched) matched.is_ready = true;
        }
      }
    );
  } catch (e) {
    console.warn('⚠️ 依赖下载进度事件监听未激活:', e);
  }
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
});
</script>

<template>
  <div
    v-if="ui.isDependencyModalOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/60 backdrop-blur-md animate-in fade-in duration-150 select-none pointer-events-auto"
  >
    <div
      class="max-w-[380px] sm:max-w-[400px] w-full p-6 sm:p-7 rounded-[32px] bg-white border border-slate-100 shadow-[0_25px_60px_rgba(0,0,0,0.35)] text-center space-y-4 flex flex-col items-center animate-in zoom-in-95 duration-150 text-slate-900"
    >
      <!-- 顶部圆形下载图标 -->
      <div
        class="w-14 h-14 rounded-full bg-[#18181b] text-white flex items-center justify-center shadow-lg shrink-0"
      >
        <Download :size="24" class="stroke-[2.5]" />
      </div>

      <!-- 极简标题与说明 -->
      <div class="space-y-1.5 w-full">
        <h3 class="text-lg sm:text-xl font-black text-slate-900 tracking-tight">
          {{ modalTitle }}
        </h3>
        <p class="text-xs sm:text-[13px] text-slate-500 font-medium leading-relaxed max-w-[320px] mx-auto">
          {{ modalDesc }}
        </p>
      </div>

      <!-- 依赖项清单 -->
      <div class="w-full bg-[#f4f4f5]/90 border border-slate-200/60 rounded-2xl p-3.5 space-y-2 text-left">
        <div
          v-for="item in ui.dependencyItems"
          :key="item.id"
          class="flex justify-between items-center text-xs py-1"
        >
          <span class="font-medium text-slate-700 truncate max-w-[200px]" :title="item.name">
            {{ item.name }} · {{ item.size_formatted }}
          </span>
          <span
            class="font-mono text-[11.5px] font-bold shrink-0"
            :class="getItemStatusClass(item)"
          >
            {{ getItemStatusText(item) }}
          </span>
        </div>
      </div>

      <!-- 下载总体进度条 -->
      <div v-if="isDownloading" class="w-full space-y-1.5 pt-0.5">
        <div class="w-full h-2 bg-slate-100 rounded-full overflow-hidden border border-slate-200/50">
          <div
            class="h-full bg-[#18181b] transition-all duration-300 rounded-full"
            :style="{ width: `${overallPercent}%` }"
          ></div>
        </div>
        <div class="flex justify-between text-[10px] font-mono text-slate-400 font-bold">
          <span>正在下载...</span>
          <span>{{ overallPercent }}%</span>
        </div>
      </div>

      <!-- 操作按钮组 -->
      <div class="w-full grid grid-cols-2 gap-3 pt-1">
        <button
          type="button"
          @click="handleCancel"
          class="py-3 px-4 rounded-2xl bg-[#f4f4f5] hover:bg-[#e4e4e7] text-slate-800 font-bold text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center shadow-sm"
        >
          取消
        </button>
        <button
          type="button"
          :disabled="isDownloading"
          @click="handleStartDownload"
          class="py-3 px-4 rounded-2xl bg-[#18181b] hover:bg-[#27272a] disabled:bg-slate-700 text-white font-bold text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1.5 shadow-md"
        >
          <Loader2 v-if="isDownloading" :size="14" class="animate-spin" />
          <span>{{ isDownloading ? `正在下载 (${overallPercent}%)...` : '立即下载' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
