<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import CountUp from './CountUp.vue';

type Token = { cls: 'kw' | 'comp' | 'attr' | 'punc' | 'num' | 'str'; text: string };

type Convo = {
  q: string;
  line1: Token[];
  line2: Token[];
  badgeText: string;
  highlightNum: number;
  highlightUnit: string;
  decimals?: number;
};

const AI_CONVOS: Convo[] = [
  {
    q: 'translate "system-level optimization" --to zh',
    line1: [
      { cls: 'kw', text: 'import ' },
      { cls: 'punc', text: '{ ' },
      { cls: 'comp', text: 'translate' },
      { cls: 'punc', text: ' } ' },
      { cls: 'kw', text: 'from ' },
      { cls: 'str', text: '"@ai-toolkit/core"' }
    ],
    line2: [
      { cls: 'kw', text: 'const ' },
      { cls: 'comp', text: 'res' },
      { cls: 'punc', text: ' = ' },
      { cls: 'kw', text: 'await ' },
      { cls: 'comp', text: 'translate' },
      { cls: 'punc', text: '({ ' },
      { cls: 'attr', text: 'target' },
      { cls: 'punc', text: ': ' },
      { cls: 'str', text: '"Chinese"' },
      { cls: 'punc', text: ' })' }
    ],
    badgeText: 'Hy-MT2 1.8B 热推演耗时:',
    highlightNum: 386.67,
    highlightUnit: 'ms (120+ Tokens/s)',
    decimals: 2
  },
  {
    q: 'translate cache query (SQLite WAL)',
    line1: [
      { cls: 'kw', text: 'import ' },
      { cls: 'punc', text: '{ ' },
      { cls: 'comp', text: 'cacheQuery' },
      { cls: 'punc', text: ' } ' },
      { cls: 'kw', text: 'from ' },
      { cls: 'str', text: '"@ai-toolkit/core"' }
    ],
    line2: [
      { cls: 'kw', text: 'const ' },
      { cls: 'comp', text: 'hit' },
      { cls: 'punc', text: ' = ' },
      { cls: 'kw', text: 'await ' },
      { cls: 'comp', text: 'cacheQuery' },
      { cls: 'punc', text: '({ ' },
      { cls: 'attr', text: 'hash' },
      { cls: 'punc', text: ': ' },
      { cls: 'str', text: '"0x9f2a"' },
      { cls: 'punc', text: ' })' }
    ],
    badgeText: 'SQLite WAL 缓存碰撞:',
    highlightNum: 1.39,
    highlightUnit: 'ms (提速 1080 倍)',
    decimals: 2
  },
  {
    q: 'upscale image to 8k (RealESRGAN x4)',
    line1: [
      { cls: 'kw', text: 'import ' },
      { cls: 'punc', text: '{ ' },
      { cls: 'comp', text: 'upscale8k' },
      { cls: 'punc', text: ' } ' },
      { cls: 'kw', text: 'from ' },
      { cls: 'str', text: '"@ai-toolkit/core"' }
    ],
    line2: [
      { cls: 'kw', text: 'const ' },
      { cls: 'comp', text: 'img' },
      { cls: 'punc', text: ' = ' },
      { cls: 'kw', text: 'await ' },
      { cls: 'comp', text: 'upscale8k' },
      { cls: 'punc', text: '({ ' },
      { cls: 'attr', text: 'scale' },
      { cls: 'punc', text: ': ' },
      { cls: 'num', text: '4' },
      { cls: 'punc', text: ' })' }
    ],
    badgeText: '纯 GPU 渲染吞吐:',
    highlightNum: 2.54,
    highlightUnit: 'MPixels/s (6008万像素)',
    decimals: 2
  }
];

const idx = ref(0);
const typed = ref('');
const phase = ref<'prompt' | 'thinking' | 'code'>('prompt');
const codeLines = ref(0);

let timers: ReturnType<typeof setTimeout>[] = [];

const conv = computed(() => AI_CONVOS[idx.value]);

function clearTimers() {
  timers.forEach(clearTimeout);
  timers = [];
}

function schedule(fn: () => void, ms: number) {
  const id = setTimeout(fn, ms);
  timers.push(id);
}

function runConvo(i: number) {
  const c = AI_CONVOS[i];
  typed.value = '';
  phase.value = 'prompt';
  codeLines.value = 0;

  let delay = 300;
  for (let k = 0; k <= c.q.length; k++) {
    const slice = c.q.slice(0, k);
    schedule(() => {
      typed.value = slice;
    }, delay);
    delay += 40;
  }

  schedule(() => {
    phase.value = 'thinking';
  }, delay);
  delay += 600;

  schedule(() => {
    phase.value = 'code';
    codeLines.value = 1;
  }, delay);
  delay += 250;
  schedule(() => {
    codeLines.value = 2;
  }, delay);
  delay += 2600;

  schedule(() => {
    idx.value = (idx.value + 1) % AI_CONVOS.length;
  }, delay);
}

onMounted(() => {
  runConvo(0);
});

onUnmounted(() => {
  clearTimers();
});

watch(idx, i => {
  if (i === 0) return;
  clearTimers();
  runConvo(i);
});
</script>

<template>
  <div class="ln-feat-aichat w-full h-56 p-4 rounded-2xl bg-slate-950/80 border border-slate-800/80 font-mono text-xs flex flex-col justify-between overflow-hidden shadow-inner">
    <div :key="idx" class="ln-feat-aichat-inner space-y-2.5">


      <div class="flex items-center justify-between pb-2 border-b border-slate-800/80">
        <div class="flex items-center gap-1.5">
          <span class="w-2.5 h-2.5 rounded-full bg-rose-500/80 inline-block"></span>
          <span class="w-2.5 h-2.5 rounded-full bg-amber-500/80 inline-block"></span>
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500/80 inline-block"></span>
        </div>
        <span class="text-[10px] text-slate-400 flex items-center gap-1">
          <span>Gateway UDS IPC</span>
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        </span>
      </div>


      <div class="flex items-center gap-2 text-slate-200">
        <span class="text-[#A5F3FC] font-bold">$</span>
        <span class="text-slate-100 font-semibold">{{ typed }}</span>
        <span v-if="phase === 'prompt'" class="w-2 h-4 bg-[#A5F3FC] animate-pulse inline-block"></span>
      </div>


      <div v-if="phase === 'thinking'" class="flex items-center gap-1.5 py-1.5">
        <span class="w-2 h-2 rounded-full bg-[#A5F3FC] animate-ping"></span>
        <span class="w-2 h-2 rounded-full bg-[#D8B4F8] animate-ping [animation-delay:0.2s]"></span>
        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping [animation-delay:0.4s]"></span>
        <span class="text-[11px] text-slate-400 ml-1">Rust UDS 协议解码中...</span>
      </div>


      <div v-if="phase === 'code'" class="space-y-1 pt-0.5">
        <div v-if="codeLines >= 1" class="flex items-center gap-3 text-slate-300">
          <span class="text-slate-600 select-none">1</span>
          <div class="truncate">
            <template v-for="(t, j) in conv.line1" :key="j">
              <span :data-token-idx="j" :class="t.cls === 'kw' ? 'text-[#D8B4F8] font-bold' : t.cls === 'comp' ? 'text-[#A5F3FC]' : t.cls === 'str' ? 'text-amber-200' : 'text-slate-300'">
                {{ t.text }}
              </span>
            </template>
          </div>
        </div>

        <div v-if="codeLines >= 2" class="flex items-center gap-3 text-slate-300">
          <span class="text-slate-600 select-none">2</span>
          <div class="truncate">
            <template v-for="(t, j) in conv.line2" :key="j">
              <span :data-token-idx="j" :class="t.cls === 'kw' ? 'text-[#D8B4F8] font-bold' : t.cls === 'comp' ? 'text-[#A5F3FC]' : t.cls === 'attr' ? 'text-cyan-300' : t.cls === 'num' ? 'text-emerald-400' : 'text-slate-300'">
                {{ t.text }}
              </span>
            </template>
          </div>
        </div>


        <div class="mt-2.5 pt-2 border-t border-slate-800/60 flex items-center justify-between text-[11px]">
          <span class="text-slate-400">{{ conv.badgeText }}</span>
          <span class="px-2 py-0.5 rounded bg-indigo-500/10 border border-indigo-500/30 text-[#A5F3FC] font-bold flex items-center gap-1">
            <CountUp :to="conv.highlightNum" :decimals="conv.decimals || 0" :duration="1.2" />
            <span>{{ conv.highlightUnit }}</span>
          </span>
        </div>
      </div>

    </div>
  </div>
</template>
