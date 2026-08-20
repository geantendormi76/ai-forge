<script setup lang="ts">
import { Activity, Mic } from 'lucide-vue-next';
import CountUp from './CountUp.vue';
import ElectricBorder from './ElectricBorder.vue';
</script>

<template>
  <ElectricBorder color="#A5F3FC" :speed="0.6" :chaos="0.08" :borderRadius="16" class="w-full h-56">
    <div class="relative w-full h-full p-4 sm:p-5 bg-slate-950/90 rounded-2xl flex flex-col justify-between overflow-hidden shadow-inner font-mono">


      <div class="space-y-1 relative z-10">
        <div class="flex items-center justify-between text-xs text-slate-400">
          <span class="flex items-center gap-1.5 text-[#A5F3FC] font-semibold">
            <Mic class="w-3.5 h-3.5 text-[#A5F3FC]" />
            MOSS 0.9B ASR 离线转写
          </span>
          <span class="px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-[10px]">
            VramGuard 1.8G 锁定
          </span>
        </div>

        <div class="flex items-baseline gap-2 pt-1">
          <div class="text-3xl font-black tracking-tight bg-gradient-to-r from-[#A5F3FC] via-slate-100 to-[#D8B4F8] bg-clip-text text-transparent">
            <CountUp :to="3.47" :decimals="2" suffix="x" :duration="1.8" />
          </div>
          <span class="text-xs text-slate-400">实时转写倍速</span>
        </div>
      </div>


      <div class="relative w-full h-16 my-1">
        <svg class="w-full h-full" viewBox="0 0 200 50" fill="none" preserveAspectRatio="none">
          <defs>
            <linearGradient id="starFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#A5F3FC" stop-opacity="0.3" />
              <stop offset="100%" stop-color="#D8B4F8" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path
            d="M0 40 C20 35, 40 38, 60 28 C80 18, 100 25, 120 15 C140 5, 160 18, 180 8 C190 3, 195 5, 200 2 L200 50 L0 50Z"
            fill="url(#starFill)"
          />
          <path
            class="ln-feat-stars-line"
            d="M0 40 C20 35, 40 38, 60 28 C80 18, 100 25, 120 15 C140 5, 160 18, 180 8 C190 3, 195 5, 200 2"
            stroke="#A5F3FC"
            stroke-width="2"
            stroke-linecap="round"
          />
        </svg>
      </div>


      <div class="flex items-center justify-between text-[11px] text-slate-400 border-t border-slate-800/80 pt-2 relative z-10">
        <span class="flex items-center gap-1.5">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
          时间戳精度 0.01s | 角色 [S01]/[S02]
        </span>
        <Activity class="w-3.5 h-3.5 text-[#D8B4F8] animate-pulse" />
      </div>

    </div>
  </ElectricBorder>
</template>

<style scoped>
.ln-feat-stars-line {
  stroke-dasharray: 240;
  stroke-dashoffset: 240;
  animation: drawLine 2s ease-out forwards;
}

@keyframes drawLine {
  to {
    stroke-dashoffset: 0;
  }
}
</style>
