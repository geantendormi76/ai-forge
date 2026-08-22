pub mod crypto;
pub mod gatekeeper;
pub mod license;
pub mod portable;

pub use portable::PortableEngine;

use hmac::{Hmac, Mac};
use mac_address::get_mac_address;
use sha2::Sha256;
use sysinfo::System;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("HMAC 加密初始化失败: {0}")]
    HmacError(String),
}

type HmacSha256 = Hmac<Sha256>;

pub struct DeviceFingerprint {
    components: Vec<String>,
}

impl Default for DeviceFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceFingerprint {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn add_cpu_info(mut self) -> Self {
        let sys = System::new_all();
        if let Some(cpu) = sys.cpus().first() {
            self.components.push(format!("{}-{}", cpu.vendor_id(), cpu.brand()));
        } else {
            self.components.push("UNKNOWN_CPU".to_string());
        }
        self
    }

    pub fn add_mac_address(mut self) -> Self {
        match get_mac_address() {
            Ok(Some(mac)) => self.components.push(mac.to_string()),
            _ => self.components.push("UNKNOWN_MAC".to_string()),
        }
        self
    }

    pub fn add_system_info(mut self) -> Self {
        let host_name = System::host_name().unwrap_or_else(|| "UNKNOWN_HOST".to_string());
        let os_name = System::name().unwrap_or_else(|| "UNKNOWN_OS".to_string());
        self.components.push(format!("{}-{}", host_name, os_name));
        self
    }

    pub fn generate(self, secret_key: &str) -> Result<String, SecurityError> {
        let raw_string = self.components.join("|");
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .map_err(|e| SecurityError::HmacError(e.to_string()))?;
        mac.update(raw_string.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }
}
