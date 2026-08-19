<script setup lang="ts">
import { ref, computed } from 'vue';
import { useUIStore } from '../store/uiStore';
import Aurora from '../components/effects/Aurora.vue';
import DotField from '../components/effects/DotField.vue';
import GradientText from '../components/effects/GradientText.vue';
import ChromaGrid, { type ChromaCardItem } from '../components/effects/ChromaGrid.vue';
import videoCoverSvg from '../assets/tools/video-subtitle/cover.svg';
import pdfCoverSvg from '../assets/tools/pdf-parse/cover.svg';
import formatCoverSvg from '../assets/tools/format-converter/cover.svg';
import {
  Film,
  FileText,
  Search,
  Sparkles,
  Zap
} from 'lucide-vue-next';

const ui = useUIStore();

// 🌟 1. 全库已竣工与已规划工具资产总注册表 (Master Tool Registry)
interface ToolItem {
  id: string;
  name: string;
  image?: string;
  subtitle: string;
  category: 'core' | 'document' | 'media' | 'system';
  desc: string;
  viewTarget?: 'asr' | 'format' | 'pdf';
}

const masterToolList: ToolItem[] = [
  {
    id: 'asr',
    name: '视频字幕生成',
    image: videoCoverSvg,
    subtitle: '端侧离线语音识别、双语神经翻译与显卡硬压',
    category: 'media',
    desc: '端侧离线语音识别、双语神经翻译与显卡硬压',
    viewTarget: 'asr',
  },
  {
    id: 'pdf',
    name: 'PDF 智能解析',
    image: pdfCoverSvg,
    subtitle: '双轨排版重构、公式转写与表格还原',
    category: 'document',
    desc: '双轨多模态排版重构、公式转写与表格还原',
    viewTarget: 'pdf',
  },
  {
    id: 'format',
    name: '全能格式转换',
    image: formatCoverSvg,
    subtitle: '音频解密、结构数据、电子书与图标',
    category: 'core',
    desc: '音频解密、表格清洗、电子书与原生图标转换',
    viewTarget: 'format',
  },
];

// 🌟 2. 核心算法：基于用户高频使用统计动态计算浮现 Top 3
const flagshipTools = computed<ChromaCardItem[]>(() => {
  // 按实际使用频次降序排序
  const sorted = [...masterToolList].sort((a, b) => {
    const countA = ui.toolUsageCounts[a.id] || 0;
    const countB = ui.toolUsageCounts[b.id] || 0;
    return countB - countA;
  });

  // 严格截取前 3 名
  return sorted.slice(0, 3).map((t) => ({
    id: t.id,
    image: t.image,
    title: t.name,
    subtitle: t.subtitle,
    viewTarget: t.viewTarget,
  }));
});

// 🌟 3. 下半区：全能工具导航矩阵与搜索分类
const searchQuery = ref('');
const activeCategory = ref('all');

const categoryList = [
  { id: 'all', label: '全部工具', icon: Sparkles },
  { id: 'core', label: '核心工具', icon: Zap },
  { id: 'document', label: '文档重构', icon: FileText },
  { id: 'media', label: '音视频矩阵', icon: Film },
];

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return masterToolList.filter((tool) => {
    const matchCategory = activeCategory.value === 'all' || tool.category === activeCategory.value;
    const matchSearch =
      !q ||
      tool.name.toLowerCase().includes(q) ||
      tool.desc.toLowerCase().includes(q);
    return matchCategory && matchSearch;
  });
});

const handleToolClick = (tool: ToolItem) => {
  ui.recordToolUsage(tool.id);
  if (tool.viewTarget === 'format') {
    ui.currentView = 'format';
  } else if (tool.viewTarget === 'pdf') {
    ui.currentView = 'pdf';
  } else if (tool.viewTarget === 'asr') {
    ui.currentView = 'asr';
  } else {
    ui.弹出提示(`💡 [${tool.name}] 正在端侧并网构建中，敬请期待...`, 'info');
  }
};

const handleChromaClick = (item: ChromaCardItem) => {
  if (item.id) {
    ui.recordToolUsage(item.id);
  }
  if (item.viewTarget === 'format') {
    ui.currentView = 'format';
  } else if (item.viewTarget === 'pdf') {
    ui.currentView = 'pdf';
  } else if (item.viewTarget === 'asr') {
    ui.currentView = 'asr';
  } else {
    ui.弹出提示('💡 该算子正在端侧并网构建中...', 'info');
  }
};
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans flex flex-col">
    
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <Aurora
        :speed="0.8"
        :amplitude="1.2"
        :color-stops="['#101e21', '#02c3b4', '#00d2ff']"
        class="absolute inset-0 opacity-40"
      />
    </div>

    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <DotField
        :dot-radius="1.2"
        :dot-spacing="16"
        :bulge-strength="55"
        :glow-radius="180"
        gradient-from="rgba(2, 195, 180, 0.35)"
        gradient-to="rgba(0, 210, 255, 0.12)"
        glow-color="#02c3b4"
        class="absolute inset-0 opacity-60"
      />
    </div>

    <div
      class="relative z-10 w-full h-full overflow-y-auto custom-scrollbar pt-10 pb-8 transition-all duration-300 ease-in-out pointer-events-auto"
      :class="ui.侧边栏收起 ? 'px-6 sm:px-10 lg:px-12' : 'pl-[256px] pr-6 sm:pr-10 lg:pr-12'"
    >
      <div class="max-w-[1580px] w-full mx-auto space-y-6 flex flex-col justify-between pt-1">

        <!-- 🌟 上半区：4:8 黄金栅格，根据频次动态浮现的 Top 3 旗舰卡片 -->
        <section class="grid grid-cols-1 lg:grid-cols-12 gap-5 items-center">
          <!-- 左翼：紫电理念 (4 列) -->
          <div class="lg:col-span-4 flex flex-col justify-center space-y-2">
            <div>
              <GradientText
                text="紫电AI"
                :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
                :animation-speed="6"
                class="text-3xl sm:text-4xl lg:text-5xl font-black tracking-tight"
              />
            </div>

            <p class="text-slate-300 text-xs sm:text-[13px] leading-relaxed max-w-sm font-sans">
              <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
              <span class="block text-slate-100 mt-0.5">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
            </p>
          </div>

          <!-- 右翼：流体等分网格 (8 列，动态 Top 3 卡片) -->
          <div class="lg:col-span-8 flex flex-col justify-center w-full min-w-0">
            <ChromaGrid
              :items="flagshipTools"
              @card-click="handleChromaClick"
            />
          </div>
        </section>

        <!-- 🌟 下半区：全能工具导航矩阵 (自适应等宽流) -->
        <section id="tool-explorer-section" class="space-y-4 pt-2">
          <div class="flex flex-col md:flex-row md:items-center justify-between gap-3 border-b border-white/[0.06] pb-3">
            <div>
              <h2 class="text-base sm:text-lg font-bold text-white tracking-wide">全能工具导航矩阵</h2>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
              <div class="flex items-center gap-1.5">
                <button
                  v-for="cat in categoryList"
                  :key="cat.id"
                  type="button"
                  @click="activeCategory = cat.id"
                  class="h-7 px-3 rounded-xl text-xs font-bold flex items-center gap-1 transition-all cursor-pointer select-none"
                  :class="activeCategory === cat.id ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/50 text-[#02c3b4] shadow-[0_0_12px_rgba(2,195,180,0.2)]' : 'bg-white/[0.02] border border-white/[0.06] text-[#8b999b] hover:text-white hover:bg-white/[0.04]'"
                >
                  <component :is="cat.icon" :size="12" />
                  <span>{{ cat.label }}</span>
                </button>
              </div>

              <div class="relative w-full sm:w-[240px]">
                <Search :size="13" class="absolute left-3 top-1/2 -translate-y-1/2 text-[#8b999b]" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索全库算子..."
                  class="w-full h-8 pl-8 pr-3 bg-white/[0.02] border border-white/[0.08] hover:border-white/[0.15] focus:border-[#02c3b4]/50 focus:ring-1 focus:ring-[#02c3b4]/30 rounded-xl text-xs text-[#f5f5f3] outline-none transition-all placeholder-[#5b696b] backdrop-blur-md"
                />
              </div>
            </div>
          </div>

          <!-- 🌟 下半区卡片网格 -->
          <div v-if="filteredTools.length > 0" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
            <article
              v-for="tool in filteredTools"
              :key="tool.id"
              @click="handleToolClick(tool)"
              class="group relative flex flex-col w-full min-w-0 p-2.5 rounded-[22px] overflow-hidden border border-white/10 hover:border-[#02c3b4]/60 bg-[#182022]/85 hover:bg-[#182022]/95 backdrop-blur-2xl transition-all duration-300 shadow-[0_12px_36px_rgba(0,0,0,0.35)] hover:shadow-[0_16px_48px_rgba(2,195,180,0.18)] cursor-pointer select-none"
            >
              <div v-if="tool.image" class="relative z-10 w-full aspect-[16/10] overflow-hidden rounded-[16px] border border-white/[0.08] bg-[#0a0f10]">
                <img
                  :src="tool.image"
                  :alt="tool.name"
                  loading="lazy"
                  class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-500 block select-none pointer-events-none"
                />
              </div>

              <div class="relative z-10 px-2 pt-2.5 pb-1 space-y-1 flex-1 flex flex-col justify-between">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm sm:text-base font-black text-white tracking-tight group-hover:text-[#02c3b4] transition-colors truncate">
                    {{ tool.name }}
                  </h3>
                  <span class="text-[#02c3b4] font-bold text-sm opacity-0 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all duration-200 shrink-0">
                    ➔
                  </span>
                </div>
                <p class="m-0 text-[11px] sm:text-[12px] text-[#8b999b] leading-relaxed group-hover:text-[#d1dddf] transition-colors line-clamp-1">
                  {{ tool.desc }}
                </p>
              </div>
            </article>
          </div>

          <div v-else class="py-10 text-center text-[#8b999b] border border-white/[0.04] rounded-2xl bg-white/[0.01]">
            <p class="text-xs">未找到与 "{{ searchQuery }}" 匹配的工具算子</p>
          </div>
        </section>

      </div>
    </div>

  </div>
</template>
