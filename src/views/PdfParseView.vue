<script setup lang="ts">
import { ref } from 'vue';
import { Sigma, Table, Image as ImageIcon, Sparkles } from 'lucide-vue-next';
import { useToolWorkflow } from '../composables/useToolWorkflow';
import ToolWorkbenchLayout, { type GuideItem } from '../components/layout/ToolWorkbenchLayout.vue';
import { parsePdf, type PdfParseResult } from '../bindings/tools/pdf-parse';

// 1. 核心多模态能力开关状态机
const enableFormula = ref<boolean>(true);
const enableTable = ref<boolean>(true);
const enableFigure = ref<boolean>(true);
const enableRxycSort = ref<boolean>(true);

// 2. 多任务工作流实例 (专属绑定 PDF 文件过滤器)
const workflow = useToolWorkflow<PdfParseResult>({
  dialogTitle: '选择待解析 PDF 文档',
  dialogFilters: [
    { name: 'PDF 文档 (*.pdf)', extensions: ['pdf'] },
    { name: '全部文件 (*.*)', extensions: ['*'] },
  ],
});

// 3. 左侧 6 大商业级技术配置指南
const optionGuides: GuideItem[] = [
  {
    title: '1. 双轨自适应分流引擎',
    tag: '智能全自动调度',
    desc: '毫秒级自动判别原生数字矢量流与扫描位图，动态分配最优算力。',
  },
  {
    title: '2. 神经数学公式识别',
    tag: 'LaTeX 语法',
    desc: '端侧深度转写行内与行间复杂公式，输出标准 KaTeX 格式。',
  },
  {
    title: '3. 多维表格结构语义还原',
    tag: 'HTML 语义重建',
    desc: '高保真解析复杂嵌套表格与跨页合并单元格，输出标准 HTML。',
  },
  {
    title: '4. 高清视觉资产无损提取',
    tag: '无损独立切片',
    desc: '独立提取原片插图与矢量图表，以相对路径高保真嵌入文档。',
  },
  {
    title: '5. 拓扑排版流智能重构',
    tag: '自然阅读序',
    desc: '双栏、多栏与学术复杂排版智能防跳读，恢复自然阅读顺序。',
  },
  {
    title: '6. 本地物理原生物理交付',
    tag: 'Markdown + images',
    desc: '纯本地运算与物理文件直出，0 数据上传，彻底守护隐私安全。',
  },
];

// 4. 执行批量解析流水线
const handleExecute = async () => {
  await workflow.executeBatch(
    async (file) => {
      return await parsePdf(file.path);
    },
    {
      targetFormat: 'Markdown',
      statusPrefix: '正在智能多模态解析',
    }
  );
};
</script>

<template>
  <ToolWorkbenchLayout
    :title="'PDF 智能解析'"
    :steps="['添加文档', '定制选项', '完成交付']"
    :guides="optionGuides"
    :workflow="workflow"
    :upload-button-text="'选择 PDF 文件'"
    :queue-title="'待解析 PDF 队列'"
    :action-button-text="'开始智能多模态解析'"
    :dropzone-title="'把需要解析的 PDF 文档拖放到这里'"
    :dropzone-subtitle="'支持学术论文、商业财报、书籍与扫描件，可多选批量排队'"
    @execute="handleExecute"
  >
    <template #options>
      <div class="space-y-3">
        <div class="flex items-center justify-between pb-1">
          <span class="text-xs font-bold text-white/90">多模态解析能力定制</span>
          <span class="text-[11px] font-mono text-[#02c3b4]">端侧全自动自适应分流</span>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div
            @click="enableFormula = !enableFormula"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="enableFormula ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="enableFormula ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <Sigma :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">数学公式转写 (LaTeX)</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="enableFormula ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="enableFormula ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>

          <div
            @click="enableTable = !enableTable"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="enableTable ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="enableTable ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <Table :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">表格结构解析 (HTML)</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="enableTable ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="enableTable ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>

          <div
            @click="enableFigure = !enableFigure"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="enableFigure ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="enableFigure ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <ImageIcon :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">插图独立切片</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="enableFigure ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="enableFigure ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>

          <div
            @click="enableRxycSort = !enableRxycSort"
            class="p-3.5 bg-white/[0.02] rounded-2xl flex items-center justify-between gap-3 cursor-pointer transition-all duration-150 select-none hover:bg-white/[0.04]"
            :class="enableRxycSort ? 'border border-[#02c3b4]/50 shadow-[0_0_16px_rgba(2,195,180,0.12)]' : 'border border-transparent'"
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-7 h-7 rounded-xl flex items-center justify-center shrink-0 transition-colors"
                :class="enableRxycSort ? 'bg-[#02c3b4]/20 text-[#02c3b4]' : 'bg-white/[0.04] text-[#8b999b]'"
              >
                <Sparkles :size="15" />
              </div>
              <span class="text-xs sm:text-sm font-bold text-white">学术双栏防跳读</span>
            </div>
            <div
              class="w-9 h-5 rounded-full p-0.5 transition-colors shrink-0"
              :class="enableRxycSort ? 'bg-[#02c3b4] shadow-[0_0_10px_rgba(2,195,180,0.5)]' : 'bg-white/15'"
            >
              <div
                class="w-4 h-4 rounded-full bg-black shadow-sm transform transition-transform"
                :class="enableRxycSort ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>
        </div>

      </div>
    </template>
  </ToolWorkbenchLayout>
</template>
