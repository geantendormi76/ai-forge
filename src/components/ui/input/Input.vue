<script setup lang="ts">
import { computed } from 'vue'

interface 输入框属性 {
  defaultValue?: string | number | null
  modelValue?: string | number | null
  variant?: 'default' | 'teal' | 'ghost'
  class?: string
  type?: string
  placeholder?: string
  disabled?: boolean
}

const props = withDefaults(defineProps<输入框属性>(), {
  type: 'text',
  variant: 'default',
})

const 触发更新 = defineEmits<{
  (时间事件: 'update:modelValue', 新值: string | number | null): void
}>()

const 绑定值 = computed({
  get: () => props.modelValue ?? props.defaultValue ?? '',
  set: (新值) => 触发更新('update:modelValue', 新值),
})

const 变体样式 = computed(() => {
  if (props.variant === 'ghost') {
    return 'bg-transparent border-none shadow-none focus:ring-0 focus:outline-none w-full h-full'
  }
  const 映射 = {
    default: 'h-8 px-3 bg-white border border-slate-200 rounded-xl shadow-sm focus:ring-2 focus:border-slate-400 focus:ring-[#DCA54C]/15',
    teal: 'h-8 px-3 bg-white border border-teal-200 rounded-xl shadow-sm focus:ring-2 focus:border-teal-400 focus:ring-teal-500/15 text-teal-800 font-bold'
  }
  return 映射[props.variant] || 映射.default
})
</script>

<template>
  <input
    v-model="绑定值"
    :type="props.type"
    :placeholder="props.placeholder"
    :disabled="props.disabled"
    :class="[
      'outline-none transition-all disabled:cursor-not-allowed disabled:opacity-50 text-left',
      props.variant !== 'ghost' ? 'sentence-input' : '',
      变体样式,
      props.class
    ]"
  />
</template>

<style scoped>
.sentence-input {
  font-size: 12px !important;
  font-weight: 600 !important;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: #334155;
  letter-spacing: 0.02em;
}
</style>