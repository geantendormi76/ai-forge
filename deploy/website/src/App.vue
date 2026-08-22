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
        </div>
        <div class="flex items-center gap-4">
          <a href="#showcase" class="text-xs font-bold text-[#8b999b] hover:text-white transition-colors hidden sm:inline-block">
            功能展台
          </a>
          <a href="#ecosystem" class="text-xs font-bold text-[#8b999b] hover:text-white transition-colors hidden sm:inline-block">
            前瞻规划
          </a>
          <a
            :href="clientDownloadUrl"
            class="h-9 px-5 rounded-full bg-gradient-to-r from-[#A5F3FC] via-[#D8B4F8] to-[#f0abfc] hover:opacity-95 text-slate-950 font-black text-xs transition-all active:scale-95 flex items-center gap-1.5 shadow-[0_0_20px_rgba(216,180,248,0.3)]"
          >
            <Download :size="14" class="stroke-[2.5]" />
            <span>下载客户端</span>
          </a>
        </div>
      </div>
    </header>

    <!-- 核心主体 -->
    <main class="flex-1">
      <!-- 英雄区：纯粹大画幅 WebGL 波浪 + 粒子 + 居中极简大标 -->
      <section class="relative w-full overflow-hidden bg-[#14110E] pt-20 pb-24 sm:pt-28 sm:pb-32">
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

        <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 relative z-20 text-center">
          <div class="flex flex-col items-center space-y-7 max-w-3xl mx-auto">
            <div>
              <GradientText
                text="紫电AI"
                :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
                :animation-speed="6"
                class="text-6xl sm:text-8xl lg:text-9xl font-black tracking-tight"
              />
            </div>

            <p class="text-slate-300 text-sm sm:text-base md:text-lg leading-relaxed max-w-2xl font-sans space-y-1">
              <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
              <span class="block text-slate-100 font-medium">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
            </p>

            <!-- 双圆角胶囊按钮组 -->
            <div class="pt-4 flex flex-col sm:flex-row items-center justify-center gap-4 w-full select-none">
              <a
                :href="clientDownloadUrl"
                class="h-12 px-8 rounded-full bg-gradient-to-r from-[#A5F3FC] via-[#D8B4F8] to-[#f0abfc] hover:opacity-95 hover:scale-105 text-slate-950 font-black text-sm tracking-wide shadow-[0_12px_36px_rgba(216,180,248,0.35)] transition-all duration-200 active:scale-95 flex items-center justify-center gap-2 group cursor-pointer"
              >
                <span>下载 Windows 客户端</span>
              </a>

              <a
                href="#showcase"
                class="h-12 px-8 rounded-full bg-white/[0.08] hover:bg-white/[0.14] hover:scale-105 border border-white/15 hover:border-white/30 text-white font-bold text-sm tracking-wide transition-all duration-200 active:scale-95 flex items-center justify-center gap-2 backdrop-blur-md cursor-pointer shadow-lg"
              >
                <span>查看更新日志</span>
              </a>
            </div>
          </div>
        </div>
      </section>

      <!-- 下半区：One tool. Endless content. 交互展示台 -->
      <section id="showcase" class="relative z-20">
        <LiveDemo />
      </section>

      <!-- 前瞻规划与生态展台 -->
      <section id="ecosystem" class="py-20 border-t border-white/[0.06] bg-black/40 relative z-20 select-none">
        <div class="max-w-5xl mx-auto px-6 space-y-10">
          <div class="text-center space-y-2">
            <div class="inline-flex items-center gap-1.5 px-3.5 py-1 rounded-full bg-[#00ffa9]/10 text-[#00ffa9] font-mono text-xs font-bold border border-[#00ffa9]/25 shadow-[0_0_12px_rgba(0,255,169,0.15)]">
              <Sparkles :size="13" class="text-[#00ffa9]" />
              <span>公测免费开放 · 前瞻生态</span>
            </div>
            <h2 class="text-2xl sm:text-3xl font-black text-white tracking-tight">更多端侧 AI 算力正在构建中</h2>
            <p class="text-xs sm:text-sm text-[#8b999b] max-w-lg mx-auto">内测期间全功能完全免费开放，专注打磨工业级离线隐私与极致性能</p>
          </div>

          <!-- 4 大前瞻规划卡片 -->
          <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
            <div class="p-6 rounded-3xl bg-white/[0.02] hover:bg-white/[0.04] border border-white/10 hover:border-[#00ffa9]/40 transition-all duration-300 space-y-3 text-center group shadow-lg">
              <div class="w-10 h-10 rounded-2xl bg-white/[0.04] border border-white/10 flex items-center justify-center mx-auto text-[#8b999b] group-hover:text-[#00ffa9] transition-colors">
                <Workflow :size="20" class="stroke-[2]" />
              </div>
              <div class="space-y-1">
                <div class="text-sm font-bold text-white">RPA 自动化</div>
                <div class="text-[10px] font-mono text-[#00ffa9] font-bold bg-[#00ffa9]/10 px-2 py-0.5 rounded-full inline-block">
                  正在构建中
                </div>
              </div>
              <p class="text-[11px] text-[#5b696b] leading-relaxed">
                桌面级跨软件工作流无感自动编排
              </p>
            </div>

            <div class="p-6 rounded-3xl bg-white/[0.02] hover:bg-white/[0.04] border border-white/10 hover:border-[#00ffa9]/40 transition-all duration-300 space-y-3 text-center group shadow-lg">
              <div class="w-10 h-10 rounded-2xl bg-white/[0.04] border border-white/10 flex items-center justify-center mx-auto text-[#8b999b] group-hover:text-[#00ffa9] transition-colors">
                <Database :size="20" class="stroke-[2]" />
              </div>
              <div class="space-y-1">
                <div class="text-sm font-bold text-white">RAG 知识库</div>
                <div class="text-[10px] font-mono text-[#00ffa9] font-bold bg-[#00ffa9]/10 px-2 py-0.5 rounded-full inline-block">
                  正在构建中
                </div>
              </div>
              <p class="text-[11px] text-[#5b696b] leading-relaxed">
                本地私有化向量数据库与多维问答
              </p>
            </div>

            <div class="p-6 rounded-3xl bg-white/[0.02] hover:bg-white/[0.04] border border-white/10 hover:border-[#00ffa9]/40 transition-all duration-300 space-y-3 text-center group shadow-lg">
              <div class="w-10 h-10 rounded-2xl bg-white/[0.04] border border-white/10 flex items-center justify-center mx-auto text-[#8b999b] group-hover:text-[#00ffa9] transition-colors">
                <Languages :size="20" class="stroke-[2]" />
              </div>
              <div class="space-y-1">
                <div class="text-sm font-bold text-white">离线高精翻译</div>
                <div class="text-[10px] font-mono text-[#00ffa9] font-bold bg-[#00ffa9]/10 px-2 py-0.5 rounded-full inline-block">
                  正在构建中
                </div>
              </div>
              <p class="text-[11px] text-[#5b696b] leading-relaxed">
                百万字长文档多语言神经翻译直出
              </p>
            </div>

            <div class="p-6 rounded-3xl bg-white/[0.02] hover:bg-white/[0.04] border border-white/10 hover:border-[#00ffa9]/40 transition-all duration-300 space-y-3 text-center group shadow-lg">
              <div class="w-10 h-10 rounded-2xl bg-white/[0.04] border border-white/10 flex items-center justify-center mx-auto text-[#8b999b] group-hover:text-[#00ffa9] transition-colors">
                <Wand2 :size="20" class="stroke-[2]" />
              </div>
              <div class="space-y-1">
                <div class="text-sm font-bold text-white">AI 无痕修改</div>
                <div class="text-[10px] font-mono text-[#00ffa9] font-bold bg-[#00ffa9]/10 px-2 py-0.5 rounded-full inline-block">
                  正在构建中
                </div>
              </div>
              <p class="text-[11px] text-[#5b696b] leading-relaxed">
                图像无痕智能抹除、瑕疵修复与重绘
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- 底部 CTA -->
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
import { Download, Zap, Sparkles, Workflow, Database, Languages, Wand2 } from 'lucide-vue-next';
import GradientText from './components/GradientText.vue';
import HeroBand from './components/HeroBand.vue';
import DotField from './components/DotField.vue';
import LiveDemo from './components/LiveDemo.vue';
import CTA from './components/CTA.vue';

// 官方安装包直链 (指向 R2 资产存储桶)
const clientDownloadUrl = "https://assets.geantendormi.top/downloads/zidian-ai-setup.exe";
</script>
