<script setup lang="ts">
import { Cpu, Zap, FileCheck } from 'lucide-vue-next';
import CountUp from './CountUp.vue';
</script>

<template>
  <div class="ln-feat-stack relative w-full h-56 overflow-hidden">


    <div class="ln-feat-stack-chip ln-feat-stack-chip--rust">
      <span class="ln-feat-stack-mark text-amber-400">
        <Zap class="w-4 h-4" />
      </span>
      <span class="ln-feat-stack-name font-mono flex items-center gap-1">
        <span>C++ 原生物理引擎:</span>
        <span class="text-amber-300 font-bold">
          <CountUp :to="233" suffix="ms" :duration="1.5" />
        </span>
      </span>
    </div>


    <div class="ln-feat-stack-chip ln-feat-stack-chip--uv">
      <span class="ln-feat-stack-mark text-[#A5F3FC]">
        <FileCheck class="w-4 h-4" />
      </span>
      <span class="ln-feat-stack-name font-mono flex items-center gap-1">
        <span>物理抹图瘦身率:</span>
        <span class="text-[#A5F3FC] font-bold">
          <CountUp :to="98.5" :decimals="1" suffix="%" :duration="1.8" />
        </span>
      </span>
    </div>


    <div class="ln-feat-stack-chip ln-feat-stack-chip--gpu">
      <span class="ln-feat-stack-mark text-[#D8B4F8]">
        <Cpu class="w-4 h-4" />
      </span>
      <span class="ln-feat-stack-name font-mono flex items-center gap-1">
        <span>显存占用:</span>
        <span class="text-emerald-400 font-bold">0 MB (纯 C++ 算子)</span>
      </span>
    </div>

  </div>
</template>

<style scoped>
.ln-feat-stack-chip {
  position: absolute;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: rgba(15, 23, 42, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  z-index: 1;
  white-space: nowrap;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
}

.ln-feat-stack-chip--rust {
  top: 18%;
  left: 8%;
  animation:
    driftA 7s ease-in-out infinite,
    pulseRust 6s ease-in-out infinite;
}

.ln-feat-stack-chip--uv {
  top: 48%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2;
  animation:
    driftB 8s ease-in-out infinite,
    pulseUv 6s ease-in-out infinite 2s;
}

.ln-feat-stack-chip--gpu {
  bottom: 18%;
  right: 8%;
  animation:
    driftC 7.5s ease-in-out infinite,
    pulseGpu 6s ease-in-out infinite 4s;
}

@keyframes driftA {
  0%, 100% { transform: translate(0, 0) rotate(-4deg); }
  50% { transform: translate(6px, -8px) rotate(-2deg); }
}

@keyframes driftB {
  0%, 100% { transform: translate(-50%, -50%) rotate(0deg); }
  50% { transform: translate(calc(-50% - 5px), calc(-50% + 6px)) rotate(2deg); }
}

@keyframes driftC {
  0%, 100% { transform: translate(0, 0) rotate(4deg); }
  50% { transform: translate(-7px, -5px) rotate(6deg); }
}

@keyframes pulseRust {
  0%, 70%, 100% {
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }
  15%, 25% {
    border-color: #f59e0b;
    box-shadow: 0 0 24px rgba(245, 158, 11, 0.4);
    background: rgba(245, 158, 11, 0.1);
  }
}

@keyframes pulseUv {
  0%, 70%, 100% {
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: 0 4px 20px rgba(165, 243, 252, 0.4);
    background: rgba(165, 243, 252, 0.1);
  }
}

@keyframes pulseGpu {
  0%, 70%, 100% {
    border-color: rgba(255, 255, 255, 0.12);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }
  15%, 25% {
    border-color: #D8B4F8;
    box-shadow: 0 0 24px rgba(216, 180, 248, 0.4);
    background: rgba(216, 180, 248, 0.1);
  }
}

.ln-feat-stack-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.ln-feat-stack-name {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 600;
}
</style>
