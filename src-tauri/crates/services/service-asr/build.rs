fn main() {
    if cfg!(target_os = "windows") {
        let cuda_path = std::env::var("CUDA_PATH")
            .unwrap_or_else(|_| r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.1".to_string());
        
        let lib_dir = std::path::PathBuf::from(&cuda_path).join("lib").join("x64");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            println!("cargo:rustc-link-lib=cudart");
            println!("cargo:rustc-link-lib=cublas");
            println!("cargo:rustc-link-lib=cuda");
        } else {
            println!("cargo:warning=⚠️ 未在 {:?} 找到 CUDA x64 库目录！", lib_dir);
        }
    }
}
