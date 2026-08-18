<script setup lang="ts">
import { ref, computed } from 'vue';
import { useUIStore } from '../store/uiStore';
import Aurora from '../components/effects/Aurora.vue';
import DotField from '../components/effects/DotField.vue';
import GradientText from '../components/effects/GradientText.vue';
import ChromaGrid, { type ChromaCardItem } from '../components/effects/ChromaGrid.vue';
import videoCoverSvg from '../assets/tools/video-subtitle/cover.svg';
import {
  Film,
  Search,
  Sparkles,
} from 'lucide-vue-next';

const ui = useUIStore();

// 🌟 1. 上半区右翼：三层立体旗舰级已就绪算子 (装配 cover.svg)
const flagshipTools: ChromaCardItem[] = [
  {
    id: 'asr',
    image: videoCoverSvg,
    title: '视频字幕生成',
    subtitle: '端侧离线语音识别、双语神经翻译与显卡硬字幕压制',
    viewTarget: 'asr',
  },
];

// 🌟 2. 下半区：全能工具导航矩阵 (1:1 对齐图二三层海报卡片)
const searchQuery = ref('');
const activeCategory = ref('all');

interface ToolItem {
  id: string;
  name: string;
  image?: string;
  category: 'core' | 'document' | 'media' | 'system';
  desc: string;
  viewTarget?: 'asr' | 'format' | 'pdf';
}

// 🛡️ 生产级收敛：1:1 挂载高清封面与作用
const toolMatrix: ToolItem[] = [
  {
    id: 'asr',
    name: '视频字幕生成',
    image: videoCoverSvg,
    category: 'media',
    desc: '端侧离线语音识别、双语神经翻译与显卡硬字幕压制',
    viewTarget: 'asr',
  },
];

const categoryList = [
  { id: 'all', label: '全部工具', icon: Sparkles },
  { id: 'media', label: '音视频矩阵', icon: Film },
];

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return toolMatrix.filter((tool) => {
    const matchCategory = activeCategory.value === 'all' || tool.category === activeCategory.value;
    const matchSearch =
      !q ||
      tool.name.toLowerCase().includes(q) ||
      tool.desc.toLowerCase().includes(q);
    return matchCategory && matchSearch;
  });
});

const handleToolClick = (tool: ToolItem) => {
  if (tool.viewTarget === 'asr') {
    ui.currentView = 'asr';
  } else {
    ui.弹出提示(`💡 [${tool.name}] 正在端侧并网构建中，敬请期待...`, 'info');
  }
};

const handleChromaClick = (item: ChromaCardItem) => {
  if (item.viewTarget === 'asr') {
    ui.currentView = 'asr';
  } else {
    ui.弹出提示('💡 该算子正在端侧并网构建中...', 'info');
  }
};
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans flex flex-col">
    
    <!-- 全屏极光背景 (优雅微光，不夺目) -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <Aurora
        :speed="0.8"
        :amplitude="1.2"
        :color-stops="['#101e21', '#02c3b4', '#00d2ff']"
        class="absolute inset-0 opacity-40"
      />
    </div>

    <!-- 全屏交互点阵力场 -->
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

    <!-- 主工作区滚动容器 -->
    <div
      class="relative z-10 w-full h-full overflow-y-auto custom-scrollbar pt-12 pb-8 transition-all duration-300 ease-in-out pointer-events-auto"
      :class="ui.侧边栏收起 ? 'px-8 sm:px-12 lg:px-14' : 'pl-[260px] pr-8 sm:pr-12 lg:pr-14'"
    >
      <div class="max-w-[1580px] w-full mx-auto space-y-8 flex flex-col justify-between pt-2">

        <!-- 🌟 上半区：紫电 AI 理念 (左 5 列) + ChromaGrid 三层立体卡片 (右 7 列) -->
        <section class="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center pt-2">
          
          <!-- 左翼 (5列 / 41.7%)：纯净品牌与理念 -->
          <div class="lg:col-span-5 flex flex-col justify-center space-y-3">
            <!-- 核心紫电 AI 渐变字标 -->
            <div>
              <GradientText
                text="紫电AI"
                :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
                :animation-speed="6"
                class="text-6xl sm:text-7xl font-black tracking-tight"
              />
            </div>

            <!-- 标语 -->
            <p class="text-slate-300 text-xs sm:text-sm leading-relaxed max-w-xl font-sans">
              <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
              <span class="block text-slate-100 mt-0.5">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
            </p>
          </div>

          <!-- 右翼 (7列 / 58.3%)：三层高定 ChromaGrid 封面卡片 -->
          <div class="lg:col-span-7 flex flex-col justify-center">
            <ChromaGrid
              :items="flagshipTools"
              @card-click="handleChromaClick"
            />
          </div>
        </section>

        <!-- 🌟 下半区：全能工具导航矩阵与搜索分类 (1:1 图二海报卡片) -->
        <section id="tool-explorer-section" class="space-y-4 pt-1">
          <div class="flex flex-col md:flex-row md:items-center justify-between gap-3 border-b border-white/[0.06] pb-3">
            <div>
              <h2 class="text-base sm:text-lg font-bold text-white tracking-wide">全能工具导航矩阵</h2>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
              <!-- 分类切换按钮组 -->
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

              <!-- 全局快速搜索框 -->
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

          <!-- 🌟 1:1 对齐图二的三层立体海报卡片流 -->
          <div v-if="filteredTools.length > 0" class="flex flex-wrap justify-center sm:justify-start items-stretch gap-5">
            <article
              v-for="tool in filteredTools"
              :key="tool.id"
              @click="handleToolClick(tool)"
              class="group relative flex flex-col w-full sm:w-[320px] lg:w-[335px] p-2.5 rounded-[22px] overflow-hidden border border-white/10 hover:border-[#02c3b4]/60 bg-[#182022]/85 hover:bg-[#182022]/95 backdrop-blur-2xl transition-all duration-300 shadow-[0_12px_36px_rgba(0,0,0,0.35)] hover:shadow-[0_16px_48px_rgba(2,195,180,0.18)] cursor-pointer select-none"
            >
              <!-- 🌟 第 1 层：16:10 高清海报封面 -->
              <div v-if="tool.image" class="relative z-10 w-full aspect-[16/10] overflow-hidden rounded-[16px] border border-white/[0.08] bg-[#0a0f10]">
                <img
                  :src="tool.image"
                  :alt="tool.name"
                  loading="lazy"
                  class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-500 block select-none pointer-events-none"
                />
              </div>

              <!-- 🌟 第 2 & 3 层：标题与作用说明 -->
              <div class="relative z-10 px-2 pt-2.5 pb-1 space-y-1 flex-1 flex flex-col justify-between">
                <div class="flex items-center justify-between">
                  <h3 class="text-base sm:text-lg font-black text-white tracking-tight group-hover:text-[#02c3b4] transition-colors">
                    {{ tool.name }}
                  </h3>
                  <span class="text-[#02c3b4] font-bold text-sm opacity-0 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all duration-200">
                    ➔
                  </span>
                </div>
                <p class="m-0 text-[11px] sm:text-[12px] text-[#8b999b] leading-relaxed group-hover:text-[#d1dddf] transition-colors line-clamp-1">
                  {{ tool.desc }}
                </p>
              </div>
            </article>
          </div>

          <!-- 搜索无结果空状态 -->
          <div v-else class="py-10 text-center text-[#8b999b] border border-white/[0.04] rounded-2xl bg-white/[0.01]">
            <p class="text-xs">未找到与 "{{ searchQuery }}" 匹配的工具算子</p>
          </div>
        </section>

      </div>
    </div>

  </div>
</template>
