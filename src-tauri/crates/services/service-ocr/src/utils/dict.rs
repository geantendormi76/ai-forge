use crate::core::OCRError;
use std::path::Path;

pub fn read_character_dict(path: &Path) -> Result<Vec<String>, OCRError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        OCRError::ConfigError(format!(
            "Failed to read character dictionary from '{}': {}",
            path.display(),
            e
        ))
    })?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}
