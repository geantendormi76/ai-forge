<script setup lang="ts">
import ParticlesEffect from '../components/effects/ParticlesEffect.vue'
import ShinyText from '../components/effects/ShinyText.vue'
</script>

<template>
  <div class="flex-1 flex flex-col items-center justify-center relative w-full h-full">
    <!-- WebGL 3D 浮动粒子星空 -->
    <div class="absolute inset-0 z-0 overflow-hidden pointer-events-none">
      <ParticlesEffect
        :particle-count="300"
        :particle-spread="12"
        :speed="0.15"
        :particle-colors="['#bc05ff']"
        :move-particles-on-hover="true"
        :particle-hover-factor="1.2"
        :alpha-particles="true"
        :particle-base-size="100"
        :size-randomness="1.5"
        :camera-distance="24"
        class="w-full h-full opacity-90"
      />
    </div>

    <!-- 径向紫色高光模糊光晕 -->
    <div class="absolute w-[500px] h-[500px] bg-gradient-to-tr from-[#bc05ff]/10 to-transparent rounded-full blur-3xl opacity-80 pointer-events-none -translate-y-12 z-0"></div>

    <!-- 前景居中 Logo 与流光字标 -->
    <div class="relative z-10 flex flex-col items-center justify-center select-none pointer-events-none -translate-y-10">
      <img
        src="/1.png"
        class="w-auto h-12 object-contain transition-transform duration-300 filter drop-shadow-[0_4px_12px_rgba(188,5,255,0.3)] mb-4"
        alt="紫电极光"
      />
      <ShinyText
        text="紫电"
        :speed="3.5"
        :spread="150"
        color="#bc05ff"
        shine-color="#EED111"
        class="text-2xl font-black tracking-[0.3em] uppercase"
      />
    </div>
  </div>
</template>
