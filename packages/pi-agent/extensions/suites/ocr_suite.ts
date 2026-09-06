// 🛡️ 紫电 AI - OCR 视觉文字提取套件 (ocr_suite.ts)
async function callRustTool(tool_name: string, params: any) {
  try {
    const res = await fetch("http://127.0.0.1:18000/api/tool", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tool_name, params })
    });
    const json = await res.json();
    if (!json.success) {
      return { content: [{ type: "text", text: `❌ OCR 识别失败: ${json.error}` }], details: json };
    }
    return { content: [{ type: "text", text: JSON.stringify(json.data, null, 2) }], details: json.data };
  } catch (err: any) {
    return { content: [{ type: "text", text: `❌ 连接 Rust 底座异常: ${err.message}` }], details: { error: err.message } };
  }
}

export function registerOcrSuite(pi: any) {
  pi.registerTool({
    name: "ocr_recognize_image",
    label: "OCR 视觉文字与坐标提取",
    description: "调用 PP-OCRv6 神经大模型，毫秒级提取图片或发票中的中英文文本流与物理 BBox 坐标。",
    parameters: {
      type: "object",
      properties: {
        image_path: { type: "string", description: "待识别图像的物理绝对路径 (PNG/JPG/BMP)" }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("ocr_recognize_image", params);
    }
  });
}
