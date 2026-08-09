use base64::prelude::*;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LicensePayload {
    pub device_fingerprint: String,
    pub plan: String,
    pub expires_at_utc: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseFile {
    pub payload: LicensePayload,
    pub signature_base64: String,
}

pub struct LicenseVerifier {
    verifying_key: VerifyingKey,
}

impl LicenseVerifier {
    pub fn from_hex_public_key(pub_key_hex: &str) -> Result<Self, String> {
        let key_bytes = hex::decode(pub_key_hex)
            .map_err(|e| format!("无效的公钥 Hex 格式: {}", e))?;

        let key_arr: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| "Ed25519 公钥必须正好是 32 字节".to_string())?;

        let verifying_key = VerifyingKey::from_bytes(&key_arr)
            .map_err(|e| format!("解析 Ed25519 公钥失败: {}", e))?;

        Ok(Self { verifying_key })
    }

    pub fn verify_license(
        &self,
        license_file: &LicenseFile,
        current_device_fingerprint: &str,
    ) -> Result<LicensePayload, String> {
        let payload_bytes = serde_json::to_vec(&license_file.payload)
            .map_err(|e| format!("Payload 序列化失败: {}", e))?;

        let sig_bytes = BASE64_STANDARD
            .decode(&license_file.signature_base64)
            .map_err(|e| format!("签名 Base64 解码失败: {}", e))?;

        let sig_arr: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| "Ed25519 签名必须正好是 64 字节".to_string())?;

        let signature = Signature::from_bytes(&sig_arr);

        self.verifying_key
            .verify(&payload_bytes, &signature)
            .map_err(|_| "🚨 [离线验签失败] 许可证签名不一致或被篡改！".to_string())?;

        if license_file.payload.device_fingerprint != current_device_fingerprint {
            return Err("🚨 [设备不匹配] 离线许可证不属于当前物理设备！".into());
        }

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now_sec > license_file.payload.expires_at_utc {
            return Err("🚨 [许可证已过期] 当前离线授权已到期！".into());
        }

        Ok(license_file.payload.clone())
    }

    pub fn try_verify_file(
        &self,
        license_path: &Path,
        current_device_fingerprint: &str,
    ) -> Result<LicensePayload, String> {
        if !license_path.exists() {
            return Err("许可证文件不存在".into());
        }

        let content = std::fs::read_to_string(license_path)
            .map_err(|e| format!("读取许可证文件失败: {}", e))?;

        let lic_file: LicenseFile = serde_json::from_str(&content)
            .map_err(|e| format!("反序列化 .lic 文件失败: {}", e))?;

        self.verify_license(&lic_file, current_device_fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn test_ed25519_license_verification() {
        // 1. 模拟密钥对生成 (由固定 32 字节私钥种子派生)
        let secret_bytes = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let pub_key_hex = hex::encode(verifying_key.to_bytes());

        let test_fingerprint = "a1385cf90acee43d496d5155315ef7ee34c4857c6e2bb502a2c7ee6115147820";

        // 2. 构建模拟 Payload 并签名
        let payload = LicensePayload {
            device_fingerprint: test_fingerprint.to_string(),
            plan: "enterprise_offline".to_string(),
            expires_at_utc: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 86400 * 365,
        };

        let payload_bytes = serde_json::to_vec(&payload).unwrap();
        let signature = signing_key.sign(&payload_bytes);
        let sig_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let lic_file = LicenseFile {
            payload: payload.clone(),
            signature_base64: sig_base64,
        };

        // 3. 客户端使用公钥验证
        let verifier = LicenseVerifier::from_hex_public_key(&pub_key_hex).unwrap();
        let verified = verifier
            .verify_license(&lic_file, test_fingerprint)
            .expect("Ed25519 离线许可证验证必须通过");

        assert_eq!(verified.plan, "enterprise_offline");
        println!("\n🎉 [Ed25519 验签测试成功] 离线 .lic 许可证验签完全正确！");
    }
}
