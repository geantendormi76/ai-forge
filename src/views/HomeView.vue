<script setup lang="ts">
import { ref, computed } from 'vue'
import { useUIStore } from '../store/uiStore'
import Aurora from '../components/effects/Aurora.vue'
import DotField from '../components/effects/DotField.vue'
import GradientText from '../components/effects/GradientText.vue'
import {
  Zap, FileText, Film, Scan, Database, Search,
  ArrowRight, ArrowUpRight, Sparkles,
  ArrowUp, Music, Table, BookOpen, Image as ImageIcon
} from 'lucide-vue-next'

const ui = useUIStore()

const searchQuery = ref('')
const activeCategory = ref('all')
const workspaceRef = ref<HTMLDivElement | null>(null)
const showBackToTop = ref(false)

// 核心工具算子矩阵定义
interface ToolItem {
  id: string
  name: string
  category: 'core' | 'document' | 'media' | 'system'
  tag: string
  icon: any
  desc: string
  badge?: string
  viewTarget?: 'format' | 'pdf' | 'asr'
}

const toolMatrix: ToolItem[] = [
  {
    id: 'format',
    name: '全能格式转换',
    category: 'core',
    tag: 'PURE RUST / 0 VRAM',
    icon: Zap,
    desc: '音频母带解密、表格清洗、电子书重排与 ICO 图标流式合成',
    badge: '主力算子',
    viewTarget: 'format'
  },
  {
    id: 'pdf',
    name: 'PDF 智能混合解析',
    category: 'document',
    tag: 'VECTOR & GPU LAYOUT',
    icon: FileText,
    desc: 'CPU 矢量毫秒级提取 + GPU 多模态神经网络版面重构',
    badge: '毫秒级',
    viewTarget: 'pdf'
  },
  {
    id: 'asr',
    name: '视频双语字幕工坊',
    category: 'media',
    tag: 'MOSS 0.9B + HY-MT2',
    icon: Film,
    desc: '端侧离线语音识别、混元神经翻译与 NVENC 极速压制',
    badge: '100% 本地',
    viewTarget: 'asr'
  },
  {
    id: 'super_res',
    name: '8K 视觉超分辨率',
    category: 'media',
    tag: 'REAL-ESRGAN / ONNX',
    icon: Scan,
    desc: '基于 ONNX C-FFI 直推的图像超分重建与画质修复',
    badge: 'GPU 优化'
  },
  {
    id: 'knowledge',
    name: '本地离线知识库',
    category: 'core',
    tag: 'EMBEDDING / RAG',
    icon: Database,
    desc: '纯端侧向量数据库检索，文档隐私 100% 物理驻留'
  },
  {
    id: 'audio_master',
    name: '音频解密与母带',
    category: 'media',
    tag: 'NCM / QMC / KGMA',
    icon: Music,
    desc: '网易云、QQ 音乐等无损加密音频格式一键还原'
  },
  {
    id: 'table_clean',
    name: '多维表格清洗',
    category: 'document',
    tag: 'CSV / TSV / JSON',
    icon: Table,
    desc: '超大结构化表格数据流式转换与 JSON 树解析'
  },
  {
    id: 'ebook_gen',
    name: '电子书重排排版',
    category: 'document',
    tag: 'EPUB / MOBI / DOCX',
    icon: BookOpen,
    desc: 'Markdown 与纯文本自动转为标准出版级电子书'
  },
  {
    id: 'icon_craft',
    name: '原生多尺寸图标',
    category: 'core',
    tag: 'ICO / PNG / BMP',
    icon: ImageIcon,
    desc: '一键生成 Windows 标准应用图标与各分辨率位图'
  }
]

const favoriteTools = computed(() => {
  return toolMatrix.slice(0, 6)
})

const categoryList = [
  { id: 'all', label: '全部工具', icon: Sparkles },
  { id: 'core', label: '核心中台', icon: Zap },
  { id: 'document', label: '文档解析', icon: FileText },
  { id: 'media', label: '音视频矩阵', icon: Film },
]

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return toolMatrix.filter(tool => {
    const matchCategory = activeCategory.value === 'all' || tool.category === activeCategory.value
    const matchSearch = !q || tool.name.toLowerCase().includes(q) || tool.desc.toLowerCase().includes(q) || tool.tag.toLowerCase().includes(q)
    return matchCategory && matchSearch
  })
})

const handleToolClick = (tool: ToolItem) => {
  if (tool.viewTarget) {
    ui.currentView = tool.viewTarget
  } else {
    ui.弹出提示(`💡 [${tool.name}] 正在并网适配中，即将就绪...`, 'info')
  }
}

const handleScroll = (e: Event) => {
  const target = e.target as HTMLElement
  showBackToTop.value = target.scrollTop > 300
}

const scrollToTop = () => {
  workspaceRef.value?.scrollTo({ top: 0, behavior: 'smooth' })
}

const scrollToExplorer = () => {
  const el = document.getElementById('tool-explorer-section')
  if (el && workspaceRef.value) {
    workspaceRef.value.scrollTo({ top: el.offsetTop - 20, behavior: 'smooth' })
  }
}
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans">
    
    <!-- 背景层 1：全息极光光子云层 (Aurora) -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <Aurora
        :speed="0.8"
        :amplitude="1.2"
        :color-stops="['#101e21', '#00ffa9', '#00d2ff']"
        class="absolute inset-0 opacity-60"
      />
    </div>

    <!-- 背景层 2：高精度交互点阵力场 (DotField) -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <DotField
        :dot-radius="1.5"
        :dot-spacing="14"
        :bulge-strength="67"
        :glow-radius="200"
        gradient-from="rgba(0, 255, 169, 0.45)"
        gradient-to="rgba(0, 210, 255, 0.35)"
        glow-color="#00ffa9"
        class="absolute inset-0 opacity-70"
      />
    </div>

    <!-- 主工作区滚动容器 -->
    <div
      ref="workspaceRef"
      @scroll="handleScroll"
      class="relative z-10 w-full h-full overflow-y-auto custom-scrollbar px-8 pb-16"
    >
      <div class="max-w-[1380px] mx-auto space-y-14 pt-2">

        <!-- 1. HERO 标杆展示区 -->
        <section class="space-y-6 pt-6 max-w-4xl">
          
          <!-- 胶囊状态徽章 -->
          <div
            @click="scrollToExplorer"
            class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#182022]/80 border border-white/10 text-xs font-mono text-slate-300 hover:border-[#00ffa9]/50 transition-colors shadow-lg backdrop-blur-md group w-fit cursor-pointer"
          >
            <span class="px-2 py-0.5 rounded-full bg-[#00ffa9] text-slate-950 font-extrabold text-[11px] tracking-wide">
              SOTA 2026
            </span>
            <span class="flex items-center gap-1.5 font-medium">
              RTX 3060 12G 算力直连
              <ArrowRight class="w-3.5 h-3.5 text-[#00ffa9] group-hover:translate-x-0.5 transition-transform" />
            </span>
          </div>

          <!-- 核心紫电 AI 渐变字标 -->
          <div class="space-y-2">
            <GradientText
              text="紫电AI"
              :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
              :animation-speed="6"
              class="text-6xl sm:text-8xl font-black tracking-tight"
            />
          </div>

          <!-- 标语 -->
          <p class="text-slate-300 text-sm sm:text-base leading-relaxed max-w-2xl font-sans space-y-1">
            <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
            <span class="block text-slate-100">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
          </p>

          <!-- 行动按钮 -->
          <div class="flex flex-wrap items-center gap-4 pt-2">
            <button
              @click="scrollToExplorer"
              class="px-6 py-3.5 rounded-2xl bg-gradient-to-r from-[#A5F3FC] to-[#D8B4F8] hover:opacity-90 text-slate-950 font-extrabold text-sm tracking-wide transition-all shadow-lg shadow-purple-500/20 hover:scale-[1.02] active:scale-95 flex items-center gap-2 cursor-pointer"
            >
              <span>探索 AI 算力工具</span>
              <ArrowRight class="w-4 h-4" />
            </button>

            <button
              @click="ui.currentView = 'pdf'"
              class="px-6 py-3.5 rounded-2xl bg-[#182022]/90 hover:bg-[#182022] text-slate-200 border border-white/10 font-semibold text-sm transition-all flex items-center gap-2 backdrop-blur-md active:scale-95 cursor-pointer hover:border-[#ffb74d]/50"
            >
              <Sparkles class="w-4 h-4 text-[#ffb74d]" />
              <span>体验 PDF 解析</span>
            </button>
          </div>

          <!-- 底层算力标签 -->
          <div class="flex items-center gap-6 pt-3 text-[11px] font-mono text-[#8b999b] uppercase">
            <span class="flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              MSVC Native x64
            </span>
            <span class="flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              Direct C-FFI
            </span>
            <span class="flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
              100% Private Offline
            </span>
          </div>
        </section>

        <!-- 2. 我的收藏 -->
        <section class="space-y-4 pt-2">
          <div class="flex items-end justify-between border-b border-white/[0.08] pb-3">
            <div>
              <span class="text-[11px] font-mono text-[#8b999b] uppercase tracking-wider">01 / YOUR WORKBENCH</span>
              <h2 class="text-xl font-bold text-white mt-0.5">我的收藏算子</h2>
            </div>
            <span class="text-xs text-[#8b999b] font-mono">Pinned Tools</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
              v-for="tool in favoriteTools"
              :key="tool.id"
              @click="handleToolClick(tool)"
              class="group p-5 rounded-2xl border border-white/[0.08] bg-[#182022]/60 hover:bg-[#182022]/90 hover:border-white/20 backdrop-blur-md transition-all duration-200 cursor-pointer flex flex-col justify-between min-h-[140px] relative shadow-lg"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="w-10 h-10 rounded-xl border border-white/10 bg-white/[0.03] flex items-center justify-center text-white group-hover:text-[#00ffa9] group-hover:border-[#00ffa9]/40 transition-colors">
                  <component :is="tool.icon" :size="18" class="stroke-[2]" />
                </div>
                <div class="flex items-center gap-1 text-[#8b999b] group-hover:text-white transition-colors">
                  <span class="text-[10px] font-mono tracking-wider uppercase">{{ tool.tag }}</span>
                  <ArrowUpRight :size="14" />
                </div>
              </div>

              <div class="mt-4">
                <h4 class="text-sm font-bold text-[#f2f2ef] group-hover:text-white transition-colors">{{ tool.name }}</h4>
                <p class="text-xs text-[#8b999b] mt-1 line-clamp-2 leading-relaxed">{{ tool.desc }}</p>
              </div>
            </div>
          </div>
        </section>

        <!-- 3. 搜索与全库矩阵 -->
        <section id="tool-explorer-section" class="space-y-6 pt-2">
          <div class="flex flex-col md:flex-row md:items-end justify-between gap-4 border-b border-white/[0.08] pb-4">
            <div>
              <span class="text-[11px] font-mono text-[#8b999b] uppercase tracking-wider">02 / TOOL EXPLORER</span>
              <h2 class="text-xl font-bold text-white mt-0.5">搜索与探索全部工具</h2>
            </div>

            <!-- 搜索框 -->
            <div class="relative w-full md:w-[360px]">
              <Search :size="15" class="absolute left-3.5 top-1/2 -translate-y-1/2 text-[#8b999b]" />
              <input
                v-model="searchQuery"
                type="text"
                placeholder="搜索格式转换、PDF、字幕、超分、表格..."
                class="w-full h-10 pl-9 pr-4 bg-[#182022]/60 border border-white/10 hover:border-white/20 focus:border-[#00ffa9]/50 rounded-xl text-xs text-[#f2f2ef] outline-none transition-all placeholder-[#5b696b] backdrop-blur-md"
              />
            </div>
          </div>

          <!-- 分类胶囊 -->
          <div class="flex items-center gap-2 flex-wrap">
            <button
              v-for="cat in categoryList"
              :key="cat.id"
              @click="activeCategory = cat.id"
              class="h-8 px-4 rounded-full text-xs font-medium flex items-center gap-1.5 transition-all cursor-pointer"
              :class="activeCategory === cat.id ? 'bg-[#f2f2ef] text-[#0a0a0a] font-bold shadow-sm' : 'bg-[#182022]/50 border border-white/[0.08] text-[#8b999b] hover:text-white hover:bg-white/[0.06]'"
            >
              <component :is="cat.icon" :size="13" />
              <span>{{ cat.label }}</span>
            </button>
          </div>

          <!-- 工具卡片栅格流 -->
          <div v-if="filteredTools.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div
              v-for="tool in filteredTools"
              :key="tool.id"
              @click="handleToolClick(tool)"
              class="p-5 rounded-2xl border border-white/[0.06] bg-[#182022]/50 hover:bg-[#182022]/80 hover:border-white/20 backdrop-blur-md transition-all duration-150 cursor-pointer flex flex-col justify-between min-h-[130px] group shadow-md"
            >
              <div class="flex items-center justify-between">
                <div class="w-8 h-8 rounded-lg border border-white/10 bg-white/[0.02] flex items-center justify-center text-[#8b999b] group-hover:text-white group-hover:border-white/25 transition-colors">
                  <component :is="tool.icon" :size="15" />
                </div>
                <span class="text-[9px] font-mono uppercase text-[#5b696b] tracking-wider">{{ tool.tag }}</span>
              </div>

              <div class="mt-3">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-bold text-[#f2f2ef] group-hover:text-white">{{ tool.name }}</span>
                  <span v-if="tool.badge" class="px-1.5 py-0.2 rounded text-[9px] font-mono bg-white/10 text-[#00ffa9]">
                    {{ tool.badge }}
                  </span>
                </div>
                <p class="text-[11px] text-[#8b999b] mt-1 line-clamp-2 leading-relaxed">{{ tool.desc }}</p>
              </div>
            </div>
          </div>

          <!-- 空搜索结果 -->
          <div v-else class="py-16 text-center text-[#8b999b] border border-white/[0.04] rounded-2xl bg-[#182022]/30">
            <p class="text-xs">未找到与 "{{ searchQuery }}" 匹配的工具算子</p>
          </div>
        </section>

        <!-- 底部落款 -->
        <footer class="flex justify-between items-center text-[11px] font-mono text-[#5b696b] border-t border-white/[0.06] pt-8 pb-4">
          <span>ZIDIAN AI DESKTOP 2026 / HIGH PERFORMANCE WORKBENCH</span>
          <span>ALL COMPUTATION STAYS ON YOUR DEVICE</span>
        </footer>

      </div>
    </div>

    <!-- 回到顶部悬浮球 -->
    <button
      v-if="showBackToTop"
      @click="scrollToTop"
      class="fixed right-8 bottom-8 z-50 w-10 h-10 rounded-full bg-[#f2f2ef] hover:bg-white text-[#0a0a0a] shadow-2xl flex items-center justify-center transition-all duration-200 active:scale-95 cursor-pointer"
      title="回到顶部"
    >
      <ArrowUp :size="16" class="stroke-[2.5]" />
    </button>

  </div>
</template>
