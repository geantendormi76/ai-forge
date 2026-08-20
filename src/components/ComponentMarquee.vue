<script setup lang="ts">
type Item = { name: string; route: string; highlight?: boolean };

const ROW_A: Item[] = [
  { name: 'MOSS 0.9B ASR (3.47x 实时)', route: '/asr', highlight: true },
  { name: 'RealESRGAN x4 (2.54 MPixels/s)', route: '/upscale-8k', highlight: true },
  { name: 'Hy-MT2 1.8B (386ms 热推演)', route: '/translate', highlight: true },
  { name: 'PP-DocLayoutV3 (8.53s 降维解析)', route: '/pdf-parse', highlight: true },
  { name: 'PyMuPDF C++ (233ms / 98.5% 瘦身)', route: '/pdf-edit', highlight: true },
  { name: 'SQLite WAL (1.39ms 缓存碰撞)', route: '/translate' },
];

const ROW_B: Item[] = [
  { name: 'Axum 0.7 算力网关 (9600 端口)', route: '/' },
  { name: 'VramTokenGuard (11,000 MB 硬锁)', route: '/', highlight: true },
  { name: 'RTX 3060 12G CUDA 13.0', route: '/' },
  { name: 'UDS 常驻进程池 (<1ms IPC)', route: '/' },
  { name: 'HMAC-SHA256 签名鉴权', route: '/' },
  { name: '0.01s 毫秒级字幕对齐', route: '/asr' },
];

const rowADoubled = [...ROW_A, ...ROW_A, ...ROW_A];
const rowBDoubled = [...ROW_B, ...ROW_B, ...ROW_B];
</script>

<template>
  <div class="ln-feat-marquee overflow-hidden relative w-full py-4 space-y-3 select-none">

    <div class="flex whitespace-nowrap overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_10%,black_90%,transparent)]">
      <div class="flex shrink-0 gap-3 animate-marquee">
        <router-link
          v-for="(item, i) in rowADoubled"
          :key="'a-' + i"
          :to="item.route"
          class="px-3.5 py-1.5 rounded-xl bg-slate-900/90 border text-xs font-mono transition-all cursor-pointer backdrop-blur-md"
          :class="item.highlight ? 'border-[#A5F3FC]/50 text-[#A5F3FC] font-bold bg-[#A5F3FC]/10 hover:bg-[#A5F3FC]/20' : 'border-slate-800 text-slate-300 hover:text-[#A5F3FC] hover:border-[#A5F3FC]/50'"
        >
          {{ item.name }}
        </router-link>
      </div>
    </div>


    <div class="flex whitespace-nowrap overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_10%,black_90%,transparent)]">
      <div class="flex shrink-0 gap-3 animate-marquee-rev">
        <router-link
          v-for="(item, i) in rowBDoubled"
          :key="'b-' + i"
          :to="item.route"
          class="px-3.5 py-1.5 rounded-xl bg-slate-900/90 border text-xs font-mono transition-all cursor-pointer backdrop-blur-md"
          :class="item.highlight ? 'border-[#D8B4F8]/50 text-[#D8B4F8] font-bold bg-[#D8B4F8]/10 hover:bg-[#D8B4F8]/20' : 'border-slate-800 text-slate-300 hover:text-[#D8B4F8] hover:border-[#D8B4F8]/50'"
        >
          {{ item.name }}
        </router-link>
      </div>
    </div>
  </div>
</template>

<style scoped>
@keyframes marquee {
  0% { transform: translateX(0%); }
  100% { transform: translateX(-33.333%); }
}

@keyframes marquee-rev {
  0% { transform: translateX(-33.333%); }
  100% { transform: translateX(0%); }
}

.animate-marquee {
  animation: marquee 25s linear infinite;
}

.animate-marquee-rev {
  animation: marquee-rev 25s linear infinite;
}

.animate-marquee:hover,
.animate-marquee-rev:hover {
  animation-play-state: paused;
}
</style>
