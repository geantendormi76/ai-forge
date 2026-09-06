use core_onnx_infer::config::ensure_cuda_dll_registered;
use service_upscale::{RealESRGANModel, UpscaleService, UpscaleTask};
use std::path::Path;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_upscale_whitebox_diagnostics() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ort=warn,core_onnx_infer=info,service_upscale=info")),
        )
        .try_init();

    // 🛡️ 节点 0: 必须在任何 ORT 接口调用前，第一优先级完成 CUDA 动态库寻址注册
    ensure_cuda_dll_registered();

    println!("\n======================================================================");
    println!("🚀 [8K 超分 GPU 满血白盒测试点火启动]");

    let model_path = UpscaleService::resolve_default_model_path();
    let t_load = Instant::now();
    let mut model = RealESRGANModel::new(&model_path, 4).expect("初始化 RealESRGANModel 失败");
    println!("⏱️ 模型加载与 Session 创建耗时: {:.2} ms", t_load.elapsed().as_secs_f64() * 1000.0);

    let dummy_img = image::DynamicImage::new_rgb8(276, 276);
    let t_inf = Instant::now();
    let out = model.forward_tile(&dummy_img).expect("单块推理失败");
    let inf_ms = t_inf.elapsed().as_secs_f64() * 1000.0;
    println!("✅ 单块 GPU 硬件推演完成 (尺寸: {}x{}), 耗时: {:.2} ms", out.width(), out.height(), inf_ms);
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
        println!("🎉 真实物理图像任务完成，端到端总耗时: {} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms as f64 / 1000.0);
    }
}
