//! 🛡️ 紫电 AI 桌面工坊 - 离线高精翻译与图像 OCR 业务适配器 (translation/src/lib.rs)
//! 深度对齐 2026 SOTA 黑匣子日志标准与 Hy-MT2 1.8B 神经翻译引擎 (降噪优化版)

use base64::Engine;
use serde::{Deserialize, Serialize};
use shared_contracts::{TaskWeight, VramTokenGuard};
use service_translation::{PureTranslationRequest, TranslationProgressCallback, TranslationService};
use service_ocr::OcrService;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 翻译风格枚举 (兼容保留)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationStyle {
    General,
    Academic,
    Spoken,
    Business,
    Literary,
}

/// 纯文本高精翻译请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTask {
    /// 待翻译的文本段落列表
    pub texts: Vec<String>,
    /// 源语种 (可选)
    pub source_lang: Option<String>,
    /// 目标语种 (如 "Chinese", "English", "Japanese" 等 38 种语言)
    pub target_lang: String,
    /// 翻译风格设定 (可选)
    pub style: Option<TranslationStyle>,
    /// 可选：输出文件的绝对路径
    pub output_file_path: Option<String>,
}

/// 纯文本高精翻译交付结果契约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub success: bool,
    pub translations: Vec<String>,
    pub output_file_path: Option<String>,
    pub total_segments: usize,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// 图像与剪贴板截图 OCR 翻译请求载荷 (支持物理文件与 Base64 内存直推)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTranslationTask {
    /// 可选：图像物理绝对路径
    pub image_path: Option<String>,
    /// 可选：图像 Base64 原始数据 (剪贴板粘贴直接内存直推，0 磁盘 I/O)
    pub image_base64: Option<String>,
    /// 目标语种 (如 "Chinese", "English" 等)
    pub target_lang: String,
}

/// 图像与剪贴板截图 OCR 翻译交付结果契约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTranslationResult {
    pub success: bool,
    /// 全文合并的 OCR 原始文本
    pub full_source_text: String,
    /// 全文合并的翻译译文
    pub full_translated_text: String,
    /// OCR 视觉提取耗时 (毫秒)
    pub ocr_elapsed_ms: u64,
    /// 神经翻译耗时 (毫秒)
    pub trans_elapsed_ms: u64,
    /// 端到端总耗时 (毫秒)
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub struct TranslationTool;

impl TranslationTool {
    pub fn build_prompt(text: &str, target_lang: &str) -> String {
        let clean_text = text.trim();
        if clean_text.is_empty() {
            return String::new();
        }
        format!(
            "将以下文本翻译为 {}，注意只需要输出翻译后的结果，不要额外解释：\n\n{}",
            target_lang, clean_text
        )
    }

    /// 执行纯文本高精翻译流水线
    pub async fn execute(
        task: TranslationTask,
        vram_guard: Option<&VramTokenGuard>,
        cancel_token: Option<Arc<AtomicBool>>,
        on_progress: Option<TranslationProgressCallback>,
    ) -> Result<TranslationResult, String> {
        let start_t = Instant::now();
        let total_count = task.texts.len();

        // 🌟 节点 1: [任务入口] 宏观业务日志
        log::info!(
            "🚀 [离线高精翻译] 收到文本翻译请求: 待处理段落数={}, 目标语种='{}'",
            total_count,
            task.target_lang
        );

        if total_count == 0 {
            return Ok(TranslationResult {
                success: true,
                translations: Vec::new(),
                output_file_path: task.output_file_path,
                total_segments: 0,
                elapsed_ms: 0,
                error: None,
            });
        }

        // 🌟 节点 2: [模型加载与显存寻址]
        let _permit = if let Some(guard) = vram_guard {
            match guard.acquire(TaskWeight::Medium).await {
                Ok(p) => Some(p),
                Err(e) => {
                    log::warn!("⚠️ [显存守卫] 显存令牌申请失败 ({})，直接调度推理", e);
                    None
                }
            }
        } else {
            None
        };

        let formatted_prompts: Vec<String> = task
            .texts
            .iter()
            .map(|t| Self::build_prompt(t, &task.target_lang))
            .collect();

        // 🌟 节点 3: [阶段推演] 仅向前端广播 IPC 进度，日志降噪
        let progress_hook = on_progress.clone();
        let cancel_flag = cancel_token.clone();

        let req = PureTranslationRequest {
            texts: formatted_prompts,
            target_lang: Some(task.target_lang.clone()),
        };

        let wrapped_cb: TranslationProgressCallback = Arc::new(move |current, total, msg| {
            if let Some(ref flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
            }
            // 降噪优化：仅在 debug 级别记录微小进度，不污染生产黑匣子
            log::debug!("  ⚡ [Hy-MT2 神经翻译] 进度 ({}/{}) - {}", current, total, msg);
            if let Some(ref cb) = progress_hook {
                cb(current, total, msg);
            }
        });

        let resp = TranslationService::run_translation_pipeline_with_progress(req, Some(wrapped_cb))
            .await
            .map_err(|e| {
                log::error!("🚨 [离线高精翻译] 推理执行异常: {}", e);
                e
            })?;

        if let Some(ref out_path) = task.output_file_path {
            let combined = resp.translations.join("\n\n");
            let _ = std::fs::write(out_path, &combined);
        }

        let elapsed_ms = start_t.elapsed().as_millis() as u64;

        // 🌟 节点 4: [产物交付与端到端耗时]
        log::info!(
            "🏆 [离线高精翻译交付] 成功完成 {} 段文本翻译, 端到端总耗时: {} ms ({:.2} s)",
            resp.translations.len(),
            elapsed_ms,
            elapsed_ms as f64 / 1000.0
        );

        Ok(TranslationResult {
            success: true,
            translations: resp.translations,
            output_file_path: task.output_file_path,
            total_segments: total_count,
            elapsed_ms,
            error: None,
        })
    }

    /// 执行图像与剪贴板截图 OCR 翻译流水线
    pub async fn execute_image_ocr(
        task: ImageTranslationTask,
        vram_guard: Option<&VramTokenGuard>,
        cancel_token: Option<Arc<AtomicBool>>,
        on_progress: Option<TranslationProgressCallback>,
    ) -> Result<ImageTranslationResult, String> {
        let start_t = Instant::now();

        // 🌟 节点 1: [任务入口]
        log::info!(
            "🚀 [图像 OCR 翻译] 收到请求: 目标语种='{}', 来源={}",
            task.target_lang,
            if task.image_base64.is_some() { "剪贴板 Base64 内存流" } else { "本地物理文件" }
        );

        // 解码图像
        let dyn_img = if let Some(ref b64) = task.image_base64 {
            let clean_b64 = b64
                .trim_start_matches("data:image/png;base64,")
                .trim_start_matches("data:image/jpeg;base64,")
                .trim_start_matches("data:image/jpg;base64,")
                .trim_start_matches("data:image/webp;base64,");
            let bytes = base64::prelude::BASE64_STANDARD
                .decode(clean_b64)
                .map_err(|e| format!("Base64 解码图像失败: {e}"))?;
            image::load_from_memory(&bytes).map_err(|e| format!("内存解析图像失败: {e}"))?
        } else if let Some(ref p_str) = task.image_path {
            let p = Path::new(p_str);
            if !p.exists() {
                return Err(format!("图像物理文件不存在: {:?}", p));
            }
            image::open(p).map_err(|e| format!("读取图像文件失败: {e}"))?
        } else {
            return Err("未提供可用的图像数据 (image_path 或 image_base64)".to_string());
        };

        // 🌟 节点 2/3: PP-OCRv6 视觉识别
        let ocr_t0 = Instant::now();
        let mut ocr_engine = OcrService::default_engine()
            .map_err(|e| format!("初始化 PP-OCRv6 引擎失败: {e}"))?;
        let regions = ocr_engine
            .process_image(dyn_img)
            .map_err(|e| format!("PP-OCRv6 图像推导异常: {e}"))?;

        let ocr_elapsed_ms = ocr_t0.elapsed().as_millis() as u64;
        log::info!(
            "  ⚡ [PP-OCRv6 视觉识别] 成功提取 {} 个文本框 (耗时: {} ms)",
            regions.len(),
            ocr_elapsed_ms
        );

        if regions.is_empty() {
            return Ok(ImageTranslationResult {
                success: true,
                full_source_text: "（未在图像中识别到可见文字）".to_string(),
                full_translated_text: "（无识别文字，无需翻译）".to_string(),
                ocr_elapsed_ms,
                trans_elapsed_ms: 0,
                elapsed_ms: ocr_elapsed_ms,
                error: None,
            });
        }

        let raw_texts: Vec<String> = regions
            .iter()
            .map(|r| r.text.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        let full_source = raw_texts.join("\n");

        // 调度 Hy-MT2 神经翻译
        let trans_t0 = Instant::now();
        let trans_task = TranslationTask {
            texts: raw_texts,
            source_lang: None,
            target_lang: task.target_lang,
            style: None,
            output_file_path: None,
        };

        let trans_res = Self::execute(trans_task, vram_guard, cancel_token, on_progress).await?;
        let trans_elapsed_ms = trans_t0.elapsed().as_millis() as u64;
        let full_translated = trans_res.translations.join("\n");
        let total_elapsed = start_t.elapsed().as_millis() as u64;

        // 🌟 节点 4: [产物交付与总耗时]
        log::info!(
            "🏆 [图像 OCR 翻译交付] 完成识别与翻译, 总耗时: {} ms (OCR: {} ms, 翻译: {} ms)",
            total_elapsed,
            ocr_elapsed_ms,
            trans_elapsed_ms
        );

        Ok(ImageTranslationResult {
            success: true,
            full_source_text: full_source,
            full_translated_text: full_translated,
            ocr_elapsed_ms,
            trans_elapsed_ms,
            elapsed_ms: total_elapsed,
            error: None,
        })
    }
}
