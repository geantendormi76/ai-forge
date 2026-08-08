<template>
  <div class="min-h-screen bg-slate-50 text-slate-800 p-8 font-sans">
    <!-- 头部标语区 -->
    <div class="max-w-6xl mx-auto mb-8 border-b border-slate-200 pb-6 flex justify-between items-end">
      <div>
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 flex items-center gap-3">
          <span class="p-2 bg-blue-50 text-blue-600 rounded-xl border border-blue-100">📄</span>
          {{ t('pdf.title') }}
        </h1>
        <p class="text-slate-500 mt-2 text-sm">{{ t('pdf.subtitle') }}</p>
      </div>

      <!-- 语言切换与显存守卫小挂件 -->
      <div class="flex items-center gap-4">
        <div class="bg-white border border-slate-200 px-3 py-1.5 rounded-lg shadow-sm text-xs flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          <span class="text-slate-600">{{ t('pdf.vramAvailable') }}: <strong class="text-slate-900 font-mono">11,000 MB</strong></span>
        </div>
        <button 
          @click="toggleLanguage" 
          class="bg-white border border-slate-200 hover:bg-slate-50 text-slate-700 px-3 py-1.5 rounded-lg shadow-sm text-xs font-medium transition-all"
        >
          🌐 {{ currentLang === 'zh-CN' ? 'English' : '简体中文' }}
        </button>
      </div>
    </div>

    <!-- 主工作区卡片 (RPA 纯白精细卡片风格) -->
    <div class="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-12 gap-8">
      
      <!-- 左侧控制面板 (5 栏) -->
      <div class="lg:col-span-5 space-y-6">
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h2 class="text-base font-semibold text-slate-900 mb-4 flex items-center gap-2">
            <span class="w-1.5 h-4 bg-blue-600 rounded-full"></span>
            文件选择与排队
          </h2>

          <!-- 拖拽上传区 -->
          <div 
            @click="triggerFileSelect"
            @dragover.prevent
            @drop.prevent="handleFileDrop"
            class="border-2 border-dashed border-slate-200 hover:border-blue-400 bg-slate-50/50 hover:bg-blue-50/30 transition-all rounded-xl p-8 text-center cursor-pointer group"
          >
            <div class="w-12 h-12 bg-white border border-slate-200 text-blue-600 rounded-xl flex items-center justify-center mx-auto mb-3 shadow-sm group-hover:scale-110 transition-transform">
              📂
            </div>
            <p class="text-sm font-medium text-slate-700">{{ selectedFilePath ? selectedFileName : t('pdf.dropzone') }}</p>
            <p class="text-xs text-slate-400 mt-1">支持 .pdf / .png / .jpg 格式</p>
          </div>

          <!-- 解析控制按钮 -->
          <button 
            @click="executeParse" 
            :disabled="!selectedFilePath || isParsing"
            class="w-full mt-6 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-300 text-white font-medium py-3 px-4 rounded-xl shadow-sm transition-all flex items-center justify-center gap-2 text-sm"
          >
            <span v-if="isParsing" class="inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
            {{ isParsing ? t('pdf.parsing') : t('pdf.startParse') }}
          </button>
        </div>

        <!-- 任务路由提示面板 -->
        <div v-if="parseResult" class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-3">分流路由与性能</h3>
          <div class="text-xs font-medium text-slate-700 bg-slate-50 p-3 rounded-lg border border-slate-100 mb-3">
            {{ parseResult.route_label }}
          </div>
          <div class="flex justify-between text-xs text-slate-500">
            <span>总耗时: <strong class="text-slate-900 font-mono">{{ parseResult.elapsed_ms }} ms</strong></span>
            <span>状态: <strong class="text-emerald-600">解析成功</strong></span>
          </div>

          <!-- 打包 Zip 下载按钮 -->
          <a 
            v-if="parseResult.download_zip_url" 
            :href="parseResult.download_zip_url"
            target="_blank"
            class="mt-4 w-full bg-slate-900 hover:bg-slate-800 text-white font-medium py-2.5 px-4 rounded-lg text-xs transition-all flex items-center justify-center gap-2 block text-center shadow-sm"
          >
            📦 {{ t('pdf.downloadZip') }}
          </a>
        </div>
      </div>

      <!-- 右侧 Markdown 实时渲染预览 (7 栏) -->
      <div class="lg:col-span-7">
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm min-h-[600px] flex flex-col">
          <div class="flex justify-between items-center border-b border-slate-100 pb-4 mb-4">
            <h2 class="text-base font-semibold text-slate-900 flex items-center gap-2">
              <span class="w-1.5 h-4 bg-emerald-500 rounded-full"></span>
              {{ t('pdf.previewTitle') }}
            </h2>
            <span class="text-xs text-slate-400">GFM Markdown</span>
          </div>

          <!-- Markdown 预览主框 -->
          <div 
            v-if="renderedMarkdownHtml" 
            class="prose prose-slate max-w-none text-sm leading-relaxed overflow-y-auto max-h-[700px] p-2"
            v-html="renderedMarkdownHtml"
          ></div>
          <div v-else class="flex-1 flex flex-col items-center justify-center text-slate-400 py-24">
            <div class="text-4xl mb-3">📑</div>
            <p class="text-sm">暂无解析预览，请在左侧上传 PDF 并开始解析</p>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

const { t, locale } = useI18n();

const currentLang = ref<'zh-CN' | 'en-US'>('zh-CN');
const selectedFilePath = ref<string>('/home/zhz/ai-forge/test/fixtures/tool-pdf-parse-deep.pdf');
const selectedFileName = ref<string>('tool-pdf-parse-deep.pdf');
const isParsing = ref<boolean>(false);
const parseResult = ref<any>(null);

const toggleLanguage = () => {
  currentLang.value = currentLang.value === 'zh-CN' ? 'en-US' : 'zh-CN';
  locale.value = currentLang.value;
};

const triggerFileSelect = () => {
  // 提示：默认演示预填测试集文件路径
};

const handleFileDrop = (e: DragEvent) => {
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    selectedFileName.value = file.name;
    selectedFilePath.value = (file as any).path || file.name;
  }
};

const executeParse = async () => {
  if (!selectedFilePath.value) return;
  isParsing.value = true;
  parseResult.value = null;

  try {
    const res: any = await invoke('parse_pdf', { filePath: selectedFilePath.value });
    parseResult.value = res;
  } catch (err: any) {
    alert('🚨 解析失败: ' + err);
  } finally {
    isParsing.value = false;
  }
};

const renderedMarkdownHtml = computed(() => {
  if (!parseResult.value || !parseResult.value.markdown) return '';
  return marked.parse(parseResult.value.markdown);
});
</script>
