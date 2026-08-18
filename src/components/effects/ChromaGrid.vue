<script setup lang="ts">
import { computed } from 'vue';

export interface ChromaCardItem {
  id?: string;
  image?: string;
  title: string;
  subtitle: string;
  borderColor?: string;
  url?: string;
  viewTarget?: string;
}

interface GridMotionProps {
  items?: ChromaCardItem[];
  className?: string;
}

const props = withDefaults(defineProps<GridMotionProps>(), {
  items: () => [],
  className: '',
});

const emit = defineEmits<{
  (e: 'cardClick', item: ChromaCardItem): void;
}>();

const data = computed(() => props.items);

const handleCardClick = (item: ChromaCardItem) => {
  if (item.url) {
    window.open(item.url, '_blank', 'noopener,noreferrer');
  }
  emit('cardClick', item);
};

const handleCardMove = (e: MouseEvent) => {
  const c = e.currentTarget as HTMLElement;
  const rect = c.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  c.style.setProperty('--mouse-x', `${x}px`);
  c.style.setProperty('--mouse-y', `${y}px`);
};
</script>

<template>
  <div
    :class="['relative w-full flex flex-wrap justify-center sm:justify-start items-stretch gap-4 select-none', className]"
  >
    <article
      v-for="(c, i) in data"
      :key="i"
      class="group relative flex flex-col w-full sm:w-[320px] lg:w-[335px] p-2.5 rounded-[22px] overflow-hidden border border-white/10 hover:border-[#02c3b4]/60 bg-[#182022]/85 hover:bg-[#182022]/95 backdrop-blur-2xl transition-all duration-300 shadow-[0_12px_36px_rgba(0,0,0,0.35)] hover:shadow-[0_16px_48px_rgba(2,195,180,0.18)] cursor-pointer"
      :style="{
        '--mouse-x': '50%',
        '--mouse-y': '50%',
        '--spotlight-color': 'rgba(2, 195, 180, 0.25)',
      }"
      @mousemove="handleCardMove"
      @click="() => handleCardClick(c)"
    >
      <!-- 动态聚光灯漫反射 -->
      <div
        class="absolute inset-0 pointer-events-none transition-opacity duration-500 z-20 opacity-0 group-hover:opacity-100"
        :style="{
          background:
            'radial-gradient(circle 220px at var(--mouse-x) var(--mouse-y), var(--spotlight-color), transparent 75%)'
        }"
      />

      <!-- 🌟 第 1 层：黄金 16:10 视觉封面区 (70% 紧凑画幅) -->
      <div v-if="c.image" class="relative z-10 w-full aspect-[16/10] overflow-hidden rounded-[16px] border border-white/[0.08] bg-[#0a0f10]">
        <img
          :src="c.image"
          :alt="c.title"
          loading="lazy"
          class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-500 block select-none pointer-events-none"
        />
      </div>

      <!-- 🌟 下半区文字容器 -->
      <div class="relative z-10 px-2 pt-2.5 pb-1 space-y-1 flex-1 flex flex-col justify-between">
        <!-- 🌟 第 2 层：标题与行动导向区 -->
        <div class="flex items-center justify-between">
          <h3 class="text-base sm:text-lg font-black text-white tracking-tight group-hover:text-[#02c3b4] transition-colors">
            {{ c.title }}
          </h3>
          <span class="text-[#02c3b4] font-bold text-sm opacity-0 group-hover:opacity-100 group-hover:translate-x-0.5 transition-all duration-200">
            ➔
          </span>
        </div>

        <!-- 🌟 第 3 层：语义说明与微文案 -->
        <p class="m-0 text-[11px] sm:text-[12px] text-[#8b999b] leading-relaxed group-hover:text-[#d1dddf] transition-colors line-clamp-1">
          {{ c.subtitle }}
        </p>
      </div>
    </article>
  </div>
</template>
