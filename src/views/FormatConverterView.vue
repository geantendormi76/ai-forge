<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useUIStore } from '../store/uiStore';
import { useToolWorkflow } from '../composables/useToolWorkflow';
import { runFormatConvert, type FormatConvertTask, type FormatConvertResult } from '../bindings/index';
import ToolWorkbenchLayout, { type GuideItem } from '../components/layout/ToolWorkbenchLayout.vue';

const ui = useUIStore();

// 1. 全域支持格式白名单
const SUPPORTED_EXTS = [
  'ncm', 'mflac', 'mgg', 'qmc0', 'qmc3', 'qmcflac', 'qmcogg', 'kgma', 'vpr', 'kgg', 'kwm', 'av3a', 'm4a', 'mp3', 'wav', 'flac', 'ogg', 'aac',
  'csv', 'tsv', 'json', 'xml',
  'md', 'txt', 'mobi', 'html',
  'ico', 'png', 'jpg', 'jpeg', 'bmp', 'heic', 'heif',
  'zip'
];

// 2. 初始化通用多任务排队工作流 (极简 6 字提示)
const workflow = useToolWorkflow<FormatConvertResult>({
  dialogTitle: '选择待转换文件 (支持多选)',
  allowedExtensions: SUPPORTED_EXTS,
  unsupportedPrompt: () => '格式暂不支持',
  dialogFilters: [
    {
      name: '支持的全部全能格式',
      extensions: SUPPORTED_EXTS
    },
    { name: '音频文件 (解密/转码)', extensions: ['ncm', 'mflac', 'mgg', 'qmc0', 'qmc3', 'qmcflac', 'qmcogg', 'kgma', 'vpr', 'kgg', 'kwm', 'av3a', 'mp3', 'wav', 'flac', 'm4a', 'ogg', 'aac'] },
    { name: '表格与结构化数据', extensions: ['csv', 'tsv', 'json', 'xml'] },
    { name: '文档与电子书', extensions: ['md', 'txt', 'mobi', 'html'] },
    { name: '图像与图标', extensions: ['ico', 'png', 'jpg', 'jpeg', 'bmp', 'heic', 'heif'] },
    { name: '归档压缩包', extensions: ['zip'] }
  ]
});

// 3. 状态配置项
const qualityPreference = ref<'lossless' | 'compatible'>('lossless');
const selectedTargetFormat = ref<string>('auto');

// 4. 5 大领域格式字典与规则映射中枢
interface TargetOption {
  format: string;
  label: string;
  badge?: string;
  desc: string;
}

interface FormatCategorySpec {
  id: string;
  name: string;
  extensions: string[];
  getTargets: (exts: string[]) => TargetOption[];
}

const CATEGORY_SPECS: FormatCategorySpec[] = [
  {
    id: 'audio',
    name: '音频母带与转码',
    extensions: ['ncm', 'mflac', 'mgg', 'qmc0', 'qmc3', 'qmcflac', 'qmcogg', 'kgma', 'vpr', 'kgg', 'kwm', 'av3a', 'm4a', 'mp3', 'wav', 'flac', 'ogg', 'aac'],
    getTargets: (exts) => {
      const isPlainAudio = exts.every(e => ['mp3', 'wav', 'flac', 'm4a', 'ogg', 'aac'].includes(e));
      if (isPlainAudio) {
        return [
          { format: 'flac', label: 'FLAC 无损', badge: '最高音质', desc: '转为无损 FLAC 音频' },
          { format: 'mp3', label: 'MP3 兼容', desc: '转为标准 MP3 格式' },
          { format: 'wav', label: 'WAV 波形', desc: '未压缩原始波形音频' }
        ];
      }
      return [
        { format: 'auto', label: '自动还原', badge: '推荐', desc: '自动嗅探无损 FLAC 或高保真 MP3' },
        { format: 'flac', label: 'FLAC 无损', desc: '100% 还原原始高保真无损音轨' },
        { format: 'mp3', label: 'MP3 兼容', desc: '适合车载与通用移动设备播放' },
        { format: 'av3a', label: 'AV3A 裸流', desc: '纯血 M4A 容器内音频裸流提取' }
      ];
    }
  },
  {
    id: 'table',
    name: '表格与结构化数据',
    extensions: ['csv', 'tsv', 'json', 'xml'],
    getTargets: (exts) => {
      const isXml = exts.every(e => e === 'xml');
      const isJson = exts.every(e => e === 'json');
      if (isXml) {
        return [{ format: 'json', label: 'JSON 结构数据', badge: '标准', desc: '保留 CDATA 属性与层级数组' }];
      }
      if (isJson) {
        return [
          { format: 'csv', label: 'CSV 扁平表格', badge: 'RFC4180', desc: '智能展平多层级键值与数组' },
          { format: 'md', label: 'Markdown 预览表格', desc: 'GFM 兼容美化表格' }
        ];
      }
      return [
        { format: 'json', label: 'JSON 对象数组', badge: '首选', desc: '表格行转换为标准 JSON 数组' },
        { format: 'md', label: 'Markdown 表格', desc: '渲染为美观的 Markdown 语法表格' },
        { format: 'csv', label: 'CSV 标准表格', desc: 'TSV 制表符归一化为逗号表格' }
      ];
    }
  },
  {
    id: 'document',
    name: '文档排版与电子书',
    extensions: ['md', 'txt', 'mobi', 'html'],
    getTargets: (exts) => {
      const isMobi = exts.some(e => e === 'mobi');
      if (isMobi) {
        return [
          { format: 'txt', label: 'TXT 纯文本', badge: '精简', desc: '提取 Kindle MOBI 正文文本' },
          { format: 'html', label: 'HTML 网页', desc: '提取完整富文本排版' },
          { format: 'docx', label: 'Word DOCX', desc: '转为带样式的 Word 文档' }
        ];
      }
      return [
        { format: 'epub', label: 'EPUB 电子书', badge: '自动分章', desc: '生成带 TOC 目录的精美电子书' },
        { format: 'docx', label: 'Word DOCX', badge: 'OpenXML', desc: '纯原生样式、标题与段落排版' }
      ];
    }
  },
  {
    id: 'image',
    name: '原生图像与全尺寸图标',
    extensions: ['ico', 'png', 'jpg', 'jpeg', 'bmp', 'heic', 'heif'],
    getTargets: (exts) => {
      const isIco = exts.every(e => e === 'ico');
      if (isIco) {
        return [
          { format: 'png', label: 'PNG 高清帧提取', badge: '最高分辨率', desc: '提取图标内最大位深 PNG' },
          { format: 'bmp', label: 'BMP 位图', desc: '解包为标准未压缩位图' }
        ];
      }
      return [
        { format: 'pdf', label: 'PDF 图像文档', badge: '极速直出', desc: '原生极速直封 PDF' },
        { format: 'ico', label: 'Windows ICO', desc: '生成 256x256 高清多分辨率图标' }
      ];
    }
  },
  {
    id: 'archive',
    name: '归档与解压',
    extensions: ['zip'],
    getTargets: () => [
      { format: 'extract', label: '解压至专属文件夹', badge: '安全防穿越', desc: '内存流式安全净化解压' }
    ]
  }
];

// 5. 智能多模态分类与动态格式推荐状态机
const currentExts = computed(() => {
  return Array.from(
    new Set(
      workflow.queue.value
        .map(f => f.name.split('.').pop()?.toLowerCase() || '')
        .filter(Boolean)
    )
  );
});

const detectedCategory = computed(() => {
  const exts = currentExts.value;
  if (exts.length === 0) return null;

  for (const spec of CATEGORY_SPECS) {
    if (exts.some(e => spec.extensions.includes(e))) {
      return spec;
    }
  }
  return null;
});

const availableTargets = computed<TargetOption[]>(() => {
  if (!detectedCategory.value) {
    return [
      { format: 'auto', label: '智能自适应转换', badge: 'AI 嗅探', desc: '根据源文件类型自动匹配最优输出格式' },
      { format: 'flac', label: 'FLAC / MP3', desc: '音频默认' },
      { format: 'pdf', label: 'PDF 图像文档', desc: '图片极速直出' },
      { format: 'json', label: 'JSON / CSV', desc: '表格数据互转' },
      { format: 'docx', label: 'DOCX / EPUB', desc: '文档电子书排版' },
      { format: 'extract', label: 'ZIP 解压', desc: '归档安全提取' }
    ];
  }
  return detectedCategory.value.getTargets(currentExts.value);
});

watch(availableTargets, (newTargets) => {
  if (newTargets.length > 0 && !newTargets.some(t => t.format === selectedTargetFormat.value)) {
    selectedTargetFormat.value = newTargets[0].format;
  }
}, { immediate: true });

// 6. 批处理执行中枢 (就地原则)
const handleExecuteConvert = async () => {
  if (workflow.queue.value.length === 0) {
    ui.弹出提示('请先添加待转换文件', 'info');
    return;
  }

  const targetFormat = selectedTargetFormat.value;

  await workflow.executeBatch(
    async (file) => {
      const ext = file.name.split('.').pop()?.toLowerCase() || '';
      
      let finalFormat = targetFormat;
      if (finalFormat === 'auto') {
        if (['ncm', 'mflac', 'mgg', 'qmc0', 'qmc3', 'qmcflac', 'qmcogg', 'kgma', 'vpr', 'kgg', 'kwm'].includes(ext)) {
          finalFormat = qualityPreference.value === 'lossless' ? 'flac' : 'mp3';
        } else if (ext === 'mp3') finalFormat = 'flac';
        else if (['jpg', 'jpeg', 'bmp', 'png'].includes(ext)) finalFormat = 'pdf';
        else if (ext === 'csv') finalFormat = 'json';
        else if (ext === 'tsv') finalFormat = 'csv';
        else if (ext === 'json') finalFormat = 'csv';
        else if (ext === 'xml') finalFormat = 'json';
        else if (ext === 'md' || ext === 'txt') finalFormat = 'epub';
        else if (ext === 'mobi') finalFormat = 'docx';
        else if (ext === 'ico') finalFormat = 'png';
        else if (ext === 'zip') finalFormat = 'extract';
        else finalFormat = 'flac';
      }

      const task: FormatConvertTask = {
        input_path: file.path,
        target_format: finalFormat,
        output_dir: null
      };

      const result = await runFormatConvert(task);
      if (!result.success) {
        throw new Error(result.message || '转换未能生成有效产物');
      }
      return result;
    },
    {
      targetFormat: selectedTargetFormat.value.toUpperCase(),
      statusPrefix: '正在极速转换'
    }
  );
};

// 🌟 7. 左翼 1-4 极简配置指南 (单行紧凑不折行)
const guideItems: GuideItem[] = [
  {
    title: '1. 支持格式',
    tag: '全能多模态',
    desc: '支持加密音频、数据表格、电子书、图片与压缩包。'
  },
  {
    title: '2. 目标格式',
    tag: '智能推荐',
    desc: '系统自动嗅探源文件，仅展示可转换的合法格式。'
  },
  {
    title: '3. 转换质量',
    tag: '无损 / 兼容',
    desc: '默认优先保留原始无损母带与高清图像画质。'
  },
  {
    title: '4. 交付位置',
    tag: '同级目录',
    desc: '产物保存在源文件同级目录，可随时一键打开。'
  }
];
</script>

<template>
  <ToolWorkbenchLayout
    title="全能格式转换工坊"
    :workflow="workflow"
    :guides="guideItems"
    dropzone-title="把需要转换的文件拖放到这里"
    dropzone-subtitle="支持多选批量排队，本地离线极速处理"
    upload-button-text="选择待转换文件"
    queue-title="待转换文件队列"
    action-button-text="开始一键极速转换"
    @execute="handleExecuteConvert"
  >
    <template #options>
      <!-- 🌟 右翼极简模块化选项矩阵 -->
      <div class="space-y-4 select-none">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          
          <!-- 目标输出格式 -->
          <div class="p-4 bg-white/[0.02] border border-white/[0.04] rounded-2xl space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-white">目标输出格式</span>
            </div>
            <div class="flex flex-wrap gap-1.5 pt-0.5">
              <button
                v-for="target in availableTargets"
                :key="target.format"
                type="button"
                @click="selectedTargetFormat = target.format"
                class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer select-none"
                :class="[
                  selectedTargetFormat === target.format
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/60 text-[#02c3b4] shadow-[0_0_12px_rgba(2,195,180,0.2)]'
                    : 'bg-white/[0.03] border border-white/[0.06] text-[#8b999b] hover:text-white hover:bg-white/[0.06]'
                ]"
              >
                {{ target.label }}
              </button>
            </div>
          </div>

          <!-- 质量与转换策略 -->
          <div class="p-4 bg-white/[0.02] border border-white/[0.04] rounded-2xl space-y-2.5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-bold text-white">质量与输出策略</span>
            </div>
            <div class="flex flex-wrap gap-1.5 pt-0.5">
              <button
                type="button"
                @click="qualityPreference = 'lossless'"
                class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer select-none"
                :class="[
                  qualityPreference === 'lossless'
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/60 text-[#02c3b4] shadow-[0_0_12px_rgba(2,195,180,0.2)]'
                    : 'bg-white/[0.03] border border-white/[0.06] text-[#8b999b] hover:text-white hover:bg-white/[0.06]'
                ]"
              >
                无损母带 / 原质
              </button>
              <button
                type="button"
                @click="qualityPreference = 'compatible'"
                class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer select-none"
                :class="[
                  qualityPreference === 'compatible'
                    ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/60 text-[#02c3b4] shadow-[0_0_12px_rgba(2,195,180,0.2)]'
                    : 'bg-white/[0.03] border border-white/[0.06] text-[#8b999b] hover:text-white hover:bg-white/[0.06]'
                ]"
              >
                通用兼容 / 精简
              </button>
            </div>
          </div>

        </div>
      </div>
    </template>
  </ToolWorkbenchLayout>
</template>
