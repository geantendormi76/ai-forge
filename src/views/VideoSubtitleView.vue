<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Mic, Languages, HardDrive, FileCheck2 } from 'lucide-vue-next';
import { useToolWorkflow } from '../composables/useToolWorkflow';
import ToolWorkbenchLayout from '../components/layout/ToolWorkbenchLayout.vue';
import {
  runVideoSubtitle,
  type VideoSubtitleOptions,
  type VideoSubtitleResult,
  type SubtitleDisplayMode,
  type SubtitleOutputMode,
} from '../bindings/tools/video-subtitle';

const { t } = useI18n();

// 1. 业务参数选项
const targetLang = ref<string>('Chinese');
const displayMode = ref<SubtitleDisplayMode>('bilingual');
const outputMode = ref<SubtitleOutputMode>('soft_mkv');
const fontSizeMultiplier = ref<number>(1.8);
const showSpeaker = ref<boolean>(false);
const maskHardsub = ref<boolean>(false); // 电影级柔和羽化遮罩开关
const hotwords = ref<string>('');

// 2. 初始化批量多任务状态机
const workflow = useToolWorkflow<VideoSubtitleResult>();

// 3. 语言快捷选项
const targetLangOptions = [
  { label: '🇨🇳 简体中文', value: 'Chinese' },
  { label: '🇺🇸 英语', value: 'English' },
  { label: '🇯🇵 日语', value: 'Japanese' },
  { label: '🇭🇰 繁体中文', value: 'zh-Hant' },
  { label: '🇰🇷 韩语', value: 'Korean' },
];

// 4. 封装格式快捷选项
const outputModeOptions = [
  { label: 'MKV (0.5s 无损软挂载)', value: 'soft_mkv' },
  { label: 'MP4 (NVENC 显卡硬压)', value: 'hard_mp4_nvenc' },
];

// 5. 底层 4 栏特性展示
const featureCards = [
  { icon: Mic, text: 'MOSS 0.9B ASR 纯血 C-FFI 原生语音转写与多说话人分离。' },
  { icon: Languages, text: 'Hy-MT2 1.8B 腾讯混元神经翻译模型，多语言精准对齐。' },
  { icon: HardDrive, text: '0.5s 无损软挂载 MKV / NVIDIA NVENC 显卡硬压 MP4。' },
  { icon: FileCheck2, text: '支持电影级半透明羽化遮罩，柔和覆盖原片硬字幕且不遮挡焦点。' },
];

// 6. 执行批量流水线
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
      statusPrefix: '正在生成双语字幕与压制',
    }
  );
};
</script>

<template>
  <ToolWorkbenchLayout
    :kicker="'VIDEO SUBTITLE & TRANSLATION'"
    :title="t('subtitle.title')"
    :description="'支持 MP4、MKV、MOV、WebM 多视频批量排队一键生成中英双语字幕，纯血 C-FFI 显存直推，NVENC 极速压制。'"
    :local-note="'视频与语音 100% 在本机 GPU 显存中转写与神经翻译，数据零上传，断网亦可全速运行。'"
    :badge-text="'SOTA 2026 MULTI-MODAL'"
    :vram-cost-mb="8000"
    :steps="['01 添加视频', '02 目标语言', '03 批量转写', '04 打开结果']"
    :meta-rows="[
      { label: '语音识别', value: 'MOSS 0.9B ASR' },
      { label: '神经翻译', value: 'Hy-MT2 1.8B' },
      { label: '渲染遮罩', value: 'Cinematic Soft Vignette' }
    ]"
    :feature-cards="featureCards"
    :workflow="workflow"
    :accepted-formats-text="t('subtitle.supportedFormats')"
    :dropzone-title="'把需要生成双语字幕的视频放到这里'"
    :dropzone-subtitle="'支持拖拽多个视频或点击上传，上传后进入队列，再选择目标语言统一转写。'"
    :upload-button-text="'上传视频文件'"
    :queue-title="'待转写视频队列'"
    :action-button-text="'开始一键双语转写'"
    :bottom-hint-text="'确认文件和目标语言后开始处理，转写完成后会自动弹出结果提示并可直接定位目录。'"
    @execute="handleExecute"
  >
    <!-- 插槽：目标语言与渲染模式胶囊选择行 -->
    <template #options>
      <div class="space-y-4">
        <!-- 目标语言选择 -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-[11px] font-bold text-white font-mono uppercase tracking-wider">目标语言 (TARGET LANGUAGE)</label>
            <span class="text-[11px] text-[#8b8b87] font-mono">腾讯混元神经翻译多语系</span>
          </div>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="lang in targetLangOptions"
              :key="lang.value"
              type="button"
              @click="targetLang = lang.value"
              class="px-4 py-2 rounded-full border text-xs font-bold transition-all cursor-pointer"
              :class="[
                targetLang === lang.value
                  ? 'bg-white text-black border-white shadow-md scale-[1.02]'
                  : 'bg-black/40 border-white/10 text-[#8b8b87] hover:border-white/25 hover:text-white'
              ]"
            >
              {{ lang.label }}
            </button>
          </div>
        </div>

        <!-- 封装模式与电影级柔和羽化遮罩开关 -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-3 border-t border-white/[0.06] items-center">
          <div class="space-y-2">
            <label class="text-[11px] font-bold text-white font-mono uppercase tracking-wider block">输出封装 (OUTPUT CONTAINER)</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="mode in outputModeOptions"
                :key="mode.value"
                type="button"
                @click="outputMode = mode.value as SubtitleOutputMode"
                class="px-3.5 py-1.5 rounded-full border text-xs font-bold transition-all cursor-pointer"
                :class="[
                  outputMode === mode.value
                    ? 'bg-white text-black border-white shadow-sm'
                    : 'bg-black/40 border-white/10 text-[#8b8b87] hover:border-white/25 hover:text-white'
                ]"
              >
                {{ mode.label }}
              </button>
            </div>
          </div>

          <!-- 电影级柔和羽化遮罩开关 -->
          <div class="p-3 bg-black/40 border border-white/10 rounded-2xl flex items-center justify-between gap-3">
            <div class="min-w-0 space-y-0.5">
              <label class="text-xs font-bold text-white block">🛡️ 覆盖原片硬字幕</label>
              <p class="text-[10px] text-[#8b8b87]">注入半透明羽化暗角，温润遮挡旧文字</p>
            </div>
            <input
              type="checkbox"
              v-model="maskHardsub"
              class="w-4 h-4 rounded border-white/20 bg-black/50 text-white focus:ring-white cursor-pointer"
            />
          </div>
        </div>
      </div>
    </template>
  </ToolWorkbenchLayout>
</template>
