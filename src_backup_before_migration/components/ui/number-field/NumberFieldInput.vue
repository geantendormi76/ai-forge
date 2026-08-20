<script setup lang="ts">
import { inject } from 'vue'

const { modelValue, disabled, updateValue } = inject('numberField') as any

const onInput = (e: Event) => {
  const val = Number((e.target as HTMLInputElement).value)
  if (!isNaN(val)) {
    updateValue(val)
  }
}
</script>

<template>
  <input
    type="number"
    :value="modelValue"
    :disabled="disabled"
    @input="onInput"
    class="w-full h-full bg-transparent px-1 text-center text-xs font-semibold text-slate-700 outline-none border-none [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
  />
</template>