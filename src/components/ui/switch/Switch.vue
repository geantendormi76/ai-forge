<script setup lang="ts">
const props = defineProps<{
  modelValue?: boolean | null
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const toggle = () => {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
}
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="!!modelValue"
    :disabled="disabled"
    @click="toggle"
    class="peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#DCA54C] focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 shadow-inner"
    :class="modelValue ? 'bg-[#DCA54C]' : 'bg-slate-200'"
  >
    <span
      class="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 ease-in-out"
      :class="modelValue ? 'translate-x-4' : 'translate-x-0'"
    />
  </button>
</template>