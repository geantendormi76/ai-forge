<script setup lang="ts">
import { useUIStore } from '../../store/uiStore';
import ShinyText from '../effects/ShinyText.vue';
import {
  LayoutDashboard,
  Film,
  Zap,
  FileText,
  Scan,
  Database,
  Music,
  Table,
  BookOpen,
  Image as ImageIcon,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  User,
} from 'lucide-vue-next';

const ui = useUIStore();

// 1. 已竣工主力工坊
const mainWorkbenches = [
  { id: 'home', label: '工坊主页', icon: LayoutDashboard },
  { id: 'upscale', label: '4K/8K 图像超分', icon: Scan },
  { id: 'format', label: '全能格式转换', icon: Zap },
  { id: 'pdf', label: 'PDF 智能解析', icon: FileText },
  { id: 'asr', label: '视频字幕生成', icon: Film },
];

// 2. 本地算力扩展矩阵
const extensionOperators = [
  { id: 'knowledge', label: '本地离线知识库', icon: Database },
  { id: 'audio_master', label: '音频母带增强', icon: Music },
  { id: 'table_clean', label: '多维表格清洗', icon: Table },
  { id: 'ebook_gen', label: '电子书重排排版', icon: BookOpen },
  { id: 'icon_craft', label: '原生多尺寸图标', icon: ImageIcon },
];

const switchView = (id: string) => {
  if (['home', 'upscale', 'format', 'asr', 'pdf'].includes(id)) {
    ui.currentView = id as any;
  } else {
    const target = extensionOperators.find((o) => o.id === id);
    ui.弹出提示(`💡 [${target ? target.label : '算子'}] 正在端侧并网构建中，敬请期待...`, 'info');
  }
};

const handleSettingsClick = () => {
  ui.弹出提示('⚙️ 系统设置面板（模型缓存、硬件算力分配）正在就绪中...', 'info');
};

const handleAccountClick = () => {
  ui.弹出提示('👤 本地账户：离线专业版 · 永久授权激活', 'success');
};
</script>

<template>
  <aside
    class="fixed left-3 top-3 bottom-3 bg-[#182022]/95 backdrop-blur-2xl rounded-[24px] shadow-[0_16px_48px_rgba(0,0,0,0.6)] flex flex-col pt-4 pb-3 z-40 transition-all duration-300 ease-in-out select-none border border-white/[0.08]"
    :class="ui.侧边栏收起 ? 'w-[64px]' : 'w-[236px]'"
    style="-webkit-app-region: drag;"
  >
    <!-- ==================== 顶部品牌与切换开关 ==================== -->
    <div class="px-3.5 mb-3 flex items-center justify-between pointer-events-auto" style="-webkit-app-region: no-drag;">
      
      <!-- 展开态头部 -->
      <div v-if="!ui.侧边栏收起" class="flex items-center justify-between w-full">
        <div
          @click="ui.currentView = 'home'"
          class="cursor-pointer hover:opacity-80 active:scale-95 transition-all flex items-center gap-2"
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
          @click="ui.切换侧边栏"
          class="w-7 h-7 rounded-lg text-[#8b999b] hover:text-white hover:bg-white/[0.08] flex items-center justify-center transition-colors cursor-pointer shrink-0"
          title="收起侧边栏"
        >
          <PanelLeftClose :size="15" class="stroke-[2]" />
        </button>
      </div>

      <!-- 收起态头部 (迷你图轨开关) -->
      <div v-else class="flex flex-col items-center justify-center w-full space-y-2">
        <button
          type="button"
          @click="ui.切换侧边栏"
          class="w-9 h-9 rounded-xl bg-white/[0.04] hover:bg-white/[0.1] border border-white/10 hover:border-[#02c3b4]/50 text-[#8b999b] hover:text-[#02c3b4] flex items-center justify-center transition-all cursor-pointer group relative"
        >
          <PanelLeftOpen :size="16" class="stroke-[2]" />
          
          <!-- 悬浮气泡: 打开边栏 -->
          <div class="absolute left-12 bg-black/90 text-white font-bold text-[11px] px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50">
            打开边栏
          </div>
        </button>
      </div>

    </div>

    <!-- ==================== 中部菜单滚动区 ==================== -->
    <div class="flex-1 overflow-y-auto custom-scrollbar px-2 space-y-1.5 pointer-events-auto" style="-webkit-app-region: no-drag;">
      
      <!-- 主力工坊组 -->
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

        <!-- 收起态 Hover 悬浮气泡 -->
        <div
          v-if="ui.侧边栏收起"
          class="absolute left-14 bg-black/90 text-white font-bold text-xs px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50"
        >
          {{ item.label }}
        </div>
      </div>

      <!-- 分隔线与扩展区 (展开时显示) -->
      <template v-if="!ui.侧边栏收起">
        <div class="pt-3 pb-1 px-2 flex items-center justify-between">
          <span class="text-[10px] font-mono font-bold text-[#5b696b] uppercase tracking-wider">本地算力扩展</span>
          <span class="text-[9px] font-mono text-[#02c3b4]/70 bg-[#02c3b4]/10 px-1.5 py-0.2 rounded">5 算子</span>
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
          <span class="text-[9px] font-mono text-[#435154] group-hover:text-[#02c3b4] transition-colors">离线</span>
        </div>
      </template>

    </div>

    <!-- ==================== 底部：设置与账户常驻区 ==================== -->
    <div class="px-2 pt-2 border-t border-white/[0.06] space-y-1 pointer-events-auto" style="-webkit-app-region: no-drag;">
      
      <!-- 设置按钮 -->
      <div
        @click="handleSettingsClick"
        class="relative flex items-center rounded-xl cursor-pointer transition-all border border-transparent text-[#8b999b] hover:text-white hover:bg-white/[0.04] group"
        :class="ui.侧边栏收起 ? 'justify-center p-2.5' : 'px-3 py-2 gap-2.5'"
        title="系统设置"
      >
        <Settings :size="17" class="stroke-[2] text-[#8b999b] group-hover:text-white" />
        <span v-if="!ui.侧边栏收起" class="text-[12.5px] font-bold">设置</span>

        <div
          v-if="ui.侧边栏收起"
          class="absolute left-14 bg-black/90 text-white font-bold text-xs px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50"
        >
          系统设置
        </div>
      </div>

      <!-- 账户头像按钮 -->
      <div
        @click="handleAccountClick"
        class="relative flex items-center rounded-xl cursor-pointer transition-all border border-transparent text-[#8b999b] hover:text-white hover:bg-white/[0.04] group"
        :class="ui.侧边栏收起 ? 'justify-center p-2' : 'px-2.5 py-1.5 gap-2.5'"
        title="个人账户"
      >
        <div class="w-7 h-7 rounded-full bg-gradient-to-tr from-[#DCA54C] to-[#02c3b4] p-[1.5px] shrink-0">
          <div class="w-full h-full rounded-full bg-[#182022] flex items-center justify-center text-white">
            <User :size="14" class="stroke-[2.5]" />
          </div>
        </div>
        
        <div v-if="!ui.侧边栏收起" class="flex flex-col min-w-0">
          <span class="text-xs font-bold text-white truncate">紫电开发者</span>
          <span class="text-[10px] font-mono text-[#02c3b4] leading-tight">Pro License</span>
        </div>

        <div
          v-if="ui.侧边栏收起"
          class="absolute left-14 bg-black/90 text-white font-bold text-xs px-2.5 py-1 rounded-lg border border-white/15 whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-150 shadow-xl z-50"
        >
          紫电开发者 (Pro)
        </div>
      </div>

    </div>
  </aside>
</template>
