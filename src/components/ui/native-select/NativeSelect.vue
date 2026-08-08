<!-- ui/src/components/ui/native-select/NativeSelect.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'

const props = defineProps<{
  modelValue?: any
  class?: any // 🚀 强类型修复：将类型升级为 any，无缝支持数组、对象及条件类，根治 TS2345 报错
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: any): void
}>()

const value = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})
</script>

<template>
  <div class="relative inline-block w-full select-none">
    <select
      v-model="value"
      :disabled="disabled"
      :class="[
        'w-full h-10 px-4 pr-8 rounded-xl text-xs font-black outline-none transition-all duration-200 cursor-pointer shadow-sm appearance-none focus:ring-2 disabled:cursor-not-allowed disabled:opacity-50',
        props.class ? props.class : 'bg-white border border-slate-200 text-slate-700 focus:border-[#DCA54C] focus:ring-[#DCA54C]/15'
      ]"
    >
      <slot />
    </select>
    <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none select-none text-slate-400">
      <ChevronDown :size="11" class="stroke-[2.5]" aria-hidden="true" />
    </div>
  </div>
</template>