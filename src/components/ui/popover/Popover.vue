<script setup lang="ts">
import { ref, provide } from 'vue'

const isOpen = ref(false)

const togglePopover = () => {
  isOpen.value = !isOpen.value
}

const closePopover = () => {
  isOpen.value = false
}

provide('popover', {
  isOpen,
  togglePopover,
  closePopover
})
</script>

<template>
  <div class="relative inline-block text-left">
    <slot />
  </div>
</template>
