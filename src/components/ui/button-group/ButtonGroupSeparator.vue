<script setup lang="ts">
interface ButtonGroupSeparatorProps {
  orientation?: 'horizontal' | 'vertical'
  class?: string
}

const props = withDefaults(defineProps<ButtonGroupSeparatorProps>(), {
  orientation: 'vertical'
})
</script>

<template>
  <div
    :class="[
      'bg-slate-200 shrink-0 self-center',
      orientation === 'vertical' ? 'h-5 w-[1px] mx-1' : 'h-[1px] w-5 my-1',
      props.class
    ]"
  />
</template>