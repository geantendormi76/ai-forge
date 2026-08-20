<!-- ui/src/components/ui/native-select/NativeSelectOption.vue -->
<script setup lang="ts">
defineProps<{
  value: any
  disabled?: boolean
  class?: string
}>()
</script>

<template>
  <option
    :value="value"
    :disabled="disabled"
    :class="[
      'text-[11px] font-bold text-slate-700 bg-white disabled:text-slate-300',
      $props.class
    ]"
  >
    <slot />
  </option>
</template>