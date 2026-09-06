<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import {
  getLocalGgufModels,
  switchGgufModel,
  type GgufModelInfo
} from '../../bindings/agent';
import { openPath } from '@tauri-apps/plugin-opener';
import {
  Cpu,
  Eye,
  FolderOpen,
  RotateCw,
  Check,
  Loader2,
  X,
  Sparkles,
  MessageSquareCode,
  HardDrive
} from 'lucide-vue-next';

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'switched', model: GgufModelInfo): void;
}>();

const models = ref<GgufModelInfo[]>([]);
const isLoading = ref(false);
const switchingId = ref<string | null>(null);
const isRefreshSuccess = ref(false);

const loadModels = async () => {
  isLoading.value = true;
  try {
    const list = await getLocalGgufModels();
    models.value = list;
  } catch (err) {
    console.error('获取 GGUF 模型列表失败:', err);
  } finally {
    isLoading.value = false;
  }
};

const handleRefresh = async () => {
  await loadModels();
  isRefreshSuccess.value = true;
  setTimeout(() => {
    isRefreshSuccess.value = false;
  }, 1200);
};

const openModelsFolder = async () => {
  try {
    await openPath('C:\\dev\\ai-forge\\models');
  } catch (err) {
    console.error('打开模型目录失败:', err);
  }
};

const handleSwitch = async (m: GgufModelInfo) => {
  if (m.is_active || switchingId.value) return;
  switchingId.value = m.id;
  try {
    const updated = await switchGgufModel(m.id);
    await loadModels();
    emit('switched', updated);
  } catch (err) {
    console.error('切换模型失败:', err);
  } finally {
    switchingId.value = null;
  }
};

watch(() => props.isOpen, (open) => {
  if (open) {
    loadModels();
  }
});

onMounted(() => {
  if (props.isOpen) {
    loadModels();
  }
});
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60 backdrop-blur-xs p-4 select-none font-sans"
    @click.self="emit('close')"
  >
    <div
      class="w-full max-w-[620px] rounded-2xl bg-[#fcfbf9] dark:bg-[#1c1a17] border border-[#e4e1da] dark:border-[#33302a] shadow-2xl flex flex-col overflow-hidden max-h-[85vh] animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- ==================== 1. 顶栏控制器 ==================== -->
      <header class="px-5 py-4 border-b border-[#e4e1da] dark:border-[#33302a] flex items-center justify-between bg-white/70 dark:bg-[#26231f]/70 backdrop-blur-md shrink-0">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-xl bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 flex items-center justify-center text-[#1764e8]">
            <Cpu :size="16" />
          </div>
          <div>
            <h3 class="text-sm font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono tracking-tight flex items-center gap-2">
              <span>本地 GGUF 大模型热拔插中枢</span>
              <span class="text-[10px] font-mono bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/40 px-2 py-0.2 rounded-full">
                12GB 显存安全锁
              </span>
            </h3>
            <p class="text-xs text-[#746f66] dark:text-[#8f8a81] mt-0.5">
              纯血端侧推理 · 秒级释放与动态挂载 · 支持多模态眼球自动配对
            </p>
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <!-- 刷新目录 -->
          <button
            type="button"
            @click="handleRefresh"
            class="w-8 h-8 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] text-[#57534a] dark:text-[#a19d92] flex items-center justify-center transition-all cursor-pointer"
            title="重新扫描 models 目录"
          >
            <Check v-if="isRefreshSuccess" :size="14" class="text-emerald-600 stroke-[2.5]" />
            <RotateCw v-else :size="14" :class="{ 'animate-spin': isLoading }" />
          </button>
          <!-- 打开模型文件夹 -->
          <button
            type="button"
            @click="openModelsFolder"
            class="h-8 px-2.5 rounded-lg border border-[#e4e1da] dark:border-[#33302a] bg-[#f0eeea] dark:bg-[#26231f] hover:bg-[#e4e1da] text-[#57534a] dark:text-[#a19d92] flex items-center gap-1 text-xs font-mono font-medium transition-all cursor-pointer"
            title="在文件资源管理器中打开 models 文件夹"
          >
            <FolderOpen :size="13" class="text-[#1764e8]" />
            <span>打开目录</span>
          </button>
          <!-- 关闭模态框 -->
          <button
            type="button"
            @click="emit('close')"
            class="w-8 h-8 rounded-lg border border-transparent hover:bg-[#f0eeea] dark:hover:bg-[#26231f] text-[#746f66] hover:text-[#1c1a17] dark:hover:text-[#faf9f7] flex items-center justify-center transition-all cursor-pointer"
          >
            <X :size="15" />
          </button>
        </div>
      </header>

      <!-- ==================== 2. 模型资产列表 ==================== -->
      <main class="p-5 overflow-y-auto custom-scrollbar space-y-3 bg-[#f7f6f3]/60 dark:bg-[#141210]/60 flex-1">
        <div v-if="isLoading && models.length === 0" class="py-12 flex flex-col items-center justify-center space-y-2 text-[#746f66] dark:text-[#8f8a81]">
          <Loader2 :size="24" class="animate-spin text-[#1764e8]" />
          <span class="text-xs font-mono">正在扫描 models 目录下的 GGUF 资产...</span>
        </div>

        <div v-else-if="models.length === 0" class="py-12 flex flex-col items-center justify-center space-y-2 text-center text-[#746f66] dark:text-[#8f8a81]">
          <HardDrive :size="28" class="text-[#746f66]/60" />
          <p class="text-xs font-mono">未在 C:\dev\ai-forge\models 目录下检测到可用主脑模型 (.gguf)</p>
          <button
            type="button"
            @click="openModelsFolder"
            class="mt-2 px-3 py-1.5 rounded-lg bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] text-[#1764e8] text-xs font-mono font-bold cursor-pointer"
          >
            放入模型文件后点击刷新
          </button>
        </div>

        <div
          v-for="m in models"
          :key="m.id"
          class="p-4 rounded-xl border transition-all duration-150 flex flex-col gap-3"
          :class="[
            m.is_active
              ? 'bg-white dark:bg-[#1c1a17] border-[#1764e8] shadow-sm'
              : 'bg-white/80 dark:bg-[#1c1a17]/80 hover:bg-white border-[#e4e1da] dark:border-[#33302a]'
          ]"
        >
          <!-- 卡片顶栏：模型名称与状态标签 -->
          <div class="flex items-start justify-between gap-2">
            <div class="space-y-1 min-w-0 flex-1">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-xs font-bold text-[#1c1a17] dark:text-[#faf9f7] font-mono truncate">
                  {{ m.id }}
                </span>
                <!-- 激活态胶囊 -->
                <span
                  v-if="m.is_active"
                  class="px-2 py-0.2 rounded-full bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800/40 text-[10px] font-mono font-bold flex items-center gap-1"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                  <span>当前主脑运行中</span>
                </span>
              </div>
              <div class="flex items-center gap-2 text-[11px] font-mono text-[#746f66] dark:text-[#8f8a81]">
                <span>{{ m.size_formatted }}</span>
                <span>•</span>
                <span class="truncate max-w-[280px]">{{ m.file_name }}</span>
              </div>
            </div>

            <!-- 右侧模态能力徽标 -->
            <div class="shrink-0 flex items-center gap-1.5">
              <span
                v-if="m.supports_vision"
                class="px-2 py-0.5 rounded-full bg-[#edf4ff] dark:bg-[#1e293b] border border-[#cce0ff] dark:border-[#3b82f6]/40 text-[#1764e8] dark:text-[#60a5fa] text-[10.5px] font-mono font-semibold flex items-center gap-1"
              >
                <Eye :size="11" />
                <span>原生视觉 VLM</span>
              </span>
              <span
                v-else
                class="px-2 py-0.5 rounded-full bg-[#fbeee0] dark:bg-[#3a2a1f] border border-[#f0dac5] dark:border-[#523e2e] text-[#8b5e2c] dark:text-[#e8b07a] text-[10.5px] font-mono font-semibold flex items-center gap-1"
              >
                <MessageSquareCode :size="11" />
                <span>纯文本语言</span>
              </span>
            </div>
          </div>

          <!-- 视觉眼球配对详情 (如有) -->
          <div
            v-if="m.mmproj_path"
            class="px-2.5 py-1.5 rounded-lg bg-[#f0eeea]/60 dark:bg-[#26231f]/60 border border-[#e4e1da]/60 dark:border-[#33302a] text-[10.5px] font-mono text-[#57534a] dark:text-[#a19d92] flex items-center gap-2 truncate"
          >
            <Sparkles :size="12" class="text-amber-500 shrink-0" />
            <span class="shrink-0 font-bold">已挂载眼球:</span>
            <span class="truncate">{{ m.mmproj_path.split('\\').pop() }}</span>
          </div>

          <!-- 操作底栏 -->
          <div class="flex items-center justify-between pt-1 border-t border-[#f0eeea] dark:border-[#26231f] text-xs font-mono">
            <span class="text-[10.5px] text-[#746f66] dark:text-[#8f8a81]">
              {{ m.is_active ? '已常驻本地 8000 端口' : '点击将此权重热重载为主脑' }}
            </span>

            <button
              v-if="!m.is_active"
              type="button"
              :disabled="switchingId === m.id"
              @click="handleSwitch(m)"
              class="h-7 px-3 rounded-lg bg-[#1c1a17] dark:bg-[#faf9f7] text-white dark:text-[#1c1a17] font-bold text-xs flex items-center gap-1.5 shadow-xs hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
            >
              <Loader2 v-if="switchingId === m.id" :size="12" class="animate-spin" />
              <span>{{ switchingId === m.id ? '热重载与显存切换中...' : '一键激活此模型' }}</span>
            </button>
            <div v-else class="text-emerald-600 dark:text-emerald-400 font-bold text-xs flex items-center gap-1">
              <Check :size="13" />
              <span>当前在役</span>
            </div>
          </div>
        </div>
      </main>

      <!-- ==================== 3. 底栏提示 ==================== -->
      <footer class="p-3.5 px-5 bg-white dark:bg-[#1c1a17] border-t border-[#e4e1da] dark:border-[#33302a] text-[11px] font-mono text-[#746f66] dark:text-[#8f8a81] flex items-center justify-between">
        <span>切换模型后将自动释放旧显存，无需重启客户端</span>
        <span class="text-[#1764e8] font-bold">零数据上传 · 100% 本地</span>
      </footer>
    </div>
  </div>
</template>
