<template>
  <div class="flex h-screen bg-slate-50 overflow-hidden font-sans select-none">
    <!-- 侧边栏 Sidebar (紫电 AI 纯净白风格) -->
    <aside class="w-64 bg-white border-r border-slate-200 flex flex-col justify-between shadow-sm z-20">
      <div>
        <!-- 应用 Logo 标语区 -->
        <div class="p-6 border-b border-slate-100 flex items-center gap-3">
          <div class="w-10 h-10 bg-[#DCA54C] rounded-xl flex items-center justify-center text-white text-xl shadow-md font-bold shrink-0">
            ⚡
          </div>
          <div class="overflow-hidden">
            <ShinyText
              text="紫电 AI"
              :speed="3.5"
              :spread="150"
              color="#bc05ff"
              shine-color="#DCA54C"
              class="text-base font-black tracking-tight leading-tight block"
            />
            <p class="text-[11px] text-slate-400 font-medium truncate mt-0.5">端侧隐私智能工坊</p>
          </div>
        </div>

        <!-- 侧边栏导航菜单 -->
        <nav class="p-4 space-y-1.5">
          <!-- 1. 全能格式转换 (主力引流层) -->
          <button
            @click="activeTab = 'format'"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold transition-all cursor-pointer',
              activeTab === 'format'
                ? 'bg-amber-50 text-[#DCA54C] border border-amber-100 shadow-sm'
                : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 border border-transparent'
            ]"
          >
            <span class="text-base">⚡</span>
            <span class="truncate">{{ t('nav.formatConverter') }}</span>
          </button>

          <!-- 2. PDF 智能解析 -->
          <button
            @click="activeTab = 'pdf'"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold transition-all cursor-pointer',
              activeTab === 'pdf'
                ? 'bg-blue-50 text-blue-600 border border-blue-100 shadow-sm'
                : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 border border-transparent'
            ]"
          >
            <span class="text-base">📄</span>
            <span class="truncate">{{ t('nav.pdfParse') }}</span>
          </button>

          <!-- 3. 视频双语字幕工坊 -->
          <button
            @click="activeTab = 'asr'"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold transition-all cursor-pointer',
              activeTab === 'asr'
                ? 'bg-purple-50 text-purple-600 border border-purple-100 shadow-sm'
                : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 border border-transparent'
            ]"
          >
            <span class="text-base">🎬</span>
            <span class="truncate">{{ t('nav.asr') }}</span>
          </button>
        </nav>
      </div>

      <!-- 底部硬件与算力状态卡片 -->
      <div class="p-4 border-t border-slate-100">
        <div class="bg-slate-50 border border-slate-200/80 rounded-xl p-3 text-xs">
          <div class="flex items-center justify-between text-slate-600 mb-1">
            <span class="font-medium">端侧算力节点</span>
            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          </div>
          <p class="text-[11px] text-slate-400 font-mono">RTX 3060 12GB (Local)</p>
        </div>
      </div>
    </aside>

    <!-- 主工作区内容视图 -->
    <main class="flex-1 overflow-y-auto relative">
      <FormatConverterView v-if="activeTab === 'format'" />
      <PdfParseView v-else-if="activeTab === 'pdf'" />
      <VideoSubtitleView v-else-if="activeTab === 'asr'" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import ShinyText from './components/effects/ShinyText.vue';
import FormatConverterView from './views/FormatConverterView.vue';
import PdfParseView from './views/PdfParseView.vue';
import VideoSubtitleView from './views/VideoSubtitleView.vue';

const { t } = useI18n();
const activeTab = ref<'format' | 'pdf' | 'asr'>('format');
</script>
