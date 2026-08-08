<script setup lang="ts">
interface ButtonGroupProps {
  orientation?: 'horizontal' | 'vertical'
  class?: string
}

const props = withDefaults(defineProps<ButtonGroupProps>(), {
  orientation: 'horizontal'
})
</script>

<template>
  <div
    role="group"
    :class="[
      'inline-flex items-center bg-white border border-slate-200 rounded-xl overflow-hidden shadow-sm shrink-0',
      orientation === 'horizontal' ? 'flex-row' : 'flex-col',
      props.class
    ]"
  >
    <slot />
  </div>
</template>