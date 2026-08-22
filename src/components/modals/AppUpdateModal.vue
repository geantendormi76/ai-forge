<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { check } from '@tauri-apps/plugin-updater';
import { openUrl } from '@tauri-apps/plugin-opener';
import { isPortable } from '../../bindings/index';
import { Sparkles, Rocket, RefreshCw, X, ExternalLink, HardDrive } from 'lucide-vue-next';

const isVisible = ref(false);
const isChecking = ref(false);
const isUpdating = ref(false);
const isPortableMode = ref(false);
const latestVersion = ref('');
const updateBody = ref('');
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const errorMessage = ref('');
let unlistenProgress: UnlistenFn | null = null;

const DIRECT_DOWNLOAD_URL = 'https://assets.geantendormi.top/downloads/zidian-ai-setup.exe';

const progressPercent = computed(() => {
  if (totalBytes.value <= 0) return 0;
  return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100));
});

const formatBytes = (bytes: number) => {
  if (bytes <= 0) return '0 MB';
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
};

/** 探测云端是否有更高版本发布 (1:1 锚定 Handy 架构) */
const checkForUpdates = async (manual = false) => {
  if (isChecking.value || isUpdating.value) return;
  isChecking.value = true;
  errorMessage.value = '';
  try {
    isPortableMode.value = await isPortable();
    const update = await check();
    if (update && update.available) {
      latestVersion.value = update.version;
      updateBody.value = update.body || '';
      isVisible.value = true;
    }
  } catch (err: any) {
    console.warn('⚠️ 检查更新异常:', err);
    if (manual) {
      errorMessage.value = String(err?.message || err);
    }
  } finally {
    isChecking.value = false;
  }
};

/** 便携模式下：直连外部下载最新发布包 */
const handlePortableDownload = async () => {
  try {
    await openUrl(DIRECT_DOWNLOAD_URL);
    isVisible.value = false;
  } catch (e) {
    console.error('打开外部下载链接失败:', e);
  }
};

/** 安装版模式下：触发 Rust 纯血后端流式下载、验签与自动重启 */
const handleStartUpdate = async () => {
  if (isUpdating.value) return;
  isUpdating.value = true;
  downloadedBytes.value = 0;
  totalBytes.value = 0;
  errorMessage.value = '';
  try {
    unlistenProgress = await listen<{ downloaded: number; total: number; percent: number }>(
      'app-update-progress',
      (event) => {
        downloadedBytes.value = event.payload.downloaded;
        totalBytes.value = event.payload.total;
      }
    );
    await invoke('install_app_update');
  } catch (err: any) {
    isUpdating.value = false;
    errorMessage.value = `更新失败: ${err?.message || err}`;
    console.error('🚨 更新异常:', err);
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  }
};

const handleDismiss = () => {
  if (isUpdating.value) return;
  isVisible.value = false;
};

onMounted(() => {
  setTimeout(() => {
    checkForUpdates(false);
  }, 2000);
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
});

defineExpose({
  checkForUpdates,
});
</script>

<template>
  <div
    v-if="isVisible"
    class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none pointer-events-auto"
  >
    <div
      class="max-w-[440px] w-full p-6 sm:p-7 rounded-[32px] bg-[#161e20] border border-white/15 shadow-[0_25px_60px_rgba(0,0,0,0.8)] text-white space-y-4 animate-in zoom-in-95 duration-150 relative"
    >
      <!-- 右上角关闭按钮 -->
      <button
        v-if="!isUpdating"
        type="button"
        @click="handleDismiss"
        class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-[#8b999b] hover:text-white flex items-center justify-center absolute top-5 right-5 transition-colors cursor-pointer"
      >
        <X :size="16" />
      </button>

      <!-- 头部标识 -->
      <div class="flex items-center gap-3">
        <div
          class="w-12 h-12 rounded-2xl bg-gradient-to-tr from-[#bc05ff] to-[#02c3b4] flex items-center justify-center shadow-[0_0_16px_rgba(2,195,180,0.35)] shrink-0"
        >
          <Rocket :size="22" class="text-white" />
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h3 class="text-base font-black text-white">发现新版本</h3>
            <span
              v-if="latestVersion"
              class="px-2 py-0.5 rounded-full bg-[#02c3b4]/15 border border-[#02c3b4]/40 text-[#02c3b4] font-mono text-[10px] font-bold"
            >
              v{{ latestVersion }}
            </span>
            <span
              v-if="isPortableMode"
              class="px-2 py-0.5 rounded-full bg-amber-500/15 border border-amber-500/40 text-amber-300 font-mono text-[10px] font-bold flex items-center gap-1"
            >
              <HardDrive :size="10" />
              便携版
            </span>
          </div>
          <p class="text-xs text-[#8b999b] font-medium mt-0.5">
            {{ isPortableMode ? '检测到绿色便携运行环境，已激活数据安全保护' : '紫电 AI 官方更新就绪，享受最新算子与极致体验' }}
          </p>
        </div>
      </div>

      <!-- 更新日志说明框 -->
      <div
        class="p-4 rounded-2xl bg-white/[0.03] border border-white/10 space-y-2 text-xs text-slate-300 max-h-[160px] overflow-y-auto custom-scrollbar"
      >
        <div class="flex items-center gap-1.5 text-[#02c3b4] font-bold">
          <Sparkles :size="13" />
          <span>更新内容速览：</span>
        </div>
        <div class="whitespace-pre-wrap leading-relaxed text-[#c4d4d6]">
          {{ updateBody || '• 全系统协同性能与显存守卫深度优化\n• 修复已知问题，提升端侧推演流畅度' }}
        </div>
      </div>

      <!-- 便携模式专用提示 -->
      <div
        v-if="isPortableMode"
        class="p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-xs text-amber-200/90 leading-relaxed"
      >
        🎒 <strong>便携模式提示：</strong>当前生产数据保存在 <code>./Data/</code> 目录。为防止安装程序覆盖便携配置，请点击下方按钮直接下载最新安装包或便携包手动替换。
      </div>

      <!-- 安装版下载进度条 -->
      <div v-else-if="isUpdating" class="w-full space-y-1.5 pt-1">
        <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-[#02c3b4] to-[#bc05ff] rounded-full transition-all duration-200"
            :style="{ width: `${progressPercent}%` }"
          ></div>
        </div>
        <div class="flex justify-between text-[11px] font-mono text-[#8b999b]">
          <span>正在由 Rust 核心极速拉取并校验签名...</span>
          <span>{{ formatBytes(downloadedBytes) }} / {{ formatBytes(totalBytes) }} ({{ progressPercent }}%)</span>
        </div>
      </div>

      <!-- 错误提示 -->
      <div v-if="errorMessage" class="text-xs text-rose-400 bg-rose-500/10 p-2.5 rounded-xl border border-rose-500/20">
        {{ errorMessage }}
      </div>

      <!-- 底部按钮组 -->
      <div class="w-full flex items-center justify-between gap-3 pt-1">
        <button
          v-if="!isUpdating"
          type="button"
          @click="handleDismiss"
          class="flex-1 py-3 px-4 rounded-2xl bg-white/[0.04] hover:bg-white/[0.08] text-[#8b999b] hover:text-white font-bold text-xs transition-all active:scale-95 cursor-pointer border border-white/10"
        >
          稍后提醒
        </button>

        <!-- 便携模式按钮 -->
        <button
          v-if="isPortableMode"
          type="button"
          @click="handlePortableDownload"
          class="flex-1 py-3 px-4 rounded-2xl bg-gradient-to-r from-amber-400 to-amber-500 hover:opacity-95 text-slate-950 font-black text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1.5 shadow-[0_0_20px_rgba(245,158,11,0.3)]"
        >
          <ExternalLink :size="14" />
          <span>下载最新安装包</span>
        </button>

        <!-- 安装版模式按钮 -->
        <button
          v-else
          type="button"
          :disabled="isUpdating"
          @click="handleStartUpdate"
          class="flex-1 py-3 px-4 rounded-2xl bg-gradient-to-r from-[#02c3b4] to-[#00d2ff] hover:opacity-95 text-slate-950 font-black text-xs transition-all active:scale-95 cursor-pointer flex items-center justify-center gap-1.5 shadow-[0_0_20px_rgba(2,195,180,0.3)]"
        >
          <RefreshCw v-if="isUpdating" :size="14" class="animate-spin" />
          <span>{{ isUpdating ? `正在安装更新 (${progressPercent}%)...` : '立即升级并重启' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
