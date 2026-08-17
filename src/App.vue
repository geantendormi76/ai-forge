<script setup lang="ts">
import { onMounted } from 'vue'
import { useUIStore } from './store/uiStore'
import { useTauriWindow } from './composables/useTauriWindow'
import Sidebar from './components/layout/Sidebar.vue'
import HomeView from './views/HomeView.vue'
import FormatConverterView from './views/FormatConverterView.vue'
import PdfParseView from './views/PdfParseView.vue'
import VideoSubtitleView from './views/VideoSubtitleView.vue'
import {
  Minus, Square, Copy, X, Check, AlertCircle, Info,
  PanelLeftOpen
} from 'lucide-vue-next'

const ui = useUIStore()
const { 是否已最大化, 触发最小化, 触发最大化还原, 触发关闭 } = useTauriWindow()

onMounted(() => {
  if (!import.meta.env.DEV) {
    window.addEventListener('contextmenu', (e) => e.preventDefault())
  }
})
</script>

<template>
  <div class="h-screen w-full flex overflow-hidden font-sans select-none bg-[#060607] text-[#f5f5f3] relative">
    
    <!-- 悬浮侧边栏展开按钮 (当侧边栏收起时显现) -->
    <div
      v-if="ui.侧边栏收起"
      class="fixed left-4 top-3 z-[100] pointer-events-auto group/expand"
      style="-webkit-app-region: no-drag;"
    >
      <button
        @click="ui.切换侧边栏"
        class="h-9 w-9 rounded-xl bg-[#0e0e10]/85 backdrop-blur-xl border border-white/10 text-[#8b8b87] hover:text-white hover:bg-white/10 shadow-[0_8px_24px_rgba(0,0,0,0.6)] cursor-pointer transition-all hover:scale-105 flex items-center justify-center"
        title="展开侧边栏"
      >
        <PanelLeftOpen :size="16" class="stroke-[2]" />
      </button>
      <div class="absolute top-11 left-1/2 -translate-x-1/2 bg-[#0e0e10]/95 backdrop-blur-xl border border-white/15 rounded-xl px-3 py-1.5 shadow-[0_12px_32px_rgba(0,0,0,0.8)] flex items-center justify-center pointer-events-none opacity-0 scale-95 group-hover/expand:opacity-100 group-hover/expand:scale-100 transition-all duration-200 select-none z-50">
        <span class="text-[11px] font-bold text-[#f2f2ef] whitespace-nowrap">展开侧边栏</span>
      </div>
    </div>

    <!-- 悬浮暗黑胶囊侧边栏 -->
    <Sidebar />

    <!-- 主工作区内容容器 -->
    <main class="flex-1 flex flex-col relative min-w-0">
      
      <!-- 无边框原生顶栏与物理窗口控制器 -->
      <div class="h-12 w-full flex justify-between items-center shrink-0 absolute top-0 left-0 right-0 z-50 pointer-events-none">
        <div data-tauri-drag-region class="h-full flex-1 pointer-events-auto" style="-webkit-app-region: drag;"></div>
        <div class="flex items-center h-full pointer-events-auto pr-3 gap-1" style="-webkit-app-region: no-drag;">
          <button
            @click="触发最小化"
            class="h-7 w-9 flex items-center justify-center text-[#8b8b87] hover:text-white hover:bg-white/10 rounded-lg transition-colors cursor-pointer"
            title="最小化"
          >
            <Minus :size="13" class="stroke-[2.5]" />
          </button>
          <button
            @click="触发最大化还原"
            class="h-7 w-9 flex items-center justify-center text-[#8b8b87] hover:text-white hover:bg-white/10 rounded-lg transition-colors cursor-pointer"
            :title="是否已最大化 ? '向下还原' : '最大化'"
          >
            <Square v-if="!是否已最大化" :size="11" class="stroke-[2.5]" />
            <Copy v-else :size="11" class="stroke-[2.5]" />
          </button>
          <button
            @click="触发关闭"
            class="h-7 w-9 flex items-center justify-center text-[#8b8b87] hover:text-white hover:bg-rose-600 rounded-lg transition-colors cursor-pointer"
            title="关闭"
          >
            <X :size="14" class="stroke-[2.5]" />
          </button>
        </div>
      </div>

      <!-- 视图路由分发区 (带顶部 48px 留白避开拖拽轨) -->
      <div class="flex-1 w-full h-full overflow-hidden relative z-10 pt-12">
        <HomeView v-if="ui.currentView === 'home'" />
        <FormatConverterView v-else-if="ui.currentView === 'format'" />
        <PdfParseView v-else-if="ui.currentView === 'pdf'" />
        <VideoSubtitleView v-else-if="ui.currentView === 'asr'" />
      </div>
    </main>

    <!-- 全局暗黑 Toast 提示气泡 -->
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
        <div class="px-5 py-3 rounded-2xl bg-[#0e0e10]/95 backdrop-blur-2xl border border-white/15 shadow-[0_20px_60px_rgba(0,0,0,0.8)] flex items-center gap-3 select-none pointer-events-auto max-w-[420px]">
          <div
            class="flex items-center justify-center w-6 h-6 rounded-full shrink-0"
            :class="[
              ui.toast类型 === 'success' ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30' :
              ui.toast类型 === 'error' ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30' :
              'bg-blue-500/15 text-blue-400 border border-blue-500/30'
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
