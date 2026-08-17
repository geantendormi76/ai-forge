<script setup lang="ts">
import { Film, FileCode2, X } from 'lucide-vue-next';
import type { ToolFileMetadata } from '../../composables/useToolWorkflow';

defineProps<{
  file: ToolFileMetadata;
  iconType?: 'video' | 'file';
}>();

const emit = defineEmits<{
  (e: 'clear'): void;
}>();
</script>

<template>
  <div class="w-full bg-[#0e0e10]/90 border border-white/10 rounded-2xl p-4 sm:p-5 flex items-center justify-between shadow-xl backdrop-blur-xl transition-all select-none group">
    <div class="flex items-center gap-3.5 min-w-0 pr-4">
      <div class="w-10 h-10 rounded-xl bg-white/[0.04] border border-white/10 text-white/80 flex items-center justify-center shrink-0">
        <Film v-if="iconType === 'video' || file.name.match(/\.(mp4|mkv|mov|avi|webm)$/i)" :size="18" class="stroke-[2]" />
        <FileCode2 v-else :size="18" class="stroke-[2]" />
      </div>

      <div class="min-w-0 space-y-0.5">
        <p class="text-xs sm:text-sm font-bold text-white truncate" :title="file.path || file.name">
          {{ file.name }}
        </p>
        <p class="text-[11px] font-mono text-[#8b8b87] truncate flex items-center gap-2">
          <span v-if="file.durationFormatted">{{ file.durationFormatted }}</span>
          <span v-if="file.durationFormatted && file.sizeFormatted">—</span>
          <span v-if="file.sizeFormatted">{{ file.sizeFormatted }}</span>
        </p>
      </div>
    </div>

    <button
      type="button"
      @click.stop="emit('clear')"
      class="w-8 h-8 rounded-lg text-[#5b5b58] hover:text-white hover:bg-white/10 flex items-center justify-center transition-colors cursor-pointer shrink-0"
      title="清除并重新选择"
    >
      <X :size="16" class="stroke-[2]" />
    </button>
  </div>
</template>
