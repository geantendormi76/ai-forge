<template>
  <div class="min-h-screen bg-slate-50 text-slate-800 p-8 font-sans relative overflow-hidden">
    <!-- 原生隐藏文件选择框 -->
    <input
      type="file"
      ref="fileInputRef"
      multiple
      class="hidden"
      @change="handleFileInputChange"
    />

    <!-- 顶部标语与算力指示区 -->
    <div class="max-w-6xl mx-auto mb-8 border-b border-slate-200 pb-6 flex justify-between items-end relative z-10">
      <div>
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 flex items-center gap-3">
          <span class="p-2 bg-amber-50 text-[#DCA54C] rounded-xl border border-amber-100 shadow-sm">⚡</span>
          全能格式转换工坊
        </h1>
        <p class="text-slate-500 mt-2 text-sm">
          纯血 Rust 内存直推 ｜ 零显存占用 ｜ 全离线隐私安全 ｜ 涵盖音频母带、表格数据、电子书与原生图标
        </p>
      </div>

      <div class="flex items-center gap-3">
        <div class="bg-white border border-slate-200 px-3 py-1.5 rounded-lg shadow-sm text-xs flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
          <span class="text-slate-600">计算中台: <strong class="text-slate-900 font-mono">CPU 内存直推 (0 MB 显存)</strong></span>
        </div>
      </div>
    </div>

    <!-- 主工作区卡片栅格 -->
    <div class="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-12 gap-8 relative z-10">
      <!-- 左侧：拖拽上传与任务排队 (7 栏) -->
      <div class="lg:col-span-7 space-y-6">
        <!-- 拖拽上传区 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h2 class="text-base font-semibold text-slate-900 mb-4 flex items-center gap-2">
            <span class="w-1.5 h-4 bg-[#DCA54C] rounded-full"></span>
            选择待转换文件
          </h2>

          <div
            @click="triggerSelectFiles"
            @dragover.prevent
            @drop.prevent="handleFileDrop"
            class="border-2 border-dashed border-slate-200 hover:border-[#DCA54C] bg-slate-50/50 hover:bg-amber-50/20 transition-all rounded-xl p-8 text-center cursor-pointer group select-none"
          >
            <div class="w-12 h-12 bg-white border border-slate-200 text-[#DCA54C] rounded-xl flex items-center justify-center mx-auto mb-3 shadow-sm group-hover:scale-110 transition-transform text-xl">
              📂
            </div>
            <p class="text-sm font-medium text-slate-700">点击浏览选择，或将文件直接拖拽至此处</p>
            <p class="text-xs text-slate-400 mt-1">
              支持 NCM / QMC / KGMA / KWM / CSV / TSV / JSON / XML / MD / TXT / MOBI / ICO / PNG / BMP / ZIP 等
            </p>
          </div>
        </div>

        <!-- 任务队列列表 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm flex flex-col min-h-[380px]">
          <div class="flex items-center justify-between border-b border-slate-100 pb-3 mb-4">
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-semibold text-slate-900">待转换任务队列</h3>
              <span class="text-xs px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 font-mono">
                {{ taskQueue.length }} 个任务
              </span>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="taskQueue.length > 0"
                @click="clearAllTasks"
                :disabled="isConverting"
                class="text-xs text-slate-400 hover:text-rose-600 px-2 py-1 transition-colors disabled:opacity-40"
              >
                清空列表
              </button>
              <button
                @click="startBatchConvert"
                :disabled="taskQueue.length === 0 || isConverting"
                class="bg-[#DCA54C] hover:bg-amber-600 disabled:bg-slate-200 disabled:text-slate-400 text-white text-xs font-black px-4 py-2 rounded-xl transition-all shadow-sm active:scale-95 flex items-center gap-1.5 cursor-pointer"
              >
                <span v-if="isConverting" class="inline-block w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                <span>{{ isConverting ? '正在转换中...' : '🚀 开始全部转换' }}</span>
              </button>
            </div>
          </div>

          <!-- 任务列表滚动区 -->
          <div v-if="taskQueue.length > 0" class="flex-1 overflow-y-auto max-h-[420px] space-y-2.5 pr-1 custom-scrollbar">
            <div
              v-for="(item, index) in taskQueue"
              :key="index"
              class="p-3.5 bg-slate-50 border border-slate-200/80 rounded-xl flex items-center justify-between transition-all hover:border-slate-300"
            >
              <div class="flex items-center gap-3 overflow-hidden mr-3">
                <span class="text-lg shrink-0">{{ getCategoryIcon(item.ext) }}</span>
                <div class="overflow-hidden">
                  <p class="text-xs font-bold text-slate-800 truncate" :title="item.filePath">
                    {{ item.fileName }}
                  </p>
                  <p class="text-[11px] text-slate-400 font-mono mt-0.5">
                    源格式: .{{ item.ext }} ➔ 目标:
                    <strong class="text-slate-700">.{{ item.targetFormat }}</strong>
                  </p>
                </div>
              </div>

              <!-- 右侧控制项：格式选择与状态徽章 -->
              <div class="flex items-center gap-3 shrink-0">
                <select
                  v-if="item.status === 'idle'"
                  v-model="item.targetFormat"
                  class="bg-white border border-slate-200 text-xs font-semibold text-slate-700 rounded-lg px-2 py-1 outline-none focus:border-[#DCA54C]"
                >
                  <option
                    v-for="opt in getAvailableTargetFormats(item.ext)"
                    :key="opt.value"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>

                <!-- 状态徽章 -->
                <span
                  v-if="item.status === 'idle'"
                  class="text-[11px] font-bold text-slate-400 bg-slate-200/60 px-2 py-0.5 rounded-md"
                >
                  等待中
                </span>
                <span
                  v-else-if="item.status === 'converting'"
                  class="text-[11px] font-bold text-amber-600 bg-amber-50 border border-amber-200/60 px-2 py-0.5 rounded-md flex items-center gap-1"
                >
                  <span class="inline-block w-2.5 h-2.5 border-2 border-amber-600 border-t-transparent rounded-full animate-spin"></span>
                  转换中
                </span>
                <span
                  v-else-if="item.status === 'success'"
                  class="text-[11px] font-bold text-emerald-600 bg-emerald-50 border border-emerald-200 px-2 py-0.5 rounded-md flex items-center gap-1"
                >
                  ✓ 完成 ({{ item.elapsedMs }}ms)
                </span>
                <span
                  v-else-if="item.status === 'error'"
                  class="text-[11px] font-bold text-rose-600 bg-rose-50 border border-rose-200 px-2 py-0.5 rounded-md"
                  :title="item.errorMessage"
                >
                  失败
                </span>

                <!-- 删除按钮 -->
                <button
                  v-if="!isConverting"
                  @click="removeTask(index)"
                  class="text-slate-400 hover:text-rose-500 p-1 transition-colors"
                  title="移除任务"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>

          <!-- 空任务提示 -->
          <div v-else class="flex-1 flex flex-col items-center justify-center text-slate-400 py-16">
            <div class="text-4xl mb-2">⚡</div>
            <p class="text-xs">暂无待转换任务，请在上方添加文件</p>
          </div>
        </div>
      </div>

      <!-- 右侧：转换产物与功能指引 (5 栏) -->
      <div class="lg:col-span-5 space-y-6">
        <!-- 转换产物看板 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm">
          <h2 class="text-base font-semibold text-slate-900 mb-4 flex items-center gap-2">
            <span class="w-1.5 h-4 bg-emerald-500 rounded-full"></span>
            转换产物交付
          </h2>

          <div v-if="completedTasks.length > 0" class="space-y-3 max-h-[380px] overflow-y-auto pr-1 custom-scrollbar">
            <div
              v-for="(task, idx) in completedTasks"
              :key="idx"
              class="p-3 bg-slate-50 border border-slate-100 rounded-xl space-y-2"
            >
              <div class="flex justify-between items-start">
                <span class="text-xs font-bold text-slate-800 truncate block max-w-[200px]" :title="task.fileName">
                  {{ task.fileName }}
                </span>
                <span class="text-[10px] font-mono text-emerald-600 font-black bg-emerald-100/60 px-1.5 py-0.5 rounded">
                  .{{ task.resultFormat }}
                </span>
              </div>

              <div class="text-[11px] font-mono text-slate-500 bg-white p-2 rounded border border-slate-200/60 truncate select-all" :title="task.outputPath">
                {{ task.outputPath }}
              </div>

              <div class="flex justify-end gap-2 pt-1">
                <button
                  @click="copyToClipboard(task.outputPath)"
                  class="text-[11px] font-bold text-slate-600 bg-white border border-slate-200 hover:bg-slate-100 px-2.5 py-1 rounded-lg transition-all"
                >
                  复制路径
                </button>
                <button
                  v-if="task.outputPath"
                  @click="openOutputFolder(task.outputPath)"
                  class="text-[11px] font-bold text-blue-600 bg-blue-50 border border-blue-100 hover:bg-blue-100 px-2.5 py-1 rounded-lg transition-all"
                >
                  打开所在目录
                </button>
              </div>
            </div>
          </div>

          <div v-else class="text-center text-slate-400 py-12">
            <div class="text-3xl mb-2">📦</div>
            <p class="text-xs">转换成功后，产物路径将自动汇聚在此</p>
          </div>
        </div>

        <!-- 引擎能力矩阵说明 -->
        <div class="bg-white border border-slate-200 rounded-2xl p-6 shadow-sm space-y-3">
          <h3 class="text-xs font-bold text-slate-400 uppercase tracking-wider">原生支持特性矩阵</h3>
          <div class="grid grid-cols-2 gap-2 text-xs text-slate-600">
            <div class="p-2 bg-slate-50 rounded-lg border border-slate-100 flex items-center gap-1.5">
              <span>🎵</span> <strong>音频解密:</strong> NCM/QMC/KGM
            </div>
            <div class="p-2 bg-slate-50 rounded-lg border border-slate-100 flex items-center gap-1.5">
              <span>📊</span> <strong>表格清洗:</strong> CSV/JSON/TSV
            </div>
            <div class="p-2 bg-slate-50 rounded-lg border border-slate-100 flex items-center gap-1.5">
              <span>📚</span> <strong>电子书排版:</strong> EPUB/DOCX/MOBI
            </div>
            <div class="p-2 bg-slate-50 rounded-lg border border-slate-100 flex items-center gap-1.5">
              <span>🖼️</span> <strong>图标合成:</strong> ICO/PNG/BMP
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, useTemplateRef } from 'vue';
import { commands, type FormatConvertTask, type FormatConvertResult } from '../bindings';
import { revealItemInDir } from '@tauri-apps/plugin-opener';

interface ConvertQueueItem {
  filePath: string;
  fileName: string;
  ext: string;
  targetFormat: string;
  status: 'idle' | 'converting' | 'success' | 'error';
  outputPath?: string;
  resultFormat?: string;
  errorMessage?: string;
  elapsedMs?: number;
}

const fileInputRef = useTemplateRef<HTMLInputElement>('fileInputRef');
const taskQueue = ref<ConvertQueueItem[]>([]);
const isConverting = ref(false);

const FORMAT_OPTIONS_MAP: Record<string, Array<{ label: string; value: string }>> = {
  ncm: [{ label: '自动音频解密 (FLAC/MP3)', value: 'auto' }],
  qmc0: [{ label: '自动音频解密', value: 'auto' }],
  qmc3: [{ label: '自动音频解密', value: 'auto' }],
  qmcflac: [{ label: '无损 FLAC', value: 'auto' }],
  qmcogg: [{ label: 'OGG 音频', value: 'auto' }],
  mflac: [{ label: 'FLAC 音频', value: 'auto' }],
  mgg: [{ label: 'OGG 音频', value: 'auto' }],
  kgma: [{ label: '自动音频解密 (FLAC/MP3)', value: 'auto' }],
  vpr: [{ label: '自动音频解密', value: 'auto' }],
  kwm: [{ label: '自动音频解密', value: 'auto' }],
  av3a: [{ label: '提取 AV3A 裸流 (.av3a)', value: 'av3a' }],
  csv: [
    { label: 'JSON 对象数组 (.json)', value: 'json' },
    { label: 'Markdown 语法表格 (.md)', value: 'md' },
  ],
  tsv: [{ label: '标准 CSV 表格 (.csv)', value: 'csv' }],
  json: [{ label: '扁平化 CSV 表格 (.csv)', value: 'csv' }],
  xml: [{ label: 'JSON 数据树 (.json)', value: 'json' }],
  md: [
    { label: 'Word 文档 (.docx)', value: 'docx' },
    { label: 'EPUB 电子书 (.epub)', value: 'epub' },
  ],
  txt: [
    { label: 'EPUB 电子书 (.epub)', value: 'epub' },
    { label: 'Word 文档 (.docx)', value: 'docx' },
  ],
  mobi: [
    { label: '纯文本 (.txt)', value: 'txt' },
    { label: '网页源码 (.html)', value: 'html' },
  ],
  ico: [{ label: '最优 PNG 帧 (.png)', value: 'png' }],
  png: [{ label: 'Windows 图标 (.ico)', value: 'ico' }],
  jpg: [{ label: 'Windows 图标 (.ico)', value: 'ico' }],
  jpeg: [{ label: 'Windows 图标 (.ico)', value: 'ico' }],
  bmp: [{ label: '纯流式 PDF (.pdf)', value: 'pdf' }],
  zip: [{ label: '解压至同名目录', value: 'extract' }],
};

const getAvailableTargetFormats = (ext: string) => {
  return FORMAT_OPTIONS_MAP[ext.toLowerCase()] || [{ label: '默认转换', value: 'default' }];
};

const getCategoryIcon = (ext: string): string => {
  const e = ext.toLowerCase();
  if (['ncm', 'qmc0', 'qmc3', 'qmcflac', 'qmcogg', 'mflac', 'mgg', 'kgma', 'vpr', 'kwm', 'av3a'].includes(e)) return '🎵';
  if (['csv', 'tsv', 'json', 'xml'].includes(e)) return '📊';
  if (['md', 'txt', 'mobi', 'epub', 'docx'].includes(e)) return '📚';
  if (['ico', 'png', 'jpg', 'jpeg', 'bmp', 'pdf'].includes(e)) return '🖼️';
  if (['zip'].includes(e)) return '📦';
  return '📄';
};

const completedTasks = computed(() => {
  return taskQueue.value.filter(t => t.status === 'success' && t.outputPath);
});

const triggerSelectFiles = () => {
  fileInputRef.value?.click();
};

const addFilesToQueue = (files: FileList | File[]) => {
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const fullPath = (file as any).path || file.name;
    const name = file.name;
    const ext = name.split('.').pop() || '';
    const available = getAvailableTargetFormats(ext);
    const defaultTarget = available.length > 0 ? available[0].value : 'default';

    taskQueue.value.push({
      filePath: fullPath,
      fileName: name,
      ext: ext.toLowerCase(),
      targetFormat: defaultTarget,
      status: 'idle',
    });
  }
};

const handleFileInputChange = (e: Event) => {
  const target = e.target as HTMLInputElement;
  if (target.files && target.files.length > 0) {
    addFilesToQueue(target.files);
  }
};

const handleFileDrop = (e: DragEvent) => {
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    addFilesToQueue(files);
  }
};

const removeTask = (index: number) => {
  taskQueue.value.splice(index, 1);
};

const clearAllTasks = () => {
  taskQueue.value = [];
};

const startBatchConvert = async () => {
  if (isConverting.value || taskQueue.value.length === 0) return;
  isConverting.value = true;

  for (const item of taskQueue.value) {
    if (item.status === 'success') continue;

    item.status = 'converting';
    const startTime = performance.now();

    const task: FormatConvertTask = {
      input_path: item.filePath,
      target_format: item.targetFormat,
      output_dir: null,
    };

    try {
      const res: FormatConvertResult = await commands.runFormatConvert(task);
      const elapsed = Math.round(performance.now() - startTime);

      if (res.success && res.output_path) {
        item.status = 'success';
        item.outputPath = res.output_path;
        item.resultFormat = res.detected_format;
        item.elapsedMs = elapsed;
      } else {
        item.status = 'error';
        item.errorMessage = res.message || '转换失败';
      }
    } catch (err: any) {
      item.status = 'error';
      item.errorMessage = String(err);
    }
  }

  isConverting.value = false;
};

const copyToClipboard = (text?: string) => {
  if (!text) return;
  navigator.clipboard.writeText(text);
  alert('✅ 路径已复制至剪贴板:\n' + text);
};

const openOutputFolder = async (outputPath?: string) => {
  if (!outputPath) return;
  try {
    await revealItemInDir(outputPath);
  } catch (e) {
    alert('无法直接打开目录，路径已复制: ' + outputPath);
  }
};
</script>
