use crate::{crypto, license::LicenseVerifier, DeviceFingerprint};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// 预留公钥：未来由管理员私钥签发后配置
const OFFICIAL_ED25519_PUBLIC_KEY: &str =
    "a1385cf90acee43d496d5155315ef7ee34c4857c6e2bb502a2c7ee6115147820a1385cf90ace";

#[derive(Debug, Serialize)]
struct QuotaDeductRequest<'a> {
    device_fingerprint: &'a str,
    points_needed: u32,
    tool_name: &'a str,
}

#[derive(Debug, Deserialize)]
struct QuotaDeductResponse {
    success: bool,
    remaining_points: Option<i64>,
    error: Option<String>,
}

pub struct Gatekeeper;

impl Gatekeeper {
    /// 算力收费站：在调起本地 Python 沙箱前，优先检查离线 .lic 许可；若无离线许可则向 Cloudflare 核销额度
    pub async fn check_permission(tool_name: &str) -> Result<(), String> {
        let secret = "ai-forge-commercial-secret-2026";

        // 1. 采集本地硬件指纹
        let fingerprint = DeviceFingerprint::new()
            .add_cpu_info()
            .add_mac_address()
            .add_system_info()
            .generate(secret)
            .map_err(|e| format!("指纹生成失败: {}", e))?;

        tracing::info!(
            "🛡️ [Gatekeeper] 触发算力鉴权 | 工具: {} | 设备指纹: {}",
            tool_name,
            fingerprint
        );

        // 2. 优先防线：校验本地是否存在 Ed25519 离线 .lic 授权文件 (政企/断网免扣费)
        let lic_path = Path::new("license.lic");
        if lic_path.exists() {
            if let Ok(verifier) = LicenseVerifier::from_hex_public_key(OFFICIAL_ED25519_PUBLIC_KEY) {
                if let Ok(lic_payload) = verifier.try_verify_file(lic_path, &fingerprint) {
                    tracing::info!(
                        "🎉 [Gatekeeper 离线授权通过] 方案: {} | 有效期至 UTC {}",
                        lic_payload.plan,
                        lic_payload.expires_at_utc
                    );
                    return Ok(());
                } else {
                    tracing::warn!("⚠️ [Gatekeeper] 本地 .lic 离线许可证无效或已被篡改，转入云端动态鉴权...");
                }
            }
        }

        // 3. 次要防线：向 Cloudflare 边缘节点核销额度
        let points_needed = match tool_name {
            "tool-pdf-parse" => 1,
            "tool-ASR" => 3,
            _ => 1,
        };

        let req_payload = QuotaDeductRequest {
            device_fingerprint: &fingerprint,
            points_needed,
            tool_name,
        };

        let body_bytes = serde_json::to_vec(&req_payload)
            .map_err(|e| format!("序列化请求体失败: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let signature = crypto::compute_signature(secret, timestamp, &body_bytes);

        let auth_url = "https://ai.geantendormi.top/api/v1/quota/deduct";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("HTTP 客户端初始化失败: {}", e))?;

        let res = client
            .post(auth_url)
            .header("Content-Type", "application/json")
            .header("X-Timestamp", timestamp.to_string())
            .header("X-Signature", signature)
            .body(body_bytes)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status = response.status();
                let res_text = response.text().await.unwrap_or_default();

                if status.is_success() {
                    if let Ok(res_json) = serde_json::from_str::<QuotaDeductResponse>(&res_text) {
                        if res_json.success {
                            let rem = res_json.remaining_points.unwrap_or(0);
                            tracing::info!(
                                "✅ [Gatekeeper] 云端鉴权通过！今日剩余额度: {} 点",
                                rem
                            );
                            return Ok(());
                        } else {
                            let err_msg = res_json.error.unwrap_or_else(|| "额度不足".into());
                            return Err(format!("🚨 算力拦截: {}", err_msg));
                        }
                    }
                }

                if status.as_u16() == 402 || status.as_u16() == 403 || status.as_u16() == 401 {
                    return Err(format!("🚨 算力拦截 [HTTP {}]: {}", status.as_u16(), res_text));
                }

                tracing::warn!("⚠️ [Gatekeeper 云端未连通] 状态码: {} | 弹性放行", status);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("⚠️ [Gatekeeper 离线模式] 无法连接云端鉴权中台 ({}) | 离线放行", e);
                Ok(())
            }
        }
    }
}
