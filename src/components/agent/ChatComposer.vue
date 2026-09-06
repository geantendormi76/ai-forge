<script setup lang="ts">
import { ref, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Image as ImageIcon,
  Paperclip,
  Cpu,
  Lightbulb,
  Zap,
  Minimize2,
  Square,
  Send,
  X
} from 'lucide-vue-next';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    isGenerating?: boolean;
    modelName?: string;
    thinkingEnabled?: boolean;
    toolPreset?: string;
    attachedFiles?: string[];
  }>(),
  {
    modelValue: '',
    isGenerating: false,
    modelName: 'Qwen3-VL-8B (VLM)',
    thinkingEnabled: true,
    toolPreset: '标准',
    attachedFiles: () => [],
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', val: string): void;
  (e: 'send', text: string): void;
  (e: 'abort'): void;
  (e: 'toggleThinking'): void;
  (e: 'toggleModel'): void;
  (e: 'compact'): void;
  (e: 'addFile', filePath: string): void;
  (e: 'removeFile', index: number): void;
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);

const canSend = computed(() => {
  return props.modelValue.trim().length > 0 || props.attachedFiles.length > 0;
});

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    if (props.isGenerating) {
      emit('abort');
    } else if (canSend.value) {
      emit('send', props.modelValue);
    }
  }
};

const handleInput = (e: Event) => {
  const target = e.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
  target.style.height = 'auto';
  target.style.height = `${Math.min(target.scrollHeight, 180)}px`;
};

// 调起 Tauri 原生 Windows 文件选择器添加视觉图像附件
const handlePickImage = async () => {
  try {
    const selected = await open({
      multiple: true,
      title: '选择待注入原生视觉的图像文件',
      filters: [
        {
          name: '图像文件',
          extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'],
        },
      ],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      for (const p of paths) {
        if (typeof p === 'string' && !props.attachedFiles.includes(p)) {
          emit('addFile', p);
        }
      }
    }
  } catch (err) {
    console.warn('调起文件选择器失败:', err);
  }
};
</script>

<template>
  <div class="w-full relative select-none font-sans">
    <!-- 附件切片预览胶囊栏 -->
    <div v-if="attachedFiles.length > 0" class="flex flex-wrap gap-2 mb-2 px-1 animate-in fade-in duration-150">
      <div
        v-for="(f, idx) in attachedFiles"
        :key="idx"
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] text-xs font-mono text-[#1c1a17] dark:text-[#faf9f7] shadow-xs"
      >
        <Paperclip :size="11" class="text-[#1764e8]" />
        <span class="truncate max-w-[240px]" :title="f">{{ f.split('\\').pop() || f }}</span>
        <button
          type="button"
          @click="emit('removeFile', idx)"
          class="text-[#746f66] hover:text-rose-500 cursor-pointer transition-colors"
          title="移除此附件"
        >
          <X :size="12" />
        </button>
      </div>
    </div>

    <!-- 悬浮式白底 Composer 输入容器 (1:1 像素级复刻 pi-desktop) -->
    <div class="w-full rounded-2xl bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] shadow-[0_8px_24px_rgba(0,0,0,0.06)] dark:shadow-[0_8px_24px_rgba(0,0,0,0.35)] p-3 flex flex-col gap-2.5 transition-all duration-200 focus-within:border-[#1764e8]/60 focus-within:shadow-[0_8px_28px_rgba(23,100,232,0.12)]">
      <!-- 文本输入域 -->
      <textarea
        ref="textareaRef"
        :value="modelValue"
        rows="2"
        placeholder="输入消息... 支持上传图片以激活原生 VLM 视觉鹰眼"
        class="w-full bg-transparent text-[#1c1a17] dark:text-[#faf9f7] placeholder-[#746f66] dark:placeholder-[#8f8a81] text-[13.5px] leading-relaxed outline-none resize-none custom-scrollbar font-sans"
        @input="handleInput"
        @keydown="handleKeyDown"
      />

      <!-- 底栏功能控制条 -->
      <div class="flex items-center justify-between gap-2 pt-1 border-t border-[#f0eeea] dark:border-[#26231f] text-xs font-mono">
        <!-- 左翼：图片上传与模型选择胶囊 -->
        <div class="flex items-center gap-1.5 flex-wrap">
          <button
            type="button"
            @click="handlePickImage"
            class="h-7 px-2 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#fcfbf9] dark:bg-[#26231f] hover:bg-[#f0eeea] text-[#57534a] dark:text-[#a19d92] flex items-center gap-1 cursor-pointer transition-colors"
            title="添加图像附件 (原生 VLM 视觉)"
          >
            <ImageIcon :size="13" class="text-[#1764e8]" />
            <span class="text-[11px]">图片</span>
          </button>

          <!-- 模型标识胶囊 -->
          <button
            type="button"
            @click="emit('toggleModel')"
            class="h-7 px-2.5 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#fcfbf9] dark:bg-[#26231f] hover:bg-[#f0eeea] text-[#1c1a17] dark:text-[#faf9f7] font-semibold flex items-center gap-1.5 cursor-pointer transition-colors"
            title="当前端侧推理模型 (点击热拔插)"
          >
            <Cpu :size="13" class="text-[#1764e8]" />
            <span>{{ modelName }}</span>
          </button>
        </div>

        <!-- 右翼：思考开关、性能、压缩与发送按钮 -->
        <div class="flex items-center gap-1.5 flex-wrap">
          <!-- 思考开关 -->
          <button
            type="button"
            @click="emit('toggleThinking')"
            class="h-7 px-2 rounded-lg border border-[#e4e1da] dark:border-[#33302a] flex items-center gap-1 text-[11px] cursor-pointer transition-colors"
            :class="thinkingEnabled ? 'bg-[#edf4ff] dark:bg-[#1e293b] border-[#cce0ff] text-[#1764e8]' : 'bg-[#fcfbf9] dark:bg-[#26231f] text-[#746f66]'"
            title="思维链深度思考开关"
          >
            <Lightbulb :size="12" />
            <span>{{ thinkingEnabled ? '思考:开' : '思考:关' }}</span>
          </button>

          <!-- 模式胶囊 -->
          <span class="h-7 px-2 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#fcfbf9] dark:bg-[#26231f] text-[#57534a] dark:text-[#a19d92] flex items-center gap-1 text-[11px]">
            <Zap :size="12" class="text-amber-500" />
            <span>{{ toolPreset }}</span>
          </span>

          <!-- 压缩上下文 -->
          <button
            type="button"
            @click="emit('compact')"
            class="h-7 px-2 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#fcfbf9] dark:bg-[#26231f] hover:bg-[#f0eeea] text-[#57534a] dark:text-[#a19d92] flex items-center gap-1 text-[11px] cursor-pointer transition-colors"
            title="压缩上下文历史"
          >
            <Minimize2 :size="12" />
            <span>压缩</span>
          </button>

          <!-- 截停 / 发送按钮 (黑色药丸) -->
          <button
            v-if="isGenerating"
            type="button"
            @click="emit('abort')"
            class="h-7 px-3.5 rounded-lg bg-rose-600 text-white font-bold text-xs flex items-center gap-1 shadow-sm hover:bg-rose-700 active:scale-95 transition-all cursor-pointer"
          >
            <Square :size="11" class="fill-white" />
            <span>截停</span>
          </button>

          <button
            v-else
            type="button"
            :disabled="!canSend"
            @click="emit('send', modelValue)"
            class="h-7 px-4 rounded-lg font-bold text-xs flex items-center gap-1 shadow-sm transition-all duration-150 cursor-pointer"
            :class="[
              canSend
                ? 'bg-[#1c1a17] dark:bg-[#faf9f7] text-white dark:text-[#1c1a17] hover:opacity-90 active:scale-95'
                : 'bg-[#f0eeea] dark:bg-[#26231f] text-[#746f66] dark:text-[#57534a] cursor-not-allowed'
            ]"
          >
            <Send :size="11" />
            <span>发送</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
