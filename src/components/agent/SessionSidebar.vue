<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  Search,
  Settings,
  Folder,
  RotateCw,
  Check,
  Trash2,
  Edit3,
  MoreHorizontal,
  Clock
} from 'lucide-vue-next';

export interface SessionItemData {
  id: string;
  name?: string;
  firstMessage?: string;
  cwd?: string;
  modified: number;
  messageCount: number;
}

const props = withDefaults(
  defineProps<{
    selectedSessionId?: string | null;
    selectedCwd?: string | null;
    sessions?: SessionItemData[];
    isRunning?: boolean;
  }>(),
  {
    selectedSessionId: null,
    selectedCwd: 'C:\\dev\\ai-forge',
    sessions: () => [],
    isRunning: false,
  }
);

const emit = defineEmits<{
  (e: 'selectSession', id: string): void;
  (e: 'newSession'): void;
  (e: 'openSettings'): void;
  (e: 'renameSession', id: string, newName: string): void;
  (e: 'deleteSession', id: string): void;
  (e: 'changeCwd'): void;
}>();

const searchQuery = ref('');
const isRefreshSuccess = ref(false);
const editingSessionId = ref<string | null>(null);
const editNameValue = ref('');
const activeMenuId = ref<string | null>(null);

// 过滤后的会话列表
const filteredSessions = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return props.sessions;
  return props.sessions.filter(s =>
    (s.name && s.name.toLowerCase().includes(q)) ||
    (s.firstMessage && s.firstMessage.toLowerCase().includes(q))
  );
});

// 相对时间格式化 (33m ago / 2h ago / 昨天)
const formatRelativeTime = (timestamp: number): string => {
  const diffSec = Math.floor((Date.now() - timestamp) / 1000);
  if (diffSec < 60) return '刚刚';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  return `${Math.floor(diffSec / 86400)}d ago`;
};

// 按照“今天 / 最近 7 天 / 更早”分组
const groupedSessions = computed(() => {
  const today: SessionItemData[] = [];
  const recent: SessionItemData[] = [];
  const older: SessionItemData[] = [];

  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const sevenDaysAgo = startOfToday - 7 * 24 * 3600 * 1000;

  for (const s of filteredSessions.value) {
    if (s.modified >= startOfToday) {
      today.push(s);
    } else if (s.modified >= sevenDaysAgo) {
      recent.push(s);
    } else {
      older.push(s);
    }
  }

  return [
    { label: '今天', items: today },
    { label: '最近 7 天', items: recent },
    { label: '更早', items: older },
  ].filter(g => g.items.length > 0);
});

const handleRefreshClick = () => {
  isRefreshSuccess.value = true;
  setTimeout(() => {
    isRefreshSuccess.value = false;
  }, 1200);
};

const startRename = (s: SessionItemData) => {
  editingSessionId.value = s.id;
  editNameValue.value = s.name || s.firstMessage || '未命名会话';
  activeMenuId.value = null;
};

const commitRename = (id: string) => {
  if (editNameValue.value.trim()) {
    emit('renameSession', id, editNameValue.value.trim());
  }
  editingSessionId.value = null;
};

const toggleActionMenu = (id: string, e: MouseEvent) => {
  e.stopPropagation();
  activeMenuId.value = activeMenuId.value === id ? null : id;
};
</script>

<template>
  <aside class="w-[260px] h-full flex flex-col bg-[#fcfbf9] dark:bg-[#1c1a17] border-r border-[#e4e1da] dark:border-[#33302a] select-none font-sans flex-shrink-0 z-20">
    <!-- 1. 顶部标牌与新建按钮 -->
    <div class="p-3.5 pb-2.5 border-b border-[#e4e1da] dark:border-[#33302a] flex flex-col gap-2.5">
      <!-- 品牌 Title 与刷新 -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="w-5 h-5 rounded-[5px] bg-[#1c1a17] dark:bg-[#faf9f7] text-[#1764e8] dark:text-[#1c1a17] flex items-center justify-center font-mono text-xs font-black">
            $
          </div>
          <span class="font-bold text-[13.5px] tracking-tight text-[#1c1a17] dark:text-[#faf9f7] font-mono">
            Pi Agent Desktop
          </span>
        </div>

        <button
          type="button"
          @click="handleRefreshClick"
          class="w-7 h-7 rounded-md border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] text-[#746f66] dark:text-[#8f8a81] flex items-center justify-center transition-all cursor-pointer"
          title="刷新会话"
        >
          <Check v-if="isRefreshSuccess" :size="13" class="text-emerald-600 stroke-[2.5]" />
          <RotateCw v-else :size="13" />
        </button>
      </div>

      <!-- 核心全宽黑色新建会话药丸 -->
      <button
        type="button"
        @click="emit('newSession')"
        class="w-full h-8 rounded-lg bg-[#1c1a17] dark:bg-[#faf9f7] text-[#f7f6f3] dark:text-[#1c1a17] font-mono text-xs font-bold flex items-center justify-center gap-1.5 shadow-sm hover:opacity-90 active:scale-[0.98] transition-all cursor-pointer"
      >
        <span class="text-sm leading-none">+</span>
        <span>新建会话</span>
      </button>

      <!-- 工作目录路径胶囊 -->
      <button
        type="button"
        @click="emit('changeCwd')"
        class="w-full px-2.5 py-1.5 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea]/60 dark:bg-[#26231f]/60 hover:bg-[#f0eeea] text-[#57534a] dark:text-[#a19d92] flex items-center gap-1.5 text-[11px] font-mono transition-all cursor-pointer text-left"
        :title="selectedCwd || '选择项目工作目录'"
      >
        <Folder :size="12" class="text-[#1764e8] shrink-0" />
        <span class="truncate flex-1">{{ selectedCwd || '~/Desktop' }}</span>
      </button>
    </div>

    <!-- 2. 搜索框与列表标头 -->
    <div class="px-3 pt-3 pb-1.5 flex flex-col gap-2">
      <div class="flex items-center justify-between text-[11px] font-mono font-bold text-[#746f66] dark:text-[#8f8a81] uppercase tracking-wider px-1">
        <span>会话历史</span>
        <span>{{ filteredSessions.length }}</span>
      </div>

      <div class="relative">
        <Search :size="13" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[#746f66] dark:text-[#8f8a81]" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索会话..."
          class="w-full h-7 pl-7 pr-3 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-white dark:bg-[#141210] text-xs text-[#1c1a17] dark:text-[#faf9f7] placeholder-[#746f66] outline-none focus:border-[#1764e8] transition-colors font-mono"
        />
      </div>
    </div>

    <!-- 3. 时间轴分组会话树 -->
    <nav class="flex-1 overflow-y-auto px-2 py-1 space-y-3 custom-scrollbar">
      <div
        v-for="group in groupedSessions"
        :key="group.label"
        class="space-y-1"
      >
        <div class="px-2 text-[10.5px] font-bold text-[#746f66] dark:text-[#8f8a81] font-mono">
          {{ group.label }}
        </div>

        <div
          v-for="s in group.items"
          :key="s.id"
          @click="emit('selectSession', s.id)"
          class="group relative w-full p-2 rounded-lg border transition-all duration-150 cursor-pointer flex flex-col gap-1"
          :class="[
            selectedSessionId === s.id
              ? 'bg-[#edf4ff] dark:bg-[#1e293b] border-[#cce0ff] dark:border-[#3b82f6]/40 shadow-xs'
              : 'bg-transparent hover:bg-[#f0eeea] dark:hover:bg-[#26231f] border-transparent'
          ]"
        >
          <div class="flex items-center justify-between gap-1.5 min-w-0">
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <span
                v-if="selectedSessionId === s.id && isRunning"
                class="w-2 h-2 rounded-full bg-[#1764e8] animate-ping shrink-0"
              />
              <span
                v-else
                class="w-1.5 h-1.5 rounded-full shrink-0"
                :class="selectedSessionId === s.id ? 'bg-[#1764e8]' : 'bg-[#746f66]/50'"
              />

              <input
                v-if="editingSessionId === s.id"
                v-model="editNameValue"
                @blur="commitRename(s.id)"
                @keydown.enter="commitRename(s.id)"
                class="w-full text-xs font-medium px-1 bg-white border border-[#1764e8] rounded outline-none"
                autofocus
                @click.stop
              />
              <span
                v-else
                class="text-xs font-semibold text-[#1c1a17] dark:text-[#faf9f7] truncate"
              >
                {{ s.name || s.firstMessage || '未命名会话' }}
              </span>
            </div>

            <button
              type="button"
              @click="toggleActionMenu(s.id, $event)"
              class="opacity-0 group-hover:opacity-100 p-0.5 rounded text-[#746f66] hover:text-[#1c1a17] transition-opacity cursor-pointer"
            >
              <MoreHorizontal :size="14" />
            </button>
          </div>

          <div class="flex items-center gap-2 text-[10.5px] font-mono text-[#746f66] dark:text-[#8f8a81] pl-3">
            <span class="flex items-center gap-0.5">
              <Clock :size="10" />
              <span>{{ formatRelativeTime(s.modified) }}</span>
            </span>
            <span class="px-1.5 py-0.2 rounded bg-[#fbeee0] dark:bg-[#3a2a1f] text-[#8b5e2c] dark:text-[#e8b07a] font-bold">
              {{ s.messageCount }} msgs
            </span>
          </div>

          <div
            v-if="activeMenuId === s.id"
            class="absolute right-2 top-8 z-30 w-28 bg-white dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] rounded-lg shadow-lg py-1 text-xs font-mono"
            @click.stop
          >
            <button
              type="button"
              @click="startRename(s)"
              class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-[#f0eeea] text-[#1c1a17] dark:text-[#faf9f7] text-left cursor-pointer"
            >
              <Edit3 :size="12" />
              <span>重命名</span>
            </button>
            <button
              type="button"
              @click="emit('deleteSession', s.id); activeMenuId = null;"
              class="w-full px-3 py-1.5 flex items-center gap-2 hover:bg-rose-50 text-rose-600 text-left cursor-pointer"
            >
              <Trash2 :size="12" />
              <span>删除会话</span>
            </button>
          </div>
        </div>
      </div>
    </nav>

    <!-- 4. 底部设置按钮 -->
    <div class="p-2 border-t border-[#e4e1da] dark:border-[#33302a]">
      <button
        type="button"
        @click="emit('openSettings')"
        class="w-full h-8 px-3 rounded-lg border border-transparent hover:bg-[#f0eeea] dark:hover:bg-[#26231f] text-[#57534a] dark:text-[#a19d92] hover:text-[#1c1a17] flex items-center gap-2 text-xs font-medium font-mono transition-all cursor-pointer"
      >
        <Settings :size="14" />
        <span>设置与开发工具</span>
      </button>
    </div>
  </aside>
</template>
