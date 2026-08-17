<script setup lang="ts">
import { useUIStore } from '../../store/uiStore'
import ShinyText from '../effects/ShinyText.vue'
import {
  Zap, FileText, Film,
  Scan, Database, PanelLeftClose,
  LayoutDashboard
} from 'lucide-vue-next'

const ui = useUIStore()

// 核心工坊主力算子
const menuGroup1 = [
  { id: 'home', label: '工坊主页', icon: LayoutDashboard },
  { id: 'format', label: '全能格式转换', icon: Zap },
  { id: 'pdf', label: 'PDF 智能解析', icon: FileText },
  { id: 'asr', label: '视频双语字幕', icon: Film },
]

// 扩展算子与模型
const menuGroup2 = [
  { id: 'super_res', label: '8K 视觉超分', icon: Scan },
  { id: 'knowledge', label: '本地离线知识库', icon: Database },
]

const switchView = (id: string) => {
  if (['home', 'format', 'pdf', 'asr'].includes(id)) {
    ui.currentView = id as 'home' | 'format' | 'pdf' | 'asr'
  } else {
    ui.弹出提示("💡 该 AI 模块正在并网建设中...", "info")
  }
}
</script>

<template>
  <aside
    class="h-[calc(100vh-32px)] my-4 bg-[#0e0e10]/80 backdrop-blur-2xl rounded-[24px] shadow-[0_16px_48px_rgba(0,0,0,0.5)] flex flex-col pt-6 pb-4 flex-shrink-0 z-20 transition-all duration-300 ease-in-out select-none border border-white/[0.08]"
    :class="ui.侧边栏收起 ? 'w-0 opacity-0 ml-0 mr-0 my-0 py-0 px-0 overflow-hidden pointer-events-none' : 'w-[220px] ml-4'"
    style="-webkit-app-region: drag;"
  >
    <!-- 头部品牌标识与收起按钮 -->
    <div class="px-5 mb-6 flex items-center justify-between pointer-events-auto select-none" style="-webkit-app-region: no-drag;">
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
        @click="ui.切换侧边栏"
        class="w-7 h-7 rounded-lg text-[#8b8b87] hover:text-white hover:bg-white/[0.08] flex items-center justify-center transition-colors cursor-pointer shrink-0"
        title="收起侧边栏"
      >
        <PanelLeftClose :size="15" class="stroke-[2]" />
      </button>
    </div>

    <!-- 菜单滚动区 -->
    <div class="flex-1 overflow-y-auto custom-scrollbar px-3 space-y-1.5 pointer-events-auto" style="-webkit-app-region: no-drag;">
      
      <!-- 主力工坊组 -->
      <div
        v-for="item in menuGroup1"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center gap-3 px-3.5 py-2.5 rounded-xl cursor-pointer transition-all duration-150 border"
        :class="[
          ui.currentView === item.id
            ? 'bg-white/[0.08] border-white/15 text-white font-bold shadow-[0_4px_16px_rgba(0,0,0,0.3)]'
            : 'border-transparent text-[#8b8b87] hover:bg-white/[0.04] hover:text-[#f2f2ef] font-medium'
        ]"
      >
        <component
          :is="item.icon"
          :size="16"
          :class="ui.currentView === item.id ? 'text-[#DCA54C] stroke-[2.5]' : 'text-[#8b8b87] stroke-[2]'"
        />
        <span class="text-[13px] tracking-wide">{{ item.label }}</span>
      </div>

      <!-- 分组标头 -->
      <div class="pt-5 pb-1.5 px-3">
        <span class="text-[10px] font-mono font-bold text-[#5b5b58] uppercase tracking-wider">本地算力扩展</span>
      </div>

      <!-- 扩展算子组 -->
      <div
        v-for="item in menuGroup2"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center justify-between px-3.5 py-2.5 rounded-xl cursor-pointer transition-colors border border-transparent text-[#8b8b87] hover:bg-white/[0.04] hover:text-[#f2f2ef] font-medium"
      >
        <div class="flex items-center gap-3">
          <component :is="item.icon" :size="16" class="text-[#5b5b58] stroke-[2]" />
          <span class="text-[13px] tracking-wide">{{ item.label }}</span>
        </div>
      </div>

    </div>

    <!-- 底部状态指示灯 -->
    <div class="px-5 pt-3 border-t border-white/[0.06] flex items-center justify-between text-[11px] font-mono text-[#5b5b58] pointer-events-auto" style="-webkit-app-region: no-drag;">
      <span class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
        RTX 3060 Ready
      </span>
      <span class="text-[10px]">v2.0</span>
    </div>
  </aside>
</template>
