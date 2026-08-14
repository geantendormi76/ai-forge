<script setup lang="ts">
import { useUIStore } from '../../store/uiStore'
import ShinyText from '../effects/ShinyText.vue'
import Button from '../ui/button/Button.vue'
import {
  Zap, FileText, Film,
  Scan, Database, PanelLeftClose
} from 'lucide-vue-next'

const ui = useUIStore()

// 核心工坊菜单组
const menuGroup1 = [
  { id: 'format', label: '全能格式转换', icon: Zap },
  { id: 'pdf', label: 'PDF 智能解析', icon: FileText },
  { id: 'asr', label: '视频双语字幕', icon: Film },
]

// 扩展工具菜单组
const menuGroup2 = [
  { id: 'super_res', label: '8K 视觉超分', icon: Scan },
  { id: 'knowledge', label: '本地知识库', icon: Database },
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
    class="h-[calc(100vh-32px)] my-4 bg-white rounded-[24px] shadow-[0_4px_24px_rgba(0,0,0,0.02)] flex flex-col pt-7 pb-4 flex-shrink-0 z-20 transition-all duration-300 ease-in-out select-none border border-slate-100/80"
    :class="ui.侧边栏收起 ? 'w-0 opacity-0 ml-0 mr-0 my-0 py-0 px-0 overflow-hidden pointer-events-none' : 'w-[220px] ml-4'"
    style="-webkit-app-region: drag;"
  >
    <!-- 头部品牌标识与收起按钮 -->
    <div class="px-6 mb-5 flex items-center justify-between pointer-events-auto select-none" style="-webkit-app-region: no-drag;">
      <div
        @click="ui.currentView = 'home'"
        class="cursor-pointer hover:opacity-80 active:scale-95 transition-all"
        title="返回紫电主页"
      >
        <ShinyText
          text="紫电 AI"
          :speed="3.5"
          :spread="150"
          color="var(--zidian-purple)"
          shine-color="var(--zidian-gold)"
          class="text-[20px] font-black tracking-tight"
        />
      </div>
      <Button
        @click="ui.切换侧边栏"
        variant="ghost"
        size="icon-sm"
        class="text-slate-400 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all cursor-pointer shrink-0"
        title="收起侧边栏"
      >
        <PanelLeftClose :size="16" class="stroke-[2]" />
      </Button>
    </div>

    <!-- 菜单滚动区 -->
    <div class="flex-1 overflow-y-auto scrollbar-hide px-2 space-y-1 pointer-events-auto" style="-webkit-app-region: no-drag;">
      <div
        v-for="item in menuGroup1"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center gap-3 px-4 py-2.5 rounded-2xl cursor-pointer transition-all duration-150"
        :class="ui.currentView === item.id ? 'bg-[#F4F4F5] text-slate-900 font-bold shadow-xs' : 'text-slate-600 hover:bg-slate-50 hover:text-slate-900 font-medium'"
      >
        <component
          :is="item.icon"
          :size="16"
          :class="ui.currentView === item.id ? 'text-[#DCA54C] stroke-[2.5]' : 'text-slate-500 stroke-[2]'"
        />
        <span class="text-[13px]">{{ item.label }}</span>
      </div>

      <div class="pt-5 pb-2 px-4">
        <span class="text-[11px] font-bold text-slate-400">本地算力扩展</span>
      </div>

      <div
        v-for="item in menuGroup2"
        :key="item.id"
        @click="switchView(item.id)"
        class="flex items-center justify-between px-4 py-2.5 rounded-2xl cursor-pointer transition-colors text-slate-600 hover:bg-slate-50 hover:text-slate-900 font-medium"
      >
        <div class="flex items-center gap-3">
          <component :is="item.icon" :size="16" class="text-slate-400 stroke-[2]" />
          <span class="text-[13px]">{{ item.label }}</span>
        </div>
      </div>
    </div>
  </aside>
</template>
