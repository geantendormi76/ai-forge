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
} from 'lucide-vue-next';

const ui = useUIStore();

// 1. 已竣工主力工坊组
const mainWorkbenches = [
  { id: 'home', label: '工坊主页', icon: LayoutDashboard },
  { id: 'asr', label: '视频字幕生成', icon: Film },
];

// 2. 本地算力扩展矩阵 (规划建设中的离线端侧算子)
const extensionOperators = [
  { id: 'pdf', label: 'PDF 智能混合解析', icon: FileText },
  { id: 'format', label: '全能格式转换', icon: Zap },
  { id: 'super_res', label: '8K 视觉超分', icon: Scan },
  { id: 'knowledge', label: '本地离线知识库', icon: Database },
  { id: 'audio_master', label: '音频解密与母带', icon: Music },
  { id: 'table_clean', label: '多维表格清洗', icon: Table },
  { id: 'ebook_gen', label: '电子书重排排版', icon: BookOpen },
  { id: 'icon_craft', label: '原生多尺寸图标', icon: ImageIcon },
];

const switchView = (id: string) => {
  if (['home', 'asr'].includes(id)) {
    ui.currentView = id as 'home' | 'asr';
  } else {
    const target = extensionOperators.find((o) => o.id === id);
    ui.弹出提示(`💡 [${target ? target.label : '算子'}] 正在端侧并网构建中，敬请期待...`, 'info');
  }
};
</script>

<template>
  <aside
    class="fixed left-4 top-4 bottom-4 w-[230px] bg-[#182022]/90 backdrop-blur-2xl rounded-[24px] shadow-[0_16px_48px_rgba(0,0,0,0.5)] flex flex-col pt-6 pb-4 z-40 transition-all duration-300 ease-in-out select-none border border-white/[0.08]"
    :class="ui.侧边栏收起 ? '-translate-x-[270px] opacity-0 pointer-events-none' : 'translate-x-0 opacity-100'"
    style="-webkit-app-region: drag;"
  >
    <!-- 头部品牌标识 -->
    <div class="px-5 mb-5 flex items-center justify-between pointer-events-auto select-none" style="-webkit-app-region: no-drag;">
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

    <!-- 菜单滚动区 -->
    <div class="flex-1 overflow-y-auto custom-scrollbar px-3 space-y-1 pointer-events-auto" style="-webkit-app-region: no-drag;">
      
      <!-- 主力工坊组 -->
      <div
        v-for="item in mainWorkbenches"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center justify-between px-3.5 py-2.5 rounded-2xl cursor-pointer transition-all duration-150 border select-none group"
        :class="[
          ui.currentView === item.id
            ? 'bg-white/[0.06] border-[#02c3b4]/30 text-white font-bold shadow-[0_4px_20px_rgba(2,195,180,0.12)]'
            : 'border-transparent text-[#8b999b] hover:bg-white/[0.04] hover:text-[#f2f2ef] font-medium'
        ]"
      >
        <div class="flex items-center gap-3">
          <component
            :is="item.icon"
            :size="16"
            :class="ui.currentView === item.id ? 'text-[#02c3b4] stroke-[2.5]' : 'text-[#8b999b] group-hover:text-white stroke-[2]'"
          />
          <span class="text-[13px] tracking-wide">{{ item.label }}</span>
        </div>

        <span
          v-if="ui.currentView === item.id"
          class="w-1.5 h-1.5 rounded-full bg-[#02c3b4] shadow-[0_0_8px_#02c3b4]"
        />
      </div>

      <!-- 分隔线与标题 -->
      <div class="pt-4 pb-1.5 px-3 flex items-center justify-between">
        <span class="text-[10px] font-mono font-bold text-[#5b696b] uppercase tracking-wider">本地算力扩展</span>
        <span class="text-[9px] font-mono text-[#02c3b4]/70 bg-[#02c3b4]/10 px-1.5 py-0.2 rounded">8 算子</span>
      </div>

      <!-- 扩展算子矩阵组 -->
      <div
        v-for="item in extensionOperators"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center justify-between px-3.5 py-2 rounded-2xl cursor-pointer transition-all border border-transparent text-[#6e7d80] hover:bg-white/[0.03] hover:text-[#c4d4d6] hover:border-white/[0.04] font-medium select-none group"
      >
        <div class="flex items-center gap-3">
          <component :is="item.icon" :size="15" class="text-[#526063] group-hover:text-[#8b999b] stroke-[2]" />
          <span class="text-[12.5px] tracking-wide">{{ item.label }}</span>
        </div>
        <span class="text-[9px] font-mono text-[#435154] group-hover:text-[#02c3b4] transition-colors">离线</span>
      </div>

    </div>

    <!-- 底部状态指示灯 -->
    <div class="px-5 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[11px] font-mono text-[#5b696b] pointer-events-auto" style="-webkit-app-region: no-drag;">
      <span class="flex items-center gap-1.5 text-[#8b999b]">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        RTX 3060 Ready
      </span>
      <span class="text-[10px] text-[#02c3b4]">v2.0</span>
    </div>
  </aside>
</template>
