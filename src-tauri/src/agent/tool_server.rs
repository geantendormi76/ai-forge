// 🛡️ 紫电 AI - 本地原子工具高性能 HTTP 直通服务 (tool_server.rs)
use super::dispatcher;
use serde::Deserialize;
use serde_json::{json, Value};
use shared_contracts::VramTokenGuard;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
struct ToolRpcRequest {
    tool_name: String,
    params: Value,
}

pub struct ToolRpcServer;

impl ToolRpcServer {
    /// 启动常驻后台工具直通服务器 (127.0.0.1:18000)
    pub fn start(vram_guard: Arc<VramTokenGuard>) {
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("🚨 构建 ToolRpcServer Tokio 运行时失败: {e}");
                    return;
                }
            };

            rt.block_on(async move {
                let addr = "127.0.0.1:18000";
                let listener = match TcpListener::bind(addr).await {
                    Ok(l) => {
                        tracing::info!("🚀 [ToolRpcServer] 原子工具直通网络总线已就绪: http://{}", addr);
                        l
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ [ToolRpcServer] 绑定端口 18000 失败 (可能已在运行): {e}");
                        return;
                    }
                };

                loop {
                    if let Ok((mut socket, _)) = listener.accept().await {
                        let vram_clone = vram_guard.clone();
                        tokio::spawn(async move {
                            let mut buf = vec![0u8; 65536];
                            let mut total_read = 0;

                            while let Ok(n) = socket.read(&mut buf[total_read..]).await {
                                if n == 0 { break; }
                                total_read += n;
                                if buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
                                    break;
                                }
                                if total_read >= buf.len() { break; }
                            }

                            let req_str = String::from_utf8_lossy(&buf[..total_read]);
                            let body_idx = req_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
                            let body_str = &req_str[body_idx..];

                            let response_body = if let Ok(req) = serde_json::from_str::<ToolRpcRequest>(body_str.trim()) {
                                match dispatcher::dispatch(&req.tool_name, req.params, vram_clone).await {
                                    Ok(res_val) => serde_json::to_string(&json!({ "success": true, "data": res_val })).unwrap(),
                                    Err(err) => serde_json::to_string(&json!({ "success": false, "error": err })).unwrap(),
                                }
                            } else {
                                serde_json::to_string(&json!({ "success": false, "error": "无效的 ToolRpcRequest 格式" })).unwrap()
                            };

                            let http_resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                response_body.as_bytes().len(),
                                response_body
                            );

                            let _ = socket.write_all(http_resp.as_bytes()).await;
                            let _ = socket.flush().await;
                        });
                    }
                }
            });
        });
    }
}
