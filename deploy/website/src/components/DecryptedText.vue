<template>
  <span ref="containerRef" :class="`inline-block whitespace-pre-wrap ${parentClassName}`" v-bind="animateListeners">
    <span class="sr-only">{{ displayText }}</span>

    <span aria-hidden="true">
      <span
        v-for="(char, index) in displayText.split('')"
        :key="index"
        :class="isRevealedOrDone(index) ? className : encryptedClassName"
      >
        {{ char }}
      </span>
    </span>
  </span>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';

type AnimateOn = 'view' | 'hover' | 'inViewHover';

interface DecryptedTextProps {
  text: string;
  speed?: number;
  maxIterations?: number;
  useOriginalCharsOnly?: boolean;
  characters?: string;
  className?: string;
  encryptedClassName?: string;
  parentClassName?: string;
  animateOn?: AnimateOn;
}

const props = withDefaults(defineProps<DecryptedTextProps>(), {
  speed: 40,
  maxIterations: 8,
  useOriginalCharsOnly: false,
  characters: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*',
  className: '',
  encryptedClassName: 'text-[#A5F3FC]/60 font-mono',
  parentClassName: '',
  animateOn: 'view'
});

const containerRef = ref<HTMLSpanElement | null>(null);

const displayText = ref<string>(props.text);
const isAnimating = ref<boolean>(false);
const revealedIndices = ref<Set<number>>(new Set());
const hasAnimated = ref<boolean>(false);
const isDecrypted = ref<boolean>(false);

let intervalId: ReturnType<typeof setInterval> | null = null;

const availableChars = computed<string[]>(() => {
  return props.useOriginalCharsOnly
    ? Array.from(new Set(props.text.split(''))).filter(char => char !== ' ')
    : props.characters.split('');
});

function shuffleText(originalText: string, currentRevealed: Set<number>): string {
  return originalText
    .split('')
    .map((char, i) => {
      if (char === ' ') return ' ';
      if (currentRevealed.has(i)) return originalText[i];
      return availableChars.value[Math.floor(Math.random() * availableChars.value.length)];
    })
    .join('');
}

function triggerDecrypt() {
  revealedIndices.value = new Set();
  isAnimating.value = true;
}

function stopInterval() {
  if (intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
}

function startInterval() {
  stopInterval();
  let currentIteration = 0;

  intervalId = setInterval(() => {
    displayText.value = shuffleText(props.text, revealedIndices.value);
    currentIteration++;
    if (currentIteration >= props.maxIterations) {
      stopInterval();
      isAnimating.value = false;
      displayText.value = props.text;
      isDecrypted.value = true;
    }
  }, props.speed);
}

watch(isAnimating, val => {
  if (val) startInterval();
  else stopInterval();
});

function triggerHoverDecrypt() {
  if (isAnimating.value) return;
  revealedIndices.value = new Set();
  isDecrypted.value = false;
  displayText.value = props.text;
  isAnimating.value = true;
}

function resetToPlainText() {
  stopInterval();
  isAnimating.value = false;
  revealedIndices.value = new Set();
  displayText.value = props.text;
  isDecrypted.value = true;
}

const animateListeners = computed(() => {
  if (props.animateOn === 'hover' || props.animateOn === 'inViewHover') {
    return {
      onMouseenter: triggerHoverDecrypt,
      onMouseleave: resetToPlainText
    };
  }
  return {};
});

function isRevealedOrDone(index: number): boolean {
  return revealedIndices.value.has(index) || (!isAnimating.value && isDecrypted.value);
}

let intersectionObserver: IntersectionObserver | null = null;

onMounted(() => {
  if (props.animateOn === 'view' || props.animateOn === 'inViewHover') {
    intersectionObserver = new IntersectionObserver(
      entries => {
        entries.forEach(entry => {
          if (entry.isIntersecting && !hasAnimated.value) {
            triggerDecrypt();
            hasAnimated.value = true;
          }
        });
      },
      { rootMargin: '0px', threshold: 0.1 }
    );
    if (containerRef.value) intersectionObserver.observe(containerRef.value);
  } else {
    displayText.value = props.text;
    isDecrypted.value = true;
  }
});

onUnmounted(() => {
  stopInterval();
  intersectionObserver?.disconnect();
});
</script>
