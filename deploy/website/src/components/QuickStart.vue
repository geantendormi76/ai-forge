<script setup lang="ts">
import { computed, ref } from 'vue';
import { Check, Copy } from 'lucide-vue-next';
import GradientText from './GradientText.vue';

type Mode = 'tauri' | 'gateway';

const mode = ref<Mode>('gateway');
const copied = ref(false);

const commands = {
  gateway: 'curl http://127.0.0.1:9600/health # 验证 RTX 3060 算力网关',
  tauri: 'pnpm tauri dev # 启动 AI Toolkit 桌面端应用'
};

const currentCmd = computed(() => commands[mode.value]);

async function copyCmd() {
  await navigator.clipboard.writeText(currentCmd.value);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
}
</script>

<template>
  <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 space-y-8 py-12 relative z-10">
    <!-- Header -->
    <div class="text-center space-y-2">
      <GradientText
        text="Get started in seconds"
        :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC']"
        :animation-speed="5"
        class="text-3xl sm:text-4xl font-extrabold"
      />
      <p class="text-xs text-slate-300">本地 127.0.0.1:9600 加密通道，一键唤醒 RTX 3060 算力节点</p>
    </div>

    <!-- Terminal Box -->
    <div class="max-w-3xl mx-auto bg-slate-900/90 border border-slate-800/90 rounded-2xl shadow-2xl backdrop-blur-xl overflow-hidden">
      <!-- Tab Bar -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-slate-800/80 bg-slate-950/60">
        <div class="flex items-center gap-2">
          <button
            @click="mode = 'gateway'"
            class="px-3 py-1 rounded-lg text-xs font-mono transition-all"
            :class="mode === 'gateway' ? 'bg-[#A5F3FC]/20 text-[#A5F3FC] border border-[#A5F3FC]/40 font-bold' : 'text-slate-400 hover:text-slate-200'"
          >
            Gateway API (9600)
          </button>
          <button
            @click="mode = 'tauri'"
            class="px-3 py-1 rounded-lg text-xs font-mono transition-all"
            :class="mode === 'tauri' ? 'bg-[#D8B4F8]/20 text-[#D8B4F8] border border-[#D8B4F8]/40 font-bold' : 'text-slate-400 hover:text-slate-200'"
          >
            Tauri App
          </button>
        </div>
        <span class="text-[10px] font-mono text-slate-500">Localhost Ready</span>
      </div>

      <!-- Command Line -->
      <div class="p-5 flex items-center justify-between gap-4 font-mono text-xs">
        <div class="flex items-center gap-3 text-slate-200 overflow-x-auto">
          <span class="text-[#A5F3FC] font-bold">$</span>
          <span class="text-slate-100 font-semibold">{{ currentCmd }}</span>
        </div>
        <button
          @click="copyCmd"
          class="px-3 py-1.5 rounded-lg bg-slate-950 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-800 text-xs font-mono transition-colors flex items-center gap-1.5 shrink-0"
        >
          <Check v-if="copied" class="w-3.5 h-3.5 text-[#A5F3FC]" />
          <Copy v-else class="w-3.5 h-3.5 text-slate-400" />
          <span>{{ copied ? '已复制' : '复制命令' }}</span>
        </button>
      </div>
    </div>
  </section>
</template>
