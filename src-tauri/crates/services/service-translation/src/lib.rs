use core_ipc::{spawn_uv_worker, IpcMessage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PureTranslationRequest {
    pub texts: Vec<String>,
    pub target_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PureTranslationResponse {
    pub translations: Vec<String>,
    pub elapsed_ms: f64,
}

pub struct TranslationService;

impl TranslationService {
    fn resolve_service_dir() -> PathBuf {
        let candidates = [
            PathBuf::from(r"C:\dev\ai-forge\src-tauri\crates\services\service-translation"),
            PathBuf::from(r"C:\dev\ai-toolkit\src-tauri\crates\services\service-translation"),
            PathBuf::from("/home/zhz/ai-forge/src-tauri/crates/services/service-translation"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return PathBuf::from(manifest_dir);
        }
        PathBuf::from("src-tauri/crates/services/service-translation")
    }

    /// 纯净翻译管道：只接收字符串列表 ➔ 只输出字符串列表，0 领域/字幕污染
    pub async fn run_translation_pipeline(
        req: PureTranslationRequest,
    ) -> Result<PureTranslationResponse, String> {
        if req.texts.is_empty() {
            return Ok(PureTranslationResponse {
                translations: Vec::new(),
                elapsed_ms: 0.0,
            });
        }

        let service_dir = Self::resolve_service_dir();
        let script_path = service_dir.join("scripts/worker.py");

        let (mut child, mut channel) = spawn_uv_worker(&service_dir, &script_path)
            .map_err(|e| format!("启动 service-translation 进程失败: {e}"))?;

        let ready_msg = channel.recv().await
            .map_err(|e| format!("接收 Translation Worker 就绪信号失败: {e}"))?;

        if ready_msg.method != "system.ready" {
            tracing::warn!("⚠️ 收到非预期就绪信号: {}", ready_msg.method);
        }

        let req_payload = serde_json::json!({
            "texts": req.texts,
            "target_lang": req.target_lang.unwrap_or_else(|| "Chinese".into())
        });

        let req_msg = IpcMessage {
            method: "translate".to_string(),
            params: req_payload,
        };

        let t0 = std::time::Instant::now();
        channel.send(&req_msg).await
            .map_err(|e| format!("发送翻译请求失败: {e}"))?;

        let res_msg = channel.recv().await
            .map_err(|e| format!("读取翻译响应失败: {e}"))?;

        let exit_msg = IpcMessage {
            method: "exit".to_string(),
            params: serde_json::json!({}),
        };
        let _ = channel.send(&exit_msg).await;
        let _ = child.wait().await;

        if res_msg.method == "error" {
            let err_msg = res_msg.params["message"].as_str().unwrap_or("翻译推演发生错误");
            return Err(format!("Hy-MT2 翻译失败: {err_msg}"));
        }

        let elapsed_ms = t0.elapsed().as_millis() as f64;

        let result_val = &res_msg.params["result"];
        let translations: Vec<String> = serde_json::from_value(result_val["translations"].clone())
            .map_err(|e| format!("解析 translations 响应失败: {e}"))?;

        Ok(PureTranslationResponse {
            translations,
            elapsed_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_purity() {
        let req = PureTranslationRequest {
            texts: vec!["Hello world".into()],
            target_lang: Some("Chinese".into()),
        };
        let json_str = serde_json::to_string(&req).unwrap();
        assert!(!json_str.contains("speaker"));
        assert!(!json_str.contains("start_sec"));
        println!("\n✅ [service-translation 纯净性测试通过] 零领域污染数据契约: {}", json_str);
    }

    fn resolve_asr_output_path(filename: &str) -> PathBuf {
        let candidates = [
            PathBuf::from(format!(r"C:\dev\ai-forge\test\outs\service-asr\{}", filename)),
            PathBuf::from(format!(r"C:\dev\ai-toolkit\test\outs\service-asr\{}", filename)),
            PathBuf::from(format!("/home/zhz/ai-forge/test/outs/service-asr/{}", filename)),
        ];
        for candidate in candidates {
            if candidate.exists() {
                return candidate;
            }
        }
        PathBuf::from(format!(r"C:\dev\ai-forge\test\outs\service-asr\{}", filename))
    }

    fn resolve_outs_dir() -> PathBuf {
        let candidate = PathBuf::from(r"C:\dev\ai-forge\test\outs\service-translation");
        let _ = std::fs::create_dir_all(&candidate);
        candidate
    }

    #[tokio::test]
    async fn test_service_translation_e2e_from_english_asr() {
        let fixture_json = resolve_asr_output_path("asr_english_pure.json");
        if !fixture_json.exists() {
            println!("⚠️ [跳过测试] 纯净英文 ASR JSON 不存在: {:?}", fixture_json);
            return;
        }

        let outs_dir = resolve_outs_dir();

        println!("\n🚀 [TDD 纯净打靶 - 混元 Hy-MT2 1.8B] 正在读取英文纯净 ASR JSON，批量翻译为中文...");

        let json_content = std::fs::read_to_string(&fixture_json).unwrap();
        let asr_val: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        let raw_segs = asr_val["segments"].as_array().expect("segments 必须是数组");
        let english_texts: Vec<String> = raw_segs
            .iter()
            .map(|s| s["text"].as_str().unwrap_or("").to_string())
            .collect();

        let req = PureTranslationRequest {
            texts: english_texts.clone(),
            target_lang: Some("Chinese".into()),
        };

        let res = TranslationService::run_translation_pipeline(req)
            .await
            .expect("Hy-MT2 1.8B 英文跨语言翻译失败");

        println!("\n🎉 ===== [Hy-MT2 1.8B 纯净英译中 翻译完成] =====");
        println!("  ⏱️ 物理总耗时: {:.2} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms / 1000.0);
        println!("  翻译总句数: {} 句", res.translations.len());

        assert_eq!(res.translations.len(), english_texts.len());

        let out_json_path = outs_dir.join("english_to_chinese_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 纯净英译中 JSON 产物落盘至: {:?}", out_json_path);
        println!("  预览前 3 句英文➔中文翻译效果:");
        for i in 0..3.min(english_texts.len()) {
            println!("     [{}] 英文: {} ➔ 中文: {}", i + 1, english_texts[i], res.translations[i]);
        }
        println!("============================================\n");
    }

    #[tokio::test]
    async fn test_service_translation_e2e_from_japanese_asr() {
        let fixture_json = resolve_asr_output_path("asr_japanese_pure.json");
        if !fixture_json.exists() {
            println!("⚠️ [跳过测试] 纯净日文 ASR JSON 不存在: {:?}", fixture_json);
            return;
        }

        let outs_dir = resolve_outs_dir();

        println!("\n🚀 [TDD 纯净打靶 - 混元 Hy-MT2 1.8B] 正在读取日文纯净 ASR JSON，批量翻译为中文...");

        let json_content = std::fs::read_to_string(&fixture_json).unwrap();
        let asr_val: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        let raw_segs = asr_val["segments"].as_array().expect("segments 必须是数组");
        let japanese_texts: Vec<String> = raw_segs
            .iter()
            .map(|s| s["text"].as_str().unwrap_or("").to_string())
            .collect();

        let req = PureTranslationRequest {
            texts: japanese_texts.clone(),
            target_lang: Some("Chinese".into()),
        };

        let res = TranslationService::run_translation_pipeline(req)
            .await
            .expect("Hy-MT2 1.8B 日文跨语言翻译失败");

        println!("\n🎉 ===== [Hy-MT2 1.8B 纯净日译中 翻译完成] =====");
        println!("  ⏱️ 物理总耗时: {:.2} ms ({:.2} s)", res.elapsed_ms, res.elapsed_ms / 1000.0);
        println!("  翻译总句数: {} 句", res.translations.len());

        assert_eq!(res.translations.len(), japanese_texts.len());

        let out_json_path = outs_dir.join("japanese_to_chinese_pure.json");
        let json_data = serde_json::to_string_pretty(&res).unwrap();
        std::fs::write(&out_json_path, &json_data).unwrap();

        println!("  💾 纯净日译中 JSON 产物落盘至: {:?}", out_json_path);
        println!("  预览前 3 句日语➔中文翻译效果:");
        for i in 0..3.min(japanese_texts.len()) {
            println!("     [{}] 日文: {} ➔ 中文: {}", i + 1, japanese_texts[i], res.translations[i]);
        }
        println!("============================================\n");
    }
}
