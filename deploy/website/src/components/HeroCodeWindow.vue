<template>
  <div class="relative w-full max-w-2xl rounded-3xl border border-white/15 bg-[#14110E]/30 backdrop-blur-2xl shadow-[0_25px_60px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,255,255,0.15)] overflow-hidden transition-all duration-300 group hover:border-[#00ffa9]/50 font-mono select-none p-5 sm:p-6 space-y-4">


    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-[#00ffa9]/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>🌐 Hy-MT2 极速翻译</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-300 text-[10px] font-bold border border-cyan-500/30 shrink-0">
            1.8B GGUF
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">热推演:</span>
            <span class="text-xl sm:text-2xl font-black text-[#00ffa9] whitespace-nowrap">
              <CountUp :to="386.67" :decimals="2" suffix=" ms" :duration="1.8" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>缓存碰撞: <strong class="text-emerald-400">1.39 ms</strong></span>
            <span class="text-slate-400">120+ Tokens/s</span>
          </div>
        </div>
      </div>


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-amber-500/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>🎨 8K 图像超分重构</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-300 text-[10px] font-bold border border-amber-500/30 shrink-0">
            CUDA 13.0
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">GPU 渲染:</span>
            <span class="text-xl sm:text-2xl font-black text-amber-300 whitespace-nowrap">
              <CountUp :to="2.54" :decimals="2" suffix=" MP/s" :duration="1.8" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>极限画质:</span>
            <span class="text-slate-200 font-medium">8192×7335 (6008万像素)</span>
          </div>
        </div>
      </div>


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-[#A5F3FC]/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>🎙️ MOSS 0.9B 语音听写</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-purple-500/15 text-[#D8B4F8] text-[10px] font-bold border border-purple-500/30 shrink-0">
            Speech-LLM
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">实时转写:</span>
            <span class="text-xl sm:text-2xl font-black text-[#A5F3FC] whitespace-nowrap">
              <CountUp :to="3.47" :decimals="2" suffix="x" :duration="1.8" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>精度 / 角色:</span>
            <span class="text-emerald-400 font-bold">0.01s | 100% [S01]/[S02]</span>
          </div>
        </div>
      </div>


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-blue-500/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>📑 PDF 智能图文解析</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-blue-500/15 text-blue-300 text-[10px] font-bold border border-blue-500/30 shrink-0">
            PP-DocLayout
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">热解析耗时:</span>
            <span class="text-xl sm:text-2xl font-black text-blue-300 whitespace-nowrap">
              <CountUp :to="8.53" :decimals="2" suffix=" s" :duration="1.8" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>5维探针路由:</span>
            <span class="text-emerald-400 font-bold">100% 还原表格与 KaTeX</span>
          </div>
        </div>
      </div>


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-emerald-500/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>✂️ PDF 页面切片瘦身</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-emerald-500/15 text-emerald-400 text-[10px] font-bold border border-emerald-500/30 shrink-0">
            C++ 原生算子
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">物理抹图:</span>
            <span class="text-xl sm:text-2xl font-black text-emerald-400 whitespace-nowrap">
              <CountUp :to="233" suffix=" ms" :duration="1.8" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>瘦身率: <strong class="text-emerald-300">98.5%</strong></span>
            <span class="text-slate-400">14.3M ➔ 0.21M</span>
          </div>
        </div>
      </div>


      <div class="p-3.5 rounded-2xl bg-black/25 border border-white/10 hover:border-rose-500/60 hover:bg-black/40 backdrop-blur-md transition-all duration-300 space-y-2 shadow-lg">
        <div class="flex items-center justify-between gap-1 whitespace-nowrap">
          <span class="font-extrabold text-slate-100 text-xs sm:text-sm flex items-center gap-1.5 truncate">
            <span>🎬 音视频提取与裁剪</span>
          </span>
          <span class="px-1.5 py-0.5 rounded bg-rose-500/15 text-rose-300 text-[10px] font-bold border border-rose-500/30 shrink-0">
            FFmpeg 引擎
          </span>
        </div>

        <div class="space-y-0.5">
          <div class="flex items-baseline justify-between gap-1">
            <span class="text-slate-300/80 text-[11px]">无损切片:</span>
            <span class="text-xl sm:text-2xl font-black text-rose-300 whitespace-nowrap">
              <CountUp :to="0.5" :decimals="1" suffix=" s" :duration="1.5" />
            </span>
          </div>
          <div class="text-[10px] text-slate-400 pt-1 border-t border-white/10 flex items-center justify-between whitespace-nowrap">
            <span>音视频合并:</span>
            <span class="text-slate-200 font-medium">B站/YouTube 4K 直提取</span>
          </div>
        </div>
      </div>

    </div>

  </div>
</template>

<script setup lang="ts">
import CountUp from './CountUp.vue';
</script>
