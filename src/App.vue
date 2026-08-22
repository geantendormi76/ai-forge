<script setup lang="ts">
import { onMounted } from 'vue';
import { useUIStore } from './store/uiStore';
import { useTauriWindow } from './composables/useTauriWindow';
import Sidebar from './components/layout/Sidebar.vue';
import DependencyGateModal from './components/modals/DependencyGateModal.vue';
import AppUpdateModal from './components/modals/AppUpdateModal.vue';
import QuotaModal from './components/modals/QuotaModal.vue';
import HomeView from './views/HomeView.vue';
import Upscale48kView from './views/Upscale48kView.vue';
import FormatConverterView from './views/FormatConverterView.vue';
import PdfParseView from './views/PdfParseView.vue';
import VideoSubtitleView from './views/VideoSubtitleView.vue';
import {
  Minus,
  Square,
  Copy,
  X,
  Check,
  AlertCircle,
  Info,
} from 'lucide-vue-next';

const ui = useUIStore();
const { 是否已最大化, 触发最小化, 触发最大化还原, 触发关闭 } = useTauriWindow();

onMounted(() => {
  if (!import.meta.env.DEV) {
    window.addEventListener('contextmenu', (e) => e.preventDefault());
  }
});
</script>

<template>
  <div class="h-screen w-full overflow-hidden font-sans select-none bg-[#20292b] text-[#f5f5f3] relative">
    <!-- 1. 全景主工作区 -->
    <main class="absolute inset-0 w-full h-full overflow-hidden z-0">
      <HomeView v-if="ui.currentView === 'home'" />
      <Upscale48kView v-else-if="ui.currentView === 'upscale'" />
      <FormatConverterView v-else-if="ui.currentView === 'format'" />
      <PdfParseView v-else-if="ui.currentView === 'pdf'" />
      <VideoSubtitleView v-else-if="ui.currentView === 'asr'" />
    </main>

    <!-- 2. 🌟 SOTA 原生拖拽顶栏 -->
    <div class="h-11 w-full flex justify-between items-center absolute top-0 left-0 right-0 z-20 pointer-events-none">
      <div data-tauri-drag-region class="h-full flex-1 pointer-events-auto select-none" style="-webkit-app-region: drag;"></div>
      <!-- 右上角物理窗口控制器 (最高层级 z-50) -->
      <div class="flex items-center h-full pointer-events-auto pr-3 gap-1 z-50" style="-webkit-app-region: no-drag;">
        <button
          type="button"
          @click="触发最小化"
          class="h-7 w-9 flex items-center justify-center text-[#8b999b] hover:text-white hover:bg-white/10 rounded-lg transition-colors cursor-pointer"
          title="最小化"
        >
          <Minus :size="13" class="stroke-[2.5]" />
        </button>
        <button
          type="button"
          @click="触发最大化还原"
          class="h-7 w-9 flex items-center justify-center text-[#8b999b] hover:text-white hover:bg-white/10 rounded-lg transition-colors cursor-pointer"
          :title="是否已最大化 ? '向下还原' : '最大化'"
        >
          <Square v-if="!是否已最大化" :size="11" class="stroke-[2.5]" />
          <Copy v-else :size="11" class="stroke-[2.5]" />
        </button>
        <button
          type="button"
          @click="触发关闭"
          class="h-7 w-9 flex items-center justify-center text-[#8b999b] hover:text-white hover:bg-rose-600 rounded-lg transition-colors cursor-pointer"
          title="关闭"
        >
          <X :size="14" class="stroke-[2.5]" />
        </button>
      </div>
    </div>

    <!-- 3. 🌟 智能双态胶囊侧边栏 (最高交互层级 z-50) -->
    <Sidebar />

    <!-- 4. 🌟 模型依赖感知下载模态框 -->
    <DependencyGateModal />

    <!-- 5. 🌟 2026 SOTA 全生命周期版本热更新模态框 -->
    <AppUpdateModal />

    <!-- 6. 🌟 端侧算力配额与账单明细模态框 -->
    <QuotaModal />

    <!-- 7. 全局暗黑微光 Toast 提示气泡 -->
    <div v-if="ui.toast显示" class="fixed inset-0 z-[9999] flex items-center justify-center pointer-events-none">
      <Transition
        appear
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-95 translate-y-4"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100 translate-y-0"
        leave-to-class="opacity-0 scale-95 translate-y-2"
      >
        <div class="px-5 py-3 rounded-2xl bg-[#182022]/95 backdrop-blur-2xl border border-white/15 shadow-[0_20px_60px_rgba(0,0,0,0.8)] flex items-center gap-3 select-none pointer-events-auto max-w-[420px]">
          <div
            class="flex items-center justify-center w-6 h-6 rounded-full shrink-0"
            :class="[
              ui.toast类型 === 'success' ? 'bg-[#02c3b4]/20 text-[#02c3b4] border border-[#02c3b4]/40 shadow-[0_0_12px_rgba(2,195,180,0.3)]' :
              ui.toast类型 === 'error' ? 'bg-rose-500/20 text-rose-400 border border-rose-500/30' :
              'bg-blue-500/20 text-blue-400 border border-blue-500/30'
            ]"
          >
            <Check v-if="ui.toast类型 === 'success'" :size="12" class="stroke-[3]" />
            <AlertCircle v-else-if="ui.toast类型 === 'error'" :size="12" class="stroke-[2.5]" />
            <Info v-else :size="12" class="stroke-[2.5]" />
          </div>
          <span class="text-[13px] font-medium text-[#f2f2ef] tracking-wide truncate" :title="ui.toast消息 || ''">
            {{ ui.toast消息 }}
          </span>
        </div>
      </Transition>
    </div>
  </div>
</template>
