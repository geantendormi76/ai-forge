<script setup lang="ts">
import { ref, computed } from 'vue';
import { useUIStore } from '../store/uiStore';
import Aurora from '../components/effects/Aurora.vue';
import DotField from '../components/effects/DotField.vue';
import GradientText from '../components/effects/GradientText.vue';
import {
  Zap,
  FileText,
  Film,
  Scan,
  Database,
  Search,
  ArrowUpRight,
  Sparkles,
  Music,
  Table,
  BookOpen,
  Image as ImageIcon,
} from 'lucide-vue-next';

const ui = useUIStore();

const searchQuery = ref('');
const activeCategory = ref('all');

interface ToolItem {
  id: string;
  name: string;
  category: 'core' | 'document' | 'media' | 'system';
  tag: string;
  icon: any;
  desc: string;
  badge?: string;
  viewTarget?: 'format' | 'pdf' | 'asr';
}

// 核心 9 大工具算子矩阵
const toolMatrix: ToolItem[] = [
  {
    id: 'asr',
    name: '视频字幕生成',
    category: 'media',
    tag: 'MOSS 0.9B + HY-MT2',
    icon: Film,
    desc: '端侧离线语音识别、混元神经翻译与 NVENC 极速压制',
    badge: '100% 本地',
    viewTarget: 'asr',
  },
  {
    id: 'pdf',
    name: 'PDF 智能混合解析',
    category: 'document',
    tag: 'VECTOR & GPU LAYOUT',
    icon: FileText,
    desc: 'CPU 矢量毫秒级提取 + GPU 多模态神经网络版面重构',
    badge: '毫秒级',
    viewTarget: 'pdf',
  },
  {
    id: 'format',
    name: '全能格式转换',
    category: 'core',
    tag: 'PURE RUST / 0 VRAM',
    icon: Zap,
    desc: '音频母带解密、表格清洗、电子书重排与 ICO 图标流式合成',
    badge: '主力算子',
    viewTarget: 'format',
  },
  {
    id: 'super_res',
    name: '8K 视觉超分辨率',
    category: 'media',
    tag: 'REAL-ESRGAN / ONNX',
    icon: Scan,
    desc: '基于 ONNX C-FFI 直推的图像超分重建与画质修复',
    badge: 'GPU 优化',
  },
  {
    id: 'knowledge',
    name: '本地离线知识库',
    category: 'core',
    tag: 'EMBEDDING / RAG',
    icon: Database,
    desc: '纯端侧向量数据库检索，文档隐私 100% 物理驻留',
  },
  {
    id: 'audio_master',
    name: '音频解密与母带',
    category: 'media',
    tag: 'NCM / QMC / KGMA',
    icon: Music,
    desc: '网易云、QQ 音乐等无损加密音频格式一键还原',
  },
  {
    id: 'table_clean',
    name: '多维表格清洗',
    category: 'document',
    tag: 'CSV / TSV / JSON',
    icon: Table,
    desc: '超大结构化表格数据流式转换与 JSON 树解析',
  },
  {
    id: 'ebook_gen',
    name: '电子书重排排版',
    category: 'document',
    tag: 'EPUB / MOBI / DOCX',
    icon: BookOpen,
    desc: 'Markdown 与纯文本自动转为标准出版级电子书',
  },
  {
    id: 'icon_craft',
    name: '原生多尺寸图标',
    category: 'core',
    tag: 'ICO / PNG / BMP',
    icon: ImageIcon,
    desc: '一键生成 Windows 标准应用图标与各分辨率位图',
  },
];

// 顶部右侧：常驻 3 个最常用核心算子
const pinnedQuickTools = computed(() => {
  return toolMatrix.slice(0, 3);
});

const categoryList = [
  { id: 'all', label: '全部工具', icon: Sparkles },
  { id: 'core', label: '核心中台', icon: Zap },
  { id: 'document', label: '文档解析', icon: FileText },
  { id: 'media', label: '音视频矩阵', icon: Film },
];

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return toolMatrix.filter((tool) => {
    const matchCategory = activeCategory.value === 'all' || tool.category === activeCategory.value;
    const matchSearch =
      !q ||
      tool.name.toLowerCase().includes(q) ||
      tool.desc.toLowerCase().includes(q) ||
      tool.tag.toLowerCase().includes(q);
    return matchCategory && matchSearch;
  });
});

const handleToolClick = (tool: ToolItem) => {
  if (tool.viewTarget) {
    ui.currentView = tool.viewTarget;
  } else {
    ui.弹出提示(`💡 [${tool.name}] 正在并网适配中，即将就绪...`, 'info');
  }
};
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans flex flex-col">
    
    <!-- 全屏极光背景 (优雅微光，不夺目) -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <Aurora
        :speed="0.8"
        :amplitude="1.2"
        :color-stops="['#101e21', '#02c3b4', '#00d2ff']"
        class="absolute inset-0 opacity-40"
      />
    </div>

    <!-- 全屏交互点阵力场 -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden">
      <DotField
        :dot-radius="1.2"
        :dot-spacing="16"
        :bulge-strength="55"
        :glow-radius="180"
        gradient-from="rgba(2, 195, 180, 0.35)"
        gradient-to="rgba(0, 210, 255, 0.12)"
        glow-color="#02c3b4"
        class="absolute inset-0 opacity-60"
      />
    </div>

    <!-- 主工作区滚动容器 -->
    <div
      class="relative z-10 w-full h-full overflow-y-auto custom-scrollbar pt-12 pb-6 transition-all duration-300 ease-in-out pointer-events-auto"
      :class="ui.侧边栏收起 ? 'px-8 sm:px-12 lg:px-14' : 'pl-[256px] pr-8 sm:pr-12 lg:pr-14'"
    >
      <div class="max-w-[1680px] w-full mx-auto space-y-6 flex flex-col justify-between">

        <!-- 🌟 上半区：紫电 AI 理念 (左) + 常用快捷工具 (右) -->
        <section class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch pt-2">
          
          <!-- 左翼 (5列 / 41.7%)：纯净品牌与理念 -->
          <div class="lg:col-span-5 flex flex-col justify-center space-y-3">
            
            <!-- 核心紫电 AI 渐变字标 -->
            <div>
              <GradientText
                text="紫电AI"
                :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
                :animation-speed="6"
                class="text-6xl sm:text-7xl font-black tracking-tight"
              />
            </div>

            <!-- 标语 -->
            <p class="text-slate-300 text-xs sm:text-sm leading-relaxed max-w-xl font-sans">
              <span class="block">不执着于单点技术的极致拔尖，而是通过全系统协同优化</span>
              <span class="block text-slate-100 mt-0.5">让 <span class="text-[#ffb74d] font-bold">廉价 AI + 极致框架</span> 创造最大化生产力</span>
            </p>
          </div>

          <!-- 右翼 (7列 / 58.3%)：常用快捷跳转卡片 -->
          <div class="lg:col-span-7 bg-[#182022]/85 border border-white/[0.08] rounded-3xl p-5 shadow-2xl backdrop-blur-2xl flex flex-col justify-between space-y-3">
            <div class="flex items-center justify-between pb-2 border-b border-white/[0.04]">
              <span class="text-xs font-mono font-bold text-[#02c3b4] tracking-wider flex items-center gap-1.5">
                <span>⚡ 常用快捷算子</span>
                <span class="text-[#5b696b] text-[11px] font-normal">（点击秒级直达主力工作台）</span>
              </span>
              <span class="text-[11px] font-mono text-[#5b696b]">3 项就绪</span>
            </div>

            <!-- 3 大常用主力卡片横向平铺 -->
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-3.5 flex-1 items-stretch">
              <div
                v-for="tool in pinnedQuickTools"
                :key="tool.id"
                @click="handleToolClick(tool)"
                class="group p-4 rounded-2xl border border-white/[0.06] bg-white/[0.02] hover:bg-white/[0.06] hover:border-[#02c3b4]/50 transition-all duration-200 cursor-pointer flex flex-col justify-between shadow-md active:scale-95"
              >
                <div class="flex items-start justify-between">
                  <div class="w-9 h-9 rounded-xl border border-white/10 bg-white/[0.04] flex items-center justify-center text-[#8b999b] group-hover:text-[#02c3b4] group-hover:border-[#02c3b4]/40 group-hover:bg-[#02c3b4]/10 transition-all">
                    <component :is="tool.icon" :size="17" class="stroke-[2]" />
                  </div>
                  <ArrowUpRight :size="14" class="text-[#5b696b] group-hover:text-[#02c3b4] transition-colors" />
                </div>

                <div class="mt-3">
                  <div class="flex items-center gap-1.5">
                    <h4 class="text-xs sm:text-sm font-bold text-[#f2f2ef] group-hover:text-white transition-colors truncate">{{ tool.name }}</h4>
                  </div>
                  <p class="text-[11px] text-[#8b999b] mt-1 line-clamp-2 leading-relaxed">{{ tool.desc }}</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- 🌟 下半区：全量工具导航矩阵与搜索分类 -->
        <section id="tool-explorer-section" class="space-y-4 pt-1">
          <div class="flex flex-col md:flex-row md:items-center justify-between gap-3 border-b border-white/[0.06] pb-3">
            <div>
              <span class="text-[11px] font-mono text-[#02c3b4] uppercase tracking-wider font-bold">02 / ALL CAPABILITIES</span>
              <h2 class="text-base sm:text-lg font-bold text-white mt-0.5">全能工具导航矩阵</h2>
            </div>

            <div class="flex items-center gap-3 flex-wrap">
              <div class="flex items-center gap-1.5">
                <button
                  v-for="cat in categoryList"
                  :key="cat.id"
                  type="button"
                  @click="activeCategory = cat.id"
                  class="h-7 px-3 rounded-xl text-xs font-bold flex items-center gap-1 transition-all cursor-pointer select-none"
                  :class="activeCategory === cat.id ? 'bg-[#02c3b4]/15 border border-[#02c3b4]/50 text-[#02c3b4] shadow-[0_0_12px_rgba(2,195,180,0.2)]' : 'bg-white/[0.02] border border-white/[0.06] text-[#8b999b] hover:text-white hover:bg-white/[0.04]'"
                >
                  <component :is="cat.icon" :size="12" />
                  <span>{{ cat.label }}</span>
                </button>
              </div>

              <div class="relative w-full sm:w-[240px]">
                <Search :size="13" class="absolute left-3 top-1/2 -translate-y-1/2 text-[#8b999b]" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索全库算子..."
                  class="w-full h-8 pl-8 pr-3 bg-white/[0.02] border border-white/[0.08] hover:border-white/[0.15] focus:border-[#02c3b4]/50 focus:ring-1 focus:ring-[#02c3b4]/30 rounded-xl text-xs text-[#f5f5f3] outline-none transition-all placeholder-[#5b696b] backdrop-blur-md"
                />
              </div>
            </div>
          </div>

          <div v-if="filteredTools.length > 0" class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-5 gap-3.5">
            <div
              v-for="tool in filteredTools"
              :key="tool.id"
              @click="handleToolClick(tool)"
              class="p-4 rounded-2xl border border-white/[0.06] bg-[#182022]/80 hover:bg-[#182022]/95 hover:border-[#02c3b4]/50 backdrop-blur-2xl transition-all duration-150 cursor-pointer flex flex-col justify-between min-h-[110px] group shadow-md"
            >
              <div class="flex items-center justify-between">
                <div class="w-7 h-7 rounded-lg border border-white/10 bg-white/[0.04] flex items-center justify-center text-[#8b999b] group-hover:text-[#02c3b4] group-hover:border-[#02c3b4]/30 transition-colors">
                  <component :is="tool.icon" :size="14" />
                </div>
                <span class="text-[9px] font-mono uppercase text-[#02c3b4] font-bold tracking-wider">{{ tool.tag }}</span>
              </div>

              <div class="mt-2.5">
                <div class="flex items-center gap-1.5">
                  <span class="text-xs font-bold text-[#f2f2ef] group-hover:text-white">{{ tool.name }}</span>
                  <span v-if="tool.badge" class="px-1.5 py-0.2 rounded text-[8px] font-mono bg-[#02c3b4]/15 text-[#02c3b4] font-bold border border-[#02c3b4]/30">
                    {{ tool.badge }}
                  </span>
                </div>
                <p class="text-[10.5px] text-[#8b999b] mt-0.5 line-clamp-1 leading-relaxed">{{ tool.desc }}</p>
              </div>
            </div>
          </div>

          <div v-else class="py-10 text-center text-[#8b999b] border border-white/[0.04] rounded-2xl bg-white/[0.01]">
            <p class="text-xs">未找到与 "{{ searchQuery }}" 匹配的工具算子</p>
          </div>
        </section>

      </div>
    </div>

  </div>
</template>
