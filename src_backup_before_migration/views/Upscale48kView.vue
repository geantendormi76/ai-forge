<script setup lang="ts">
import { ref } from 'vue';
import ToolWorkbenchLayout, { type GuideItem } from '../components/layout/ToolWorkbenchLayout.vue';
import { useToolWorkflow } from '../composables/useToolWorkflow';
import { runUpscale48k, type UpscaleResult, type UpscaleTask } from '../bindings/tools/upscale-48k';

// 1. 初始化 4K/8K 图像超分专属批量工作流状态机
const workflow = useToolWorkflow<UpscaleResult>({
  dialogTitle: '选择待超分辨率重建的图像文件',
  dialogFilters: [
    {
      name: '支持的图像格式',
      extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'],
    },
  ],
  allowedExtensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'],
  unsupportedPrompt: (names) =>
    `已自动过滤不受支持的文件: ${names.join(', ')}。仅支持常见图像格式。`,
});

// 2. 超分定制参数 (极简小白直觉)
const targetScale = ref<number>(4);
const selectedCapMode = ref<'fast_4k' | 'flagship_8k'>('fast_4k');

// 3. 左侧指南卡片数据：1:1 镜面对齐右侧选项
const upscaleGuides: GuideItem[] = [
  {
    title: '1. 放大倍率',
    tag: '2x / 4x / 8x',
    desc: '头像与日常截图推荐【4x】；轻微锐化选【2x】；模糊老照片选【8x】。',
  },
  {
    title: '2. 画面规格',
    tag: '4K 极速 / 8K 满血',
    desc: '日常首选【4K 极速】几秒出图不占硬盘；超大壁纸与大幅印刷选【8K 满血】。',
  },
  {
    title: '3. 透明背景保真',
    tag: '透明底 PNG / WEBP',
    desc: 'LOGO、立绘与贴纸等透明素材自动保留透明底，重构干净无白边。',
  },
  {
    title: '4. 智能硬件加速',
    tag: 'RTX 显卡直推',
    desc: '系统已全自动激活 GPU 显存极速推导与防爆切块，无需手动调节。',
  },
];

// 4. 批量执行流水线
const handleExecute = async () => {
  await workflow.executeBatch(
    async (file, _index, _total) => {
      const maxOutputSide = selectedCapMode.value === 'fast_4k' ? 3840 : 8192;
      const lastSlash = Math.max(file.path.lastIndexOf('\\'), file.path.lastIndexOf('/'));
      const outDir = lastSlash !== -1 ? file.path.substring(0, lastSlash) : '.';
      
      const lastDot = file.name.lastIndexOf('.');
      const baseName = lastDot !== -1 ? file.name.substring(0, lastDot) : file.name;
      const ext = lastDot !== -1 ? file.name.substring(lastDot + 1) : 'png';

      const modeTag = selectedCapMode.value === 'fast_4k' ? '4k' : '8k';
      const outName = `${baseName}_upscaled_${modeTag}_${targetScale.value}x.${ext}`;
      const outputPath = `${outDir}\\${outName}`;

      const task: UpscaleTask = {
        input_path: file.path,
        output_path: outputPath,
        model_path: null,
        target_scale: targetScale.value,
        max_output_side: maxOutputSide,
        tile_size: 512,
        tile_pad: 10,
      };

      return await runUpscale48k(task);
    },
    {
      targetFormat: selectedCapMode.value === 'fast_4k' ? '4K 极速超清' : '8K 旗舰巨幅',
      statusPrefix: 'RTX 显卡超分中',
    }
  );
};
</script>

<template>
  <ToolWorkbenchLayout
    title="4K/8K 图像超分"
    :steps="['1 添加图像', '2 选定倍率', '3 极速交付']"
    :guides="upscaleGuides"
    :workflow="workflow"
    queue-title="待超分图像队列"
    upload-button-text="选择图像文件"
    dropzone-title="把需要超清重建的图片拖放到这里"
    dropzone-subtitle="支持 PNG、JPG、JPEG、WEBP、BMP 格式，纯血本地 GPU 矩阵直推"
    action-button-text="开始图像超分辨率重建"
    @execute="handleExecute"
  >
    <!-- 核心定制选项仓 -->
    <template #options>
      <div class="space-y-4 select-none">
        
        <!-- 选项 1：目标放大倍率 -->
        <div class="space-y-2">
          <div>
            <span class="text-xs font-bold text-white">1. 目标放大倍率</span>
          </div>

          <div class="grid grid-cols-3 gap-2.5">
            <button
              type="button"
              @click="targetScale = 2"
              class="p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between space-y-1"
              :class="[
                targetScale === 2
                  ? 'bg-[#02c3b4]/15 border-[#02c3b4]/60 text-white shadow-[0_0_16px_rgba(2,195,180,0.2)]'
                  : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-white/[0.04] hover:text-white'
              ]"
            >
              <span class="text-xs font-bold">2x 高清锐化</span>
              <p class="text-[10.5px] opacity-70 leading-tight">轻微模糊、快速清晰</p>
            </button>

            <button
              type="button"
              @click="targetScale = 4"
              class="p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between space-y-1"
              :class="[
                targetScale === 4
                  ? 'bg-[#02c3b4]/15 border-[#02c3b4]/60 text-white shadow-[0_0_16px_rgba(2,195,180,0.2)]'
                  : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-white/[0.04] hover:text-white'
              ]"
            >
              <span class="text-xs font-bold">4x 超清重构</span>
              <p class="text-[10.5px] opacity-70 leading-tight">日常推荐、细节拉满</p>
            </button>

            <button
              type="button"
              @click="targetScale = 8"
              class="p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between space-y-1"
              :class="[
                targetScale === 8
                  ? 'bg-[#02c3b4]/15 border-[#02c3b4]/60 text-white shadow-[0_0_16px_rgba(2,195,180,0.2)]'
                  : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-white/[0.04] hover:text-white'
              ]"
            >
              <span class="text-xs font-bold">8x 巨幅放大</span>
              <p class="text-[10.5px] opacity-70 leading-tight">极小模糊老旧照片</p>
            </button>
          </div>
        </div>

        <!-- 选项 2：画面规格限制 -->
        <div class="space-y-2 pt-1 border-t border-white/[0.04]">
          <div>
            <span class="text-xs font-bold text-white">2. 画面规格限制</span>
          </div>

          <div class="grid grid-cols-2 gap-2.5">
            <button
              type="button"
              @click="selectedCapMode = 'fast_4k'"
              class="p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between space-y-1"
              :class="[
                selectedCapMode === 'fast_4k'
                  ? 'bg-[#02c3b4]/15 border-[#02c3b4]/60 text-white shadow-[0_0_16px_rgba(2,195,180,0.2)]'
                  : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-white/[0.04] hover:text-white'
              ]"
            >
              <span class="text-xs font-bold">4K 极速模式</span>
              <p class="text-[10.5px] opacity-70 leading-tight">3840px 黄金封顶，几秒出图，日常首选</p>
            </button>

            <button
              type="button"
              @click="selectedCapMode = 'flagship_8k'"
              class="p-3 rounded-2xl border text-left transition-all cursor-pointer flex flex-col justify-between space-y-1"
              :class="[
                selectedCapMode === 'flagship_8k'
                  ? 'bg-[#02c3b4]/15 border-[#02c3b4]/60 text-white shadow-[0_0_16px_rgba(2,195,180,0.2)]'
                  : 'bg-white/[0.02] border-white/[0.06] text-[#8b999b] hover:bg-white/[0.04] hover:text-white'
              ]"
            >
              <span class="text-xs font-bold">8K 满血巨幅</span>
              <p class="text-[10.5px] opacity-70 leading-tight">8192px 满血无损重构，适合海报与大幅印刷</p>
            </button>
          </div>
        </div>

      </div>
    </template>
  </ToolWorkbenchLayout>
</template>
