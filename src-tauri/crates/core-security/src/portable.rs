//! 🛡️ 紫电 AI - 企业级绿色便携免安装协议 (portable.rs)
//! 当可执行程序同级存在 `portable` 标记文件时，自动将所有数据、模型和输出重定向至 `./Data/` 目录。
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PORTABLE_DATA_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
pub const PORTABLE_MAGIC_MARKER: &str = "ZiDianAI Portable Mode";

pub struct PortableEngine;

impl PortableEngine {
    /// 初始化便携模式探测（启动时自动全局初始化一次）
    pub fn init() -> Option<PathBuf> {
        PORTABLE_DATA_DIR
            .get_or_init(|| {
                let exe_path = std::env::current_exe().ok()?;
                let exe_dir = exe_path.parent()?;

                let marker_path = exe_dir.join("portable");
                let data_dir = exe_dir.join("Data");

                let is_portable = if Self::is_valid_marker(&marker_path) {
                    true
                } else if marker_path.exists() && data_dir.exists() {
                    // 兼容旧版空标记升级
                    let _ = std::fs::write(&marker_path, PORTABLE_MAGIC_MARKER);
                    true
                } else {
                    false
                };

                if is_portable {
                    if !data_dir.exists() {
                        let _ = std::fs::create_dir_all(&data_dir);
                    }
                    tracing::info!("🎒 [便携模式已激活] 生产数据全部重定向至: {:?}", data_dir);
                    Some(data_dir)
                } else {
                    None
                }
            })
            .clone()
        }

    /// 目录标记探测判定（支持传入任意测试目录进行白盒断言）
    pub fn detect_marker_in_dir(dir: &Path) -> Option<PathBuf> {
        let marker_path = dir.join("portable");
        let data_dir = dir.join("Data");

        if Self::is_valid_marker(&marker_path) {
            Some(data_dir)
        } else if marker_path.exists() && data_dir.exists() {
            Some(data_dir)
        } else {
            None
        }
    }

    /// 校验标记文件内容
    pub fn is_valid_marker(marker_path: &Path) -> bool {
        if !marker_path.exists() {
            return false;
        }
        if let Ok(content) = std::fs::read_to_string(marker_path) {
            let trimmed = content.trim();
            trimmed.starts_with(PORTABLE_MAGIC_MARKER)
                || trimmed.starts_with("Handy Portable Mode")
                || trimmed == "portable"
        } else {
            false
        }
    }

    /// 当前环境是否处于便携模式
    pub fn is_portable() -> bool {
        PORTABLE_DATA_DIR.get().and_then(|v| v.as_ref()).is_some()
    }

    /// 获取便携模式下的 Data 绝对根目录
    pub fn data_dir() -> Option<PathBuf> {
        PORTABLE_DATA_DIR.get().and_then(|v| v.clone())
    }

    /// 重定向子模块路径（例如 "models", "outputs", "logs"）
    pub fn resolve_data_path(sub_path: &str) -> Option<PathBuf> {
        Self::data_dir().map(|d| d.join(sub_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_valid_magic_marker_detection() {
        let temp_dir = std::env::temp_dir().join("zidian_test_portable_valid");
        let _ = fs::create_dir_all(&temp_dir);
        let marker_file = temp_dir.join("portable");
        fs::write(&marker_file, PORTABLE_MAGIC_MARKER).unwrap();

        let detected = PortableEngine::detect_marker_in_dir(&temp_dir);
        assert_eq!(detected, Some(temp_dir.join("Data")));
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_empty_marker_without_data_fails() {
        let temp_dir = std::env::temp_dir().join("zidian_test_portable_empty");
        let _ = fs::create_dir_all(&temp_dir);
        let marker_file = temp_dir.join("portable");
        fs::write(&marker_file, "").unwrap();

        // 仅有空文件且没有 Data 目录时不应判定为便携模式
        let detected = PortableEngine::detect_marker_in_dir(&temp_dir);
        assert_eq!(detected, None);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_resolve_data_subpaths() {
        let temp_dir = std::env::temp_dir().join("zidian_test_portable_subpaths");
        let data_dir = temp_dir.join("Data");
        let _ = fs::create_dir_all(&data_dir);
        let marker_file = temp_dir.join("portable");
        fs::write(&marker_file, "ZiDianAI Portable Mode\n").unwrap();

        assert!(PortableEngine::is_valid_marker(&marker_file));
        assert_eq!(PortableEngine::detect_marker_in_dir(&temp_dir), Some(data_dir));
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
