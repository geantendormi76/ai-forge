<template>
  <div ref="containerRef" :class="`relative w-full ${className}`.trim()">
    <div
      ref="listRef"
      :class="`max-h-[300px] overflow-y-auto p-1 ${
        displayScrollbar
          ? '[&::-webkit-scrollbar]:w-[6px] [&::-webkit-scrollbar-track]:bg-slate-100 [&::-webkit-scrollbar-thumb]:bg-slate-300 [&::-webkit-scrollbar-thumb]:rounded-[3px]'
          : 'scrollbar-hide'
      }`"
      :style="{
        scrollbarWidth: displayScrollbar ? 'thin' : 'none',
        scrollbarColor: '#cbd5e1 #f1f5f9'
      }"
      @scroll="handleScroll"
    >
      <Motion
        v-for="(item, index) in items"
        :key="index"
        tag="div"
        :data-index="index"
        class="mb-1 cursor-pointer"
        :initial="{ scale: 0.95, opacity: 0 }"
        :animate="getItemInView(index) ? { scale: 1, opacity: 1 } : { scale: 0.95, opacity: 0 }"
        :transition="{ duration: 0.15, delay: 0.03 * Math.min(index, 5) }"
        @mouseenter="() => setSelectedIndex(index)"
        @click="
          () => {
            setSelectedIndex(index);
            emit('itemSelected', item, index);
          }
        "
      >
        <slot :item="item" :index="index" :isSelected="selectedIndex === index">
          <div :class="`p-2 bg-slate-50 hover:bg-slate-100 rounded-lg ${selectedIndex === index ? 'bg-slate-100 font-bold' : ''} ${itemClassName}`">
            <p class="text-slate-800 text-xs m-0">{{ typeof item === 'object' ? item.label : item }}</p>
          </div>
        </slot>
      </Motion>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, useTemplateRef } from 'vue';
import { Motion } from 'motion-v';

interface AnimatedListProps {
  items?: any[];
  showGradients?: boolean;
  enableArrowNavigation?: boolean;
  className?: string;
  itemClassName?: string;
  displayScrollbar?: boolean;
  initialSelectedIndex?: number;
}

const props = withDefaults(defineProps<AnimatedListProps>(), {
  items: () => [],
  showGradients: false,
  enableArrowNavigation: true,
  className: '',
  itemClassName: '',
  displayScrollbar: false,
  initialSelectedIndex: -1
});

const emit = defineEmits<{
  (e: 'itemSelected', item: any, index: number): void;
}>();

const containerRef = useTemplateRef<HTMLDivElement>('containerRef');
const listRef = useTemplateRef<HTMLDivElement>('listRef');
const selectedIndex = ref(props.initialSelectedIndex);
const keyboardNav = ref(false);
const itemsInView = ref<boolean[]>([]);

const setSelectedIndex = (index: number) => {
  selectedIndex.value = index;
};

const getItemInView = (index: number) => {
  return itemsInView.value[index] ?? true;
};

const handleScroll = (_e: Event) => {
  updateItemsInView();
};

const updateItemsInView = () => {
  if (!listRef.value) return;

  const container = listRef.value;
  const containerRect = container.getBoundingClientRect();

  itemsInView.value = (props.items || []).map((_, index) => {
    const item = container.querySelector(`[data-index="${index}"]`) as HTMLElement;
    if (!item) return true;

    const itemRect = item.getBoundingClientRect();
    const viewHeight = containerRect.height;
    const itemTop = itemRect.top - containerRect.top;
    const itemBottom = itemTop + itemRect.height;

    return itemTop < viewHeight && itemBottom > 0;
  });
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!props.items || props.items.length === 0) return;
  if (e.key === 'ArrowDown' || (e.key === 'Tab' && !e.shiftKey)) {
    e.preventDefault();
    keyboardNav.value = true;
    setSelectedIndex(Math.min(selectedIndex.value + 1, props.items.length - 1));
  } else if (e.key === 'ArrowUp' || (e.key === 'Tab' && e.shiftKey)) {
    e.preventDefault();
    keyboardNav.value = true;
    setSelectedIndex(Math.max(selectedIndex.value - 1, 0));
  } else if (e.key === 'Enter') {
    if (selectedIndex.value >= 0 && selectedIndex.value < props.items.length) {
      e.preventDefault();
      emit('itemSelected', props.items[selectedIndex.value], selectedIndex.value);
    }
  }
};

watch([selectedIndex, keyboardNav], () => {
  if (!keyboardNav.value || selectedIndex.value < 0 || !listRef.value) return;
  const container = listRef.value;
  const selectedItem = container.querySelector(`[data-index="${selectedIndex.value}"]`) as HTMLElement | null;
  if (selectedItem) {
    const extraMargin = 20;
    const containerScrollTop = container.scrollTop;
    const containerHeight = container.clientHeight;
    const itemTop = selectedItem.offsetTop;
    const itemBottom = itemTop + selectedItem.offsetHeight;
    if (itemTop < containerScrollTop + extraMargin) {
      container.scrollTo({ top: itemTop - extraMargin, behavior: 'smooth' });
    } else if (itemBottom > containerScrollTop + containerHeight - extraMargin) {
      container.scrollTo({
        top: itemBottom - containerHeight + extraMargin,
        behavior: 'smooth'
      });
    }
  }
  keyboardNav.value = false;
});

onMounted(() => {
  if (props.enableArrowNavigation) {
    window.addEventListener('keydown', handleKeyDown);
  }
  itemsInView.value = new Array((props.items || []).length).fill(true);
  setTimeout(updateItemsInView, 100);
});

onUnmounted(() => {
  if (props.enableArrowNavigation) {
    window.removeEventListener('keydown', handleKeyDown);
  }
});
</script>
