// 🛡️ 紫电 AI 桌面工坊 - 工业级批量多任务工作流状态机母线 (useToolWorkflow.ts)

import { ref, shallowRef, computed, onMounted, onUnmounted } from 'vue';
import { useUIStore } from '../store/uiStore';
import { revealItemInDir, openPath } from '@tauri-apps/plugin-opener';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { cancelCurrentTask } from '../bindings/index';

export type WorkflowState =
  | 'idle'
  | 'queued'
  | 'running'
  | 'success'
  | 'error';

export interface ToolFileMetadata {
  id: string;
  name: string;
  path: string;
  sizeBytes: number;
  sizeFormatted: string;
  durationSec?: number;
  durationFormatted?: string;
}

export interface BatchSuccessPayload {
  title: string;
  totalProcessed: number;
  failedCount?: number;
  targetFormat?: string;
  outputDir: string;
  lastOutputPath?: string;
  elapsedMs?: number;
  elapsedFormatted?: string;
  tokensConsumed?: number;
}

export interface ToolWorkflowConfig {
  dialogTitle?: string;
  dialogFilters?: { name: string; extensions: string[] }[];
  allowedExtensions?: string[];
  unsupportedPrompt?: (rejectedNames: string[]) => string;
}

export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes <= 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '00:00';
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  if (hrs > 0) {
    return `${hrs.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

export function useToolWorkflow<TResult = any>(config?: ToolWorkflowConfig) {
  const ui = useUIStore();

  const state = ref<WorkflowState>('idle');
  const queue = ref<ToolFileMetadata[]>([]);
  const isDragging = ref(false);
  const activeIndex = ref(0);
  const progress = ref(0);
  const statusText = ref('');
  const elapsedMs = ref(0);
  const errorMessage = ref('');
  const isCancelled = ref(false);

  const isProcessingModalOpen = ref(false);
  const isSuccessModalOpen = ref(false);
  const successInfo = ref<BatchSuccessPayload | null>(null);

  const latestResults = shallowRef<TResult[]>([]);
  const fileInputRef = ref<HTMLInputElement | null>(null);

  let unlistenDragDrop: (() => void) | null = null;

  const allowedSet = computed(() => {
    if (!config?.allowedExtensions || config.allowedExtensions.length === 0 || config.allowedExtensions.includes('*')) {
      return null;
    }
    return new Set(config.allowedExtensions.map((e) => e.toLowerCase().trim().replace(/^\./, '')));
  });

  const addPaths = (paths: string[]) => {
    if (!paths || paths.length === 0) return;
    const newItems: ToolFileMetadata[] = [];
    const rejectedNames: string[] = [];

    for (const rawPath of paths) {
      if (!rawPath || typeof rawPath !== 'string') continue;
      const cleanPath = rawPath.trim();
      if (!cleanPath) continue;

      const name = cleanPath.split(/[/\\]/).pop() || cleanPath;
      const ext = name.split('.').pop()?.toLowerCase() || '';

      if (allowedSet.value && (!ext || !allowedSet.value.has(ext))) {
        rejectedNames.push(name);
        continue;
      }

      if (queue.value.some((q) => q.path === cleanPath)) {
        continue;
      }

      const item: ToolFileMetadata = {
        id: `task_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
        name,
        path: cleanPath,
        sizeBytes: 0,
        sizeFormatted: '物理文件',
      };

      newItems.push(item);
    }

    if (rejectedNames.length > 0) {
      const promptText = config?.unsupportedPrompt ? config.unsupportedPrompt(rejectedNames) : '格式暂不支持';
      ui.弹出提示(promptText, 'info');
    }

    if (newItems.length > 0) {
      queue.value.push(...newItems);
      state.value = 'queued';
    }
  };

  const triggerFileSelect = async () => {
    try {
      const selected = await openDialog({
        multiple: true,
        title: config?.dialogTitle || '选择待处理文件',
        filters: config?.dialogFilters || [
          {
            name: '全部文件',
            extensions: ['*'],
          },
        ],
      });

      if (selected) {
        const pathList = Array.isArray(selected) ? selected : [selected];
        addPaths(pathList);
      }
    } catch (err) {
      console.warn('⚠️ 原生文件选择器调起异常，尝试备用方案:', err);
      fileInputRef.value?.click();
    }
  };

  const addFiles = async (files: FileList | File[] | { name: string; path?: string; size?: number }[]) => {
    const rawList = Array.isArray(files) ? files : Array.from(files);
    if (rawList.length === 0) return;

    const paths: string[] = [];
    for (const f of rawList) {
      const p = (f as any).path;
      if (p && typeof p === 'string' && p.length > 0) {
        paths.push(p);
      }
    }

    if (paths.length > 0) {
      addPaths(paths);
      return;
    }

    const newItems: ToolFileMetadata[] = [];
    const rejectedNames: string[] = [];

    for (const f of rawList) {
      const path = (f as any).path || f.name;
      const name = f.name;
      const sizeBytes = f.size || 0;
      const ext = name.split('.').pop()?.toLowerCase() || '';

      if (allowedSet.value && (!ext || !allowedSet.value.has(ext))) {
        rejectedNames.push(name);
        continue;
      }

      if (queue.value.some((q) => q.path === path)) {
        continue;
      }

      newItems.push({
        id: `task_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
        name,
        path,
        sizeBytes,
        sizeFormatted: formatBytes(sizeBytes),
      });
    }

    if (rejectedNames.length > 0) {
      const promptText = config?.unsupportedPrompt ? config.unsupportedPrompt(rejectedNames) : '格式暂不支持';
      ui.弹出提示(promptText, 'info');
    }

    if (newItems.length > 0) {
      queue.value.push(...newItems);
      state.value = 'queued';
    }
  };

  const removeFile = (index: number) => {
    queue.value.splice(index, 1);
    if (queue.value.length === 0) {
      state.value = 'idle';
      if (fileInputRef.value) fileInputRef.value.value = '';
    }
  };

  const clearQueue = () => {
    queue.value = [];
    state.value = 'idle';
    progress.value = 0;
    statusText.value = '';
    errorMessage.value = '';
    latestResults.value = [];
    if (fileInputRef.value) fileInputRef.value.value = '';
  };

  const handleFileChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      addFiles(target.files);
    }
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = true;
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = false;
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    isDragging.value = false;
    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      addFiles(e.dataTransfer.files);
    }
  };

  onMounted(async () => {
    try {
      const appWindow = getCurrentWebviewWindow();
      unlistenDragDrop = await appWindow.onDragDropEvent((event) => {
        if (event.payload.type === 'over' || event.payload.type === 'enter') {
          isDragging.value = true;
        } else if (event.payload.type === 'leave') {
          isDragging.value = false;
        } else if (event.payload.type === 'drop') {
          isDragging.value = false;
          if (event.payload.paths && event.payload.paths.length > 0) {
            addPaths(event.payload.paths);
          }
        }
      });
    } catch (e) {
      console.warn('⚠️ 物理窗口原生拖拽监听未激活:', e);
    }
  });

  onUnmounted(() => {
    if (unlistenDragDrop) {
      unlistenDragDrop();
      unlistenDragDrop = null;
    }
  });

  const cancelProcessing = async () => {
    isCancelled.value = true;
    isProcessingModalOpen.value = false;
    state.value = 'queued';
    try {
      await cancelCurrentTask();
      ui.弹出提示('🛑 已成功截停当前任务，算力已释放', 'info');
    } catch (err) {
      console.warn('⚠️ 截停指令发送异常:', err);
    }
  };

  const executeBatch = async (
    runner: (file: ToolFileMetadata, index: number, total: number) => Promise<TResult>,
    options: {
      targetFormat?: string;
      statusPrefix?: string;
      tokensPerItem?: number;
    } = {}
  ) => {
    if (queue.value.length === 0) {
      ui.弹出提示('请先添加待处理文件到队列', 'info');
      return;
    }

    isCancelled.value = false;
    state.value = 'running';
    isProcessingModalOpen.value = true;
    isSuccessModalOpen.value = false;
    errorMessage.value = '';
    latestResults.value = [];

    const beforeUsed = ui.quota.used_today;
    const total = queue.value.length;
    const t0 = performance.now();
    let lastOutput = '';
    let successCount = 0;
    let failedCount = 0;
    const failureErrors: string[] = [];

    for (let i = 0; i < total; i++) {
      if (isCancelled.value) break;

      activeIndex.value = i;
      progress.value = Math.round((i / total) * 100);
      const currentFile = queue.value[i];
      statusText.value = `${options.statusPrefix || '正在处理'} (${i + 1}/${total}): ${currentFile.name}`;

      try {
        const res = await runner(currentFile, i, total);
        latestResults.value.push(res);
        successCount++;

        if (res && typeof res === 'object') {
          if ('output_md_path' in res) lastOutput = (res as any).output_md_path;
          else if ('output_video_path' in res) lastOutput = (res as any).output_video_path;
          else if ('output_path' in res) lastOutput = (res as any).output_path;
        }
      } catch (itemErr: any) {
        if (isCancelled.value) break;
        failedCount++;
        const msg = typeof itemErr === 'string' ? itemErr : itemErr?.message || String(itemErr);
        failureErrors.push(`${currentFile.name}: ${msg}`);
        console.error(`🚨 [任务 ${i + 1}/${total} 失败]`, itemErr);
        ui.弹出提示(`⚠️ [${currentFile.name}] 处理失败: ${msg}`, 'error');
      }
    }

    // 🌟 立即刷新云端状态
    await ui.refreshQuota();

    if (isCancelled.value) return;

    const totalElapsed = Math.round(performance.now() - t0);
    elapsedMs.value = totalElapsed;
    progress.value = 100;
    isProcessingModalOpen.value = false;

    if (successCount === 0 && failedCount > 0) {
      state.value = 'error';
      errorMessage.value = failureErrors.join(';\n');
      ui.弹出提示(`🚨 全部任务处理失败`, 'error');
      return;
    }

    state.value = 'success';
    const outputDir = lastOutput
      ? lastOutput.substring(0, lastOutput.lastIndexOf('\\')) || lastOutput.substring(0, lastOutput.lastIndexOf('/'))
      : '源文件同级目录';

    const summaryTitle =
      total === 1
        ? `已将 ${queue.value[0].name} 转换完成`
        : failedCount > 0
        ? `已处理完成 ${successCount} 个文件 (另有 ${failedCount} 个失败)`
        : `已成功批量处理 ${total} 个文件`;

    const elapsedFormatted =
      totalElapsed < 1000 ? `${totalElapsed} ms` : `${(totalElapsed / 1000).toFixed(2)} 秒`;

    const deltaTokens = Math.max(0, ui.quota.used_today - beforeUsed);

    successInfo.value = {
      title: summaryTitle,
      totalProcessed: successCount,
      failedCount,
      targetFormat: options.targetFormat || '产物',
      outputDir,
      lastOutputPath: lastOutput,
      elapsedMs: totalElapsed,
      elapsedFormatted,
      tokensConsumed: deltaTokens > 0 ? deltaTokens : (options.tokensPerItem ? options.tokensPerItem * successCount : 2),
    };

    isSuccessModalOpen.value = true;
  };

  const openFolder = async (dirOrFilePath?: string) => {
    const target = dirOrFilePath || successInfo.value?.lastOutputPath || successInfo.value?.outputDir;
    if (!target) return;
    try {
      if (target.includes('.') && !target.endsWith('/') && !target.endsWith('\\')) {
        await revealItemInDir(target);
      } else {
        await openPath(target);
      }
      ui.弹出提示('✅ 已在资源管理器中定位到产物', 'success');
    } catch {
      await navigator.clipboard.writeText(target);
      ui.弹出提示('已复制文件路径到剪贴板', 'info');
    }
  };

  const closeSuccessModal = () => {
    isSuccessModalOpen.value = false;
  };

  return {
    state,
    queue,
    isDragging,
    activeIndex,
    progress,
    statusText,
    elapsedMs,
    errorMessage,
    isProcessingModalOpen,
    isSuccessModalOpen,
    successInfo,
    latestResults,
    fileInputRef,
    triggerFileSelect,
    addFiles,
    addPaths,
    removeFile,
    clearQueue,
    handleFileChange,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    executeBatch,
    cancelProcessing,
    openFolder,
    closeSuccessModal,
  };
}

export type ToolWorkflowInstance<T = any> = ReturnType<typeof useToolWorkflow<T>>;
