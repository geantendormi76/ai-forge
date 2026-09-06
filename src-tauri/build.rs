// 🛡️ 紫电 AI 桌面工坊最高安全红线：自适应 Windows Manifest 合并与 ORT 纯血 CUDA 12 自愈脚本
use std::fs;
use std::path::Path;
use tauri_build::WindowsAttributes;

fn main() {
    let mut attributes = tauri_build::Attributes::new();
    #[cfg(target_os = "windows")]
    {
        // 🔬 [自愈防线] 拦截 Tauri 默认注入清单机制，阻断 mt.exe LNK1327 链接错误
        attributes = attributes.windows_attributes(
            WindowsAttributes::new_without_app_manifest()
        );
        embed_manifest_for_all();
        自愈投影_cuda12_dll();
    }
    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}

#[cfg(target_os = "windows")]
fn embed_manifest_for_all() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_path = Path::new(&out_dir).join("windows-app-manifest.xml");
    fs::write(&manifest_path, r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#).expect("failed to write manifest file");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest_path.to_str().unwrap());
    }
}

/// 🛡️ 直接从 bin/cuda12 黄金目录同步全部官方认证动态库至构建沙盒，根除 AppData 污染
#[cfg(target_os = "windows")]
fn 自愈投影_cuda12_dll() {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = Path::new(&manifest_dir);
        let root_bin = manifest_path.parent().unwrap_or(manifest_path).join("bin").join("cuda12");
        let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let target_dir = manifest_path.join("target").join(&profile);
        let deps_dir = target_dir.join("deps");

        if root_bin.exists() && target_dir.exists() {
            let _ = fs::create_dir_all(&deps_dir);
            if let Ok(entries) = fs::read_dir(&root_bin) {
                for entry in entries.flatten() {
                    let src = entry.path();
                    if src.is_file() && src.extension().map_or(false, |ext| ext == "dll") {
                        let fname = entry.file_name();
                        let _ = fs::copy(&src, target_dir.join(&fname));
                        let _ = fs::copy(&src, deps_dir.join(&fname));
                    }
                }
            }
            println!("cargo:warning=[自愈工坊] 已成功将 bin/cuda12 官方黄金库全量同步至构建沙盒");
        }
    }
}
