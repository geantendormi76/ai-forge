<script setup lang="ts">
import { type Component } from 'vue'

interface 空状态属性 {
  title: string
  icon: Component
}

defineProps<空状态属性>()
</script>

<template>
  <div class="text-center space-y-3 pointer-events-none animate-in fade-in duration-300 p-4 flex flex-col items-center justify-center select-none">
    <!-- 🦾 图标优化：采用更精致的极细笔画，轻量级脉冲感 -->
    <component 
      :is="icon" 
      :size="22" 
      class="text-slate-400/50 mx-auto animate-pulse stroke-[1.2]" 
    />
    <!-- 🦾 字体美化：换装顶奢级无衬线字族，降低字重，收拢字距 -->
    <p class="empty-state-text text-[11px] text-center leading-relaxed whitespace-pre-line">
      {{ title }}
    </p>
  </div>
</template>

<style scoped>
.empty-state-text {
  /* 1. 换装主流顶奢、极其秀丽的无衬线字族，兼容 Windows / macOS / 移动端 */
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", "PingFang SC", "Microsoft YaHei", sans-serif;
  
  /* 2. 黄金字重控制：从 900 降归 600，杜绝粗笨与发虚，尽显锐利精致 */
  font-weight: 600;
  
  /* 3. 优雅的深板岩灰，配合轻微透光，提供极佳视觉对比度 */
  color: rgba(100, 116, 139, 0.85); /* slate-500 搭配 85% 不透明度 */
  
  /* 4. 字距微调：降维大间距，锁定 0.06em 呼吸感，最适合多行中文阅读 */
  letter-spacing: 0.06em;
}
</style>