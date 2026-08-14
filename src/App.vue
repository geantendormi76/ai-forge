<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useUIStore } from './store/uiStore'
import { useTauriWindow } from './composables/useTauriWindow'
import Sidebar from './components/layout/Sidebar.vue'
import HomeView from './views/HomeView.vue'
import FormatConverterView from './views/FormatConverterView.vue'
import PdfParseView from './views/PdfParseView.vue'
import VideoSubtitleView from './views/VideoSubtitleView.vue'
import Button from './components/ui/button/Button.vue'
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
  <div class="h-screen w-full flex overflow-hidden font-sans select-none bg-[#F7F7F8] text-slate-800 relative">
    
    <!-- 悬浮侧边栏展开按钮 (当侧边栏收起时显现) -->
    <div
      v-if="ui.侧边栏收起"
      class="fixed left-4 top-2 z-[100] pointer-events-auto group/expand"
      style="-webkit-app-region: no-drag;"
    >
      <Button
        @click="ui.切换侧边栏"
        variant="outline"
        size="icon"
        class="h-9 w-9 rounded-xl bg-white border border-slate-200 text-slate-500 hover:text-slate-800 hover:bg-slate-50 shadow-[0_4px_12px_rgba(15,23,42,0.06)] cursor-pointer transition-all hover:scale-105 flex items-center justify-center"
      >
        <PanelLeftOpen :size="16" class="stroke-[2]" />
      </Button>
      <div class="absolute top-11 left-1/2 -translate-x-1/2 bg-white border border-slate-200 rounded-xl px-3 py-1.5 shadow-[0_8px_24px_rgba(15,23,42,0.08)] flex items-center justify-center pointer-events-none opacity-0 scale-95 group-hover/expand:opacity-100 group-hover/expand:scale-100 transition-all duration-200 select-none z-50">
        <span class="text-[11px] font-black text-slate-700 whitespace-nowrap">展开侧边栏</span>
      </div>
    </div>

    <!-- 悬浮胶囊侧边栏 -->
    <Sidebar />

    <!-- 主工作区内容容器 -->
    <main class="flex-1 flex flex-col relative min-w-0">
      
      <!-- 无边框原生顶栏与物理窗口控制器 -->
      <div class="h-12 w-full flex justify-between items-center shrink-0 absolute top-0 left-0 right-0 z-50 pointer-events-none">
        <div data-tauri-drag-region class="h-full flex-1 pointer-events-auto" style="-webkit-app-region: drag;"></div>
        <div class="flex items-center h-full pointer-events-auto pr-2" style="-webkit-app-region: no-drag;">
          <button
            @click="触发最小化"
            class="h-8 w-10 flex items-center justify-center text-slate-400 hover:text-slate-700 hover:bg-slate-200/50 rounded transition-colors cursor-pointer"
            title="最小化"
          >
            <Minus :size="14" class="stroke-[2.5]" />
          </button>
          <button
            @click="触发最大化还原"
            class="h-8 w-10 flex items-center justify-center text-slate-400 hover:text-slate-700 hover:bg-slate-200/50 rounded transition-colors cursor-pointer"
            :title="是否已最大化 ? '向下还原' : '最大化'"
          >
            <Square v-if="!是否已最大化" :size="12" class="stroke-[2.5]" />
            <Copy v-else :size="12" class="stroke-[2.5]" />
          </button>
          <button
            @click="触发关闭"
            class="h-8 w-10 flex items-center justify-center text-slate-400 hover:text-white hover:bg-rose-500 rounded transition-colors cursor-pointer"
            title="关闭"
          >
            <X :size="16" class="stroke-[2.5]" />
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

    <!-- 全局 Toast 提示气泡 -->
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
        <div class="px-5 py-3 rounded-2xl bg-white border border-slate-100 shadow-[0_8px_30px_rgb(0,0,0,0.08)] flex items-center gap-3 select-none pointer-events-auto max-w-[400px]">
          <div
            class="flex items-center justify-center w-6 h-6 rounded-full shrink-0"
            :class="[
              ui.toast类型 === 'success' ? 'bg-emerald-50 text-emerald-600' :
              ui.toast类型 === 'error' ? 'bg-red-50 text-red-600' :
              'bg-blue-50 text-blue-600'
            ]"
          >
            <Check v-if="ui.toast类型 === 'success'" :size="12" class="stroke-[3]" />
            <AlertCircle v-else-if="ui.toast类型 === 'error'" :size="12" class="stroke-[2.5]" />
            <Info v-else :size="12" class="stroke-[2.5]" />
          </div>
          <span class="text-[13px] font-medium text-slate-700 tracking-wide truncate" :title="ui.toast消息 || ''">
            {{ ui.toast消息 }}
          </span>
        </div>
      </Transition>
    </div>

  </div>
</template>
