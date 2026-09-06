// 🛡️ 紫电 AI - OCR 视觉文字提取套件 (ocr_suite.ts)
// 保持工具纯血与单一职责：只输出客观识别文字，严禁掺杂业务决策与文件生成硬编码

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
        content: [{ type: "text", text: `❌ OCR 识别失败: ${json.error}` }],
        details: json
      };
    }

    const cleanText = json.data?.full_text || JSON.stringify(json.data);

    // 纯粹、客观的 Observation：不给大模型强加任何特定业务假设
    return {
      content: [{ type: "text", text: cleanText }],
      details: json.data
    };
  } catch (err: any) {
    return {
      content: [{ type: "text", text: `❌ 连接 Rust 底座异常: ${err.message}` }],
      details: { error: err.message }
    };
  }
}

export function registerOcrSuite(pi: any) {
  pi.registerTool({
    name: "ocr_recognize_image",
    label: "PP-OCRv6 视觉文字提取",
    description: "调用端侧 PP-OCRv6 视觉大模型，提取图片、单据、发票或扫描件中的中英文文本内容。",
    parameters: {
      type: "object",
      properties: {
        image_path: {
          type: "string",
          description: "待识别图像的物理绝对路径 (PNG/JPG/BMP)"
        }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("ocr_recognize_image", params);
    }
  });
}
