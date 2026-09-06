<script setup lang="ts">
import { ref } from 'vue';
import type { ToolTrace } from '../../composables/useAgentSession';
import { revealItemInDir, openPath } from '@tauri-apps/plugin-opener';
import OcrArtifactCard from '../artifacts/OcrArtifactCard.vue';
import ImageArtifactCard from '../artifacts/ImageArtifactCard.vue';
import TranslateArtifactCard from '../artifacts/TranslateArtifactCard.vue';
import {
  Layers,
  PanelRightClose,
  Maximize2,
  Minimize2,
  Copy,
  Check,
  Wrench,
  Loader2,
  AlertCircle,
  FileCode2,
  LayoutTemplate,
  FolderOpen,
  Terminal,
  Table,
  FileText
} from 'lucide-vue-next';

withDefaults(
  defineProps<{
    currentArtifact: ToolTrace | null;
    allArtifacts: Array<{ msgId: string; tool: ToolTrace }>;
    isMaximized?: boolean;
  }>(),
  {
    currentArtifact: null,
    allArtifacts: () => [],
    isMaximized: false,
  }
);

const emit = defineEmits<{
  (e: 'selectArtifact', callId: string): void;
  (e: 'toggleMaximize'): void;
  (e: 'close'): void;
}>();

const copiedRaw = ref(false);

const getToolDisplayLabel = (name: string): string => {
  const map: Record<string, string> = {
    native_upscale_image: '4K/8K 视觉超分重构',
    ocr_recognize_image: 'PP-OCRv6 视觉文字提取',
    native_neural_translate: 'Hy-MT2 神经机器翻译',
    pdf_get_meta: 'PDF 物理元数据探测',
    pdf_render_page_image: 'PDF 页面高清渲染',
    pdf_extract_raw_text: 'PDF 原生纯文本提取',
    formula_recognize_latex: 'PP-Formula 数学公式转写',
    table_recognize_structure: 'SLANet 复杂表格重构',
    native_format_convert: '全能格式解密转换',
    native_doc_parse: '多模态文档深度重构',
    execute_python_sandbox: 'Python 动态数据沙箱',
  };
  return map[name] || name;
};

const isOcrTool = (name: string) => name.includes('ocr');
const isImageTool = (name: string, result: any) => {
  if (name.includes('upscale') || name.includes('vision')) return true;
  const p = result?.output_path || result?.data?.output_path;
  return typeof p === 'string' && /\.(png|jpg|jpeg|webp|bmp|ico)$/i.test(p);
};
const isTranslateTool = (name: string) => name.includes('translate');
const isSandboxTool = (name: string) => name.includes('sandbox') || name.includes('python');
const isTableTool = (name: string) => name.includes('table');
const isDocTool = (name: string) => name.includes('doc_parse') || name.includes('pdf');

const copyRawResult = async (res: any) => {
  if (!res) return;
  await navigator.clipboard.writeText(JSON.stringify(res, null, 2));
  copiedRaw.value = true;
  setTimeout(() => { copiedRaw.value = false; }, 2000);
};

const openOutputPath = async (pathStr?: string) => {
  if (!pathStr) return;
  try {
    await revealItemInDir(pathStr);
  } catch {
    await openPath(pathStr);
  }
};
</script>

<template>
  <div class="w-full h-full flex flex-col bg-[#fcfbf9] dark:bg-[#1c1a17] border-l border-[#e4e1da] dark:border-[#33302a] overflow-hidden select-none font-sans">
    <!-- ==================== 1. 画布顶栏控制器 ==================== -->
    <header class="h-11 border-b border-[#e4e1da] dark:border-[#33302a] px-4 flex items-center justify-between bg-[#fcfbf9]/95 dark:bg-[#1c1a17]/95 backdrop-blur-md shrink-0 z-10">
      <!-- 左侧：产物标识 -->
      <div class="flex items-center gap-2 min-w-0">
        <div class="w-5 h-5 rounded-md bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 flex items-center justify-center text-[#1764e8] shrink-0">
          <Layers :size="12" />
        </div>
        <span class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono tracking-tight">产物看板 / ARTIFACTS</span>
        <span
          v-if="currentArtifact"
          class="text-[10px] font-mono font-semibold bg-[#edf4ff] dark:bg-[#1e293b] text-[#1764e8] dark:text-[#60a5fa] border border-[#cce0ff] dark:border-[#3b82f6]/40 px-2 py-0.5 rounded-full truncate"
        >
          {{ getToolDisplayLabel(currentArtifact.toolName) }}
        </span>
      </div>

      <!-- 右侧：多产物 Tab 切换与窗口控制 -->
      <div class="flex items-center gap-1.5 shrink-0">
        <!-- 多产物 Tab 切换胶囊 -->
        <div v-if="allArtifacts.length > 1" class="flex items-center gap-1 bg-[#f0eeea] dark:bg-[#26231f] p-0.5 rounded-lg border border-[#e4e1da] dark:border-[#33302a]">
          <button
            v-for="(art, aIdx) in allArtifacts"
            :key="art.tool.callId"
            type="button"
            @click="emit('selectArtifact', art.tool.callId)"
            class="px-2 py-0.5 rounded-md text-[10px] font-mono font-bold transition-all cursor-pointer select-none"
            :class="[
              currentArtifact && currentArtifact.callId === art.tool.callId
                ? 'bg-white dark:bg-[#1c1a17] text-[#1c1a17] dark:text-[#faf9f7] shadow-xs'
                : 'text-[#746f66] dark:text-[#8f8a81] hover:text-[#1c1a17] dark:hover:text-[#faf9f7]'
            ]"
          >
            #{{ aIdx + 1 }} {{ art.tool.toolName.replace('native_', '').replace('_image', '').replace('recognize_', '') }}
          </button>
        </div>

        <!-- 全屏放大 / 还原切换 -->
        <button
          type="button"
          @click="emit('toggleMaximize')"
          class="w-7 h-7 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea]/60 dark:bg-[#26231f]/60 hover:bg-[#f0eeea] dark:hover:bg-[#26231f] text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] flex items-center justify-center transition-all cursor-pointer"
          :title="isMaximized ? '还原分栏看板' : '全屏沉浸看板'"
        >
          <Minimize2 v-if="isMaximized" :size="12" />
          <Maximize2 v-else :size="12" />
        </button>

        <!-- 关闭画布抽屉 -->
        <button
          type="button"
          @click="emit('close')"
          class="w-7 h-7 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea]/60 dark:bg-[#26231f]/60 hover:bg-[#f0eeea] dark:hover:bg-[#26231f] text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] flex items-center justify-center transition-all cursor-pointer"
          title="收起产物看板"
        >
          <PanelRightClose :size="13" />
        </button>
      </div>
    </header>

    <!-- ==================== 2. 画布内容全景渲染区 ==================== -->
    <main class="flex-1 overflow-y-auto custom-scrollbar p-5 space-y-4 bg-[#f7f6f3] dark:bg-[#141210]">
      <!-- 场景 A: 存在选中的产物 -->
      <template v-if="currentArtifact">
        <!-- 产物元数据与操作条 -->
        <div class="p-3 rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] flex items-center justify-between text-xs font-mono shadow-xs">
          <div class="flex items-center gap-2 min-w-0">
            <Wrench :size="13" class="text-[#1764e8] shrink-0" />
            <span class="text-[#1c1a17] dark:text-[#faf9f7] font-semibold truncate">{{ getToolDisplayLabel(currentArtifact.toolName) }}</span>
            <span class="text-[10px] text-[#746f66] dark:text-[#8f8a81] hidden sm:inline">CallId: {{ currentArtifact.callId.slice(-8) }}</span>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button
              type="button"
              @click="copyRawResult(currentArtifact.result)"
              class="px-2.5 py-1 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] dark:hover:bg-[#33302a] text-[11px] text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] flex items-center gap-1 transition-all cursor-pointer select-none"
              title="复制底层完整 RAW JSON"
            >
              <Check v-if="copiedRaw" :size="11" class="text-emerald-600 dark:text-emerald-400" />
              <Copy v-else :size="11" />
              <span>{{ copiedRaw ? '已复制 JSON' : '复制 JSON' }}</span>
            </button>
          </div>
        </div>

        <!-- 1. 执行中 Loading 状态 -->
        <div
          v-if="currentArtifact.status === 'running'"
          class="h-[360px] rounded-2xl border border-[#e4e1da] dark:border-[#33302a] bg-[#fcfbf9] dark:bg-[#1c1a17] flex flex-col items-center justify-center space-y-4 shadow-xs"
        >
          <div class="relative flex items-center justify-center">
            <div class="w-14 h-14 rounded-full border-2 border-[#1764e8]/20 animate-ping absolute"></div>
            <div class="w-12 h-12 rounded-xl bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 flex items-center justify-center shadow-sm">
              <Loader2 :size="20" class="text-[#1764e8] animate-spin" />
            </div>
          </div>
          <div class="space-y-1 text-center">
            <h4 class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono">端侧 GPU / CPU 神经算子计算中</h4>
            <p class="text-[11px] text-[#746f66] dark:text-[#8f8a81]">100% 本地运算 · 零数据上传 · 隐私安全保障</p>
          </div>
        </div>

        <!-- 2. 执行成功：高保真多模态卡片分流 -->
        <template v-else-if="currentArtifact.result && currentArtifact.status === 'success'">
          <!-- A. OCR 视觉文字与坐标提取 -->
          <OcrArtifactCard
            v-if="isOcrTool(currentArtifact.toolName)"
            :result="currentArtifact.result"
          />

          <!-- B. 视觉超分辨率图像重构 -->
          <ImageArtifactCard
            v-else-if="isImageTool(currentArtifact.toolName, currentArtifact.result)"
            :result="currentArtifact.result"
          />

          <!-- C. 神经机器多语种翻译对照 -->
          <TranslateArtifactCard
            v-else-if="isTranslateTool(currentArtifact.toolName)"
            :result="currentArtifact.result"
            :inputParams="currentArtifact.input"
          />

          <!-- D. Python 数据分析沙箱输出 -->
          <div
            v-else-if="isSandboxTool(currentArtifact.toolName)"
            class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs"
          >
            <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2 text-xs font-mono">
              <div class="flex items-center gap-2 text-[#1c1a17] dark:text-[#faf9f7] font-bold">
                <Terminal :size="13" class="text-[#1764e8]" />
                <span>Python 沙箱执行日志与标准输出</span>
              </div>
            </div>
            <pre class="p-3.5 bg-[#f0eeea]/60 dark:bg-[#141210] rounded-lg border border-[#e4e1da] dark:border-[#26231f] text-xs font-mono text-[#1c1a17] dark:text-[#faf9f7] whitespace-pre-wrap select-text leading-relaxed overflow-x-auto custom-scrollbar">{{ currentArtifact.result.content?.[0]?.text || JSON.stringify(currentArtifact.result, null, 2) }}</pre>
          </div>

          <!-- E. 复杂表格拓扑与 HTML 渲染 -->
          <div
            v-else-if="isTableTool(currentArtifact.toolName)"
            class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs"
          >
            <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2 text-xs font-mono">
              <div class="flex items-center gap-2 text-[#1c1a17] dark:text-[#faf9f7] font-bold">
                <Table :size="13" class="text-[#1764e8]" />
                <span>表格拓扑结构化重构 HTML</span>
              </div>
            </div>
            <div
              v-if="currentArtifact.result.html_structure"
              class="p-3.5 bg-white dark:bg-[#141210] rounded-lg border border-[#e4e1da] dark:border-[#26231f] overflow-x-auto custom-scrollbar text-xs text-[#1c1a17] dark:text-[#faf9f7] prose dark:prose-invert max-w-none"
              v-html="currentArtifact.result.html_structure"
            />
            <pre v-else class="p-3 bg-[#f0eeea]/60 dark:bg-[#141210] rounded-lg text-[11px] font-mono text-[#746f66] overflow-x-auto">{{ JSON.stringify(currentArtifact.result, null, 2) }}</pre>
          </div>

          <!-- F. 多模态文档重构 (Markdown + 产物目录) -->
          <div
            v-else-if="isDocTool(currentArtifact.toolName)"
            class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs"
          >
            <div class="flex items-center justify-between border-b border-[#f0eeea] dark:border-[#26231f] pb-2 text-xs font-mono">
              <div class="flex items-center gap-2 text-[#1c1a17] dark:text-[#faf9f7] font-bold">
                <FileText :size="13" class="text-[#1764e8]" />
                <span>多模态 PDF 深度重构产物</span>
              </div>
              <button
                v-if="currentArtifact.result.output_md_path || currentArtifact.result.task_out_dir"
                type="button"
                @click="openOutputPath(currentArtifact.result.output_md_path || currentArtifact.result.task_out_dir)"
                class="px-2.5 py-1 rounded-lg bg-[#edf4ff] dark:bg-[#1e293b] hover:bg-[#cce0ff] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-xs text-[#1764e8] font-bold flex items-center gap-1.5 transition-all cursor-pointer"
              >
                <FolderOpen :size="12" />
                <span>定位产物目录</span>
              </button>
            </div>
            <div
              v-if="currentArtifact.result.markdown"
              class="p-3.5 bg-white dark:bg-[#141210] rounded-lg border border-[#e4e1da] dark:border-[#26231f] max-h-[320px] overflow-y-auto custom-scrollbar text-xs font-mono text-[#1c1a17] dark:text-[#faf9f7] whitespace-pre-wrap select-text leading-relaxed"
            >
              {{ currentArtifact.result.markdown }}
            </div>
          </div>

          <!-- G. 兜底通用结构化数据卡片 -->
          <div v-else class="rounded-xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] p-4 space-y-3 shadow-xs">
            <div class="flex items-center justify-between text-xs font-mono font-bold text-[#1c1a17] dark:text-[#faf9f7] border-b border-[#f0eeea] dark:border-[#26231f] pb-2">
              <span class="flex items-center gap-1.5">
                <FileCode2 :size="13" class="text-[#1764e8]" />
                <span>算子交付数据详情</span>
              </span>
            </div>
            <pre class="p-3.5 bg-[#f0eeea]/60 dark:bg-[#141210] rounded-lg border border-[#e4e1da] dark:border-[#26231f] text-[11px] font-mono text-[#57534a] dark:text-[#a19d92] overflow-x-auto custom-scrollbar select-text">{{ JSON.stringify(currentArtifact.result, null, 2) }}</pre>
          </div>
        </template>

        <!-- 3. 执行失败提示 -->
        <div
          v-else-if="currentArtifact.status === 'error'"
          class="p-4 rounded-xl bg-rose-50 dark:bg-rose-950/20 border border-rose-200 dark:border-rose-900/40 text-rose-700 dark:text-rose-400 text-xs font-mono space-y-2 shadow-xs"
        >
          <div class="flex items-center gap-1.5 font-bold">
            <AlertCircle :size="14" />
            <span>算子推导异常报错</span>
          </div>
          <pre class="p-2.5 bg-white/80 dark:bg-[#141210] rounded-lg text-rose-600 dark:text-rose-300 whitespace-pre-wrap overflow-x-auto border border-rose-100 dark:border-rose-900/30">{{ JSON.stringify(currentArtifact.result, null, 2) }}</pre>
        </div>
      </template>

      <!-- 场景 B: 暂无产物时的沉浸式占位区 -->
      <div
        v-else
        class="h-full min-h-[440px] rounded-2xl border border-dashed border-[#e4e1da] dark:border-[#33302a] flex flex-col items-center justify-center p-8 text-center space-y-3.5 text-[#746f66] dark:text-[#8f8a81] select-none"
      >
        <div class="w-14 h-14 rounded-2xl bg-[#f0eeea]/60 dark:bg-[#26231f] border border-[#e4e1da] dark:border-[#33302a] flex items-center justify-center text-[#746f66] dark:text-[#8f8a81] shadow-xs">
          <LayoutTemplate :size="24" class="text-[#1764e8]/70" />
        </div>
        <div class="space-y-1 max-w-sm">
          <h3 class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono">Artifacts 产物画布就绪</h3>
          <p class="text-[11.5px] leading-relaxed text-[#746f66] dark:text-[#8f8a81]">
            在左侧输入需要调用的算子任务（如 8K 图像放大、文字提取、翻译或文档解析），产物将自动投影在此处呈现！
          </p>
        </div>
      </div>
    </main>
  </div>
</template>
