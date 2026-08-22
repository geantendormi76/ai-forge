<script setup lang="ts">
import { useUIStore } from '../../store/uiStore';
import { Flame, Clock, X } from 'lucide-vue-next';

const ui = useUIStore();

// 格式化数据库中的工具英文字段为规范中文标签 (1:1 保持原版定义)
const formatToolName = (name: string) => {
  const map: Record<string, string> = {
    'upscale-48k': '🔍 4K/8K 图像超分',
    'tool-upscale-48k': '🔍 4K/8K 图像超分',
    'service-upscale': '🔍 4K/8K 图像超分',
    'video-subtitle': '🎬 视频字幕生成',
    'tool-ASR': '🎬 视频字幕生成',
    'service-asr': '🎬 视频字幕生成',
    'pdf-parse': '📄 PDF 智能解析',
    'tool-pdf-parse': '📄 PDF 智能解析',
    'format-converter': '⚡ 全能格式转换',
    'tool-format-convert': '⚡ 全能格式转换',
  };
  return map[name] || name;
};
</script>

<template>
  <!-- ==================== 模态弹窗 HTML 模板 (1:1 图一原版) ==================== -->
  <div
    v-if="ui.showQuotaModal"
    class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none pointer-events-auto"
  >
    <div class="max-w-[440px] w-full p-7 rounded-[32px] bg-[#161e20] border border-white/15 shadow-[0_25px_60px_rgba(0,0,0,0.8)] text-white space-y-5 animate-in zoom-in-95 duration-150 relative">
      
      <!-- 右上角关闭按钮 -->
      <button
        type="button"
        @click="ui.showQuotaModal = false"
        class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-[#8b999b] hover:text-white flex items-center justify-center absolute top-6 right-6 transition-colors cursor-pointer"
      >
        <X :size="16" />
      </button>

      <!-- 弹窗头部标题 -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-2xl bg-gradient-to-tr from-[#bc05ff] to-[#02c3b4] flex items-center justify-center shadow-[0_0_16px_rgba(2,195,180,0.35)]">
          <Flame :size="20" class="text-white fill-white" />
        </div>
        <div>
          <h3 class="text-base font-black text-white">
            Tokens 算力中心
          </h3>
        </div>
      </div>

      <!-- Tokens 核心资产大卡片 -->
      <div class="p-5 rounded-2xl bg-white/[0.03] border border-white/10 space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-xs text-[#8b999b] font-medium">可用 Tokens 余额</span>
          <span class="text-2xl font-black font-mono text-[#02c3b4]">
            {{ ui.quota.remaining_points }} <span class="text-xs text-[#8b999b] font-normal">Tokens</span>
          </span>
        </div>

        <!-- 动态渐变进度条 -->
        <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
          <div
            class="h-full bg-gradient-to-r from-[#02c3b4] to-[#ffb74d] rounded-full transition-all duration-500"
            :style="{ width: `${Math.min(100, Math.max(5, (ui.quota.remaining_points / (ui.quota.daily_limit || 1)) * 100))}%` }"
          />
        </div>

        <div class="grid grid-cols-2 gap-2 text-xs font-mono pt-1 text-[#8b999b]">
          <div>每日基础额度: <strong class="text-white">{{ ui.quota.daily_limit }} Tokens</strong></div>
          <div>今日已消耗: <strong class="text-white">{{ ui.quota.used_today }} Tokens</strong></div>
        </div>
      </div>

      <!-- 实时扣费明细流水 (直连 Cloudflare D1) -->
      <div class="space-y-2.5">
        <div class="text-xs font-bold text-[#8b999b] px-1 flex items-center gap-1.5">
          <Clock :size="13" />
          <span>今日任务消费账单</span>
        </div>

        <div v-if="ui.quota.recent_logs && ui.quota.recent_logs.length > 0" class="space-y-2 max-h-[190px] overflow-y-auto custom-scrollbar pr-0.5">
          <div
            v-for="log in ui.quota.recent_logs"
            :key="log.id"
            class="p-3 rounded-2xl bg-white/[0.03] hover:bg-white/[0.05] border border-white/[0.05] flex items-center justify-between text-xs transition-all"
          >
            <div class="flex items-center gap-2.5">
              <span class="font-bold text-slate-200">{{ formatToolName(log.tool_name) }}</span>
              <span class="text-[10px] font-mono text-[#5b696b]">{{ log.created_at.split(' ')[1] || log.created_at }}</span>
            </div>
            <strong class="font-mono text-rose-400 font-bold text-xs bg-rose-500/10 px-2 py-0.5 rounded-lg border border-rose-500/20">
              -{{ log.points_deducted }} Tokens
            </strong>
          </div>
        </div>

        <div v-else class="py-6 text-center text-xs text-[#5b696b] bg-white/[0.01] rounded-2xl border border-white/[0.03]">
          今日暂无扣费记录，开始你的第一个任务吧！
        </div>
      </div>

      <!-- 底部确定关闭按钮 -->
      <button
        type="button"
        @click="ui.showQuotaModal = false"
        class="w-full py-3 rounded-2xl bg-white hover:bg-slate-100 text-black font-bold text-sm transition-all active:scale-95 cursor-pointer shadow-lg mt-2"
      >
        确定
      </button>

    </div>
  </div>
</template>
