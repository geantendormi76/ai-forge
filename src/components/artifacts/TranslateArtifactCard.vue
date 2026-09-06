<script setup lang="ts">
import { ref, computed } from 'vue';
import { Copy, Check, Languages, ArrowRight } from 'lucide-vue-next';

const props = defineProps<{
  result: any;
  inputParams?: any;
}>();

const copied = ref(false);

const data = computed(() => props.result?.data || props.result || {});
const targetLang = computed(() => props.inputParams?.target_lang || data.value?.target_lang || 'English');

const translations = computed<string[]>(() => {
  if (Array.isArray(data.value?.translations)) return data.value.translations;
  if (typeof data.value?.translated_text === 'string') return [data.value.translated_text];
  if (typeof data.value === 'string') return [data.value];
  return [];
});

const originalTexts = computed<string[]>(() => {
  if (Array.isArray(props.inputParams?.texts)) return props.inputParams.texts;
  if (typeof props.inputParams?.text === 'string') return [props.inputParams.text];
  return [];
});

const fullTranslated = computed(() => translations.value.join('\n'));

const copyResult = async () => {
  if (!fullTranslated.value) return;
  await navigator.clipboard.writeText(fullTranslated.value);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
};
</script>

<template>
  <div class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs">
    <!-- 顶栏元信息与操作按钮 -->
    <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2.5">
      <div class="flex items-center gap-2">
        <Languages :size="14" class="text-[#1764e8]" />
        <span class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono">神经机器翻译对照面板</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="text-[10px] font-mono font-bold bg-[#fbeee0] dark:bg-[#3a2a1f] text-[#8b5e2c] dark:text-[#e8b07a] border border-[#f0dac5] dark:border-[#523e2e] px-2 py-0.5 rounded-full flex items-center gap-1">
          <span>译为</span>
          <ArrowRight :size="10" />
          <span>{{ targetLang }}</span>
        </span>
        <button
          type="button"
          @click="copyResult"
          class="px-2.5 py-1 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] dark:hover:bg-[#33302a] text-xs text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] font-medium flex items-center gap-1.5 transition-all active:scale-95 cursor-pointer"
        >
          <Check v-if="copied" :size="12" class="text-emerald-600 dark:text-emerald-400" />
          <Copy v-else :size="12" />
          <span>{{ copied ? '已复制' : '复制译文' }}</span>
        </button>
      </div>
    </div>

    <!-- 双语对照卡片列表 -->
    <div class="space-y-2 max-h-[260px] overflow-y-auto custom-scrollbar pr-1">
      <div
        v-for="(trans, idx) in translations"
        :key="idx"
        class="p-3 rounded-lg bg-[#f0eeea]/50 dark:bg-[#141210] border border-[#e4e1da]/60 dark:border-[#26231f] space-y-1.5 text-xs"
      >
        <div v-if="originalTexts[idx]" class="text-[#746f66] dark:text-[#8f8a81] font-sans pb-1.5 border-b border-[#e4e1da]/40 dark:border-[#26231f]">
          <span class="text-[10px] font-mono font-semibold text-[#8b5e2c] dark:text-[#e8b07a] mr-1.5">原文:</span>
          <span class="select-text leading-relaxed">{{ originalTexts[idx] }}</span>
        </div>
        <div class="text-[#1c1a17] dark:text-[#faf9f7] font-sans font-medium">
          <span class="text-[10px] font-mono font-semibold text-[#1764e8] mr-1.5">译文:</span>
          <span class="select-text leading-relaxed">{{ trans }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
