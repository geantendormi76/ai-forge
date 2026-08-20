<script setup lang="ts">
import { ref } from 'vue';

type Variant = 'shapegrid' | 'magicrings' | 'shinytext' | 'dock';

interface Props {
  variant: Variant;
  active?: boolean;
}

withDefaults(defineProps<Props>(), {
  active: false
});

const dockItems = [
  { label: 'Home', icon: 'M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z M9 22V12h6v10' },
  { label: 'Archive', icon: 'M21 8v13H3V8 M1 3h22v5H1z M10 12h4' },
  { label: 'Search', icon: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z M21 21l-4.35-4.35' },
  { label: 'Profile', icon: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z' },
  { label: 'Settings', icon: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z' }
];

const hoveredDockIndex = ref<number | null>(null);
</script>

<template>
  <div class="demo-fill relative w-full h-full min-h-[200px] bg-slate-950/90 rounded-2xl overflow-hidden flex items-center justify-center border border-slate-800/80">
    
    <!-- 1. ShapeGrid: 蜂窝六边形网格 -->
    <div v-if="variant === 'shapegrid'" class="absolute inset-0 flex items-center justify-center p-2 opacity-80">
      <svg class="w-full h-full" viewBox="0 0 400 200" fill="none">
        <pattern id="hexGrid" width="40" height="34.64" patternUnits="userSpaceOnUse">
          <path d="M 40 0 L 30 17.32 L 10 17.32 L 0 0 L 10 -17.32 L 30 -17.32 Z" fill="none" stroke="rgba(148, 163, 184, 0.15)" stroke-width="1" />
          <path d="M 20 17.32 L 10 34.64 L -10 34.64 L -20 17.32 L -10 0 L 10 0 Z" fill="none" stroke="rgba(148, 163, 184, 0.15)" stroke-width="1" />
        </pattern>
        <rect width="100%" height="100%" fill="url(#hexGrid)" />
        <path d="M 120 34.64 L 110 51.96 L 90 51.96 L 80 34.64 L 90 17.32 L 110 17.32 Z" fill="rgba(0, 255, 169, 0.35)" stroke="#00ffa9" stroke-width="1.5" class="animate-pulse" />
        <path d="M 200 69.28 L 190 86.6 L 170 86.6 L 160 69.28 L 170 51.96 L 190 51.96 Z" fill="rgba(0, 210, 255, 0.4)" stroke="#00d2ff" stroke-width="1.5" class="animate-pulse [animation-delay:0.5s]" />
      </svg>
    </div>

    <!-- 2. MagicRings: 荧光光晕圆环 -->
    <div v-else-if="variant === 'magicrings'" class="absolute inset-0 flex items-center justify-center">
      <div class="relative w-48 h-48 flex items-center justify-center">
        <div class="absolute inset-0 rounded-full border-2 border-[#00ffa9]/60 shadow-[0_0_30px_rgba(0,255,169,0.3)] animate-ping [animation-duration:3s]"></div>
        <div class="absolute w-36 h-36 rounded-full border-2 border-[#42fcff]/60 shadow-[0_0_20px_rgba(66,252,255,0.3)] animate-pulse"></div>
        <div class="absolute w-24 h-24 rounded-full border border-[#00ffa9]/80 shadow-[0_0_15px_rgba(0,255,169,0.4)]"></div>
        <div class="w-12 h-12 rounded-full bg-[#00ffa9]/20 border border-[#00ffa9] backdrop-blur-md flex items-center justify-center shadow-lg">
          <div class="w-3 h-3 rounded-full bg-[#00ffa9] animate-pulse"></div>
        </div>
      </div>
    </div>

    <!-- 3. ShinyText: 金属漫反射流光字 -->
    <div v-else-if="variant === 'shinytext'" class="absolute inset-0 flex items-center justify-center p-4">
      <div class="text-3xl sm:text-4xl font-extrabold tracking-tight bg-gradient-to-r from-slate-500 via-white to-slate-500 bg-clip-text text-transparent animate-shimmer bg-[length:200%_100%]">
        Shiny Text
      </div>
    </div>

    <!-- 4. Dock: macOS 悬浮浮动工具栏 -->
    <div v-else-if="variant === 'dock'" class="absolute inset-0 flex items-center justify-center p-4">
      <div class="flex items-center gap-3 px-4 py-2.5 rounded-2xl bg-slate-900/90 border border-slate-800/90 shadow-2xl backdrop-blur-xl">
        <div
          v-for="(item, idx) in dockItems"
          :key="idx"
          @mouseenter="hoveredDockIndex = idx"
          @mouseleave="hoveredDockIndex = null"
          class="p-2.5 rounded-xl bg-slate-950/80 border border-slate-800/80 hover:border-[#A5F3FC] transition-all duration-200 cursor-pointer transform hover:-translate-y-1.5 hover:scale-125 hover:shadow-lg hover:shadow-[#A5F3FC]/20"
        >
          <svg class="w-5 h-5 text-[#A5F3FC]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path :d="item.icon" />
          </svg>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

.animate-shimmer {
  animation: shimmer 3s infinite linear;
}
</style>
