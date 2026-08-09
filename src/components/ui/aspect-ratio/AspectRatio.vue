<script setup lang="ts">
import { computed } from 'vue'

interface 宽高比属性 {
  ratio?: number
}

const props = withDefaults(defineProps<宽高比属性>(), {
  ratio: 16 / 9,
})

const 动态比例样式 = computed(() => {
  return {
    aspectRatio: String(props.ratio)
  }
})
</script>

<template>
  <!-- 利用现代浏览器原生 CSS 属性，实现 0 毫秒布局计算 -->
  <div :style="动态比例样式" class="w-full overflow-hidden relative">
    <slot />
  </div>
</template>