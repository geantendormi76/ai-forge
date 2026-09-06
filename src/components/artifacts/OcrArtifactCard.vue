<script setup lang="ts">
import { ref, computed } from 'vue';
import { Copy, Check, FileText, Download } from 'lucide-vue-next';

const props = defineProps<{
  result: any;
}>();

const copied = ref(false);

const lines = computed<Array<{ text: string; score?: number }>>(() => {
  const res = props.result?.data || props.result;
  if (!res) return [];
  if (Array.isArray(res)) {
    return res.map((r: any) => ({ text: r.text || String(r), score: r.score }));
  }
  if (Array.isArray(res.regions)) {
    return res.regions.map((r: any) => ({ text: r.text || '', score: r.score }));
  }
  if (typeof res.full_text === 'string') {
    return res.full_text.split('\n').filter(Boolean).map((t: string) => ({ text: t }));
  }
  return [];
});

const fullText = computed(() => lines.value.map(l => l.text).join('\n'));

const copyFullText = async () => {
  if (!fullText.value) return;
  await navigator.clipboard.writeText(fullText.value);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
};

const exportTxtFile = () => {
  if (!fullText.value) return;
  const blob = new Blob([fullText.value], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `ocr_result_${Date.now()}.txt`;
  a.click();
  URL.revokeObjectURL(url);
};
</script>

<template>
  <div class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs">
    <!-- 顶栏元信息与操作按钮 -->
    <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2.5">
      <div class="flex items-center gap-2">
        <FileText :size="14" class="text-[#1764e8]" />
        <span class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono">OCR 提取结果 (共 {{ lines.length }} 行)</span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          type="button"
          @click="copyFullText"
          class="px-2.5 py-1 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] dark:hover:bg-[#33302a] text-xs text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] font-medium flex items-center gap-1.5 transition-all active:scale-95 cursor-pointer"
        >
          <Check v-if="copied" :size="12" class="text-emerald-600 dark:text-emerald-400" />
          <Copy v-else :size="12" />
          <span>{{ copied ? '已复制' : '复制全文' }}</span>
        </button>
        <button
          type="button"
          @click="exportTxtFile"
          class="px-2.5 py-1 rounded-lg bg-[#edf4ff] dark:bg-[#1e293b] hover:bg-[#cce0ff] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-xs text-[#1764e8] font-bold flex items-center gap-1.5 transition-all active:scale-95 cursor-pointer"
        >
          <Download :size="12" />
          <span>导出 .txt</span>
        </button>
      </div>
    </div>

    <!-- 结构化文字行流 -->
    <div class="space-y-1.5 max-h-[260px] overflow-y-auto custom-scrollbar pr-1">
      <div
        v-for="(line, idx) in lines"
        :key="idx"
        class="px-3 py-2 rounded-lg bg-[#f0eeea]/50 dark:bg-[#141210] hover:bg-[#f0eeea] dark:hover:bg-[#26231f] border border-[#e4e1da]/60 dark:border-[#26231f] flex items-center justify-between gap-3 text-xs transition-colors"
      >
        <div class="flex items-center gap-2.5 min-w-0">
          <span class="text-[10px] font-mono text-[#746f66] dark:text-[#8f8a81] w-5 shrink-0">{{ idx + 1 }}.</span>
          <span class="text-[#1c1a17] dark:text-[#faf9f7] font-sans truncate select-text">{{ line.text }}</span>
        </div>
        <span
          v-if="line.score !== undefined"
          class="text-[9.5px] font-mono px-1.5 py-0.5 rounded font-semibold shrink-0"
          :class="line.score > 0.8 ? 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/40' : 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-400 border border-amber-200 dark:border-amber-800/40'"
        >
          {{ (line.score * 100).toFixed(0) }}%
        </span>
      </div>
    </div>
  </div>
</template>
