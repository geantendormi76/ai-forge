<script setup lang="ts">
import { Sparkles, Mic, FileText, Zap, Languages, Video, Code, Cpu, Eye, ShieldCheck, Database } from 'lucide-vue-next';
</script>

<template>
  <div class="ln-feat-orbit relative w-full h-56 flex items-center justify-center overflow-hidden">
    
    <!-- 中心 AI 算力核心 -->
    <div class="relative z-20 w-12 h-12 rounded-2xl bg-slate-900 border border-[#A5F3FC]/40 flex items-center justify-center shadow-lg shadow-[#A5F3FC]/20 animate-pulse">
      <Sparkles class="w-6 h-6 text-[#A5F3FC]" />
    </div>

    <!-- 内轨道 (顺时针公转) -->
    <div class="absolute w-32 h-32 rounded-full border border-dashed border-[#A5F3FC]/25 animate-orbit-cw z-10 flex items-center justify-center">
      <div class="absolute -top-3.5 p-1.5 rounded-full bg-slate-900 border border-[#A5F3FC]/40 text-[#A5F3FC] shadow-sm animate-orbit-ccw">
        <Mic class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -bottom-3.5 p-1.5 rounded-full bg-slate-900 border border-[#A5F3FC]/40 text-[#A5F3FC] shadow-sm animate-orbit-ccw">
        <FileText class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -left-3.5 p-1.5 rounded-full bg-slate-900 border border-[#A5F3FC]/40 text-[#A5F3FC] shadow-sm animate-orbit-ccw">
        <Zap class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -right-3.5 p-1.5 rounded-full bg-slate-900 border border-[#A5F3FC]/40 text-[#A5F3FC] shadow-sm animate-orbit-ccw">
        <Languages class="w-3.5 h-3.5" />
      </div>
    </div>

    <!-- 外轨道 (逆时针公转) -->
    <div class="absolute w-48 h-48 rounded-full border border-dashed border-[#D8B4F8]/20 animate-orbit-ccw-slow z-0 flex items-center justify-center">
      <div class="absolute -top-3.5 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <Video class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -bottom-3.5 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <Cpu class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -left-3.5 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <Code class="w-3.5 h-3.5" />
      </div>
      <div class="absolute -right-3.5 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <ShieldCheck class="w-3.5 h-3.5" />
      </div>
      <div class="absolute top-6 left-6 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <Eye class="w-3.5 h-3.5" />
      </div>
      <div class="absolute bottom-6 right-6 p-1.5 rounded-full bg-slate-900 border border-[#D8B4F8]/40 text-[#D8B4F8] shadow-sm animate-orbit-cw-slow">
        <Database class="w-3.5 h-3.5" />
      </div>
    </div>

  </div>
</template>

<style scoped>
@keyframes orbit-cw {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes orbit-ccw {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(-360deg); }
}

.animate-orbit-cw {
  animation: orbit-cw 20s linear infinite;
}

.animate-orbit-ccw {
  animation: orbit-ccw 20s linear infinite;
}

.animate-orbit-ccw-slow {
  animation: orbit-ccw 28s linear infinite;
}

.animate-orbit-cw-slow {
  animation: orbit-cw 28s linear infinite;
}
</style>
