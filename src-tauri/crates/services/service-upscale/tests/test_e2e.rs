use ort::ep::ExecutionProvider;
use service_upscale::{RealESRGANModel, UpscaleService, UpscaleTask};
use std::path::Path;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_upscale_whitebox_diagnostics() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn,service_upscale=info")),
        )
        .try_init();

    let is_cuda_avail = ort::ep::CUDA::default().is_available().unwrap_or(false);
    println!("\n======================================================================");
    println!("⚡ ort::ep::CUDA::default().is_available() = {}", is_cuda_avail);

    let model_path = UpscaleService::resolve_default_model_path();
    let t_load = Instant::now();
    let mut model = RealESRGANModel::new(&model_path, 4).expect("初始化 RealESRGANModel 失败");
    println!("⏱️ 模型加载与 Session 创建耗时: {:.2} ms", t_load.elapsed().as_secs_f64() * 1000.0);

    let dummy_img = image::DynamicImage::new_rgb8(276, 276);
    let t_inf = Instant::now();
    let out = model.forward_tile(&dummy_img).expect("单块推理失败");
    let inf_ms = t_inf.elapsed().as_secs_f64() * 1000.0;
    println!("✅ 单块推理完成 (尺寸: {}x{}), 纯推理耗时: {:.2} ms", out.width(), out.height(), inf_ms);
    println!("======================================================================\n");

    let in_path = Path::new(r"C:\dev\ai-forge\test\input\service-upscale\1.png");
    if in_path.exists() {
        let out_path = Path::new(r"C:\dev\ai-forge\test\outs\service-upscale\whitebox_out_1.png");
        let task = UpscaleTask {
            input_path: in_path.to_string_lossy().to_string(),
            output_path: out_path.to_string_lossy().to_string(),
            model_path: None,
            target_scale: Some(4.0),
            max_output_side: Some(3840),
            tile_size: Some(256),
            tile_pad: Some(10),
        };
        let res = UpscaleService::run_upscale(&task, None, Some(|cur, total, msg: &str| {
            println!("   [进度 {}/{}] {}", cur, total, msg);
        }))
        .await
        .expect("任务执行失败");
        println!("🎉 任务完成，总耗时: {} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms as f64 / 1000.0);
    }
}
