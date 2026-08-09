use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// 计算 HMAC-SHA256 签名 (用于桌面端向云端发起鉴权请求时，证明请求未被篡改)
pub fn compute_signature(secret: &str, timestamp: u64, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC 支持任意长度密钥");
    mac.update(&timestamp.to_be_bytes());
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}
