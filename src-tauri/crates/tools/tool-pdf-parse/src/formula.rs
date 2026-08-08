#![allow(non_snake_case, dead_code, non_camel_case_types)]

use std::path::Path;

pub struct FormulaEngine;

impl FormulaEngine {
    pub fn load_vocab(vocab_json_path: &Path) -> Vec<String> {
        if !vocab_json_path.exists() {
            return Vec::new();
        }
        let content = match std::fs::read_to_string(vocab_json_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        serde_json::from_str(&content).unwrap_or_default()
    }
}
