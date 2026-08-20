<template>
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16 relative z-10">
    <div class="relative rounded-3xl border border-slate-800/80 bg-slate-900/90 p-8 sm:p-14 text-center overflow-hidden shadow-2xl backdrop-blur-xl group hover:border-[#D8B4F8]/50 transition-all">
      
      <!-- 发光背景 -->
      <div class="absolute -inset-1 bg-gradient-to-r from-[#A5F3FC]/10 via-[#D8B4F8]/10 to-[#A5F3FC]/10 rounded-3xl blur-xl opacity-50 group-hover:opacity-100 transition-opacity pointer-events-none"></div>

      <div class="relative z-10 space-y-6 max-w-2xl mx-auto">
        <GradientText
          text="Stop paying for AI subscriptions."
          :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC']"
          :animation-speed="5"
          class="text-3xl sm:text-5xl font-black tracking-tight"
        />

        <p class="text-slate-200 text-sm sm:text-base leading-relaxed">
          100% 离线、脱网稳定运行。基于本地 RTX 3060 算力直推，无隐私泄露风险，永久免费。
        </p>

        <div class="flex flex-wrap items-center justify-center gap-4 pt-4">
          <a
            href="#tools-matrix"
            class="px-8 py-4 rounded-2xl bg-gradient-to-r from-[#A5F3FC] to-[#D8B4F8] hover:opacity-90 text-slate-950 font-extrabold text-sm tracking-wide transition-all shadow-xl shadow-purple-500/20 hover:scale-[1.02] flex items-center gap-2"
          >
            <span>立即体验全套 AI 工具</span>
            <ArrowRight class="w-4 h-4" />
          </a>

          <a
            href="https://geantendormi.top"
            target="_blank"
            class="px-6 py-4 rounded-2xl bg-slate-950 hover:bg-slate-800 text-slate-200 border border-slate-800 font-semibold text-sm transition-all flex items-center gap-2"
          >
            <Globe class="w-4 h-4 text-[#A5F3FC]" />
            <span>访问公网主站</span>
          </a>
        </div>
      </div>

    </div>
  </section>
</template>

<script setup lang="ts">
import { ArrowRight, Globe } from 'lucide-vue-next';
import GradientText from './GradientText.vue';
</script>
