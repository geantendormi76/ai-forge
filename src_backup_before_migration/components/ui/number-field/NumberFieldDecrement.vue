<script setup lang="ts">
import { inject } from 'vue'
import { Minus } from 'lucide-vue-next'

const { decrement, disabled } = inject('numberField') as any
</script>

<template>
  <button
    type="button"
    :disabled="disabled"
    @click="decrement"
    class="flex items-center justify-center h-full w-8 hover:bg-slate-50 border-r border-slate-150 text-slate-400 hover:text-[#DCA54C] disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer shrink-0"
  >
    <Minus :size="11" class="stroke-[3]" />
  </button>
</template>