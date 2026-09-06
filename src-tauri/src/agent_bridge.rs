// 🛡️ 纯血 Pi 官方 RPC 协议精准解构中枢 (agent_bridge.rs)
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared_contracts::VramTokenGuard;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Error)]
pub enum AgentBridgeError {
    #[error("找不到 pi.exe 可执行文件: {0:?}")]
    PiNotFound(PathBuf),
    #[error("子进程启动失败: {0}")]
    ProcessSpawnFailed(#[from] std::io::Error),
    #[error("Stdio 通信管道已断开: {0}")]
    PipeBroken(String),
    #[error("JSON-RPC 序列化/反序列化失败: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type AgentResult<T> = Result<T, AgentBridgeError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload")]
pub enum AgentEvent {
    ThinkingDelta(String),
    ContentDelta(String),
    ToolCallStarted { tool_name: String, call_id: String, input: Value },
    ToolCallFinished { tool_name: String, call_id: String, result: Value },
    TurnFinished { total_tokens: Option<usize>, elapsed_ms: u64 },
    Error(String),
}

pub struct AgentBridge {
    stdin: Arc<Mutex<ChildStdin>>,
    cancel_token: Arc<AtomicBool>,
    _vram_guard: Arc<VramTokenGuard>,
    _child: Arc<Mutex<Child>>,
}

impl AgentBridge {
    pub fn resolve_pi_executable() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\bin\pi.exe"),
            PathBuf::from(r"src-tauri\bin\pi.exe"),
            PathBuf::from(r"bin\pi.exe"),
            PathBuf::from(r"C:\dev\bin\pi.exe"),
        ];
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(parent) = current_exe.parent() {
                let p1 = parent.join("pi.exe");
                let p2 = parent.join("bin").join("pi.exe");
                if p1.exists() { return Some(p1); }
                if p2.exists() { return Some(p2); }
            }
        }
        candidates.into_iter().find(|p| p.exists())
    }

    pub async fn spawn(
        vram_guard: Arc<VramTokenGuard>,
        extension_path: Option<&Path>,
        working_dir: Option<&Path>,
    ) -> AgentResult<(Self, mpsc::Receiver<AgentEvent>)> {
        let pi_bin = Self::resolve_pi_executable()
            .ok_or_else(|| AgentBridgeError::PiNotFound(PathBuf::from("pi.exe")))?;
        tracing::info!("🚀 [AgentBridge] 正在拉起 Pi 官方 RPC 引擎: {:?}", pi_bin);

        let mut cmd = Command::new(&pi_bin);
        cmd.arg("--mode").arg("rpc")
            .arg("--provider").arg("local")
            .arg("--model").arg("abliterated_Qwen3-VL-8B-Instruct-IQ3_XXS")
            .arg("--approve")
            .arg("--no-session");

        if let Some(ext) = extension_path {
            if ext.exists() {
                cmd.arg("-e").arg(ext);
            }
        }

        if let Some(cwd) = working_dir {
            cmd.current_dir(cwd);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000); // 静默运行 CREATE_NO_WINDOW
        }

        let mut child = cmd.spawn().map_err(AgentBridgeError::ProcessSpawnFailed)?;
        let child_pid = child.id();
        crate::agent::process_guardian::protect_child_pid(child_pid);

        let stdin = child.stdin.take().ok_or_else(|| AgentBridgeError::PipeBroken("获取 stdin 失败".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| AgentBridgeError::PipeBroken("获取 stdout 失败".into()))?;

        let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(256);
        let cancel_token = Arc::new(AtomicBool::new(false));
        let cancel_token_clone = cancel_token.clone();

        tokio::spawn(async move {
            Self::event_pump(stdout, event_tx, cancel_token_clone).await;
        });

        let bridge = Self {
            stdin: Arc::new(Mutex::new(stdin)),
            cancel_token,
            _vram_guard: vram_guard,
            _child: Arc::new(Mutex::new(child)),
        };

        Ok((bridge, event_rx))
    }

    pub async fn send_prompt(&self, prompt: &str, _session_id: Option<&str>) -> AgentResult<()> {
        self.cancel_token.store(false, Ordering::SeqCst);
        let req = json!({
            "type": "prompt",
            "id": format!("req_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            "message": prompt
        });
        let mut payload = serde_json::to_string(&req)?;
        payload.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(payload.as_bytes()).await.map_err(|e| AgentBridgeError::PipeBroken(e.to_string()))?;
        stdin.flush().await.map_err(|e| AgentBridgeError::PipeBroken(e.to_string()))?;
        Ok(())
    }

    pub async fn abort(&self) -> AgentResult<()> {
        self.cancel_token.store(true, Ordering::SeqCst);
        let req = json!({
            "type": "abort",
            "id": "abort_current"
        });
        let mut payload = serde_json::to_string(&req)?;
        payload.push('\n');

        let mut stdin = self.stdin.lock().await;
        let _ = stdin.write_all(payload.as_bytes()).await;
        let _ = stdin.flush().await;
        Ok(())
    }

    /// 规整工具返回的真实业务载荷（兼容 details / data 嵌套）
    fn normalize_tool_result(raw: &Value) -> Value {
        if let Some(details) = raw.get("details") {
            if details.is_object() && !details.as_object().unwrap().is_empty() {
                return details.clone();
            }
        }
        if let Some(data) = raw.get("data") {
            if data.is_object() {
                return data.clone();
            }
        }
        raw.clone()
    }

    async fn event_pump(
        stdout: ChildStdout,
        tx: mpsc::Sender<AgentEvent>,
        cancel_token: Arc<AtomicBool>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let start_time = std::time::Instant::now();

        while let Ok(Some(line)) = lines.next_line().await {
            if cancel_token.load(Ordering::Relaxed) {
                let _ = tx.send(AgentEvent::Error("任务已终止".into())).await;
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if let Some(msg_type) = val.get("type").and_then(Value::as_str) {
                    match msg_type {
                        // 1. 实时流式增量提取
                        "message_update" => {
                            if let Some(ev) = val.get("event") {
                                if let Some(ev_type) = ev.get("type").and_then(Value::as_str) {
                                    match ev_type {
                                        "thinking_delta" => {
                                            if let Some(delta) = ev.get("delta").and_then(Value::as_str) {
                                                let _ = tx.send(AgentEvent::ThinkingDelta(delta.to_string())).await;
                                            }
                                        }
                                        "text_delta" => {
                                            if let Some(delta) = ev.get("delta").and_then(Value::as_str) {
                                                let _ = tx.send(AgentEvent::ContentDelta(delta.to_string())).await;
                                            }
                                        }
                                        "tool_result" => {
                                            let call_id = ev.get("toolCallId").or_else(|| ev.get("id")).and_then(Value::as_str).unwrap_or("");
                                            let tool_name = ev.get("toolName").or_else(|| ev.get("name")).and_then(Value::as_str).unwrap_or("tool");
                                            let raw_res = ev.get("result").unwrap_or(&Value::Null);
                                            let normalized = Self::normalize_tool_result(raw_res);

                                            tracing::info!("✅ [AgentBridge] 收到 message_update.tool_result -> call_id: {}, 算子: {}", call_id, tool_name);
                                            let _ = tx.send(AgentEvent::ToolCallFinished {
                                                tool_name: tool_name.to_string(),
                                                call_id: call_id.to_string(),
                                                result: normalized,
                                            }).await;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        // 2. 工具调用启动
                        "tool_execution_start" | "tool_call" => {
                            let tool_name = val.get("toolName").or_else(|| val.get("name")).and_then(Value::as_str).unwrap_or("tool");
                            let call_id = val.get("toolCallId").or_else(|| val.get("id")).and_then(Value::as_str).unwrap_or("");
                            let input = val.get("input").or_else(|| val.get("arguments")).cloned().unwrap_or(json!({}));

                            tracing::info!("⚡ [AgentBridge] 工具执行开始 -> 算子: {}, call_id: {}", tool_name, call_id);
                            let _ = tx.send(AgentEvent::ToolCallStarted {
                                tool_name: tool_name.to_string(),
                                call_id: call_id.to_string(),
                                input,
                            }).await;
                        }
                        // 3. 工具调用完成
                        "tool_execution_end" => {
                            let tool_name = val.get("toolName").or_else(|| val.get("name")).and_then(Value::as_str).unwrap_or("tool");
                            let call_id = val.get("toolCallId").or_else(|| val.get("id")).and_then(Value::as_str).unwrap_or("");
                            let raw_res = val.get("result").unwrap_or(&Value::Null);
                            let normalized = Self::normalize_tool_result(raw_res);

                            tracing::info!("✅ [AgentBridge] 工具执行完成 -> 算子: {}, call_id: {}", tool_name, call_id);
                            let _ = tx.send(AgentEvent::ToolCallFinished {
                                tool_name: tool_name.to_string(),
                                call_id: call_id.to_string(),
                                result: normalized,
                            }).await;
                        }
                        // 4. 回合收敛完成报文
                        "agent_end" | "turn_finish" => {
                            if let Some(msgs) = val.get("messages").and_then(Value::as_array) {
                                for msg in msgs {
                                    if msg.get("role").and_then(Value::as_str) == Some("assistant") {
                                        if let Some(contents) = msg.get("content").and_then(Value::as_array) {
                                            for item in contents {
                                                let c_type = item.get("type").and_then(Value::as_str).unwrap_or("");
                                                if c_type == "thinking" {
                                                    if let Some(th) = item.get("thinking").and_then(Value::as_str) {
                                                        let _ = tx.send(AgentEvent::ThinkingDelta(th.to_string())).await;
                                                    }
                                                } else if c_type == "text" {
                                                    if let Some(txt) = item.get("text").and_then(Value::as_str) {
                                                        let _ = tx.send(AgentEvent::ContentDelta(txt.to_string())).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            let elapsed = start_time.elapsed().as_millis() as u64;
                            tracing::info!("🏁 [AgentBridge] 本轮回合执行完成, 耗时: {} ms", elapsed);
                            let _ = tx.send(AgentEvent::TurnFinished {
                                total_tokens: None,
                                elapsed_ms: elapsed,
                            }).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
