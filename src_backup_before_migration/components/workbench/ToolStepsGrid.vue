<script setup lang="ts">
export interface StepItem {
  step: number | string;
  title: string;
  desc: string;
}

defineProps<{
  title?: string;
  subtitle?: string;
  steps: StepItem[];
}>();
</script>

<template>
  <section class="space-y-6 pt-10 select-none">
    <div class="text-center space-y-1">
      <h3 class="text-xl sm:text-2xl font-bold text-white tracking-tight">
        {{ title || '使用步骤' }}
      </h3>
      <p class="text-xs sm:text-sm text-[#8b8b87]">
        {{ subtitle || '简单三步，在本机极速完成' }}
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div
        v-for="item in steps"
        :key="item.step"
        class="p-6 rounded-2xl border border-white/[0.06] bg-black/40 hover:bg-black/60 hover:border-white/15 transition-all text-center space-y-3 shadow-md"
      >
        <div class="w-8 h-8 rounded-xl bg-white/[0.06] border border-white/10 text-white font-mono font-bold text-xs flex items-center justify-center mx-auto">
          {{ item.step }}
        </div>
        <h4 class="text-sm font-bold text-white">
          {{ item.title }}
        </h4>
        <p class="text-xs text-[#8b8b87] leading-relaxed">
          {{ item.desc }}
        </p>
      </div>
    </div>
  </section>
</template>
