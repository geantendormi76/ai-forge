// 🛡️ 紫电 AI - 纯血 GGUF 模型资产嗅探与热拔插核心服务 (model_hub.rs)
use core_models_download::ModelManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GgufModelInfo {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub supports_vision: bool,
    pub mmproj_path: Option<String>,
    pub is_active: bool,
}

pub struct GgufModelHub;

impl GgufModelHub {
    /// 解析模型存储根目录 (优先 C:\dev\ai-forge\models，其次开发工作区自愈目录)
    pub fn resolve_models_dir() -> PathBuf {
        let hardcoded = PathBuf::from(r"C:\dev\ai-forge\models");
        if hardcoded.exists() && hardcoded.is_dir() {
            return hardcoded;
        }
        ModelManager::resolve_models_base_dir()
    }

    /// 获取当前持久化保存的激活模型 ID 记录路径
    fn resolve_active_marker_path() -> PathBuf {
        Self::resolve_models_dir().join("active_model.txt")
    }

    /// 读取当前已激活的模型 ID
    pub fn get_current_active_model_id() -> String {
        let marker = Self::resolve_active_marker_path();
        if marker.exists() {
            if let Ok(id) = fs::read_to_string(&marker) {
                let trimmed = id.trim().to_string();
                if !trimmed.is_empty() {
                    return trimmed;
                }
            }
        }
        "abliterated_Qwen3-VL-8B-Instruct-IQ3_XXS".to_string()
    }

    /// 从模型文件名中提取核心规模特征 (如 "8b", "7b", "14b", "2b", "0.5b")
    fn extract_param_size(name: &str) -> Option<String> {
        let lower = name.to_lowercase();
        let tokens: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric() && c != '.').filter(|s| !s.is_empty()).collect();
        for t in tokens {
            if t.ends_with('b') && t.len() >= 2 {
                let num_part = &t[..t.len() - 1];
                if num_part.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    return Some(t.to_string());
                }
            }
        }
        None
    }

    /// 清洗模型名称，剥离量化标记、前缀与后缀，提取纯净骨干 Token 集合
    fn extract_backbone_tokens(name: &str) -> HashSet<String> {
        let lower = name.to_lowercase();
        let noise_words: HashSet<&str> = [
            "mmproj", "abliterated", "instruct", "chat", "gguf",
            "iq3", "iq3_xxs", "iq3_s", "q4_k_m", "q4_0", "q4_1",
            "q5_k_m", "q8_0", "f16", "bf16", "fp16", "int4", "int8"
        ].into_iter().collect();

        lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && !noise_words.contains(*s))
            .map(str::to_string)
            .collect()
    }

    /// 双阶贪心眼球配对算法 (SOTA 参数量隔离 + Jaccard 骨干相似度贪心打分)
    pub fn find_best_mmproj<'a>(model_stem: &str, mmproj_candidates: &'a [PathBuf]) -> Option<&'a PathBuf> {
        let model_param_size = Self::extract_param_size(model_stem);
        let model_tokens = Self::extract_backbone_tokens(model_stem);

        let mut best_candidate: Option<&'a PathBuf> = None;
        let mut best_score = 0.0f32;

        for candidate in mmproj_candidates {
            let mm_stem = candidate.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let mm_param_size = Self::extract_param_size(mm_stem);

            // 🛡️ 第一阶门禁：参数规模一票否决防呆线
            if let (Some(ref m_size), Some(ref mm_size)) = (&model_param_size, &mm_param_size) {
                if m_size != mm_size {
                    continue;
                }
            }

            // 🛡️ 第二阶门禁：Jaccard 骨干 Token 贪心重合度打分
            let mm_tokens = Self::extract_backbone_tokens(mm_stem);
            let intersection = model_tokens.intersection(&mm_tokens).count();
            let union = model_tokens.union(&mm_tokens).count();

            if union == 0 {
                continue;
            }

            let jaccard_score = intersection as f32 / union as f32;

            if jaccard_score > best_score && jaccard_score >= 0.4 {
                best_score = jaccard_score;
                best_candidate = Some(candidate);
            }
        }

        best_candidate
    }

    /// 嗅探当前硬盘中所有可供加载的主脑 GGUF 模型并智能关联 mmproj 视觉投影器
    pub fn scan_local_models(active_model_id: Option<&str>) -> Vec<GgufModelInfo> {
        let dir = Self::resolve_models_dir();
        let mut models = Vec::new();
        let mut mmproj_files = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            tracing::warn!("⚠️ [ModelHub] 模型根目录不存在: {:?}", dir);
            return models;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(err) => {
                tracing::error!("🚨 [ModelHub] 遍历模型目录失败: {:?}", err);
                return models;
            }
        };

        let mut candidate_models = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_file() { continue; }
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if ext != "gguf" { continue; }

            let file_name = p.file_name().and_then(|f| f.to_str()).unwrap_or("").to_string();
            if file_name.starts_with("mmproj-") {
                mmproj_files.push(p);
            } else {
                candidate_models.push(p);
            }
        }

        let current_active = active_model_id
            .map(str::to_string)
            .unwrap_or_else(Self::get_current_active_model_id);

        for p in candidate_models {
            let file_name = p.file_name().and_then(|f| f.to_str()).unwrap_or("").to_string();
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(&file_name).to_string();
            let meta = fs::metadata(&p).ok();
            let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let size_formatted = ModelManager::format_bytes(size_bytes);

            let matched_mmproj = Self::find_best_mmproj(&stem, &mmproj_files);

            let (supports_vision, mmproj_path) = if let Some(mm) = matched_mmproj {
                (true, Some(mm.to_string_lossy().to_string()))
            } else {
                (false, None)
            };

            let is_active = stem == current_active;

            models.push(GgufModelInfo {
                id: stem.clone(),
                name: stem,
                file_name,
                file_path: p.to_string_lossy().to_string(),
                size_bytes,
                size_formatted,
                supports_vision,
                mmproj_path,
                is_active,
            });
        }

        models.sort_by(|a, b| a.name.cmp(&b.name));
        models
    }

    /// 一键热拔插：切换当前活动模型，自动重构 Pi 的 models.json 并持久化状态
    pub fn switch_active_model(target_model_id: &str) -> Result<GgufModelInfo, String> {
        let all_models = Self::scan_local_models(None);
        let target = all_models
            .into_iter()
            .find(|m| m.id == target_model_id)
            .ok_or_else(|| format!("未在模型目录中找到目标模型: {}", target_model_id))?;

        // 1. 持久化当前选中标记
        let marker = Self::resolve_active_marker_path();
        if let Err(e) = fs::write(&marker, &target.id) {
            tracing::warn!("⚠️ [ModelHub] 写入 active_model.txt 失败: {:?}", e);
        }

        // 2. 构建针对 Pi 官方协议的标准配置 JSON 结构
        let input_modalities = if target.supports_vision {
            vec!["text", "image"]
        } else {
            vec!["text"]
        };

        let config_payload = json!({
            "providers": {
                "local": {
                    "baseUrl": "http://127.0.0.1:8000/v1",
                    "api": "openai-completions",
                    "apiKey": "local-key",
                    "models": [
                        {
                            "id": target.id,
                            "name": target.file_name,
                            "reasoning": true,
                            "input": input_modalities,
                            "contextWindow": 16384,
                            "maxTokens": 4096
                        }
                    ]
                }
            }
        });

        let json_str = serde_json::to_string_pretty(&config_payload)
            .map_err(|e| format!("序列化 models.json 失败: {e}"))?;

        // 3. 纯血标准库获取用户主目录，原子更新工作区与系统全局 models.json
        let user_home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from);

        let target_paths = [
            Some(PathBuf::from(r"C:\dev\ai-forge\.pi\models.json")),
            user_home.map(|h| h.join(".pi").join("agent").join("models.json")),
        ];

        for path_opt in target_paths.into_iter().flatten() {
            if let Some(parent) = path_opt.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(&path_opt, &json_str) {
                tracing::warn!("⚠️ [ModelHub] 写入 {:?} 失败: {:?}", path_opt, e);
            } else {
                tracing::info!("✅ [ModelHub] 成功热拔插同步写入契约: {:?}", path_opt);
            }
        }

        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy_mmproj_anti_mistake_matching() {
        println!("\n🛡️ ===== [GgufModelHub 双阶贪心防呆匹配单元测试] =====");
        let candidates = vec![
            PathBuf::from(r"C:\dev\ai-forge\models\mmproj-Qwen3-VL-7B-Instruct-F16.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\mmproj-abliterated_Qwen3-VL-8B-Instruct-BF16.gguf"),
            PathBuf::from(r"C:\dev\ai-forge\models\mmproj-MiniCPM-V-2_6-BF16.gguf"),
        ];

        let m8b = "abliterated_Qwen3-VL-8B-Instruct-IQ3_XXS";
        let matched_8b = GgufModelHub::find_best_mmproj(m8b, &candidates).expect("8B 必须命中配对眼球");
        assert!(matched_8b.to_string_lossy().contains("8B"), "8B 主脑绝不能错配非 8B 眼球！");

        let m7b = "Qwen3-VL-7B-Instruct-Q4_K_M";
        let matched_7b = GgufModelHub::find_best_mmproj(m7b, &candidates).expect("7B 必须命中配对眼球");
        assert!(matched_7b.to_string_lossy().contains("7B"), "7B 主脑绝不能错配非 7B 眼球！");

        let m_deepseek = "DeepSeek-R1-Distill-Qwen-8B-Q4_K_M";
        let matched_ds = GgufModelHub::find_best_mmproj(m_deepseek, &candidates);
        assert!(matched_ds.is_none(), "非 VL 纯文本模型不应盲目挂载视觉眼球");
        println!("======================================================\n");
    }

    #[test]
    fn test_scan_and_switch_model_pipeline() {
        println!("\n🔄 ===== [GgufModelHub 热拔插切换自动化测试] =====");
        let target_id = "abliterated_Qwen3-VL-8B-Instruct-IQ3_XXS";
        let switched = GgufModelHub::switch_active_model(target_id).expect("一键热拔插切换必须成功");
        assert_eq!(switched.id, target_id);
        assert!(switched.supports_vision);
        println!("  🎉 成功将主脑热拔插切换为: {} (视觉支持: {})", switched.id, switched.supports_vision);

        let active_id = GgufModelHub::get_current_active_model_id();
        assert_eq!(active_id, target_id, "持久化记录的激活 ID 必须一致！");
        println!("======================================================\n");
    }
}
