<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core';
import { revealItemInDir, openPath } from '@tauri-apps/plugin-opener';
import { FolderOpen, Eye, Sparkles } from 'lucide-vue-next';

const props = defineProps<{
  result: any;
}>();

const data = props.result?.data || props.result || {};
const outputPath = data.output_path || data.outputPath || '';
const originalSize = data.original_size || data.originalSize || [0, 0];
const outputSize = data.output_size || data.outputSize || [0, 0];
const actualScale = data.actual_scale || data.actualScale || 4;

const openInExplorer = async () => {
  if (!outputPath) return;
  try {
    await revealItemInDir(outputPath);
  } catch {
    await openPath(outputPath);
  }
};
</script>

<template>
  <div class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs">
    <!-- 顶栏元信息与操作按钮 -->
    <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2.5">
      <div class="flex items-center gap-2">
        <Sparkles :size="14" class="text-[#1764e8]" />
        <span class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono">视觉超分辨率重构产物</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="text-[10px] font-mono font-bold bg-[#edf4ff] dark:bg-[#1e293b] text-[#1764e8] dark:text-[#60a5fa] border border-[#cce0ff] dark:border-[#3b82f6]/40 px-2 py-0.5 rounded-full">
          {{ actualScale }}x 放大
        </span>
        <button
          type="button"
          @click="openInExplorer"
          class="px-2.5 py-1 rounded-lg bg-[#edf4ff] dark:bg-[#1e293b] hover:bg-[#cce0ff] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-xs text-[#1764e8] font-bold flex items-center gap-1.5 transition-all active:scale-95 cursor-pointer"
        >
          <FolderOpen :size="12" />
          <span>打开文件夹</span>
        </button>
      </div>
    </div>

    <!-- 分辨率药丸对比栏 -->
    <div class="flex items-center justify-between text-[11px] font-mono bg-[#f0eeea]/60 dark:bg-[#141210] px-3 py-2 rounded-lg border border-[#e4e1da] dark:border-[#26231f] text-[#57534a] dark:text-[#a19d92]">
      <span>原始: <strong class="text-[#1c1a17] dark:text-[#faf9f7]">{{ originalSize[0] }}×{{ originalSize[1] }}</strong></span>
      <span class="text-[#1764e8] font-bold">➔</span>
      <span>超分后: <strong class="text-emerald-600 dark:text-emerald-400 font-bold">{{ outputSize[0] }}×{{ outputSize[1] }}</strong></span>
    </div>

    <!-- 图像高清缩略图预览区 -->
    <div
      v-if="outputPath"
      class="rounded-lg overflow-hidden bg-[#f0eeea]/40 dark:bg-[#141210] border border-[#e4e1da] dark:border-[#26231f] flex items-center justify-center p-1.5 relative group cursor-pointer"
      @click="openInExplorer"
      title="点击在资源管理器中查看"
    >
      <img
        :src="convertFileSrc(outputPath)"
        class="max-h-[260px] object-contain rounded select-none group-hover:scale-[1.01] transition-transform duration-200"
        alt="超分产物"
      />
      <div class="absolute inset-0 bg-[#1c1a17]/40 dark:bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-1.5 text-white text-xs font-mono font-bold pointer-events-none">
        <Eye :size="14" />
        <span>在资源管理器中定位</span>
      </div>
    </div>
  </div>
</template>
