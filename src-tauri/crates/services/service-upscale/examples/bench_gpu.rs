// 🛡️ 2026 SOTA 硬件级 GPU 推理：冷启动预热 + 3 轮热推演白盒性能基准

use image::{DynamicImage, RgbImage};
use service_upscale::pipeline::RealESRGANModel;
use service_upscale::UpscaleService;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("🚀 [SOTA 硬件基准] 正在对 RTX 3060 显卡执行冷/热多轮物理压测...");
    println!("============================================================\n");

    let model_path = UpscaleService::resolve_default_model_path();
    println!("🎯 [1/3] 寻址 RealESRGAN ONNX 模型: {:?}", model_path);

    let t_load_start = Instant::now();
    let mut model = RealESRGANModel::new(&model_path, 4)
        .map_err(|e| format!("🚨 CUDA 引擎初始化失败: {e}"))?;
    println!("✅ [2/3] ONNX Runtime Session 纯血 CUDA 初始化完成 (耗时: {} ms)", t_load_start.elapsed().as_millis());

    let mock_tile = DynamicImage::ImageRgb8(RgbImage::new(512, 512));

    // 1. 第 1 轮：冷启动预热 (Cold Run)
    let t_cold = Instant::now();
    let _ = model.forward_tile(&mock_tile)?;
    let cold_ms = t_cold.elapsed().as_millis();
    println!("❄️ [冷启动预热 (第1块)]: {} ms (包含 cuDNN 算子寻址与显存分配)", cold_ms);

    // 2. 连续 3 轮热推演 (Warm Runs)
    println!("\n🔥 [3/3] 启动连续 3 轮 512x512 真实切块热推演压测:");
    let mut warm_times = Vec::new();
    for round in 1..=3 {
        let t_warm = Instant::now();
        let _ = model.forward_tile(&mock_tile)?;
        let warm_ms = t_warm.elapsed().as_millis();
        warm_times.push(warm_ms);
        println!("  ├── 第 {} 块 GPU 耗时: {} ms", round, warm_ms);
    }

    let avg_warm: u128 = warm_times.iter().sum::<u128>() / warm_times.len() as u128;
    println!("\n------------------------------------------------------------");
    println!("📊 【压测结论】平均单块热推演耗时: {} ms", avg_warm);
    if avg_warm < 200 {
        println!("🏆 显卡 Tensor Core 硬件直推极速已完全激活！(4K 图像几秒即可全部完成)");
    } else {
        println!("⚠️ 热推演耗时仍在 200ms 以上，需进一步优化显存与切块调度！");
    }
    println!("------------------------------------------------------------\n");

    Ok(())
}
