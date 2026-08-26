<script setup lang="ts">
import { MessageSquareHeart, X } from 'lucide-vue-next';
import qrcodeImg from '../../assets/qrcode.png';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
}>();

const handleClose = () => {
  emit('update:visible', false);
};
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center p-6 bg-black/75 backdrop-blur-md animate-in fade-in duration-150 select-none pointer-events-auto"
  >
    <div class="max-w-[380px] w-full p-7 rounded-[32px] bg-[#161e20] border border-white/15 shadow-[0_25px_60px_rgba(0,0,0,0.8)] text-white space-y-5 animate-in zoom-in-95 duration-150 relative text-center flex flex-col items-center">
      <!-- 右上角关闭按钮 -->
      <button
        type="button"
        @click="handleClose"
        class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-[#8b999b] hover:text-white flex items-center justify-center absolute top-5 right-5 transition-colors cursor-pointer"
      >
        <X :size="16" />
      </button>

      <!-- 弹窗头部标题 -->
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-2xl bg-gradient-to-tr from-rose-500 to-[#bc05ff] flex items-center justify-center shadow-[0_0_16px_rgba(244,63,94,0.35)]">
          <MessageSquareHeart :size="20" class="text-white fill-white/20" />
        </div>
        <div class="text-left">
          <h3 class="text-base font-black text-white">
            联系作者
          </h3>
          <p class="text-xs text-[#8b999b]">微信扫码直接对话与技术支持</p>
        </div>
      </div>

      <!-- 二维码卡片展示区 -->
      <div class="p-4 rounded-2xl bg-white/[0.04] border border-white/10 flex flex-col items-center space-y-3 w-full">
        <div class="w-[200px] h-[200px] rounded-2xl bg-white p-2 flex items-center justify-center shadow-lg overflow-hidden">
          <img
            :src="qrcodeImg"
            alt="微信二维码"
            class="w-full h-full object-contain select-none"
          />
        </div>
        <p class="text-xs text-[#c4d4d6] leading-relaxed">
          微信扫一扫上方二维码<br />
          交流使用体验、反馈 Bug 或交流算子
        </p>
      </div>

      <!-- 底部确定关闭按钮 -->
      <button
        type="button"
        @click="handleClose"
        class="w-full py-3 rounded-2xl bg-gradient-to-r from-[#02c3b4] to-[#00d2ff] hover:opacity-95 text-slate-950 font-black text-sm transition-all active:scale-95 cursor-pointer shadow-lg"
      >
        确定
      </button>
    </div>
  </div>
</template>
