<script setup lang="ts">
import { ref } from 'vue';
import { ArrowRight, Sparkles, ImageIcon, Mic, Languages, FileText, Zap } from 'lucide-vue-next';

type ToolDemo = {
  id: string;
  step: string;
  title: string;
  inputTitle: string;
  inputDesc: string;
  outputTitle: string;
  outputDesc: string;
  icon: any;
  accentColor: string;
};

const activeIdx = ref(0);

const demos: ToolDemo[] = [
  {
    id: 'upscale-8k',
    step: '01',
    title: '8K 图像超分重构',
    inputTitle: 'Input Asset',
    inputDesc: 'Low-res photo (1080P / 老旧照片)',
    outputTitle: 'Output Result',
    outputDesc: '8K Ultra-clear (8192 × 7335 px, 2.54 MPixels/s)',
    icon: Sparkles,
    accentColor: '#A5F3FC'
  },
  {
    id: 'asr',
    step: '02',
    title: 'MOSS 0.9B 语音听写与角色对齐',
    inputTitle: 'Audio Source',
    inputDesc: 'Raw audio/video (MP4 / MP3 / WAV)',
    outputTitle: 'Transcribed Subtitle',
    outputDesc: 'Color ASS & Diarization (3.47x Realtime, [S01]/[S02])',
    icon: Mic,
    accentColor: '#D8B4F8'
  },
  {
    id: 'translate',
    step: '03',
    title: 'Hy-MT2 中英日极速翻译',
    inputTitle: 'Source Document',
    inputDesc: 'English/Japanese raw text (保持格式)',
    outputTitle: 'Translated Output',
    outputDesc: 'Fluent Chinese (386.67ms Warm, 120+ Tokens/s)',
    icon: Languages,
    accentColor: '#00ffa9'
  },
  {
    id: 'pdf-parse',
    step: '04',
    title: 'PDF 智能图文解析引擎',
    inputTitle: 'Scanned Document',
    inputDesc: 'Complex multi-column PDF (含公式与表格)',
    outputTitle: 'Structured Markdown',
    outputDesc: 'GFM Markdown & LaTeX (8.53s 5-dim Probe)',
    icon: FileText,
    accentColor: '#38BDF8'
  },
  {
    id: 'pdf-edit',
    step: '05',
    title: 'C++ PDF 页面切片抹图瘦身',
    inputTitle: 'Original PDF File',
    inputDesc: 'Large PDF with images (14.3 MB)',
    outputTitle: 'Slimmed PDF Asset',
    outputDesc: 'Text-only PDF (0.21 MB, 98.5% size reduction in 233ms)',
    icon: Zap,
    accentColor: '#34D399'
  }
];
</script>

<template>
  <section class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-16 space-y-12 select-none font-sans">


    <div class="text-center space-y-3">
      <h2 class="text-4xl sm:text-6xl font-black tracking-tight text-white">
        One tool. Endless content.
      </h2>
      <p class="text-slate-400 text-xs sm:text-sm font-medium max-w-lg mx-auto leading-relaxed">
        根据实际输入与处理需求，实时呈现本地 RTX 3060 算力节点的极速转换战果。
      </p>
    </div>


    <div class="space-y-6">

      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-white/10 pb-4">
        <div class="flex items-center gap-3">
          <span class="text-slate-500 font-mono font-bold text-base sm:text-lg">{{ demos[activeIdx].step }}</span>
          <span class="text-slate-600 font-light">—</span>
          <span class="text-white font-extrabold text-base sm:text-xl tracking-tight">{{ demos[activeIdx].title }}</span>
        </div>


        <div class="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0 custom-scrollbar">
          <button
            v-for="(d, idx) in demos"
            :key="d.id"
            @click="activeIdx = idx"
            class="px-3 py-1 rounded-full text-xs font-mono transition-all whitespace-nowrap cursor-pointer"
            :class="activeIdx === idx
              ? 'bg-white text-slate-950 font-bold'
              : 'text-slate-400 hover:text-white'"
          >
            <span>{{ d.step }}</span>
            <span class="ml-1 font-sans">{{ d.title.split(' ')[0] }}</span>
          </button>
        </div>
      </div>


      <div class="max-w-6xl mx-auto grid grid-cols-12 gap-2 sm:gap-3 items-center pt-2">


        <div class="col-span-5 space-y-3">

          <div class="relative w-full aspect-[3/4] rounded-4xl bg-slate-900/90 overflow-hidden shadow-2xl flex flex-col items-center justify-center p-6 group cursor-pointer hover:scale-[1.01] transition-transform">
            <div class="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center text-slate-400 mb-2">
              <ImageIcon class="w-6 h-6 text-slate-300" />
            </div>
            <span class="text-slate-400 text-xs font-mono font-medium">Input Asset</span>
          </div>


          <div class="pt-1">
            <div class="font-extrabold text-white text-sm sm:text-base">{{ demos[activeIdx].inputTitle }}</div>
            <div class="text-xs text-slate-400 font-sans italic mt-0.5 truncate">{{ demos[activeIdx].inputDesc }}</div>
          </div>
        </div>


        <div class="col-span-2 flex items-center justify-center">
          <ArrowRight class="w-5 h-5 text-slate-500 stroke-[1.5]" />
        </div>


        <div class="col-span-5 space-y-3">

          <div class="relative w-full aspect-[3/4] rounded-4xl bg-slate-900/90 overflow-hidden shadow-2xl flex flex-col items-center justify-center p-6 group cursor-pointer hover:scale-[1.01] transition-transform">
            <div class="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center text-slate-400 mb-2">
              <component :is="demos[activeIdx].icon" class="w-6 h-6" :style="{ color: demos[activeIdx].accentColor }" />
            </div>
            <span class="text-slate-400 text-xs font-mono font-medium">AI Result</span>
          </div>


          <div class="pt-1">
            <div class="font-extrabold text-white text-sm sm:text-base">{{ demos[activeIdx].outputTitle }}</div>
            <div class="text-xs text-slate-400 font-sans italic mt-0.5 truncate">{{ demos[activeIdx].outputDesc }}</div>
          </div>
        </div>

      </div>

    </div>

  </section>
</template>
