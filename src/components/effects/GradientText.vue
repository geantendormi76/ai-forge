<template>
  <div
    :class="[
      'relative flex max-w-fit flex-row items-center justify-center rounded-2xl transition-all duration-500 font-extrabold tracking-tight select-none',
      showBorder ? 'border border-white/10 bg-slate-950/80 px-4 py-1.5 backdrop-blur-md shadow-lg shadow-purple-500/10' : 'border-transparent bg-transparent',
      className,
    ]"
  >
    <div
      class="inline-block font-black select-none tracking-normal leading-tight"
      :style="gradientStyle"
    >
      <slot>{{ text }}</slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';

interface GradientTextProps {
  text?: string;
  colors?: string[];
  animationSpeed?: number;
  showBorder?: boolean;
  className?: string;
  yoyo?: boolean;
}

const props = withDefaults(defineProps<GradientTextProps>(), {
  text: '',
  colors: () => ['#A5F3FC', '#D8B4F8', '#A5F3FC', '#D8B4F8'],
  animationSpeed: 6,
  showBorder: false,
  className: '',
  yoyo: true,
});

const progress = ref(0);
let animationFrameId: number | null = null;
let startTime: number | null = null;

function animate(timestamp: number) {
  if (!startTime) startTime = timestamp;
  const elapsed = (timestamp - startTime) / 1000;

  const duration = props.animationSpeed;
  const cycle = (elapsed % duration) / duration;

  if (props.yoyo) {
    progress.value = (Math.sin(cycle * Math.PI * 2) + 1) / 2;
  } else {
    progress.value = cycle;
  }

  animationFrameId = requestAnimationFrame(animate);
}

onMounted(() => {
  animationFrameId = requestAnimationFrame(animate);
});

onUnmounted(() => {
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
});

const gradientStyle = computed(() => {
  const colorString = props.colors.join(', ');
  const shift = progress.value * 100;
  return {
    backgroundImage: `linear-gradient(135deg, ${colorString})`,
    backgroundSize: '300% 300%',
    backgroundRepeat: 'repeat',
    backgroundPosition: `${shift}% 50%`,
    backgroundClip: 'text',
    WebkitBackgroundClip: 'text',
    WebkitTextFillColor: 'transparent',
    color: 'transparent',
  };
});
</script>
