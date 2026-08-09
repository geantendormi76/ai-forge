<script setup lang="ts">
interface IndicatorProps {
  active?: boolean
  completed?: boolean
  failed?: boolean
  class?: string
}

// 🛡️ 物理落盘：显式解构 Vue3 SFC Props 契约
const props = defineProps<IndicatorProps>()
</script>

<template>
  <div
    class="w-6 h-6 rounded-full bg-white border flex items-center justify-center z-10 transition-all duration-300 shadow-sm shrink-0 select-none"
    :class="[
      props.active ? 'border-[#bc05ff] ring-4 ring-[#bc05ff]/10 shadow-[0_0_12px_rgba(188,5,255,0.25)]' : '',
      props.completed ? 'border-emerald-500 bg-emerald-500/5 text-emerald-500' : '',
      props.failed ? 'border-rose-600 bg-rose-600/5 text-rose-600' : '',
      (!props.active && !props.completed && !props.failed) ? 'border-slate-300 text-slate-400' : '',
      props.class
    ]"
  >
    <slot />
  </div>
</template>