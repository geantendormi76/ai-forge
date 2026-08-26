<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useUIStore } from '../store/uiStore';
import DotField from '../components/effects/DotField.vue';
import GradientText from '../components/effects/GradientText.vue';
import {
  runTranslation,
  runImageTranslation,
} from '../bindings/tools/translation';
import {
  Copy,
  Check,
  Sparkles,
  Trash2,
  ArrowRightLeft,
  Loader2,
  ArrowLeft,
  Image as ImageIcon,
  X,
  Search,
  ChevronDown,
  ChevronUp,
  Globe2
} from 'lucide-vue-next';

const ui = useUIStore();

// 1. 核心状态
const sourceText = ref('');
const translatedText = ref('');
const isProcessing = ref(false);
const isCopied = ref(false);
const elapsedMs = ref<number | null>(null);

// 2. 图像附带状态
const attachedImageBase64 = ref<string | null>(null);
const attachedImageName = ref<string | null>(null);

// 3. 目标语种状态与全量 38 语种多维注册表
const targetLang = ref('Chinese');
const isLangDrawerOpen = ref(false);
const langSearchQuery = ref('');

interface LangItem {
  name: string;
  value: string;
  category: 'east_asia' | 'europe' | 'asia_south' | 'mideast_eurasia';
}

const all38Languages: LangItem[] = [
  // 1. 东亚与中华方言
  { name: '简体中文', value: 'Chinese', category: 'east_asia' },
  { name: '英语', value: 'English', category: 'east_asia' },
  { name: '日语', value: 'Japanese', category: 'east_asia' },
  { name: '韩语', value: 'Korean', category: 'east_asia' },
  { name: '繁体中文', value: 'Traditional Chinese', category: 'east_asia' },
  { name: '粤语', value: 'Cantonese', category: 'east_asia' },
  { name: '藏语', value: 'Tibetan', category: 'east_asia' },
  { name: '蒙古语', value: 'Mongolian', category: 'east_asia' },
  { name: '维吾尔语', value: 'Uyghur', category: 'east_asia' },

  // 2. 欧美主流
  { name: '法语', value: 'French', category: 'europe' },
  { name: '德语', value: 'German', category: 'europe' },
  { name: '西班牙语', value: 'Spanish', category: 'europe' },
  { name: '葡萄牙语', value: 'Portuguese', category: 'europe' },
  { name: '意大利语', value: 'Italian', category: 'europe' },
  { name: '俄语', value: 'Russian', category: 'europe' },
  { name: '荷兰语', value: 'Dutch', category: 'europe' },
  { name: '波兰语', value: 'Polish', category: 'europe' },
  { name: '捷克语', value: 'Czech', category: 'europe' },
  { name: '乌克兰语', value: 'Ukrainian', category: 'europe' },

  // 3. 东南亚与南亚
  { name: '越南语', value: 'Vietnamese', category: 'asia_south' },
  { name: '马来语', value: 'Malay', category: 'asia_south' },
  { name: '印尼语', value: 'Indonesian', category: 'asia_south' },
  { name: '菲律宾语', value: 'Filipino', category: 'asia_south' },
  { name: '泰语', value: 'Thai', category: 'asia_south' },
  { name: '高棉语', value: 'Khmer', category: 'asia_south' },
  { name: '缅甸语', value: 'Burmese', category: 'asia_south' },
  { name: '印地语', value: 'Hindi', category: 'asia_south' },
  { name: '孟加拉语', value: 'Bengali', category: 'asia_south' },
  { name: '泰米尔语', value: 'Tamil', category: 'asia_south' },
  { name: '泰卢固语', value: 'Telugu', category: 'asia_south' },
  { name: '马拉地语', value: 'Marathi', category: 'asia_south' },
  { name: '古吉拉特语', value: 'Gujarati', category: 'asia_south' },
  { name: '乌尔都语', value: 'Urdu', category: 'asia_south' },

  // 4. 中东与欧亚
  { name: '阿拉伯语', value: 'Arabic', category: 'mideast_eurasia' },
  { name: '波斯语', value: 'Persian', category: 'mideast_eurasia' },
  { name: '希伯来语', value: 'Hebrew', category: 'mideast_eurasia' },
  { name: '土耳其语', value: 'Turkish', category: 'mideast_eurasia' },
  { name: '哈萨克语', value: 'Kazakh', category: 'mideast_eurasia' },
];

// 高频常用 6 大置顶卡槽
const topQuickLanguages = [
  { name: '简体中文', value: 'Chinese' },
  { name: '英语', value: 'English' },
  { name: '日语', value: 'Japanese' },
  { name: '韩语', value: 'Korean' },
  { name: '繁体中文', value: 'Traditional Chinese' },
  { name: '粤语', value: 'Cantonese' },
];

// 语种分类标签
const categoryLabels: Record<string, string> = {
  east_asia: '🏮 东亚与中华方言',
  europe: '🏰 欧美主流语种',
  asia_south: '🌴 东南亚与南亚',
  mideast_eurasia: '🕌 中东与欧亚大区',
};

// 过滤后的全量语种
const filteredLanguagesByCategory = computed(() => {
  const q = langSearchQuery.value.trim().toLowerCase();
  const list = all38Languages.filter((l) => {
    return (
      !q ||
      l.name.toLowerCase().includes(q) ||
      l.value.toLowerCase().includes(q)
    );
  });

  const groups: Record<string, LangItem[]> = {
    east_asia: [],
    europe: [],
    asia_south: [],
    mideast_eurasia: [],
  };

  for (const item of list) {
    if (groups[item.category]) {
      groups[item.category].push(item);
    }
  }

  return groups;
});

// 获取当前选中语言的友好显示名称
const currentLangLabel = computed(() => {
  const found = all38Languages.find((l) => l.value === targetLang.value);
  return found ? found.name : targetLang.value;
});

const selectLanguage = (val: string) => {
  targetLang.value = val;
  isLangDrawerOpen.value = false;
};

// 4. 剪贴板监听：支持直接按 Ctrl+V 粘贴截图
const handlePaste = (e: ClipboardEvent) => {
  const items = e.clipboardData?.items;
  if (!items) return;

  for (const item of items) {
    if (item.type.startsWith('image/')) {
      e.preventDefault();
      const file = item.getAsFile();
      if (file) {
        const reader = new FileReader();
        reader.onload = (event) => {
          attachedImageBase64.value = event.target?.result as string;
          attachedImageName.value = '剪贴板截屏图像';
          sourceText.value = '';
          ui.弹出提示('📷 已捕获剪贴板图像，按回车 [Enter] 即刻自动 OCR 识图直翻', 'info');
        };
        reader.readAsDataURL(file);
        return;
      }
    }
  }
};

// 5. 拖拽图像支持
const handleDrop = (e: DragEvent) => {
  e.preventDefault();
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = (event) => {
        attachedImageBase64.value = event.target?.result as string;
        attachedImageName.value = file.name;
        sourceText.value = '';
        ui.弹出提示(`📷 已载入图像 [${file.name}]，按回车 [Enter] 自动 OCR 识别`, 'info');
      };
      reader.readAsDataURL(file);
    }
  }
};

const removeAttachedImage = () => {
  attachedImageBase64.value = null;
  attachedImageName.value = null;
};

// 6. 统一执行翻译流水线
const handleTranslate = async () => {
  if (isProcessing.value) return;

  // 场景 A: 图像 OCR 识图翻译
  if (attachedImageBase64.value) {
    isProcessing.value = true;
    translatedText.value = '';
    elapsedMs.value = null;

    try {
      const res = await runImageTranslation({
        image_base64: attachedImageBase64.value,
        target_lang: targetLang.value,
      });

      if (res.success) {
        translatedText.value = res.full_translated_text;
        elapsedMs.value = res.elapsed_ms;
        ui.弹出提示(`✅ OCR 识别与翻译完成 (耗时 ${res.elapsed_ms} ms)`, 'success');
      } else {
        ui.弹出提示(res.error || '图像识别翻译失败', 'error');
      }
    } catch (err: any) {
      ui.弹出提示(`图像处理异常: ${err}`, 'error');
    } finally {
      isProcessing.value = false;
    }
    return;
  }

  // 场景 B: 纯文本高精直翻
  const textToTranslate = sourceText.value.trim();
  if (!textToTranslate) {
    ui.弹出提示('请输入待翻译的文本或按 Ctrl+V 粘贴截图', 'info');
    return;
  }

  isProcessing.value = true;
  translatedText.value = '';
  elapsedMs.value = null;

  try {
    const res = await runTranslation({
      texts: [textToTranslate],
      target_lang: targetLang.value,
      style: null,
      output_file_path: null,
    });

    if (res.success && res.translations.length > 0) {
      translatedText.value = res.translations.join('\n\n');
      elapsedMs.value = res.elapsed_ms;
      ui.弹出提示(`✅ 翻译完成 (耗时 ${res.elapsed_ms} ms)`, 'success');
    } else {
      ui.弹出提示(res.error || '翻译未返回有效结果', 'error');
    }
  } catch (err: any) {
    ui.弹出提示(`翻译异常: ${err}`, 'error');
  } finally {
    isProcessing.value = false;
  }
};

const handleClear = () => {
  sourceText.value = '';
  attachedImageBase64.value = null;
  attachedImageName.value = null;
  translatedText.value = '';
  elapsedMs.value = null;
};

const copyResult = async () => {
  if (!translatedText.value) return;
  try {
    await navigator.clipboard.writeText(translatedText.value);
    isCopied.value = true;
    ui.弹出提示('✅ 译文已成功复制到剪贴板', 'success');
    setTimeout(() => {
      isCopied.value = false;
    }, 2000);
  } catch {
    ui.弹出提示('复制失败，请手动选取文字', 'error');
  }
};

const swapLanguage = () => {
  targetLang.value = targetLang.value === 'Chinese' ? 'English' : 'Chinese';
};

// 7. 输入框键盘交互：Enter 直接翻译，Shift + Enter 换行
const handleTextareaKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
    e.preventDefault();
    handleTranslate();
  }
};

const handleGlobalKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && !ui.toast显示) {
    if (isLangDrawerOpen.value) {
      isLangDrawerOpen.value = false;
    } else {
      ui.currentView = 'home';
    }
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown);
});
</script>

<template>
  <div class="relative w-full h-full overflow-hidden bg-[#20292b] text-[#f5f5f3] select-none font-sans flex flex-col">
    <!-- 1. 背景流光点阵 (全局统一青色) -->
    <div class="absolute inset-0 pointer-events-none z-0 overflow-hidden opacity-30">
      <DotField
        :dot-radius="1.2"
        :dot-spacing="16"
        :bulge-strength="45"
        :glow-radius="180"
        gradient-from="rgba(2, 195, 180, 0.35)"
        gradient-to="rgba(0, 210, 255, 0.12)"
        glow-color="#02c3b4"
      />
    </div>

    <!-- 2. 主视窗容器 -->
    <div
      class="relative z-10 flex-1 w-full overflow-y-auto custom-scrollbar pt-12 pb-8 transition-all duration-300 ease-in-out"
      :class="ui.侧边栏收起 ? 'pl-[104px] pr-6 sm:pr-10' : 'pl-[276px] pr-6 sm:pr-10 lg:pr-14'"
    >
      <div class="max-w-[1380px] w-full mx-auto space-y-4 pt-1 flex flex-col justify-between min-h-[calc(100vh-100px)]">
        
        <!-- 顶栏：返回与标题 -->
        <header class="relative flex items-center justify-between border-b border-white/[0.06] pb-3.5 pt-1">
          <button
            type="button"
            @click="ui.currentView = 'home'"
            class="h-8 px-3.5 rounded-xl bg-white/[0.03] hover:bg-[#02c3b4]/10 border border-white/10 hover:border-[#02c3b4]/50 text-[#8b999b] hover:text-[#02c3b4] transition-all duration-200 flex items-center gap-2 text-xs font-bold shadow-md active:scale-95 cursor-pointer group hover:shadow-[0_0_12px_rgba(2,195,180,0.2)]"
            title="返回工坊主页 (Esc)"
          >
            <ArrowLeft :size="14" class="group-hover:-translate-x-0.5 transition-transform text-[#02c3b4]" />
            <span>返回主页</span>
          </button>

          <div class="absolute left-1/2 -translate-x-1/2 pointer-events-none">
            <GradientText
              text="离线高精翻译"
              :colors="['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8']"
              :animation-speed="6"
              class="text-2xl sm:text-3xl font-black tracking-tight text-center"
            />
          </div>

          <div
            @click="ui.currentView = 'home'"
            class="flex items-center gap-1.5 text-[11px] font-mono text-[#5b696b] hover:text-[#02c3b4] cursor-pointer transition-colors select-none"
            title="点击或按 ESC 返回主页"
          >
            <kbd class="px-1.5 py-0.5 rounded-md bg-white/[0.04] border border-white/10 text-[#8b999b] text-[10px] font-bold">ESC</kbd>
            <span class="hidden sm:inline">快捷返回</span>
          </div>
        </header>

        <!-- 核心工作区：上输入 · 下输出 极简大双仓 -->
        <main class="space-y-4 flex-1 flex flex-col justify-between">
          
          <!-- 仓 1：上输入区域 (多模态感知输入) -->
          <div
            class="flex-1 flex flex-col justify-between bg-white/[0.02] border border-white/[0.08] hover:border-white/[0.15] rounded-3xl p-5 sm:p-6 shadow-xl backdrop-blur-xl space-y-3 transition-all min-h-[210px]"
            @drop="handleDrop"
            @dragover.prevent
          >
            <!-- 头部信息 -->
            <div class="flex items-center justify-between border-b border-white/[0.04] pb-3">
              <div class="flex items-center gap-2.5">
                <span class="text-sm sm:text-base font-bold text-white tracking-wide flex items-center gap-1.5">
                  <span>📥 输入内容</span>
                </span>
                <span class="text-xs sm:text-[13px] text-[#02c3b4] font-bold font-mono tracking-wide hidden sm:inline">
                  (支持直接打字、粘贴文字、拖入图片或 Ctrl+V 粘贴截图)
                </span>
              </div>

              <button
                v-if="sourceText || attachedImageBase64"
                type="button"
                @click="handleClear"
                class="text-xs font-medium text-[#8b999b] hover:text-rose-400 cursor-pointer flex items-center gap-1.5 transition-colors"
                title="清空输入"
              >
                <Trash2 :size="13" />
                <span>清空</span>
              </button>
            </div>

            <!-- 图像预览胶囊 (如果粘贴了图像) -->
            <div v-if="attachedImageBase64" class="p-3.5 rounded-2xl bg-black/40 border border-[#02c3b4]/40 flex items-center justify-between gap-3 animate-in fade-in duration-150">
              <div class="flex items-center gap-3.5 min-w-0">
                <div class="w-12 h-12 rounded-xl overflow-hidden bg-white/5 border border-white/10 shrink-0">
                  <img :src="attachedImageBase64" class="w-full h-full object-cover" alt="Attached" />
                </div>
                <div class="space-y-0.5 min-w-0">
                  <span class="text-sm font-bold text-white flex items-center gap-1.5 truncate">
                    <ImageIcon :size="14" class="text-[#02c3b4]" />
                    <span>{{ attachedImageName || '已捕获待识别图像' }}</span>
                  </span>
                  <p class="text-xs text-[#a0b0b2]">已启用 PP-OCRv6 纯血离线视觉识别，回车即可直翻</p>
                </div>
              </div>
              <button
                type="button"
                @click="removeAttachedImage"
                class="w-7 h-7 rounded-lg bg-white/5 hover:bg-white/15 text-[#8b999b] hover:text-white flex items-center justify-center cursor-pointer transition-colors"
                title="移除图片"
              >
                <X :size="14" />
              </button>
            </div>

            <!-- 文本输入框 -->
            <textarea
              v-else
              v-model="sourceText"
              @paste="handlePaste"
              @keydown="handleTextareaKeyDown"
              placeholder="在此键入待翻译文本 (按 Enter 即可直接翻译，Shift+Enter 换行)，或直接按 Ctrl+V 粘贴任何截图..."
              class="w-full flex-1 min-h-[110px] bg-transparent text-sm sm:text-[14.5px] text-[#f5f5f3] placeholder-[#5b696b] outline-none resize-none custom-scrollbar font-sans leading-relaxed select-text"
            ></textarea>

            <!-- 底部状态条与即刻翻译按钮 (全局统一青色发光) -->
            <div class="flex items-center justify-between pt-2.5 border-t border-white/[0.04]">
              <div class="text-xs font-mono text-[#8b999b]">
                <span v-if="attachedImageBase64" class="text-[#02c3b4] font-bold">📷 图像 OCR 模式</span>
                <span v-else>{{ sourceText.length }} 字符</span>
              </div>

              <button
                type="button"
                :disabled="isProcessing || (!sourceText.trim() && !attachedImageBase64)"
                @click="handleTranslate"
                class="px-8 py-3 rounded-2xl font-bold text-xs sm:text-[13px] transition-all duration-200 flex items-center gap-2 cursor-pointer shadow-xl active:scale-95"
                :class="[
                  isProcessing || (!sourceText.trim() && !attachedImageBase64)
                    ? 'bg-white/[0.03] text-[#5b696b] cursor-not-allowed border border-white/[0.04]'
                    : 'bg-[#02c3b4]/15 hover:bg-[#02c3b4]/25 text-[#02c3b4] border border-[#02c3b4]/50 shadow-[0_0_24px_rgba(2,195,180,0.25)] hover:scale-[1.02]'
                ]"
              >
                <Loader2 v-if="isProcessing" :size="15" class="animate-spin" />
                <Sparkles v-else :size="15" />
                <span>{{ isProcessing ? '极速推导中...' : '即刻翻译 (Enter)' }}</span>
              </button>
            </div>
          </div>

          <!-- 仓 2：下输出区域 (译文呈现) -->
          <div class="flex-1 flex flex-col justify-between bg-white/[0.02] border border-white/[0.08] rounded-3xl p-5 sm:p-6 shadow-xl backdrop-blur-xl space-y-3 min-h-[210px]">
            <!-- 头部 -->
            <div class="flex items-center justify-between border-b border-white/[0.04] pb-3">
              <div class="flex items-center gap-2.5">
                <span class="text-sm sm:text-base font-bold text-white tracking-wide flex items-center gap-1.5">
                  <span>✨ 目标译文</span>
                </span>
                <span class="text-xs sm:text-[13px] text-[#02c3b4] font-bold font-mono tracking-wide bg-[#02c3b4]/10 px-2.5 py-0.5 rounded-lg border border-[#02c3b4]/20">
                  {{ currentLangLabel }}
                </span>
              </div>

              <div class="flex items-center gap-3">
                <span v-if="elapsedMs !== null" class="text-xs font-mono text-emerald-400 font-bold bg-emerald-500/10 px-2.5 py-0.5 rounded-lg border border-emerald-500/20">
                  ⚡ {{ elapsedMs }} ms
                </span>
                <button
                  type="button"
                  @click="copyResult"
                  v-if="translatedText"
                  class="text-xs sm:text-[13px] text-[#02c3b4] hover:text-white cursor-pointer flex items-center gap-1.5 font-bold transition-colors"
                  title="复制完整译文"
                >
                  <Check v-if="isCopied" :size="14" class="stroke-[3]" />
                  <Copy v-else :size="14" />
                  <span>{{ isCopied ? '已复制' : '一键复制' }}</span>
                </button>
              </div>
            </div>

            <!-- 译文正文 -->
            <div class="w-full flex-1 min-h-[110px] overflow-y-auto custom-scrollbar text-sm sm:text-[14.5px] text-[#f5f5f3] leading-relaxed select-text font-sans p-1">
              <p v-if="translatedText" class="whitespace-pre-wrap select-text leading-relaxed text-[#f5f5f3]">{{ translatedText }}</p>
              <div v-else-if="isProcessing" class="h-full flex items-center justify-center text-[#8b999b] gap-2.5 py-8">
                <Loader2 :size="18" class="animate-spin text-[#02c3b4]" />
                <span class="text-sm font-medium">Hy-MT2 神经引擎正在进行高精翻译...</span>
              </div>
              <div v-else class="h-full flex items-center justify-center text-[#5b696b] text-sm py-8 font-medium">
                <span>高精译文将在此实时生成</span>
              </div>
            </div>

            <!-- 底部字数 -->
            <div class="text-xs text-[#8b999b] pt-2 border-t border-white/[0.04] text-right font-mono">
              <span>{{ translatedText.length }} 字符</span>
            </div>
          </div>

          <!-- 仓 3：极简纯净语种卡槽选择仓 (全量青色强调与微光反馈) -->
          <div class="p-4 rounded-3xl bg-white/[0.02] border border-white/[0.08] shadow-2xl backdrop-blur-2xl space-y-3 relative">
            
            <!-- 上层卡槽导航：置顶卡片 + 展开全量入口 -->
            <div class="flex flex-col lg:flex-row items-center justify-between gap-3">
              <div class="flex items-center gap-2 w-full lg:w-auto justify-between lg:justify-start">
                <div class="flex items-center gap-2 text-sm font-bold text-white shrink-0">
                  <Globe2 :size="16" class="text-[#02c3b4]" />
                  <span>目标语种</span>
                </div>
                
                <!-- 中英对调按钮 (统配青色悬浮微光) -->
                <button
                  type="button"
                  @click="swapLanguage"
                  class="h-8 px-3 rounded-xl bg-white/[0.04] hover:bg-[#02c3b4]/10 border border-white/10 hover:border-[#02c3b4]/50 text-xs font-bold text-[#8b999b] hover:text-[#02c3b4] flex items-center gap-1.5 cursor-pointer transition-all duration-200 shadow-sm active:scale-95 hover:shadow-[0_0_12px_rgba(2,195,180,0.2)]"
                  title="中英快速对调"
                >
                  <ArrowRightLeft :size="12" />
                  <span>中英对调</span>
                </button>
              </div>

              <!-- 6 大常用单行卡槽 (统配青色主交互与悬浮微发光) -->
              <div class="grid grid-cols-3 sm:grid-cols-6 gap-2 w-full lg:flex-1 max-w-[780px]">
                <button
                  v-for="item in topQuickLanguages"
                  :key="item.value"
                  type="button"
                  @click="selectLanguage(item.value)"
                  class="h-9 px-3.5 rounded-2xl border text-center transition-all duration-200 cursor-pointer flex items-center justify-center text-xs sm:text-[13px] font-bold truncate active:scale-95"
                  :class="[
                    targetLang === item.value
                      ? 'bg-[#02c3b4]/20 border-[#02c3b4] text-white shadow-[0_0_16px_rgba(2,195,180,0.35)] scale-[1.02]'
                      : 'bg-white/[0.02] border-white/[0.08] text-[#8b999b] hover:bg-[#02c3b4]/10 hover:border-[#02c3b4]/50 hover:text-[#02c3b4] hover:shadow-[0_0_12px_rgba(2,195,180,0.2)]'
                  ]"
                >
                  {{ item.name }}
                </button>
              </div>

              <!-- 展开 38 语种大仓按钮 (统配青色微光) -->
              <button
                type="button"
                @click="isLangDrawerOpen = !isLangDrawerOpen"
                class="h-9 px-4 rounded-2xl border flex items-center justify-center gap-1.5 text-xs sm:text-[13px] font-bold transition-all duration-200 cursor-pointer shrink-0 shadow-md active:scale-95 w-full lg:w-auto"
                :class="[
                  isLangDrawerOpen
                    ? 'bg-[#02c3b4]/20 border-[#02c3b4] text-[#02c3b4] shadow-[0_0_16px_rgba(2,195,180,0.3)]'
                    : 'bg-white/[0.04] hover:bg-[#02c3b4]/10 border-white/10 hover:border-[#02c3b4]/50 text-white hover:text-[#02c3b4] hover:shadow-[0_0_12px_rgba(2,195,180,0.2)]'
                ]"
              >
                <span>全部语种</span>
                <ChevronUp v-if="isLangDrawerOpen" :size="14" />
                <ChevronDown v-else :size="14" />
              </button>
            </div>

            <!-- 下层折叠抽屉：全量 38 语种按大洲分类矩阵与搜索 -->
            <div
              v-if="isLangDrawerOpen"
              class="pt-3.5 border-t border-white/[0.06] space-y-3.5 animate-in fade-in zoom-in-95 duration-150"
            >
              <!-- 语种搜索栏 -->
              <div class="flex items-center justify-between gap-3">
                <div class="relative flex-1 max-w-[340px]">
                  <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-[#8b999b]" />
                  <input
                    v-model="langSearchQuery"
                    type="text"
                    placeholder="输入语种搜索 (如: 俄语, 德语, 泰语)..."
                    class="w-full h-8 pl-8 pr-3 bg-black/40 border border-white/10 focus:border-[#02c3b4]/50 rounded-xl text-xs text-white placeholder-[#5b696b] outline-none"
                  />
                </div>
                <span class="text-xs font-mono text-[#8b999b]">
                  当前已选: <strong class="text-[#02c3b4] font-bold">{{ currentLangLabel }}</strong>
                </span>
              </div>

              <!-- 分类网格 (全量统配青色悬浮与激活) -->
              <div class="space-y-3 max-h-[280px] overflow-y-auto custom-scrollbar pr-1">
                <div
                  v-for="(groupList, catKey) in filteredLanguagesByCategory"
                  :key="catKey"
                  v-show="groupList.length > 0"
                  class="space-y-2"
                >
                  <div class="text-xs font-bold text-[#8b999b] flex items-center gap-1.5 pb-1 border-b border-white/[0.03]">
                    <span>{{ categoryLabels[catKey] }}</span>
                    <span class="text-[11px] font-mono text-[#5b696b]">({{ groupList.length }})</span>
                  </div>

                  <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2">
                    <button
                      v-for="item in groupList"
                      :key="item.value"
                      type="button"
                      @click="selectLanguage(item.value)"
                      class="h-8 px-3 rounded-xl border text-center transition-all duration-200 cursor-pointer flex items-center justify-between text-xs font-bold group active:scale-95"
                      :class="[
                        targetLang === item.value
                          ? 'bg-[#02c3b4]/25 border-[#02c3b4] text-white shadow-[0_0_14px_rgba(2,195,180,0.35)]'
                          : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-[#02c3b4]/10 hover:border-[#02c3b4]/50 hover:text-[#02c3b4] hover:shadow-[0_0_10px_rgba(2,195,180,0.15)]'
                      ]"
                    >
                      <span class="truncate">{{ item.name }}</span>
                      <Check v-if="targetLang === item.value" :size="12" class="text-[#02c3b4] stroke-[3] shrink-0" />
                    </button>
                  </div>
                </div>
              </div>
            </div>

          </div>

        </main>
      </div>
    </div>
  </div>
</template>
