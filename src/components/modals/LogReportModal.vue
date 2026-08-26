<script setup lang="ts">
import { ref, watch } from 'vue';
import { useUIStore } from '../../store/uiStore';
import { getDiagnosticReport, openLogDir } from '../../bindings/index';
import { Terminal, Copy, FolderOpen, X, Check, RefreshCw } from 'lucide-vue-next';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const ui = useUIStore();
const reportText = ref('正在抓取系统硬件与运行流水日志...');
const isCopied = ref(false);
const isLoading = ref(false);

const loadReport = async () => {
  isLoading.value = true;
  try {
    const report = await getDiagnosticReport();
    reportText.value = report;
  } catch (err: any) {
    reportText.value = `获取诊断信息失败: ${err}`;
  } finally {
    isLoading.value = false;
  }
};

watch(
  () => props.visible,
  (val) => {
    if (val) {
      isCopied.value = false;
      loadReport();
    }
  }
);

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(reportText.value);
    isCopied.value = true;
    ui.弹出提示('✅ 诊断报告与流水日志已复制到剪贴板，可直接粘贴发给作者！', 'success');
    setTimeout(() => {
      isCopied.value = false;
    }, 2500);
  } catch {
    ui.弹出提示('复制失败，请手动选取文字复制', 'error');
  }
};

const handleOpenDir = async () => {
  try {
    await openLogDir();
    ui.弹出提示('📁 已在资源管理器中定位日志文件夹', 'info');
  } catch (err: any) {
    ui.弹出提示(`打开日志目录失败: ${err}`, 'error');
  }
};

const handleClose = () => {
  emit('update:visible', false);
};
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none pointer-events-auto"
  >
    <div class="max-w-[560px] w-full p-6 sm:p-7 rounded-[32px] bg-[#161e20] border border-white/15 shadow-[0_25px_60px_rgba(0,0,0,0.8)] text-white space-y-4 animate-in zoom-in-95 duration-150 relative flex flex-col">
      <!-- 右上角关闭与刷新按钮 -->
      <div class="absolute top-5 right-5 flex items-center gap-1.5">
        <button
          type="button"
          @click="loadReport"
          :disabled="isLoading"
          class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-[#8b999b] hover:text-white flex items-center justify-center transition-colors cursor-pointer"
          title="刷新诊断日志"
        >
          <RefreshCw :size="14" :class="{ 'animate-spin': isLoading }" />
        </button>
        <button
          type="button"
          @click="handleClose"
          class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-[#8b999b] hover:text-white flex items-center justify-center transition-colors cursor-pointer"
          title="关闭"
        >
          <X :size="16" />
        </button>
      </div>

      <!-- 头部标识 -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-2xl bg-gradient-to-tr from-[#02c3b4] to-[#00d2ff] flex items-center justify-center shadow-[0_0_16px_rgba(2,195,180,0.35)] shrink-0">
          <Terminal :size="20" class="text-slate-950 stroke-[2.5]" />
        </div>
        <div>
          <h3 class="text-base font-black text-white">
            日志与运行诊断
          </h3>
          <p class="text-xs text-[#8b999b] mt-0.5">硬件驱动检测与全生命周期黑匣子日志</p>
        </div>
      </div>

      <!-- 诊断报告控制台展示框 -->
      <div class="p-3.5 rounded-2xl bg-[#0a0f10] border border-white/10 font-mono text-[11px] sm:text-xs text-[#a5f3fc] leading-relaxed max-h-[300px] overflow-y-auto custom-scrollbar select-text">
        <pre class="whitespace-pre-wrap font-mono text-[11px] sm:text-xs text-[#b8f0f6] leading-relaxed select-text">{{ reportText }}</pre>
      </div>

      <!-- 底部双操作按钮组 -->
      <div class="grid grid-cols-2 gap-3 pt-1">
        <button
          type="button"
          @click="handleOpenDir"
          class="py-3 px-4 rounded-2xl bg-white/[0.05] hover:bg-white/[0.1] text-white font-bold text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-2 border border-white/10 shadow-sm"
        >
          <FolderOpen :size="14" class="text-[#02c3b4]" />
          <span>打开日志目录</span>
        </button>

        <button
          type="button"
          @click="handleCopy"
          class="py-3 px-4 rounded-2xl bg-gradient-to-r from-[#02c3b4] to-[#00d2ff] hover:opacity-95 text-slate-950 font-black text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-2 shadow-[0_0_16px_rgba(2,195,180,0.3)]"
        >
          <Check v-if="isCopied" :size="14" class="stroke-[3]" />
          <Copy v-else :size="14" class="stroke-[2.5]" />
          <span>{{ isCopied ? '已复制到剪贴板' : '一键复制诊断信息' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
