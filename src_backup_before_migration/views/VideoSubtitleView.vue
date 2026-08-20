<script setup lang="ts">
import { ref } from 'vue';
import { UserCheck, Layers } from 'lucide-vue-next';
import { useToolWorkflow } from '../composables/useToolWorkflow';
import ToolWorkbenchLayout, { type GuideItem } from '../components/layout/ToolWorkbenchLayout.vue';
import {
  runVideoSubtitle,
  type VideoSubtitleOptions,
  type VideoSubtitleResult,
  type SubtitleDisplayMode,
  type SubtitleOutputMode,
} from '../bindings/tools/video-subtitle';

// 1. 参数状态机
const targetLang = ref<string>('Chinese');
const displayMode = ref<SubtitleDisplayMode>('bilingual');
const outputMode = ref<SubtitleOutputMode>('soft_mkv');
const fontSizeMultiplier = ref<number>(1.35);
const showSpeaker = ref<boolean>(false);
const maskHardsub = ref<boolean>(false);
const hotwords = ref<string>('');

// 2. 多任务工作流
const workflow = useToolWorkflow<VideoSubtitleResult>();

// 3. 极简语言选项
const targetLangOptions = [
  { label: '中文', value: 'Chinese' },
  { label: '英语', value: 'English' },
  { label: '日语', value: 'Japanese' },
  { label: '韩语', value: 'Korean' },
];

// 4. 极简显示模式
const displayModeOptions = [
  { label: '双语对照', value: 'bilingual' },
  { label: '仅译文', value: 'target_only' },
  { label: '仅原文', value: 'source_only' },
];

// 5. 极简封装格式
const outputModeOptions = [
  { label: '软字幕 MKV', value: 'soft_mkv' },
  { label: '硬压制 MP4', value: 'hard_mp4_nvenc' },
];

// 6. 左侧 6 大选项配置指南
const optionGuides: GuideItem[] = [
  {
    title: '1. 目标语言',
    tag: '中 / 英 / 日 / 韩',
    desc: '指定神经机器翻译的目标语种。',
  },
  {
    title: '2. 字幕呈现',
    tag: '双语 / 仅译 / 仅原',
    desc: '双语适合外语学习；仅译文适合原片观影；仅原文适合听写。',
  },
  {
    title: '3. 说话人标注',
    tag: '[S01/S02]',
    desc: '区分多角色发言并分配独立色标（独白视频建议关闭）。',
  },
  {
    title: '4. 覆盖旧字幕',
    tag: '烟熏柔焦胶囊',
    desc: '原片自带硬字幕时开启，以动态微黑胶囊遮挡原字幕。',
  },
  {
    title: '5. 封装格式',
    tag: 'MKV / MP4',
    desc: '软字幕（MKV）秒级混流可自由开关；硬压制（MP4）永久烧录。',
  },
  {
    title: '6. 识别热词',
    tag: '专有名词',
    desc: '填入特定人名、术语（逗号分隔），提高 ASR 识别准确率。',
  },
];

// 7. 执行批量流水线
const handleExecute = async () => {
  await workflow.executeBatch(
    async (file) => {
      const options: VideoSubtitleOptions = {
        video_path: file.path,
        output_dir: null,
        target_lang: targetLang.value,
        display_mode: displayMode.value,
        show_speaker: showSpeaker.value,
        font_size_multiplier: fontSizeMultiplier.value,
        output_mode: outputMode.value,
        hotwords: hotwords.value.trim() ? hotwords.value.trim() : null,
        glossary: null,
        source_kind: null,
        subtitle_stream_index: null,
        mask_hardsub: maskHardsub.value,
      };
      return await runVideoSubtitle(options);
    },
    {
      targetFormat: outputMode.value === 'soft_mkv' ? 'MKV' : 'MP4',
      statusPrefix: '正在转写并封装',
    }
  );
};
</script>

<template>
  <ToolWorkbenchLayout
    :title="'视频字幕生成'"
    :steps="['添加视频', '定制选项', '完成交付']"
    :guides="optionGuides"
    :workflow="workflow"
    :upload-button-text="'选择视频文件'"
    :queue-title="'待转写视频队列'"
    :action-button-text="'开始一键双语转写'"
    @execute="handleExecute"
  >
    <!-- 插槽：高定海青绿 rgb(2, 195, 180) 参数区 -->
    <template #options>
      <div class="space-y-4">

        <!-- 第 1 行：译文语言 + 排版呈现 -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- 目标语言 -->
          <div class="space-y-2 p-3.5 rounded-2xl bg-white/[0.02]">
            <span class="text-xs font-bold text-white/90 block">目标语言</span>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="lang in targetLangOptions"
                :key="lang.value"
                type="button"
                @click="targetLang = lang.value"
                class="px-4 py-2 rounded-xl text-xs sm:text-sm font-bold transition-all cursor-pointer"
                :class="[
                  targetLang === lang.value
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/50 text-[#02c3b4] shadow-[0_0_18px_rgba(2,195,180,0.22)] scale-[1.02]'
                    : 'bg-white/[0.03] text-[#8b999b] hover:bg-white/[0.06] hover:text-white'
                ]"
              >
                {{ lang.label }}
              </button>
            </div>
          </div>

          <!-- 排版呈现 -->
          <div class="space-y-2 p-3.5 rounded-2xl bg-white/[0.02]">
            <span class="text-xs font-bold text-white/90 block">字幕呈现</span>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="mode in displayModeOptions"
                :key="mode.value"
                type="button"
                @click="displayMode = mode.value as SubtitleDisplayMode"
                class="px-4 py-2 rounded-xl text-xs sm:text-sm font-bold transition-all cursor-pointer"
                :class="[
                  displayMode === mode.value
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/50 text-[#02c3b4] shadow-[0_0_18px_rgba(2,195,180,0.22)] scale-[1.02]'
                    : 'bg-white/[0.03] text-[#8b999b] hover:bg-white/[0.06] hover:text-white'
                ]"
              >
                {{ mode.label }}
              </button>
            </div>
          </div>
        </div>

        <!-- 第 2 行：说话人标注 + 覆盖旧字幕开关 -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <!-- 说话人开关 -->
          <div
            @click="showSpeaker = !showSpeaker"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="showSpeaker ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="showSpeaker ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <UserCheck :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">说话人标注 [S01/S02]</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="showSpeaker ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="showSpeaker ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>

          <!-- 遮罩开关 -->
          <div
            @click="maskHardsub = !maskHardsub"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="maskHardsub ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="maskHardsub ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <Layers :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">覆盖原片旧字幕</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="maskHardsub ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="maskHardsub ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>
        </div>

        <!-- 第 3 行：封装格式 + 识别热词 -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- 封装容器 -->
          <div class="space-y-2 p-3.5 rounded-2xl bg-white/[0.02]">
            <span class="text-xs font-bold text-white/90 block">封装格式</span>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="mode in outputModeOptions"
                :key="mode.value"
                type="button"
                @click="outputMode = mode.value as SubtitleOutputMode"
                class="px-4 py-2 rounded-xl text-xs sm:text-sm font-bold transition-all cursor-pointer"
                :class="[
                  outputMode === mode.value
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/50 text-[#02c3b4] shadow-[0_0_18px_rgba(2,195,180,0.22)] scale-[1.02]'
                    : 'bg-white/[0.03] text-[#8b999b] hover:bg-white/[0.06] hover:text-white'
                ]"
              >
                {{ mode.label }}
              </button>
            </div>
          </div>

          <!-- ASR 热词 -->
          <div class="space-y-2 p-3.5 rounded-2xl bg-white/[0.02]">
            <span class="text-xs font-bold text-white/90 block">识别热词 (可选)</span>
            <input
              v-model="hotwords"
              type="text"
              placeholder="专有名词，逗号分隔 (如: DeepSeek, Rust, 町中華)"
              class="w-full h-8 px-3.5 bg-white/[0.03] border border-white/[0.06] hover:border-white/15 focus:border-[#02c3b4]/70 focus:ring-1 focus:ring-[#02c3b4]/30 rounded-xl text-xs sm:text-sm text-[#f5f5f3] outline-none transition-all placeholder-[#5b696b]"
            />
          </div>
        </div>

      </div>
    </template>
  </ToolWorkbenchLayout>
</template>
