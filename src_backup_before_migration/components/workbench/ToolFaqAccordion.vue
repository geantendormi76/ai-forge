<script setup lang="ts">
import { ref } from 'vue';
import { ChevronDown } from 'lucide-vue-next';

export interface FaqItem {
  id: string | number;
  question: string;
  answer: string;
}

defineProps<{
  title?: string;
  faqs: FaqItem[];
}>();

const openFaqId = ref<string | number | null>(null);

const toggleFaq = (id: string | number) => {
  openFaqId.value = openFaqId.value === id ? null : id;
};
</script>

<template>
  <section class="space-y-6 pt-10 select-none">
    <div class="text-center space-y-1">
      <h3 class="text-xl sm:text-2xl font-bold text-white tracking-tight">
        {{ title || '常见问题' }}
      </h3>
    </div>

    <div class="space-y-3 max-w-3xl mx-auto">
      <div
        v-for="faq in faqs"
        :key="faq.id"
        class="border border-white/[0.06] bg-black/40 hover:border-white/15 rounded-2xl transition-all overflow-hidden shadow-md"
      >
        <button
          type="button"
          @click="toggleFaq(faq.id)"
          class="w-full p-4 sm:p-5 flex items-center justify-between text-left cursor-pointer gap-4"
        >
          <span class="text-xs sm:text-sm font-bold text-white">
            {{ faq.question }}
          </span>
          <ChevronDown
            :size="15"
            class="text-[#8b8b87] transition-transform duration-200 shrink-0"
            :class="{ 'rotate-180 text-white': openFaqId === faq.id }"
          />
        </button>

        <div
          v-if="openFaqId === faq.id"
          class="px-5 pb-5 pt-1 text-xs text-[#8b8b87] leading-relaxed border-t border-white/[0.04] animate-in fade-in duration-200"
        >
          {{ faq.answer }}
        </div>
      </div>
    </div>
  </section>
</template>
