<script setup lang="ts">
import { onMounted } from 'vue';
import { useUIStore } from '../../store/uiStore';
import ShinyText from '../effects/ShinyText.vue';
import {
  LayoutDashboard,
  Languages,
  Film,
  Zap,
  FileText,
  Scan,
  PanelLeftClose,
  PanelLeftOpen,
  User,
  Flame,
  Workflow,
  Database,
  Wand2
} from 'lucide-vue-next';

const ui = useUIStore();

// 1. 核心五大已上线工作台 (含离线高精翻译)
const mainWorkbenches = [
  { id: 'home', label: '工坊主页', icon: LayoutDashboard },
  { id: 'trans', label: '离线高精翻译', icon: Languages },
  { id: 'upscale', label: '4K/8K 图像超分', icon: Scan },
  { id: 'format', label: '全能格式转换', icon: Zap },
  { id: 'pdf', label: 'PDF 智能解析', icon: FileText },
  { id: 'asr', label: '视频字幕生成', icon: Film },
];

// 2. 🌟 未来核心算力前瞻规划
const extensionOperators = [
  { id: 'rpa', label: 'RPA 自动化', icon: Workflow },
  { id: 'rag', label: 'RAG 知识库', icon: Database },
  { id: 'ai_edit', label: 'AI 无痕修改', icon: Wand2 },
];

const switchView = (id: string) => {
  if (['home', 'trans', 'upscale', 'format', 'asr', 'pdf'].includes(id)) {
    ui.navigateToTool(id as any);
  } else {
    const target = extensionOperators.find((o) => o.id === id);
    ui.弹出提示(`💡 [${target ? target.label : '算子'}] 正在端侧规划构建中，敬请期待...`, 'info');
  }
};

const openQuotaModal = () => {
  ui.refreshQuota();
  ui.showQuotaModal = true;
};

onMounted(() => {
  ui.refreshQuota();
});
</script>

<template>
  <aside
    class="fixed left-3 top-3 bottom-3 bg-[#182022]/95 backdrop-blur-2xl rounded-[24px] shadow-[0_16px_48px_rgba(0,0,0,0.6)] flex flex-col pt-2.5 pb-3 z-50 transition-all duration-300 ease-in-out select-none border border-white/[0.08] overflow-hidden"
    :class="ui.侧边栏收起 ? 'w-[64px]' : 'w-[236px]'"
  >
    <!-- ==================== 顶部品牌与切换开关 ==================== -->
    <div class="px-2.5 h-10 mb-2 flex items-center justify-between pointer-events-auto shrink-0 relative z-50" style="-webkit-app-region: no-drag !important;">
      <!-- 展开态头部 -->
      <div v-if="!ui.侧边栏收起" class="flex items-center justify-between w-full">
        <div
          @click="ui.navigateToTool('home')"
          class="cursor-pointer hover:opacity-80 active:scale-95 transition-all flex items-center gap-2 pl-1"
          title="返回紫电主页"
        >
          <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-[#bc05ff] to-[#DCA54C] flex items-center justify-center shadow-[0_0_12px_rgba(188,5,255,0.4)]">
            <Zap :size="15" class="text-white fill-white" />
          </div>
          <ShinyText
            text="紫电 AI"
            :speed="3.5"
            :spread="150"
            color="#f2f2ef"
            shine-color="#DCA54C"
            class="text-[17px] font-black tracking-tight"
          />
        </div>
        <button
          type="button"
          @click.stop="ui.切换侧边栏"
          class="w-8 h-8 rounded-lg text-[#8b999b] hover:text-white hover:bg-white/[0.08] active:scale-90 flex items-center justify-center transition-all cursor-pointer shrink-0"
          title="收起侧边栏"
        >
          <PanelLeftClose :size="16" class="stroke-[2.2]" />
        </button>
      </div>

      <!-- 收起态头部 -->
      <div v-else class="flex items-center justify-center w-full">
        <button
          type="button"
          @click.stop="ui.切换侧边栏"
          class="w-9 h-9 rounded-xl bg-white/[0.04] hover:bg-white/[0.1] active:scale-90 border border-white/10 hover:border-[#02c3b4]/50 text-[#8b999b] hover:text-[#02c3b4] flex items-center justify-center transition-all cursor-pointer group relative"
          title="展开侧边栏"
        >
          <PanelLeftOpen :size="16" class="stroke-[2.2]" />
          <div class="absolute left-12 bg-black/90 text-white font-bold text-[11px] px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50">
            展开侧边栏
          </div>
        </button>
      </div>
    </div>

    <!-- 中部菜单滚动区 -->
    <div
      class="flex-1 overflow-y-auto overflow-x-hidden px-2 space-y-1.5 pointer-events-auto"
      :class="ui.侧边栏收起 ? 'scrollbar-hide' : 'custom-scrollbar'"
      style="-webkit-app-region: no-drag;"
    >
      <div
        v-for="item in mainWorkbenches"
        :key="item.id"
        @click="switchView(item.id)"
        class="relative flex items-center rounded-2xl cursor-pointer transition-all duration-150 border select-none group"
        :class="[
          ui.侧边栏收起 ? 'justify-center p-2.5' : 'justify-between px-3 py-2.5',
          ui.currentView === item.id
            ? 'bg-white/[0.08] border-[#02c3b4]/40 text-white font-bold shadow-[0_4px_20px_rgba(2,195,180,0.15)]'
            : 'border-transparent text-[#8b999b] hover:bg-white/[0.04] hover:text-[#f2f2ef] font-medium'
        ]"
      >
        <div class="flex items-center gap-2.5">
          <component
            :is="item.icon"
            :size="17"
            :class="ui.currentView === item.id ? 'text-[#02c3b4] stroke-[2.5]' : 'text-[#8b999b] group-hover:text-white stroke-[2]'"
          />
          <span v-if="!ui.侧边栏收起" class="text-[13px] tracking-wide truncate">{{ item.label }}</span>
        </div>
        <span
          v-if="!ui.侧边栏收起 && ui.currentView === item.id"
          class="w-1.5 h-1.5 rounded-full bg-[#02c3b4] shadow-[0_0_8px_#02c3b4]"
        />

        <div
          v-if="ui.侧边栏收起"
          class="absolute left-14 bg-black/90 text-white font-bold text-xs px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50"
        >
          {{ item.label }}
        </div>
      </div>

      <template v-if="!ui.侧边栏收起">
        <div class="pt-3 pb-1 px-2 flex items-center justify-between">
          <span class="text-[10px] font-mono font-bold text-[#5b696b] uppercase tracking-wider">本地算力扩展</span>
          <span class="text-[9px] font-mono text-[#02c3b4]/70 bg-[#02c3b4]/10 px-1.5 py-0.2 rounded">3 规划</span>
        </div>
        <div
          v-for="item in extensionOperators"
          :key="item.id"
          @click="switchView(item.id)"
          class="flex items-center justify-between px-3 py-1.5 rounded-2xl cursor-pointer transition-all border border-transparent text-[#6e7d80] hover:bg-white/[0.03] hover:text-[#c4d4d6] hover:border-white/[0.04] font-medium select-none group"
        >
          <div class="flex items-center gap-2.5">
            <component :is="item.icon" :size="15" class="text-[#526063] group-hover:text-[#8b999b] stroke-[2]" />
            <span class="text-[12px] tracking-wide">{{ item.label }}</span>
          </div>
          <span class="text-[9px] font-mono text-[#435154] group-hover:text-[#02c3b4] transition-colors">规划</span>
        </div>
      </template>
    </div>

    <!-- 底部：Tokens 胶囊 -->
    <div class="px-2 pt-2 border-t border-white/[0.06] space-y-2 pointer-events-auto shrink-0 overflow-hidden" style="-webkit-app-region: no-drag;">
      <div
        @click="openQuotaModal"
        class="relative rounded-2xl p-2.5 bg-gradient-to-r from-white/[0.04] to-white/[0.02] hover:from-[#02c3b4]/10 hover:to-[#02c3b4]/5 border border-white/10 hover:border-[#02c3b4]/40 cursor-pointer transition-all shadow-md group"
      >
        <div v-if="!ui.侧边栏收起" class="space-y-1.5">
          <div class="flex items-center justify-between text-[11px] font-mono">
            <span class="flex items-center gap-1 text-[#02c3b4] font-bold">
              <Flame :size="12" class="fill-[#02c3b4] text-[#02c3b4]" />
              <span>Tokens</span>
            </span>
            <span class="text-white font-bold">{{ ui.quota.remaining_points }} <span class="text-[#5b696b] font-normal">/ {{ ui.quota.daily_limit }}</span></span>
          </div>
          <div class="w-full h-1 bg-white/10 rounded-full overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-[#02c3b4] to-[#ffb74d] rounded-full transition-all duration-300"
              :style="{ width: `${Math.min(100, Math.max(5, (ui.quota.remaining_points / (ui.quota.daily_limit || 1)) * 100))}%` }"
            />
          </div>
          <div class="flex items-center justify-between text-[9.5px] font-mono text-[#5b696b]">
            <span>每日重置</span>
            <span class="text-[#ffb74d]">已用 {{ ui.quota.used_today }} Tokens</span>
          </div>
        </div>

        <div v-else class="flex flex-col items-center justify-center py-1 text-[#02c3b4]">
          <Flame :size="16" class="fill-[#02c3b4]" />
          <span class="text-[9px] font-mono font-bold mt-0.5 text-white">{{ ui.quota.remaining_points }}</span>
          <div class="absolute left-14 bg-black/90 text-white font-bold text-xs px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50">
            可用: {{ ui.quota.remaining_points }} Tokens (已用 {{ ui.quota.used_today }})
          </div>
        </div>
      </div>

      <div
        @click="openQuotaModal"
        class="relative flex items-center rounded-xl cursor-pointer transition-all border border-transparent text-[#8b999b] hover:text-white hover:bg-white/[0.04] p-1.5"
        :class="ui.侧边栏收起 ? 'justify-center' : 'gap-2.5'"
        title="点击查看 Tokens 与账户"
      >
        <div class="w-7 h-7 rounded-full bg-gradient-to-tr from-[#ffb74d] to-[#02c3b4] p-[1.5px] shrink-0">
          <div class="w-full h-full rounded-full bg-[#182022] flex items-center justify-center text-white">
            <User :size="13" class="stroke-[2.5]" />
          </div>
        </div>
        <div v-if="!ui.侧边栏收起" class="flex flex-col min-w-0 flex-1">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-white truncate">紫电探索者</span>
            <span class="text-[9px] font-mono font-bold bg-[#02c3b4]/15 text-[#02c3b4] px-1 py-0.2 rounded">在线</span>
          </div>
          <span class="text-[10px] font-mono text-[#8b999b] leading-tight">每日 600 Tokens</span>
        </div>
      </div>
    </div>
  </aside>
</template>
