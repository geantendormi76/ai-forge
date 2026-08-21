// 🛡️ 紫电 AI 桌面工坊最高安全红线：自适应 Windows Manifest 合并与 ORT 测试沙箱自愈脚本
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
        自愈投影_onnxruntime_dll();
    }

    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}

#[cfg(target_os = "windows")]
fn embed_manifest_for_all() {
    // 🛡️ 动态在临时 OUT_DIR 目录下生成符合 Common Controls v6 标准的应用清单 XML 文件
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

/// 自动在 AppData 中寻找 ort 缓存的合规 DLL 并将其顺次投影至运行目标目录
/// 彻底解决因 System32 路径劫持导致的 STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139) 闪退
#[cfg(target_os = "windows")]
fn 自愈投影_onnxruntime_dll() {
    let local_appdata = match std::env::var("LOCALAPPDATA") {
        Ok(val) => val,
        _ => return,
    };

    let pyke_cache_path = Path::new(&local_appdata)
        .join("ort.pyke.io")
        .join("dfbin")
        .join("x86_64-pc-windows-msvc");

    if !pyke_cache_path.exists() {
        return;
    }

    if let Some(src_dll) = find_dll_recursive(&pyke_cache_path) {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
            let target_dir = Path::new(&manifest_dir).join("target").join(&profile);
            let deps_dir = target_dir.join("deps");

            if target_dir.exists() {
                let _ = fs::create_dir_all(&deps_dir);
                let target_main_dll = target_dir.join("onnxruntime.dll");
                let _ = fs::copy(&src_dll, &target_main_dll);

                let target_deps_dll = deps_dir.join("onnxruntime.dll");
                let _ = fs::copy(&src_dll, &target_deps_dll);

                println!(
                    "cargo:warning=[自愈工坊] 已成功自动将 AppData 物理缓存的 ONNX 动态库投影至构建沙盒: {:?}",
                    target_deps_dll
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn find_dll_recursive(dir: &Path) -> Option<std::path::PathBuf> {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(found) = find_dll_recursive(&p) {
                        return Some(found);
                    }
                } else if p.is_file() && p.file_name().and_then(|s| s.to_str()) == Some("onnxruntime.dll") {
                    return Some(p);
                }
            }
        }
    }
    None
}
