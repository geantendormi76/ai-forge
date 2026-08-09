<script setup lang="ts">
import { provide, computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: number | null
  min?: number
  max?: number
  step?: number
  disabled?: boolean
}>(), {
  modelValue: 0,
  min: 0,
  max: 9999,
  step: 1,
  disabled: false
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void
}>()

const safeValue = computed(() => props.modelValue ?? 0)

const updateValue = (val: number) => {
  if (props.disabled) return
  let clamped = val
  if (props.min !== undefined && clamped < props.min) clamped = props.min
  if (props.max !== undefined && clamped > props.max) clamped = props.max
  
  // 🛡️ 鲁棒性防线：消除 JavaScript 浮点数加减精度空难 (例如: 0.1 + 0.2 = 0.300000004)
  const decimals = props.step.toString().split('.')[1]?.length || 0
  clamped = Number(clamped.toFixed(decimals))
  
  emit('update:modelValue', clamped)
}

const increment = () => {
  updateValue(safeValue.value + props.step)
}

const decrement = () => {
  updateValue(safeValue.value - props.step)
}

provide('numberField', {
  modelValue: safeValue,
  min: props.min,
  max: props.max,
  step: props.step,
  disabled: props.disabled,
  increment,
  decrement,
  updateValue
})
</script>

<template>
  <div class="relative w-full number-field-container select-none">
    <slot />
  </div>
</template>