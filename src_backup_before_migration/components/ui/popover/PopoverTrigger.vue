<!-- ui/src/components/ui/popover/PopoverTrigger.vue -->
<script setup lang="ts">
import { inject, onMounted, useTemplateRef } from 'vue'

const { toggle, triggerEl } = inject('popover') as any
const triggerRef = useTemplateRef<HTMLElement>('triggerRef')

onMounted(() => {
  if (triggerRef.value) {
    // 🚀 获取真实触发按钮的 DOM 实例以用于坐标运算
    triggerEl.value = (triggerRef.value.children[0] as HTMLElement) || triggerRef.value
  }
})
</script>

<template>
  <div ref="triggerRef" @click.stop="toggle" class="inline-block cursor-pointer">
    <slot />
  </div>
</template>