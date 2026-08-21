<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { Sparkles, Rocket, RefreshCw, X } from 'lucide-vue-next';

const isVisible = ref(false);
const isChecking = ref(false);
const isUpdating = ref(false);
const updateInfo = ref<Update | null>(null);

const downloadedBytes = ref(0);
const totalBytes = ref(0);
const errorMessage = ref('');

const progressPercent = computed(() => {
  if (totalBytes.value <= 0) return 0;
  return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100));
});

const formatBytes = (bytes: number) => {
  if (bytes <= 0) return '0 MB';
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
};

/** 探测云端是否有更高版本发布 */
const checkForUpdates = async (manual = false) => {
  if (isChecking.value || isUpdating.value) return;
  isChecking.value = true;
  errorMessage.value = '';
  try {
    const update = await check();
    if (update && update.available) {
      updateInfo.value = update;
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

/** 触发一键下载、数字验签与自动重启升级 */
const handleStartUpdate = async () => {
  if (!updateInfo.value || isUpdating.value) return;
  isUpdating.value = true;
  downloadedBytes.value = 0;
  totalBytes.value = 0;
  errorMessage.value = '';

  try {
    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          totalBytes.value = event.data.contentLength || 0;
          break;
        case 'Progress':
          downloadedBytes.value += event.data.chunkLength;
          break;
        case 'Finished':
          break;
      }
    });

    // 升级包替换完成，发起进程热重启
    await relaunch();
  } catch (err: any) {
    isUpdating.value = false;
    errorMessage.value = `更新失败: ${err?.message || err}`;
    console.error('🚨 更新异常:', err);
  }
};

const handleDismiss = () => {
  if (isUpdating.value) return;
  isVisible.value = false;
};

onMounted(() => {
  // 应用启动 2 秒后在后台静默发起版本探针
  setTimeout(() => {
    checkForUpdates(false);
  }, 2000);
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
      class="max-w-[420px] w-full p-6 sm:p-7 rounded-[32px] bg-[#161e20] border border-white/15 shadow-[0_25px_60px_rgba(0,0,0,0.8)] text-white space-y-4 animate-in zoom-in-95 duration-150 relative"
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
              v-if="updateInfo"
              class="px-2 py-0.5 rounded-full bg-[#02c3b4]/15 border border-[#02c3b4]/40 text-[#02c3b4] font-mono text-[10px] font-bold"
            >
              v{{ updateInfo.version }}
            </span>
          </div>
          <p class="text-xs text-[#8b999b] font-medium mt-0.5">
            紫电 AI 官方更新就绪，享受最新算子与极致体验
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
          {{ updateInfo?.body || '• 性能与稳定性全系统协同优化\n• 修复已知问题，提升端侧推演流畅度' }}
        </div>
      </div>

      <!-- 下载进度条 -->
      <div v-if="isUpdating" class="w-full space-y-1.5 pt-1">
        <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-[#02c3b4] to-[#bc05ff] rounded-full transition-all duration-200"
            :style="{ width: `${progressPercent}%` }"
          ></div>
        </div>
        <div class="flex justify-between text-[11px] font-mono text-[#8b999b]">
          <span>正在高速拉取并校验签名...</span>
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
        <button
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
