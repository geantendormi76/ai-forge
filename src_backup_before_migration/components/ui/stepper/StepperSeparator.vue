<script setup lang="ts">
interface SeparatorProps {
  active?: boolean
  completed?: boolean
  class?: string
}

// 🛡️ 物理落盘：显式解构 Vue3 SFC Props 契约
const props = defineProps<SeparatorProps>()
</script>

<template>
  <div
    class="w-[1.5px] flex-1 transition-all duration-500 mt-0.5"
    :class="[
      props.completed ? 'bg-emerald-500' : '',
      props.active ? 'bg-gradient-to-b from-[#bc05ff] to-[#DCA54C] animate-pulse' : '',
      (!props.completed && !props.active) ? 'bg-slate-300' : '',
      props.class
    ]"
  />
</template>