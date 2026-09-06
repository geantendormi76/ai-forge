// 🛡️ 紫电 AI 桌面工坊 - 神经机器翻译领域能力套件 (translate_suite.ts)
// 职责边界：调用 Hy-MT2 1.8B GGUF 神经大模型，直推 18000 端口，在 38 种语言间进行高精度专业互译

async function callRustTool(tool_name: string, params: any) {
  try {
    const res = await fetch("http://127.0.0.1:18000/api/tool", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tool_name, params })
    });
    const json = await res.json();
    if (!json.success) {
      return {
        content: [{ type: "text", text: `❌ 神经翻译执行失败: ${json.error}` }],
        details: json
      };
    }

    // 核心分离：
    // 1. 给 8B 模型提供纯净干净的译文正文
    const translations = json.data?.translations || [];
    const resultText = translations.join("\n");
    const observationText = `【Hy-MT2 神经翻译完成 (目标语种: ${json.data?.target_lang || "Chinese"})】:\n\n${resultText}`;

    // 2. 给前端 TranslateArtifactCard 保留全量 details 数据
    return {
      content: [{ type: "text", text: observationText }],
      details: json.data
    };
  } catch (err: any) {
    return {
      content: [{ type: "text", text: `❌ 连接 Rust 底座异常: ${err.message}` }],
      details: { error: err.message }
    };
  }
}

export function registerTranslateSuite(pi: any) {
  pi.registerTool({
    name: "native_neural_translate",
    label: "高精神经机器翻译",
    description: "调用端侧 Hy-MT2 神经翻译大模型，在 38 种语言间进行高质量专业互译（支持中文、英语、日语、韩语、俄语、德语、法语、西班牙语等，支持多段文本或整篇文章批量互译）。如果用户需要翻译文本、文档或单据内容，请调用本工具。",
    parameters: {
      type: "object",
      properties: {
        texts: {
          type: "array",
          items: { type: "string" },
          description: "待翻译的文本段落或句子列表 (字符串数组)"
        },
        text: {
          type: "string",
          description: "单段待翻译文本 (与 texts 二选一即可)"
        },
        target_lang: {
          type: "string",
          description: "目标语言名称 (如 Chinese, English, Japanese, Korean, French, German, Spanish, Russian 等)",
          default: "Chinese"
        }
      }
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("native_neural_translate", params);
    }
  });
}
