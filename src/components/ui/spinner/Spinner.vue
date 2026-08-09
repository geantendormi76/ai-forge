<script setup lang="ts">
interface SpinnerProps {
  class?: string
}

const props = withDefaults(defineProps<SpinnerProps>(), {})
</script>

<template>
  <svg
    class="animate-spin text-current shrink-0"
    :class="props.class"
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
  >
    <circle
      class="opacity-20"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      stroke-width="3"
    />
    <path
      class="opacity-100"
      fill="url(#purpleGoldGlow)"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
    />
    <defs>
      <!-- 🎨 核心：定义紫金流光全息过渡色 -->
      <linearGradient id="purpleGoldGlow" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="#bc05ff" />
        <stop offset="100%" stop-color="#DCA54C" />
      </linearGradient>
    </defs>
  </svg>
</template>