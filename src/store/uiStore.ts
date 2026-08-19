import { defineStore } from 'pinia';
import { ref } from 'vue';

const USAGE_STORAGE_KEY = 'zidian_tool_usage_counts';

// 🌟 对齐冷启动基线权重 (asr: 10, pdf: 8, format: 6)
const DEFAULT_INITIAL_COUNTS: Record<string, number> = {
  asr: 10,
  pdf: 8,
  format: 6,
};

export const useUIStore = defineStore('ui', () => {
  const currentView = ref<'home' | 'format' | 'pdf' | 'asr'>('home');
  const showRadar = ref(false);
  const radarMode = ref<'global' | 'local'>('global');
  const toast消息 = ref<string | null>(null);
  const toast类型 = ref<'success' | 'error' | 'info'>('success');
  const toast显示 = ref(false);

  // 悬浮侧边栏收起/展开状态机
  const 侧边栏收起 = ref(false);

  let toast定时器: number | null = null;

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

  // 🌟 1. 读取本地离线持久化频次数据
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

  // 🌟 2. 原子记录工具使用并持久化
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
  };
});
