<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, useTemplateRef } from 'vue'
import { ChevronDown } from 'lucide-vue-next'
import AnimatedList from '../../effects/AnimatedList.vue'

interface SelectOption {
  value: any
  label: string
  icon?: any
  badge?: string
}

interface SelectProps {
  modelValue: any
  options: SelectOption[]
  placeholder?: string
  width?: string
  placement?: 'top' | 'bottom'
  variant?: 'default' | 'borderless'
}

const props = withDefaults(defineProps<SelectProps>(), {
  placeholder: '请选择',
  width: 'w-[160px]',
  placement: 'bottom',
  variant: 'default'
})

const emit = defineEmits<{
  (e: 'update:modelValue', val: any): void
}>()

const containerRef = useTemplateRef<HTMLDivElement>('containerRef')
const openMenu = ref(false)
const instanceId = `select_${Math.random().toString(36).substring(2, 9)}`

const activeOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue) || { value: null, label: props.placeholder, icon: null }
})

const selectItem = (item: any) => {
  openMenu.value = false
  emit('update:modelValue', item.value)
}

const toggleMenu = () => {
  openMenu.value = !openMenu.value
  if (openMenu.value) {
    window.dispatchEvent(new CustomEvent('aeforge-close-dropdowns', {
      detail: { source: instanceId }
    }))
  }
}

const clickOutsideHandler = (e: MouseEvent) => {
  const el = containerRef.value
  if (el && !el.contains(e.target as HTMLElement)) {
    openMenu.value = false
  }
}

const handleGlobalClose = (e: Event) => {
  const customEvent = e as CustomEvent
  if (customEvent.detail?.source !== instanceId) {
    openMenu.value = false
  }
}

onMounted(() => {
  window.addEventListener('click', clickOutsideHandler)
  window.addEventListener('aeforge-close-dropdowns', handleGlobalClose)
})

onUnmounted(() => {
  window.removeEventListener('click', clickOutsideHandler)
  window.removeEventListener('aeforge-close-dropdowns', handleGlobalClose)
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative inline-block select-none transition-all"
    :class="openMenu ? 'z-[100]' : 'z-10'"
  >
    <button
      @click="toggleMenu"
      type="button"
      class="h-8 transition-all duration-200 cursor-pointer inline-flex items-center gap-1.5 active:scale-95 group/pill"
      :class="[
        props.variant === 'borderless'
          ? 'bg-transparent border-none px-2 hover:bg-slate-100/60 rounded-xl text-slate-700'
          : 'px-2.5 rounded-xl bg-white border border-slate-300 hover:border-blue-500 shadow-sm'
      ]"
      :aria-expanded="openMenu"
    >
      <component
        v-if="activeOption.icon"
        :is="activeOption.icon"
        :size="12"
        class="text-slate-500 group-hover/text-blue-500 transition-colors duration-200 shrink-0"
      />
      <span class="text-[11px] font-black text-slate-700">
        {{ activeOption.label }}
      </span>
      <ChevronDown
        :size="10"
        class="text-slate-400 transition-transform duration-300 stroke-[2.5]"
        :class="{ 'rotate-180 text-blue-500': openMenu }"
      />
    </button>

    <div
      v-if="openMenu"
      :class="[
        'absolute left-0 bg-white/95 backdrop-blur-xl border border-slate-100/50 rounded-2xl shadow-[0_12px_42px_rgba(15,23,42,0.08),_0_2px_8px_rgba(15,23,42,0.04)] p-1.5 animate-in fade-in zoom-in-95 duration-150 flex flex-col gap-0.5 z-[100]',
        props.placement === 'top' ? 'bottom-full mb-2.5 origin-bottom-left' : 'top-full mt-2 origin-top-left',
        props.width
      ]"
    >
      <AnimatedList
        :items="options"
        :showGradients="false"
        :enableArrowNavigation="true"
        :displayScrollbar="false"
        className="w-full"
        @itemSelected="(item: any) => selectItem(item)"
      >
        <template #default="{ item, isSelected }">
          <div
            class="w-full px-3 py-2 rounded-xl transition-all duration-150 cursor-pointer flex items-center justify-between group/item text-left text-slate-700"
            :class="[
              isSelected || item.value === props.modelValue
                ? 'bg-blue-50 text-blue-600 font-extrabold'
                : 'hover:bg-slate-100/80 hover:text-slate-950'
            ]"
          >
            <div class="flex items-center gap-2.5 overflow-hidden">
              <div
                v-if="item.icon"
                class="p-1 rounded-lg border flex-shrink-0 flex items-center justify-center transition-colors duration-150"
                :class="[
                  isSelected || item.value === props.modelValue
                    ? 'bg-white border-blue-200 text-blue-600'
                    : 'bg-white border-slate-200 text-slate-400 group-hover/item:border-slate-300 group-hover/item:text-slate-600'
                ]"
              >
                <component :is="item.icon" :size="13" />
              </div>
              <span class="text-xs font-semibold tracking-wide whitespace-nowrap">{{ item.label }}</span>
            </div>

            <span
              v-if="item.badge"
              class="text-[7.5px] font-black uppercase px-1 py-0.5 rounded border tracking-widest shrink-0 scale-90"
              :class="[
                isSelected || item.value === props.modelValue
                  ? 'bg-blue-100 text-blue-600 border-blue-200'
                  : 'bg-slate-100 text-slate-400 border-slate-200/60'
              ]"
            >
              {{ item.badge }}
            </span>
          </div>
        </template>
      </AnimatedList>
    </div>
  </div>
</template>
