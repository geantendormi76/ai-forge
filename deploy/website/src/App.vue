<template>
  <div class="min-h-screen bg-[#14110E] text-[#f5f5f3] flex flex-col justify-between selection:bg-[#00ffa9]/30 selection:text-[#00ffa9] font-sans">
    
    <!-- 顶部导航栏 -->
    <header class="w-full border-b border-white/[0.08] bg-[#14110E]/80 backdrop-blur-xl sticky top-0 z-50">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-xl bg-gradient-to-tr from-[#bc05ff] to-[#00ffa9] flex items-center justify-center shadow-[0_0_16px_rgba(0,255,169,0.4)]">
            <Zap :size="18" class="text-slate-950 fill-slate-950 stroke-[2.5]" />
          </div>
          <span class="text-lg font-black tracking-tight text-white">紫电 AI</span>
          <span class="text-[10px] font-mono font-bold bg-[#00ffa9]/15 text-[#00ffa9] px-2 py-0.5 rounded-full border border-[#00ffa9]/30">
            SOTA 2026 离线版
          </span>
        </div>

        <div class="flex items-center gap-4">
          <a href="#showcase" class="text-xs font-bold text-[#8b999b] hover:text-white transition-colors hidden sm:inline-block">
            功能展台
          </a>
          <a href="#tokenomics" class="text-xs font-bold text-[#8b999b] hover:text-white transition-colors hidden sm:inline-block">
            算力体系
          </a>
          <a
            :href="clientDownloadUrl"
            class="h-9 px-4 rounded-xl bg-gradient-to-r from-[#00ffa9] to-[#00d2ff] hover:opacity-95 text-slate-950 font-black text-xs transition-all active:scale-95 flex items-center gap-1.5 shadow-[0_0_20px_rgba(0,255,169,0.3)]"
          >
            <Download :size="14" class="stroke-[2.5]" />
            <span>下载客户端 (35MB)</span>
          </a>
        </div>
      </div>
    </header>

    <!-- 核心主体 -->
    <main class="flex-1">
      
      <!-- 英雄区：波浪 + 粒子 + 6 颗性能战报卡片 -->
      <section class="relative w-full overflow-hidden bg-[#14110E] pt-14 pb-20 sm:pt-16 sm:pb-24">
        
        <!-- 背景 WebGL 波浪与点阵 -->
        <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
          <HeroBand
            color="#00ffa9"
            :speed="0.2"
            :frequency="1"
            :noise="0.05"
            :band-width="0.14"
            :rotation="90"
            :fade-top="0.75"
            :iterations="1"
            :intensity="1.25"
            class="absolute inset-0 opacity-90"
          />
          <DotField
            :dot-radius="1.5"
            :dot-spacing="14"
            :bulge-strength="67"
            :glow-radius="200"
            gradient-from="rgba(0, 255, 169, 0.45)"
            gradient-to="rgba(0, 210, 255, 0.35)"
            glow-color="#00ffa9"
            class="absolute inset-0 opacity-80"
          />
        </div>

        <!-- 底部平滑渐变 -->
        <svg class="absolute bottom-0 left-0 right-0 h-44 w-full pointer-events-none z-10" preserveAspectRatio="none" viewBox="0 0 1 1">
          <defs>
            <linearGradient id="hero-bottom-fade" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#14110E" stop-opacity="0" />
              <stop offset="30%" stop-color="#14110E" stop-opacity="0.05" />
              <stop offset="55%" stop-color="#14110E" stop-opacity="0.25" />
              <stop offset="75%" stop-color="#14110E" stop-opacity="0.65" />
              <stop offset="90%" stop-color="#14110E" stop-opacity="0.9" />
              <stop offset="100%" stop-color="#14110E" stop-opacity="1" />
            </linearGradient>
          </defs>
          <rect width="1" height="1" fill="url(#hero-bottom-fade)" />
        </svg>

        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative z-20">
          <div class="grid grid-cols-1 lg:grid-cols-12 gap-8 lg:gap-10 items-center min-h-[520px]">

            <!-- 左翼：大字标 + 理念 + 下载 CTA -->
            <div class="lg:col-span-6 space-y-6">
              <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-black/40 border border-white/10 text-xs font-mono text-slate-300 shadow-lg backdrop-blur-md w-fit">
                <span class="px-2 py-0.5 rounded-full bg-[#00ffa9] text-slate-950 font-extrabold text-[11px] tracking-wide">
                  SOTA 2026
                </span>
                <span class="flex items-center gap-1.5 font-medium text-slate-200">
                  RTX 3060 12G 算力直连 · 纯血 Rust Native
                </span>
              </div>

              <div>
                <GradientText
                  text="紫电AI"
                  :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
                  :animation-speed="6"
                  class="text-6xl sm:text-8xl font-black tracking-tight"
                />
              </div>

              <p class="text-slate-300 text-sm sm:text-base leading-relaxed max-w-xl font-sans space-y-1">
                <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
                <span class="block text-slate-100">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
              </p>

              <!-- 一键下载 Windows 客户端大胶囊 -->
              <div class="pt-2 flex flex-col sm:flex-row items-stretch sm:items-center gap-4">
                <a
                  :href="clientDownloadUrl"
                  class="h-13 px-8 rounded-2xl bg-gradient-to-r from-[#00ffa9] to-[#00d2ff] hover:opacity-95 text-slate-950 font-black text-sm tracking-wide shadow-[0_12px_36px_rgba(0,255,169,0.35)] transition-all active:scale-95 flex items-center justify-center gap-2.5 group"
                >
                  <Download :size="18" class="stroke-[2.5] group-hover:-translate-y-0.5 transition-transform" />
                  <span>立即下载 Windows 客户端 (.exe)</span>
                </a>

                <a
                  href="#tokenomics"
                  class="h-13 px-5 rounded-2xl bg-white/[0.04] hover:bg-white/[0.08] border border-white/10 text-slate-200 font-bold text-xs transition-all flex items-center justify-center gap-2 backdrop-blur-md"
                >
                  <Flame :size="15" class="text-[#ffb74d] fill-[#ffb74d]" />
                  <span>内测每日送 600 点</span>
                </a>
              </div>

              <p class="text-[11px] font-mono text-[#5b696b]">
                适用于 Windows 10 / 11 (x64) · 约 35 MB 极轻安装 · 100% 离线隐私推演
              </p>
            </div>

            <!-- 右翼：6 颗实时性能战报卡片 -->
            <div class="lg:col-span-6 flex items-center justify-center w-full">
              <HeroCodeWindow />
            </div>

          </div>
        </div>
      </section>

      <!-- 下半区：One tool. Endless content. 交互展示台 -->
      <section id="showcase" class="relative z-20">
        <LiveDemo />
      </section>

      <!-- 算力定价与 Tokenomics 说明 -->
      <section id="tokenomics" class="py-20 border-t border-white/[0.06] bg-black/40 relative z-20">
        <div class="max-w-5xl mx-auto px-6 space-y-10">
          <div class="text-center space-y-2">
            <div class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-[#00ffa9]/10 text-[#00ffa9] font-mono text-xs font-bold border border-[#00ffa9]/20">
              <Flame :size="13" class="fill-[#00ffa9]" />
              <span>Tokenomics 算力代币模型</span>
            </div>
            <h2 class="text-2xl sm:text-3xl font-black text-white">透明亲民的算力计费矩阵</h2>
            <p class="text-xs sm:text-sm text-[#8b999b]">1 Token = ¥0.01 · 每日 00:00 UTC 自动刷新免费配额</p>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
            <div class="p-5 rounded-2xl bg-white/[0.02] border border-white/10 space-y-2 text-center">
              <div class="text-xs text-[#8b999b]">📄 PDF 智能解析</div>
              <div class="text-xl font-black font-mono text-[#00ffa9]">1 点 / 页</div>
              <div class="text-[11px] text-[#5b696b]">折合 ¥0.01 / 页</div>
            </div>
            <div class="p-5 rounded-2xl bg-white/[0.02] border border-white/10 space-y-2 text-center">
              <div class="text-xs text-[#8b999b]">🔍 4K/8K 图像超分</div>
              <div class="text-xl font-black font-mono text-[#00ffa9]">2 点 / 张</div>
              <div class="text-[11px] text-[#5b696b]">秒级 4K 重构</div>
            </div>
            <div class="p-5 rounded-2xl bg-white/[0.02] border border-white/10 space-y-2 text-center">
              <div class="text-xs text-[#8b999b]">🎬 视频双语字幕</div>
              <div class="text-xl font-black font-mono text-[#00ffa9]">10 点 / 小时</div>
              <div class="text-[11px] text-[#5b696b]">2小时电影仅 20 点</div>
            </div>
            <div class="p-5 rounded-2xl bg-white/[0.02] border border-white/10 space-y-2 text-center">
              <div class="text-xs text-[#8b999b]">⚡ 全能格式转换</div>
              <div class="text-xl font-black font-mono text-emerald-400">永久免费</div>
              <div class="text-[11px] text-[#5b696b]">零算力流式直出</div>
            </div>
          </div>
        </div>
      </section>

      <!-- 快速上手与底部 CTA -->
      <QuickStart />
      <CTA />

    </main>

    <!-- 底部版权 -->
    <footer class="border-t border-white/[0.08] bg-[#0c1012] py-8 text-center text-xs text-[#5b696b] relative z-20">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div class="flex items-center gap-2">
          <span class="font-bold text-white">紫电 AI (Zidian AI)</span>
          <span>© 2026 工业级 SOTA 端侧离线推演架构</span>
        </div>
        <div class="flex items-center gap-4 text-[#8b999b]">
          <a href="https://geantendormi.top/" class="hover:text-white transition-colors">官方主页</a>
          <a :href="clientDownloadUrl" class="hover:text-[#00ffa9] transition-colors">客户端直链</a>
        </div>
      </div>
    </footer>

  </div>
</template>

<script setup lang="ts">
import { Download, Zap, Flame, ArrowRight } from 'lucide-vue-next';
import GradientText from './components/GradientText.vue';
import HeroBand from './components/HeroBand.vue';
import DotField from './components/DotField.vue';
import HeroCodeWindow from './components/HeroCodeWindow.vue';
import LiveDemo from './components/LiveDemo.vue';
import QuickStart from './components/QuickStart.vue';
import CTA from './components/CTA.vue';

// 官方安装包直链 (指向 R2 资产存储桶)
const clientDownloadUrl = "https://assets.geantendormi.top/downloads/zidian-ai-setup.exe";
</script>
