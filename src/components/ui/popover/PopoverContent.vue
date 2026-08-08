<script setup lang="ts">
import { inject, onUnmounted, ref, watch, nextTick } from 'vue'

const popover = inject<{
  isOpen: { value: boolean }
  closePopover: () => void
}>('popover')

const contentRef = ref<HTMLElement | null>(null)

const handleClickOutside = (event: MouseEvent) => {
  if (contentRef.value && !contentRef.value.contains(event.target as Node)) {
    popover?.closePopover()
  }
}

watch(() => popover?.isOpen.value, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      document.addEventListener('click', handleClickOutside)
    })
  } else {
    document.removeEventListener('click', handleClickOutside)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    v-if="popover?.isOpen.value"
    ref="contentRef"
    class="absolute right-0 z-50 mt-2 w-56 rounded-xl bg-white p-2 text-slate-800 shadow-lg ring-1 ring-slate-900/5 focus:outline-none"
  >
    <slot />
  </div>
</template>
