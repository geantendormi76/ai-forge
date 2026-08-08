<!-- ui/src/components/ui/native-select/NativeSelectOptGroup.vue -->
<script setup lang="ts">
defineProps<{
  label: string
  disabled?: boolean
  class?: string
}>()
</script>

<template>
  <optgroup
    :label="label"
    :disabled="disabled"
    :class="[
      'text-[10px] font-black text-slate-400 uppercase tracking-widest bg-slate-50',
      $props.class
    ]"
  >
    <slot />
  </optgroup>
</template>