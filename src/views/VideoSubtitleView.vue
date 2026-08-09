<template>
  <div class="min-h-screen bg-slate-50 text-slate-800 p-8 font-sans">
    <!-- 原生隐藏文件选择器对话框 -->
    <input 
      type="file" 
      ref="fileInputRef" 
      accept="video/*,.mp4,.mkv,.mov,.avi" 
      class="hidden" 
      @change="handleFileChange" 
    />

    <!-- 头部标语区 -->
    <div class="max-w-6xl mx-auto mb-8 border-b border-slate-200 pb-6 flex justify-between items-end">
      <div>
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 flex items-center gap-3">
          <span class="p-2 bg-blue-50 text-blue-600 rounded-xl border border-blue-100">🎬</span>
          {{ t('subtitle.title') }}
        </h1>
        <p class="text-slate-500 mt-2 text-sm">
          {{ t('subtitle.subtitle') }}
        </p>
      </div>

      <!-- 语言切换与显存守卫小挂件 -->
      <div class="flex items-center gap-4">
        <div class="bg-white border border-slate-200 px-3 py-1.5 rounded-lg shadow-sm text-xs flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          <span class="text-slate-600">{{ t('pdf.vramAvailable') }}: <strong class="text-slate-900 font-mono">11,000 MB</strong></span>
        </div>
        <button 
          @click="toggleLanguage" 
          class="bg-white border border-slate-200 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg shadow-sm text-xs font-medium transition-all cursor-pointer"
        >
          🌐 {{ currentLang === 'zh-CN' ? 'English' : '简体中文' }}
        </button>
      </div>
    </div>

    <!-- 主工作区卡片 (RPA 纯白精细卡片风格) -->
    <div class="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-12 gap-8">
      
      <!-- 左侧控制面板 (5 栏) -->
      <div class="lg:col-span-5 space-y-6">
        
        <!-- 卡片 1：文件选择与拖拽区 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h2 class="text-base font-semibold text-slate-900 mb-4 flex items-center gap-2">
            <span class="w-1.5 h-4 bg-blue-600 rounded-full"></span>
            {{ t('subtitle.step1Title') }}
          </h2>

          <div 
            @click="triggerFileSelect"
            @dragover.prevent
            @drop.prevent="handleFileDrop"
            class="border-2 border-dashed border-slate-200 hover:border-blue-400 bg-slate-50/50 hover:bg-blue-50/30 transition-all rounded-xl p-8 text-center cursor-pointer group"
          >
            <div class="w-12 h-12 bg-white border border-slate-200 text-blue-600 rounded-xl flex items-center justify-center mx-auto mb-3 shadow-sm group-hover:scale-110 transition-transform text-xl">
              📹
            </div>
            <p class="text-sm font-medium text-slate-700">{{ selectedFilePath ? selectedFileName : t('subtitle.dropzoneHint') }}</p>
            <p class="text-xs text-slate-400 mt-1">{{ t('subtitle.supportedFormats') }}</p>
          </div>
        </div>

        <!-- 卡片 2：定制配置选项 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm space-y-4">
          <h2 class="text-base font-semibold text-slate-900 mb-2 flex items-center gap-2">
            <span class="w-1.5 h-4 bg-purple-600 rounded-full"></span>
            {{ t('subtitle.step2Title') }}
          </h2>

          <!-- 目标语言选择 -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-slate-600 mb-1">{{ t('subtitle.targetLang') }}</label>
              <select v-model="targetLang" class="w-full bg-slate-50 border border-slate-200 rounded-lg p-2 text-xs font-medium focus:outline-none focus:border-blue-500 cursor-pointer">
                <option value="Chinese">{{ t('subtitle.langZH') }}</option>
                <option value="English">{{ t('subtitle.langEN') }}</option>
                <option value="Japanese">{{ t('subtitle.langJA') }}</option>
                <option value="zh-Hant">{{ t('subtitle.langZHHant') }}</option>
                <option value="Korean">{{ t('subtitle.langKO') }}</option>
              </select>
            </div>

            <!-- 显示模式 -->
            <div>
              <label class="block text-xs font-semibold text-slate-600 mb-1">{{ t('subtitle.displayMode') }}</label>
              <select v-model="displayMode" class="w-full bg-slate-50 border border-slate-200 rounded-lg p-2 text-xs font-medium focus:outline-none focus:border-blue-500 cursor-pointer">
                <option value="bilingual">{{ t('subtitle.modeBilingual') }}</option>
                <option value="target_only">{{ t('subtitle.modeTargetOnly') }}</option>
                <option value="source_only">{{ t('subtitle.modeSourceOnly') }}</option>
              </select>
            </div>
          </div>

          <!-- 输出模式 & 字号倍率 -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-slate-600 mb-1">{{ t('subtitle.outputMode') }}</label>
              <select v-model="outputMode" class="w-full bg-slate-50 border border-slate-200 rounded-lg p-2 text-xs font-medium focus:outline-none focus:border-blue-500 cursor-pointer">
                <option value="soft_mkv">{{ t('subtitle.modeSoftMkv') }}</option>
                <option value="hard_mp4_nvenc">{{ t('subtitle.modeHardMp4Nvenc') }}</option>
              </select>
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-600 mb-1">{{ t('subtitle.fontSize') }}: {{ fontSizeMultiplier }}x</label>
              <input type="range" min="1.0" max="2.5" step="0.1" v-model.number="fontSizeMultiplier" class="w-full accent-blue-600 mt-2 cursor-pointer" />
            </div>
          </div>

          <!-- 开关与热词 -->
          <div class="flex items-center justify-between border-t border-slate-100 pt-3">
            <label class="text-xs font-semibold text-slate-600 flex items-center gap-2 cursor-pointer">
              <input type="checkbox" v-model="showSpeaker" class="rounded border-slate-300 text-blue-600 focus:ring-blue-500 cursor-pointer" />
              {{ t('subtitle.showSpeaker') }}
            </label>
          </div>

          <div>
            <label class="block text-xs font-semibold text-slate-600 mb-1">{{ t('subtitle.hotwords') }}</label>
            <input v-model="hotwords" type="text" :placeholder="t('subtitle.hotwordsPlaceholder')" class="w-full bg-slate-50 border border-slate-200 rounded-lg p-2 text-xs font-medium focus:outline-none focus:border-blue-500" />
          </div>

          <!-- 一键生成按钮 -->
          <button 
            @click="executeSubtitleProcess" 
            :disabled="!selectedFilePath || isProcessing"
            class="w-full mt-4 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-300 text-white font-medium py-3 px-4 rounded-xl shadow-sm transition-all flex items-center justify-center gap-2 text-sm cursor-pointer"
          >
            <span v-if="isProcessing" class="inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
            {{ isProcessing ? t('subtitle.processing') : t('subtitle.startBtn') }}
          </button>
        </div>

        <!-- 产物性能卡片 -->
        <div v-if="subtitleResult" class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-3">{{ t('subtitle.perfTitle') }}</h3>
          <div class="space-y-2 text-xs text-slate-700 mb-4">
            <div class="flex justify-between">
              <span>{{ t('subtitle.totalTime') }}:</span>
              <strong class="font-mono text-slate-900">{{ (subtitleResult.elapsed_ms / 1000).toFixed(2) }} s</strong>
            </div>
            <div class="flex justify-between">
              <span>{{ t('subtitle.totalSegments') }}:</span>
              <strong class="font-mono text-slate-900">{{ subtitleResult.total_segments }}</strong>
            </div>
            <div class="flex justify-between items-center overflow-hidden">
              <span class="shrink-0">{{ t('subtitle.outputVideo') }}:</span>
              <span class="font-mono text-[11px] text-blue-600 truncate ml-2" :title="subtitleResult.output_video_path">
                {{ subtitleResult.output_video_path }}
              </span>
            </div>
          </div>

          <div class="grid grid-cols-2 gap-2">
            <button @click="copyPath(subtitleResult.srt_path)" class="bg-slate-100 hover:bg-slate-200 text-slate-700 font-medium py-2 px-3 rounded-lg text-xs transition-all cursor-pointer">
              {{ t('subtitle.copySrt') }}
            </button>
            <button @click="copyPath(subtitleResult.ass_path)" class="bg-slate-100 hover:bg-slate-200 text-slate-700 font-medium py-2 px-3 rounded-lg text-xs transition-all cursor-pointer">
              {{ t('subtitle.copyAss') }}
            </button>
          </div>
        </div>

      </div>

      <!-- 右侧双语字幕实时卡片预览 (7 栏) -->
      <div class="lg:col-span-7">
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm min-h-[600px] flex flex-col">
          <div class="flex justify-between items-center border-b border-slate-100 pb-4 mb-4">
            <h2 class="text-base font-semibold text-slate-900 flex items-center gap-2">
              <span class="w-1.5 h-4 bg-emerald-500 rounded-full"></span>
              {{ t('subtitle.previewTitle') }}
            </h2>
            <span class="text-xs text-slate-400" v-if="subtitleResult"># {{ subtitleResult.total_segments }}</span>
          </div>

          <!-- 字幕卡片流展示区 -->
          <div v-if="subtitleResult && subtitleResult.segments.length > 0" class="space-y-3 overflow-y-auto max-h-[650px] pr-2">
            <div 
              v-for="seg in subtitleResult.segments" 
              :key="seg.id" 
              class="p-3 bg-slate-50 border border-slate-100 hover:border-blue-200 rounded-xl transition-all space-y-1"
            >
              <div class="flex justify-between text-[11px] text-slate-400 font-mono">
                <span>[{{ seg.speaker }}] {{ formatTime(seg.start_sec) }} ➔ {{ formatTime(seg.end_sec) }}</span>
                <span>#{{ seg.id }}</span>
              </div>
              <!-- 上行译文 (紫电主色风格) -->
              <p class="text-sm font-bold text-purple-900 leading-snug">
                {{ seg.target_text || t('subtitle.noTranslation') }}
              </p>
              <!-- 下行原文 (青霜副色对照) -->
              <p class="text-xs text-teal-700 leading-normal">
                {{ seg.source_text }}
              </p>
            </div>
          </div>

          <!-- 暂无字幕空状态 -->
          <div v-else class="flex-1 flex flex-col items-center justify-center text-slate-400 py-24">
            <div class="text-4xl mb-3">💬</div>
            <p class="text-sm">{{ t('subtitle.emptyTitle') }}</p>
          </div>

        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, useTemplateRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';

const { t, locale } = useI18n();

const fileInputRef = useTemplateRef<HTMLInputElement>('fileInputRef');
const currentLang = ref<'zh-CN' | 'en-US'>('zh-CN');
const selectedFilePath = ref<string>('C:\\dev\\ai-forge\\test\\fixtures\\test_video.mp4');
const selectedFileName = ref<string>('test_video.mp4');
const isProcessing = ref<boolean>(false);
const subtitleResult = ref<any>(null);

// 参数属性
const targetLang = ref<string>('Chinese');
const displayMode = ref<string>('bilingual');
const outputMode = ref<string>('soft_mkv');
const fontSizeMultiplier = ref<number>(1.8);
const showSpeaker = ref<boolean>(false);
const hotwords = ref<string>('');

const toggleLanguage = () => {
  currentLang.value = currentLang.value === 'zh-CN' ? 'en-US' : 'zh-CN';
  locale.value = currentLang.value;
};

const triggerFileSelect = () => {
  fileInputRef.value?.click();
};

const handleFileChange = (e: Event) => {
  const input = e.target as HTMLInputElement;
  if (input.files && input.files.length > 0) {
    const file = input.files[0];
    selectedFileName.value = file.name;
    selectedFilePath.value = (file as any).path || file.name;
  }
};

const handleFileDrop = (e: DragEvent) => {
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    selectedFileName.value = file.name;
    selectedFilePath.value = (file as any).path || file.name;
  }
};

const executeSubtitleProcess = async () => {
  if (!selectedFilePath.value) return;
  isProcessing.value = true;
  subtitleResult.value = null;

  const options = {
    video_path: selectedFilePath.value,
    output_dir: null,
    target_lang: targetLang.value,
    display_mode: displayMode.value,
    show_speaker: showSpeaker.value,
    font_size_multiplier: fontSizeMultiplier.value,
    output_mode: outputMode.value,
    hotwords: hotwords.value ? hotwords.value : null,
    glossary: null
  };

  try {
    const res: any = await invoke('run_video_subtitle', { options });
    subtitleResult.value = res;
  } catch (err: any) {
    alert('🚨 Subtitle Processing Failed: ' + err);
  } finally {
    isProcessing.value = false;
  }
};

const copyPath = (path: string) => {
  navigator.clipboard.writeText(path);
  alert('✅ Path copied to clipboard:\n' + path);
};

const formatTime = (seconds: number) => {
  const mins = Math.floor(seconds / 60);
  const secs = (seconds % 60).toFixed(2);
  return `${mins.toString().padStart(2, '0')}:${secs.padStart(5, '0')}`;
};
</script>
