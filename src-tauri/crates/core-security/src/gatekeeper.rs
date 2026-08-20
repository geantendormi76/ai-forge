use crate::{crypto, license::LicenseVerifier, DeviceFingerprint};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const OFFICIAL_ED25519_PUBLIC_KEY: &str =
    "a1385cf90acee43d496d5155315ef7ee34c4857c6e2bb502a2c7ee6115147820a1385cf90ace";

const AUTH_BASE_URL: &str = "https://ai.geantendormi.top";
const COMMERCIAL_SECRET: &str = "ai-forge-commercial-secret-2026";

#[derive(Debug, Serialize)]
struct QuotaDeductRequest<'a> {
    device_fingerprint: &'a str,
    points_needed: u32,
    tool_name: &'a str,
}

#[derive(Debug, Deserialize)]
struct QuotaDeductResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    remaining_points: Option<i64>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct QuotaLogRecord {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub tool_name: String,
    #[serde(default)]
    pub points_deducted: i64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct QuotaStatus {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub device_fingerprint: String,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub daily_limit: i64,
    #[serde(default)]
    pub used_today: i64,
    #[serde(default)]
    pub bonus_points: i64,
    #[serde(default)]
    pub remaining_points: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub is_offline_pro: bool,
    #[serde(default)]
    pub recent_logs: Vec<QuotaLogRecord>,
}

#[derive(Debug, Serialize)]
struct TelemetryPayload<'a> {
    device_fingerprint: &'a str,
    tool_name: &'a str,
    elapsed_ms: u64,
    success: bool,
    error_message: Option<&'a str>,
    client_version: &'a str,
}

pub struct Gatekeeper;

impl Gatekeeper {
    pub fn get_device_fingerprint() -> Result<String, String> {
        DeviceFingerprint::new()
            .add_cpu_info()
            .add_mac_address()
            .add_system_info()
            .generate(COMMERCIAL_SECRET)
            .map_err(|e| format!("指纹生成失败: {e}"))
    }

    /// 查询设备当前 Tokens 状态与最近账单流水
    pub async fn get_quota_status() -> Result<QuotaStatus, String> {
        let fingerprint = Self::get_device_fingerprint()?;

        let lic_path = Path::new("license.lic");
        let is_offline_pro = if lic_path.exists() {
            if let Ok(verifier) = LicenseVerifier::from_hex_public_key(OFFICIAL_ED25519_PUBLIC_KEY) {
                verifier.try_verify_file(lic_path, &fingerprint).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        if is_offline_pro {
            return Ok(QuotaStatus {
                success: true,
                device_fingerprint: fingerprint,
                stage: "offline_pro".to_string(),
                daily_limit: 999999,
                used_today: 0,
                bonus_points: 999999,
                remaining_points: 999999,
                status: "active".to_string(),
                is_offline_pro: true,
                recent_logs: vec![],
            });
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let body = serde_json::json!({
            "device_fingerprint": fingerprint
        });

        let res = client
            .post(format!("{AUTH_BASE_URL}/api/v1/quota/status"))
            .json(&body)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status_code = response.status();
                let text = response.text().await.unwrap_or_default();
                if status_code.is_success() {
                    if let Ok(mut status) = serde_json::from_str::<QuotaStatus>(&text) {
                        status.is_offline_pro = false;
                        return Ok(status);
                    }
                }
            }
            _ => {}
        }

        Ok(QuotaStatus {
            success: true,
            device_fingerprint: fingerprint,
            stage: "offline_fallback".to_string(),
            daily_limit: 600,
            used_today: 0,
            bonus_points: 0,
            remaining_points: 600,
            status: "active".to_string(),
            is_offline_pro: false,
            recent_logs: vec![],
        })
    }

    pub async fn check_permission_tokens(tool_name: &str, tokens_needed: u32) -> Result<(), String> {
        if tokens_needed == 0 || tool_name == "format-converter" || tool_name == "tool-format-convert" {
            return Ok(());
        }

        let fingerprint = Self::get_device_fingerprint()?;

        let lic_path = Path::new("license.lic");
        if lic_path.exists() {
            if let Ok(verifier) = LicenseVerifier::from_hex_public_key(OFFICIAL_ED25519_PUBLIC_KEY) {
                if verifier.try_verify_file(lic_path, &fingerprint).is_ok() {
                    return Ok(());
                }
            }
        }

        let req_payload = QuotaDeductRequest {
            device_fingerprint: &fingerprint,
            points_needed: tokens_needed,
            tool_name,
        };

        let body_bytes = serde_json::to_vec(&req_payload)
            .map_err(|e| format!("序列化请求体失败: {e}"))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let signature = crypto::compute_signature(COMMERCIAL_SECRET, timestamp, &body_bytes);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("HTTP 初始化失败: {e}"))?;

        let res = client
            .post(format!("{AUTH_BASE_URL}/api/v1/quota/deduct"))
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
                            tracing::info!("✅ [Gatekeeper] 扣减 {tokens_needed} Tokens 通过！今日剩余: {rem} Tokens");
                            return Ok(());
                        } else {
                            let err_msg = res_json.error.unwrap_or_else(|| "Tokens 额度不足".into());
                            return Err(format!("🚨 算力拦截: {err_msg}"));
                        }
                    }
                }

                if status.as_u16() == 402 || status.as_u16() == 403 || status.as_u16() == 401 {
                    return Err(format!("🚨 算力拦截 [HTTP {}]: {}", status.as_u16(), res_text));
                }

                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    pub async fn check_permission(tool_name: &str) -> Result<(), String> {
        Self::check_permission_tokens(tool_name, 1).await
    }

    pub async fn report_telemetry(
        tool_name: &str,
        elapsed_ms: u64,
        success: bool,
        error_message: Option<&str>,
    ) {
        let fingerprint = Self::get_device_fingerprint().unwrap_or_else(|_| "anonymous".into());
        let payload = TelemetryPayload {
            device_fingerprint: &fingerprint,
            tool_name,
            elapsed_ms,
            success,
            error_message,
            client_version: "1.0.0",
        };

        if let Ok(c) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(3)).build() {
            let _ = c.post(format!("{AUTH_BASE_URL}/api/v1/telemetry/report"))
                .json(&payload)
                .send()
                .await;
        }
    }
}
