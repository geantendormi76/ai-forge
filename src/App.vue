<template>
  <div class="flex h-screen bg-slate-50 overflow-hidden font-sans">
    <!-- 侧边栏 Sidebar (紫电 AI RPA 纯净白风格) -->
    <aside class="w-64 bg-white border-r border-slate-200 flex flex-col justify-between shadow-sm z-10">
      <div>
        <!-- 应用 Logo 标语区 -->
        <div class="p-6 border-b border-slate-100 flex items-center gap-3">
          <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center text-white text-xl shadow-md font-bold">
            ⚡
          </div>
          <div>
            <h1 class="text-base font-bold text-slate-900 leading-tight">紫电 AI</h1>
            <p class="text-xs text-slate-400 font-medium">端侧隐私智能工坊</p>
          </div>
        </div>

        <!-- 侧边栏导航菜单 -->
        <nav class="p-4 space-y-1.5">
          <button 
            @click="activeTab = 'pdf'"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-semibold transition-all',
              activeTab === 'pdf' 
                ? 'bg-blue-50 text-blue-600 border border-blue-100 shadow-sm' 
                : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'
            ]"
          >
            <span class="text-base">📄</span>
            {{ t('nav.pdfParse') }}
          </button>

          <button 
            @click="activeTab = 'asr'"
            :class="[
              'w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-semibold transition-all',
              activeTab === 'asr' 
                ? 'bg-blue-50 text-blue-600 border border-blue-100 shadow-sm' 
                : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900'
            ]"
          >
            <span class="text-base">🎬</span>
            {{ t('nav.asr') }}
          </button>
        </nav>
      </div>

      <!-- 底部硬件与版本状态面板 -->
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
    <main class="flex-1 overflow-y-auto">
      <PdfParseView v-if="activeTab === 'pdf'" />
      <VideoSubtitleView v-else-if="activeTab === 'asr'" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import PdfParseView from './views/PdfParseView.vue';
import VideoSubtitleView from './views/VideoSubtitleView.vue';

const { t } = useI18n();
const activeTab = ref<'pdf' | 'asr'>('pdf');
</script>
