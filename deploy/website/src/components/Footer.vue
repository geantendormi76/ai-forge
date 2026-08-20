<template>
  <footer class="relative z-10 border-t border-slate-900 bg-slate-950 py-12 text-xs text-slate-400">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 space-y-8">
      
      <div class="grid grid-cols-1 md:grid-cols-12 gap-8 items-start justify-between">
        
        <!-- 品牌栏 -->
        <div class="md:col-span-6 space-y-3">
          <div class="flex items-center gap-2 font-extrabold text-base text-slate-100">
            <Sparkles class="w-5 h-5 text-[#A5F3FC]" />
            <span>AI Toolkit</span>
            <span class="px-2 py-0.5 rounded bg-[#A5F3FC]/10 border border-[#A5F3FC]/30 text-[#A5F3FC] font-mono text-[10px]">SOTA 2026</span>
          </div>
          <p class="text-slate-400 max-w-sm leading-relaxed">
            基于钱学森工程思维哲学的工业级 AI 算力工具箱。极速全栈协同优化，让算力触手可及。
          </p>
        </div>

        <!-- 硬件与链接栏 -->
        <div class="md:col-span-6 grid grid-cols-2 gap-6 font-mono">
          <div class="space-y-2">
            <span class="text-slate-200 font-bold">硬件与环境</span>
            <ul class="space-y-1 text-slate-400">
              <li>• NVIDIA RTX 3060 12GB</li>
              <li>• Intel i5-13600FK / 32G</li>
              <li>• CUDA 13.0 / WSL2 Ubuntu</li>
            </ul>
          </div>

          <div class="space-y-2">
            <span class="text-slate-200 font-bold">公网与通道</span>
            <ul class="space-y-1 text-slate-400">
              <li>• https://geantendormi.top</li>
              <li>• 网关: 127.0.0.1:9600</li>
              <li>• Specta 强类型契约</li>
            </ul>
          </div>
        </div>

      </div>

      <!-- Copyright -->
      <div class="pt-6 border-t border-slate-900 flex flex-col sm:flex-row items-center justify-between gap-4 text-slate-500 font-mono">
        <p>© 2026 AI Toolkit | geantendormi.top All rights reserved.</p>
        <p>Made with ❤️ by AI Toolkit Team</p>
      </div>

    </div>
  </footer>
</template>

<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next';
</script>
