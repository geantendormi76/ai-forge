use ort::ep::ExecutionProvider;
use service_upscale::{RealESRGANModel, UpscaleService, UpscaleTask};
use std::path::Path;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_upscale_whitebox_diagnostics() {
    // 1. 白盒探针：全开 TRACE 级底层日志，强制暴露 ONNX Runtime 内部所有 C++ 告警
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("trace,ort=trace,core_onnx_infer=trace,service_upscale=trace")),
        )
        .try_init();

    println!("\n======================================================================");
    println!("🔍 [白盒探针 1] 打印当前 Rust 测试进程内部生效的 PATH 环境变量:");
    if let Ok(path_var) = std::env::var("PATH") {
        for p in path_var.split(';').take(5) {
            println!("   - {}", p);
        }
    }
    println!("======================================================================");

    // 2. 白盒探针：直接调用 ort API 探测 CUDA Provider 硬件可用性
    println!("🔍 [白盒探针 2] 探测 ort::ep::CUDA 硬件可用性断言:");
    let cuda_ep = ort::ep::CUDA::default();
    let is_cuda_avail = cuda_ep.is_available().unwrap_or(false);
    println!("   ⚡ ort::ep::CUDA::default().is_available() = {}", is_cuda_avail);
    println!("======================================================================");

    // 3. 白盒探针：对单个模型执行独立单块点火，精确测量纯推理毫秒
    let model_path = UpscaleService::resolve_default_model_path();
    println!("🔍 [白盒探针 3] 正在加载模型会话: {:?}", model_path);
    let t_load = Instant::now();
    let mut model = RealESRGANModel::new(&model_path, 4).expect("初始化 RealESRGANModel 失败");
    println!("   ⏱️ 模型加载与 Session 创建物理耗时: {:.2} ms", t_load.elapsed().as_secs_f64() * 1000.0);

    let dummy_img = image::DynamicImage::new_rgb8(276, 276);
    println!("\n⚡ [白盒探针 4] 执行单切块 (276x276) 单独推导测速 (观察是否输出 C++ 告警)...");
    let t_inf = Instant::now();
    let out = model.forward_tile(&dummy_img).expect("单块推理失败");
    let inf_ms = t_inf.elapsed().as_secs_f64() * 1000.0;
    println!("   ✅ 单块推理完成，输出尺寸: {}x{}", out.width(), out.height());
    println!("   ⏱️ 单切块纯推理物理耗时: {:.2} ms ({:.2} s)", inf_ms, inf_ms / 1000.0);
    println!("======================================================================\n");

    // 4. 跑一个真实图片样本
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

        println!("🚀 启动完整图片任务验证: {:?}", in_path);
        let res = UpscaleService::run_upscale(&task, None, Some(|cur, total, msg: &str| {
            println!("   [进度 {}/{}] {}", cur, total, msg);
        }))
        .await
        .expect("任务执行失败");

        println!("🎉 任务完成，总耗时: {} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms as f64 / 1000.0);
    }
}
