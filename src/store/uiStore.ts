import { defineStore } from 'pinia';
import { ref } from 'vue';
import {
  getQuotaStatus,
  checkToolDependencies,
  type QuotaStatus,
  type DependencyItem,
} from '../bindings/index';

const USAGE_STORAGE_KEY = 'zidian_tool_usage_counts';
const DEFAULT_INITIAL_COUNTS: Record<string, number> = {
  upscale: 12,
  asr: 10,
  pdf: 8,
  format: 6,
};

export const useUIStore = defineStore('ui', () => {
  const currentView = ref<'home' | 'format' | 'pdf' | 'asr' | 'upscale'>('home');
  const showRadar = ref(false);
  const radarMode = ref<'global' | 'local'>('global');
  const toast消息 = ref<string | null>(null);
  const toast类型 = ref<'success' | 'error' | 'info'>('success');
  const toast显示 = ref(false);
  const 侧边栏收起 = ref(false);
  let toast定时器: number | null = null;

  // 🛡️ 依赖与模型门禁拦截状态机
  const isDependencyModalOpen = ref(false);
  const dependencyTargetTool = ref<string | null>(null);
  const dependencyItems = ref<DependencyItem[]>([]);
  const dependencyCallback = ref<(() => void) | null>(null);

  const openDependencyModal = (
    toolId: string,
    items: DependencyItem[],
    callback?: () => void
  ) => {
    dependencyTargetTool.value = toolId;
    dependencyItems.value = items;
    dependencyCallback.value = callback || null;
    isDependencyModalOpen.value = true;
  };

  const closeDependencyModal = () => {
    isDependencyModalOpen.value = false;
    dependencyTargetTool.value = null;
    dependencyItems.value = [];
    dependencyCallback.value = null;
  };

  /**
   * 🛡️ 核心：带依赖检测的智能路由跳转守卫
   * 适用于全库工具切换：如果缺失本地模型，立即拦截并呼出下载弹窗；全部就绪时才真正切入视窗！
   */
  const navigateToTool = async (
    toolId: 'home' | 'format' | 'pdf' | 'asr' | 'upscale',
    onReady?: () => void
  ) => {
    recordToolUsage(toolId);

    if (toolId === 'home') {
      currentView.value = 'home';
      onReady?.();
      return;
    }

    try {
      // 1. 毫秒级探测当前工具所需模型在本地的真实就绪状态
      const deps = await checkToolDependencies(toolId);
      const missingDeps = deps.filter((d) => !d.is_ready);

      if (missingDeps.length > 0) {
        // 2. 存在缺失依赖，触发拦截模态框
        openDependencyModal(toolId, deps, () => {
          currentView.value = toolId;
          onReady?.();
        });
        return;
      }
    } catch (e) {
      console.warn(`⚠️ 依赖探测异常，尝试常规直通:`, e);
    }

    // 3. 所有依赖均已在本地就绪，直通进入
    currentView.value = toolId;
    onReady?.();
  };

  // 算力 HUD 状态
  const showQuotaModal = ref(false);
  const quota = ref<QuotaStatus>({
    success: true,
    device_fingerprint: 'local_device',
    stage: 'beta',
    daily_limit: 600,
    used_today: 0,
    bonus_points: 0,
    remaining_points: 600,
    status: 'active',
    is_offline_pro: false,
  });

  const refreshQuota = async () => {
    try {
      const res = await getQuotaStatus();
      quota.value = res;
    } catch (e) {
      console.warn('⚠️ 获取云端算力状态异常:', e);
    }
  };

  const 弹出提示 = (消息: string, 类型: 'success' | 'error' | 'info' = 'success') => {
    toast消息.value = 消息;
    toast类型.value = 类型;
    toast显示.value = true;
    if (toast定时器) {
      clearTimeout(toast定时器);
      toast定时器 = null;
    }
    toast定时器 = window.setTimeout(() => {
      toast显示.value = false;
    }, 2500);
  };

  const 切换侧边栏 = () => {
    侧边栏收起.value = !侧边栏收起.value;
  };

  const loadUsageCounts = (): Record<string, number> => {
    try {
      const raw = localStorage.getItem(USAGE_STORAGE_KEY);
      if (raw) {
        return { ...DEFAULT_INITIAL_COUNTS, ...JSON.parse(raw) };
      }
    } catch (e) {
      console.warn('⚠️ 读取工具使用频次异常:', e);
    }
    return { ...DEFAULT_INITIAL_COUNTS };
  };

  const toolUsageCounts = ref<Record<string, number>>(loadUsageCounts());
  const recordToolUsage = (toolId: string) => {
    const current = toolUsageCounts.value[toolId] || 0;
    const nextCounts = {
      ...toolUsageCounts.value,
      [toolId]: current + 1,
    };
    toolUsageCounts.value = nextCounts;
    try {
      localStorage.setItem(USAGE_STORAGE_KEY, JSON.stringify(nextCounts));
    } catch (e) {
      console.warn('⚠️ 写入工具使用频次异常:', e);
    }
  };

  return {
    currentView,
    showRadar,
    radarMode,
    toast消息,
    toast类型,
    toast显示,
    弹出提示,
    侧边栏收起,
    切换侧边栏,
    toolUsageCounts,
    recordToolUsage,
    showQuotaModal,
    quota,
    refreshQuota,
    isDependencyModalOpen,
    dependencyTargetTool,
    dependencyItems,
    dependencyCallback,
    openDependencyModal,
    closeDependencyModal,
    navigateToTool,
  };
});
