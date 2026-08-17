<script setup lang="ts">
import { UploadCloud } from 'lucide-vue-next';

defineProps<{
  isDragging?: boolean;
  title?: string;
  subtitle?: string;
  acceptedFormatsText?: string;
  buttonText?: string;
}>();

const emit = defineEmits<{
  (e: 'click'): void;
}>();
</script>

<template>
  <div
    @click="emit('click')"
    class="w-full border border-dashed rounded-3xl p-10 sm:p-14 text-center cursor-pointer transition-all duration-200 flex flex-col items-center justify-center relative overflow-hidden group select-none"
    :class="[
      isDragging
        ? 'border-white/60 bg-white/[0.06] scale-[0.99]'
        : 'border-white/15 hover:border-white/30 bg-black/40 hover:bg-white/[0.02]'
    ]"
  >
    <div class="w-14 h-14 rounded-2xl bg-white/[0.04] border border-white/10 text-[#f2f2ef] group-hover:text-white group-hover:border-white/25 flex items-center justify-center mb-5 shadow-lg group-hover:scale-105 transition-all">
      <UploadCloud :size="24" class="stroke-[1.75]" />
    </div>

    <h3 class="text-base sm:text-lg font-bold text-white tracking-tight">
      {{ title || '拖拽视频到此处' }}
    </h3>
    <p class="text-xs sm:text-sm text-[#8b8b87] mt-1">
      {{ subtitle || '或点击浏览设备中的文件' }}
    </p>

    <button
      type="button"
      class="mt-6 px-7 py-2.5 rounded-full bg-white hover:bg-[#f2f2ef] text-black font-bold text-xs sm:text-sm shadow-md transition-all active:scale-95 cursor-pointer pointer-events-none"
    >
      {{ buttonText || '选择视频文件' }}
    </button>

    <p class="text-[11px] font-mono text-[#5b5b58] mt-4">
      {{ acceptedFormatsText || 'MP4、WebM、MOV、AVI — 最大 10 GB' }}
    </p>
  </div>
</template>
