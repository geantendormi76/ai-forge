// 🛡️ 紫电 AI 桌面工坊 - 工业级批量多任务工作流状态机母线 (useToolWorkflow.ts)
// 1:1 外科手术式对齐 ToolKnit 原生桌面端批量队列、模态遮罩与成功交付体系

import { ref, shallowRef } from 'vue';
import { useUIStore } from '../store/uiStore';
import { revealItemInDir, openPath } from '@tauri-apps/plugin-opener';

export type WorkflowState =
  | 'idle'      // 空闲：等待添加文件
  | 'queued'    // 已排队：队列已有任务，等待开始
  | 'running'   // 运行中：正在批量推导与渲染
  | 'success'   // 成功：批处理全部交付完成
  | 'error';    // 异常：运行中断或报错

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
  targetFormat?: string;
  outputDir: string;
  lastOutputPath?: string;
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

export function useToolWorkflow<TResult = any>() {
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

  // 模态弹窗响应式状态
  const isProcessingModalOpen = ref(false);
  const isSuccessModalOpen = ref(false);
  const successInfo = ref<BatchSuccessPayload | null>(null);

  const latestResults = shallowRef<TResult[]>([]);
  const fileInputRef = ref<HTMLInputElement | null>(null);

  const triggerFileSelect = () => {
    fileInputRef.value?.click();
  };

  /** 批量追加文件到队列 */
  const addFiles = async (files: FileList | File[] | { name: string; path?: string; size?: number }[]) => {
    const rawList = Array.isArray(files) ? files : Array.from(files);
    if (rawList.length === 0) return;

    const newItems: ToolFileMetadata[] = [];

    for (const f of rawList) {
      const path = (f as any).path || f.name;
      const name = f.name;
      const sizeBytes = f.size || 0;

      // 查重：避免重复添加相同物理文件
      if (queue.value.some(q => q.path === path || (q.name === name && q.sizeBytes === sizeBytes))) {
        continue;
      }

      const item: ToolFileMetadata = {
        id: `task_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
        name,
        path,
        sizeBytes,
        sizeFormatted: formatBytes(sizeBytes),
      };

      if (f instanceof File && (f.type.startsWith('video/') || f.type.startsWith('audio/'))) {
        try {
          const url = URL.createObjectURL(f);
          const media = document.createElement(f.type.startsWith('video/') ? 'video' : 'audio');
          media.preload = 'metadata';
          media.src = url;
          await new Promise<void>((resolve) => {
            media.onloadedmetadata = () => {
              item.durationSec = media.duration;
              item.durationFormatted = formatDuration(media.duration);
              URL.revokeObjectURL(url);
              resolve();
            };
            media.onerror = () => {
              URL.revokeObjectURL(url);
              resolve();
            };
          });
        } catch {
          // 忽略 DOM 嗅探异常
        }
      }

      newItems.push(item);
    }

    if (newItems.length > 0) {
      queue.value.push(...newItems);
      state.value = 'queued';
    }
  };

  /** 剔除指定索引的任务 */
  const removeFile = (index: number) => {
    queue.value.splice(index, 1);
    if (queue.value.length === 0) {
      state.value = 'idle';
      if (fileInputRef.value) fileInputRef.value.value = '';
    }
  };

  /** 一键清空任务队列 */
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

  /** 取消当前批处理 */
  const cancelProcessing = () => {
    isCancelled.value = true;
    isProcessingModalOpen.value = false;
    state.value = 'queued';
    ui.弹出提示('已取消当前任务处理', 'info');
  };

  /** 统一多任务批处理调度母线 */
  const executeBatch = async (
    runner: (file: ToolFileMetadata, index: number, total: number) => Promise<TResult>,
    options: {
      targetFormat?: string;
      statusPrefix?: string;
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

    const total = queue.value.length;
    const t0 = performance.now();
    let lastOutput = '';

    try {
      for (let i = 0; i < total; i++) {
        if (isCancelled.value) break;

        activeIndex.value = i;
        progress.value = Math.round((i / total) * 100);
        statusText.value = `${options.statusPrefix || '正在处理...'} (${i + 1}/${total})`;

        const file = queue.value[i];
        const res = await runner(file, i, total);
        latestResults.value.push(res);

        if (res && typeof res === 'object') {
          if ('output_video_path' in res) lastOutput = (res as any).output_video_path;
          else if ('output_path' in res) lastOutput = (res as any).output_path;
        }
      }

      if (isCancelled.value) return;

      progress.value = 100;
      state.value = 'success';
      isProcessingModalOpen.value = false;
      elapsedMs.value = Math.round(performance.now() - t0);

      const outputDir = lastOutput
        ? lastOutput.substring(0, lastOutput.lastIndexOf('\\')) || lastOutput.substring(0, lastOutput.lastIndexOf('/'))
        : '已保存至源文件同级目录';

      successInfo.value = {
        title: total === 1 ? `已将 ${queue.value[0].name} 处理完成` : `已成功批量处理 ${total} 个视频文件`,
        totalProcessed: total,
        targetFormat: options.targetFormat || 'MKV',
        outputDir,
        lastOutputPath: lastOutput,
      };

      isSuccessModalOpen.value = true;
    } catch (err: any) {
      state.value = 'error';
      isProcessingModalOpen.value = false;
      const msg = typeof err === 'string' ? err : err?.message || String(err);
      errorMessage.value = msg;
      ui.弹出提示(`🚨 处理失败: ${msg}`, 'error');
      throw err;
    }
  };

  /** 调起原生资源管理器打开并高亮产物所在目录 */
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
