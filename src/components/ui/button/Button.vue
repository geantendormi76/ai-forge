<script setup lang="ts">
import { computed } from 'vue'

interface 按钮属性 {
  as?: string | object
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link'
  size?: 'default' | 'sm' | 'lg' | 'icon' | 'icon-sm' | 'icon-lg'
  class?: string
}

const props = withDefaults(defineProps<按钮属性>(), {
  as: 'button',
  variant: 'default',
  size: 'default',
})

const 组合样式 = computed(() => {
  const 基础样式 = 'inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-xl text-[11px] font-black tracking-wider uppercase transition-all duration-200 select-none active:scale-95 disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#DCA54C]/50'

  const 变体样式集 = {
    default: 'bg-[#DCA54C] text-white hover:bg-amber-600 shadow-sm',
    destructive: 'bg-rose-600 text-white hover:bg-rose-700 shadow-sm',
    outline: 'border border-slate-300 bg-white text-slate-700 hover:border-[#DCA54C] hover:bg-slate-50',
    secondary: 'bg-slate-100 text-slate-800 hover:bg-slate-200 border border-slate-200/50',
    ghost: 'text-slate-600 hover:bg-slate-100 hover:text-slate-900',
    link: 'text-[#DCA54C] underline-offset-4 hover:underline bg-transparent',
  }

  const 尺寸样式集 = {
    default: 'h-8 px-4 py-2',
    sm: 'h-7 rounded-lg px-3 text-[10px]',
    lg: 'h-10 rounded-2xl px-8 text-xs',
    icon: 'h-8 w-8 p-0',
    'icon-sm': 'h-7 w-7 p-0',
    'icon-lg': 'h-10 w-10 p-0',
  }

  return [
    基础样式,
    变体样式集[props.variant] || 变体样式集.default,
    尺寸样式集[props.size] || 尺寸样式集.default,
    props.class
  ].filter(Boolean).join(' ')
})
</script>

<template>
  <component
    :is="props.as"
    :class="组合样式"
  >
    <slot />
  </component>
</template>