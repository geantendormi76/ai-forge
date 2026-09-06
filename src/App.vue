<script setup lang="ts">
import { onMounted } from 'vue';
import { useUIStore } from './store/uiStore';
import { useTauriWindow } from './composables/useTauriWindow';
import Sidebar from './components/layout/Sidebar.vue';
import DependencyGateModal from './components/modals/DependencyGateModal.vue';
import AppUpdateModal from './components/modals/AppUpdateModal.vue';
import QuotaModal from './components/modals/QuotaModal.vue';
import HomeView from './views/HomeView.vue';
import ChatAgent from './views/ChatAgent.vue';
import Upscale48kView from './views/Upscale48kView.vue';
import FormatConverterView from './views/FormatConverterView.vue';
import PdfParseView from './views/PdfParseView.vue';
import VideoSubtitleView from './views/VideoSubtitleView.vue';
import TranslationView from './views/TranslationView.vue';
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
  <div class="h-screen w-full overflow-hidden font-sans select-none bg-[#f7f6f3] dark:bg-[#141210] text-[#1c1a17] dark:text-[#faf9f7] relative">
    <!-- 1. 全景主工作区 -->
    <main class="absolute inset-0 w-full h-full overflow-hidden z-0">
      <HomeView v-if="ui.currentView === 'home'" />
      <ChatAgent v-else-if="ui.currentView === 'agent'" />
      <TranslationView v-else-if="ui.currentView === 'trans'" />
      <Upscale48kView v-else-if="ui.currentView === 'upscale'" />
      <FormatConverterView v-else-if="ui.currentView === 'format'" />
      <PdfParseView v-else-if="ui.currentView === 'pdf'" />
      <VideoSubtitleView v-else-if="ui.currentView === 'asr'" />
    </main>

    <!-- 2. 原生拖拽顶栏 -->
    <div class="h-10 w-full flex justify-between items-center absolute top-0 left-0 right-0 z-40 pointer-events-none">
      <div data-tauri-drag-region class="h-full flex-1 pointer-events-auto select-none" style="-webkit-app-region: drag;"></div>
      <!-- 右上角物理窗口控制器 -->
      <div class="flex items-center h-full pointer-events-auto pr-3 gap-1 z-50" style="-webkit-app-region: no-drag;">
        <button
          type="button"
          @click="触发最小化"
          class="h-7 w-8 flex items-center justify-center text-[#746f66] hover:text-[#1c1a17] hover:bg-black/5 rounded-md transition-colors cursor-pointer"
          title="最小化"
        >
          <Minus :size="13" class="stroke-[2]" />
        </button>
        <button
          type="button"
          @click="触发最大化还原"
          class="h-7 w-8 flex items-center justify-center text-[#746f66] hover:text-[#1c1a17] hover:bg-black/5 rounded-md transition-colors cursor-pointer"
          :title="是否已最大化 ? '向下还原' : '最大化'"
        >
          <Square v-if="!是否已最大化" :size="11" class="stroke-[2]" />
          <Copy v-else :size="11" class="stroke-[2]" />
        </button>
        <button
          type="button"
          @click="触发关闭"
          class="h-7 w-8 flex items-center justify-center text-[#746f66] hover:text-white hover:bg-rose-600 rounded-md transition-colors cursor-pointer"
          title="关闭"
        >
          <X :size="13" class="stroke-[2]" />
        </button>
      </div>
    </div>

    <!-- 3. 工具箱侧边栏（仅在非 Agent 独立工作台视图下渲染） -->
    <Sidebar v-if="ui.currentView !== 'agent'" />

    <!-- 4. 全局模态框 -->
    <DependencyGateModal />
    <AppUpdateModal />
    <QuotaModal />

    <!-- 5. 全局 Toast 提示 -->
    <div v-if="ui.toast显示" class="fixed inset-0 z-[9999] flex items-center justify-center pointer-events-none">
      <Transition
        appear
        enter-active-class="transition duration-150 ease-out"
        enter-from-class="opacity-0 scale-95 translate-y-2"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition duration-100 ease-in"
        leave-from-class="opacity-100 scale-100 translate-y-0"
        leave-to-class="opacity-0 scale-95 translate-y-2"
      >
        <div class="px-4 py-2.5 rounded-xl bg-[#1c1a17]/95 text-white backdrop-blur-xl border border-white/10 shadow-2xl flex items-center gap-2.5 select-none pointer-events-auto max-w-[420px] text-xs font-mono">
          <Check v-if="ui.toast类型 === 'success'" :size="13" class="text-emerald-400 stroke-[3]" />
          <AlertCircle v-else-if="ui.toast类型 === 'error'" :size="13" class="text-rose-400 stroke-[2.5]" />
          <Info v-else :size="13" class="text-blue-400 stroke-[2.5]" />
          <span>{{ ui.toast消息 }}</span>
        </div>
      </Transition>
    </div>
  </div>
</template>
