use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

#[derive(thiserror::Error, Debug)]
pub enum IpcError {
    #[error("IO 异常: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 序列化异常: {0}")]
    Json(#[from] serde_json::Error),
    #[error("子进程已意外退出")]
    ProcessExited,
    #[error("帧结构异常或超载")]
    InvalidFraming,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpcMessage {
    pub method: String,
    pub params: serde_json::Value,
}

pub struct IpcChannel {
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl IpcChannel {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        Self { stdin, stdout }
    }

    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), IpcError> {
        let payload = serde_json::to_vec(msg)?;
        let len = payload.len() as u32;
        self.stdin.write_u32(len).await?;
        self.stdin.write_all(&payload).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<IpcMessage, IpcError> {
        let len = match self.stdout.read_u32().await {
            Ok(l) => l,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Err(IpcError::ProcessExited),
            Err(e) => return Err(IpcError::Io(e)),
        };

        if len > 50 * 1024 * 1024 {
            return Err(IpcError::InvalidFraming);
        }

        let mut buf = vec![0u8; len as usize];
        self.stdout.read_exact(&mut buf).await?;

        let msg = serde_json::from_slice(&buf)?;
        Ok(msg)
    }
}

pub fn resolve_uv_binary() -> PathBuf {
    let binary_name = if cfg!(windows) { "uv.exe" } else { "uv" };

    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        let p = PathBuf::from(cargo_home).join("bin").join(binary_name);
        if p.exists() {
            return p;
        }
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(user_profile).join(".cargo").join("bin").join(binary_name);
        if p.exists() {
            return p;
        }
    }

    let candidates = [
        PathBuf::from(r"C:\Users\52484\.cargo\bin\uv.exe"),
        PathBuf::from("/home/zhz/.cargo/bin/uv"),
        PathBuf::from("/home/zhz/.local/bin/uv"),
        PathBuf::from("/usr/local/bin/uv"),
        PathBuf::from("/usr/bin/uv"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    PathBuf::from(binary_name)
}

/// 普通二进制进程拉起
pub fn spawn_worker(executable_path: &str, script_path: &str) -> Result<(Child, IpcChannel), IpcError> {
    let mut child = Command::new(executable_path)
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.take().ok_or(IpcError::InvalidFraming)?;
    let stdout = child.stdout.take().ok_or(IpcError::InvalidFraming)?;

    Ok((child, IpcChannel::new(stdin, stdout)))
}

/// 🛡️ SOTA: 严格遵循 Theorem 1 与 Theorem 2，使用 `--no-sync` 与离线沙箱拉起 Worker
pub fn spawn_uv_worker(project_dir: &Path, script_path: &Path) -> Result<(Child, IpcChannel), IpcError> {
    let uv_binary = resolve_uv_binary();
    let python_bin = if cfg!(windows) { "python" } else { "python3" };

    let mut child = Command::new(&uv_binary)
        .arg("run")
        .arg("--no-sync") // 🛡️ SOTA 离线隔离：禁止运行时网络/锁校验
        .arg("--project")
        .arg(project_dir)
        .arg(python_bin) // 🛡️ Windows Native: 使用 "python" 避开 Microsoft Store 虚假别名
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child.stdin.take().ok_or(IpcError::InvalidFraming)?;
    let stdout = child.stdout.take().ok_or(IpcError::InvalidFraming)?;

    Ok((child, IpcChannel::new(stdin, stdout)))
}
