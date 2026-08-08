<script setup lang="ts">
import { computed } from 'vue'

interface TextareaProps {
  defaultValue?: string
  modelValue?: string | null
  class?: string
  placeholder?: string
  disabled?: boolean
  rows?: number
}

const props = withDefaults(defineProps<TextareaProps>(), {
  rows: 3
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const value = computed({
  get: () => props.modelValue ?? props.defaultValue ?? '',
  set: (val) => emit('update:modelValue', val)
})
</script>

<template>
  <textarea
    v-model="value"
    :placeholder="props.placeholder"
    :disabled="props.disabled"
    :rows="props.rows"
    :class="[
      'sentence-textarea w-full p-3 bg-white border border-slate-200 rounded-xl outline-none transition-all shadow-sm disabled:cursor-not-allowed disabled:opacity-50 text-left focus:ring-2 focus:border-slate-400 focus:ring-[#DCA54C]/15',
      props.class
    ]"
  ></textarea>
</template>

<style scoped>
.sentence-textarea {
  font-size: 11px !important;
  font-weight: 600 !important;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: #334155;
  letter-spacing: 0.02em;
}
</style>