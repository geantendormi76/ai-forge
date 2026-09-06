<script setup lang="ts">
import { ref } from 'vue';
import { ChevronRight } from 'lucide-vue-next';

const props = withDefaults(
  defineProps<{
    messageCount?: number;
    toolCallCount?: number;
    defaultExpanded?: boolean;
  }>(),
  {
    messageCount: 1,
    toolCallCount: 0,
    defaultExpanded: false,
  }
);

const isExpanded = ref(props.defaultExpanded);
</script>

<template>
  <div class="mb-3 select-none">
    <button
      type="button"
      @click="isExpanded = !isExpanded"
      class="flex items-center gap-1.5 py-1 text-xs text-[#667783] dark:text-[#a19d92] hover:text-[#102333] dark:hover:text-[#faf9f7] font-mono cursor-pointer transition-colors"
      :title="isExpanded ? '收起过程详情' : '展开过程详情'"
    >
      <ChevronRight
        :size="13"
        class="transition-transform duration-200"
        :class="{ 'rotate-90': isExpanded }"
      />
      <span class="font-medium">
        过程详情 · {{ messageCount }} 条消息<template v-if="toolCallCount > 0"> · {{ toolCallCount }} 次工具调用</template>
      </span>
    </button>

    <div v-if="isExpanded" class="mt-2 pl-4 border-l-2 border-[#e4e1da] dark:border-[#33302a] space-y-2">
      <slot />
    </div>
  </div>
</template>
