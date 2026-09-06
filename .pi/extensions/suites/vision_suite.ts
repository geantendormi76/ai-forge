// 🛡️ 紫电 AI - 视觉超分领域能力套件 (vision_suite.ts)
async function callRustTool(tool_name: string, params: any) {
  try {
    const res = await fetch("http://127.0.0.1:18000/api/tool", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tool_name, params })
    });
    const json = await res.json();
    if (!json.success) {
      return { content: [{ type: "text", text: `❌ 超分推导失败: ${json.error}` }], details: json };
    }
    return { content: [{ type: "text", text: JSON.stringify(json.data, null, 2) }], details: json.data };
  } catch (err: any) {
    return { content: [{ type: "text", text: `❌ 连接 Rust 底座异常: ${err.message}` }], details: { error: err.message } };
  }
}

export function registerVisionSuite(pi: any) {
  pi.registerTool({
    name: "native_upscale_image",
    label: "视觉超分辨率重构",
    description: "调用端侧 CUDA 超分大模型，将低清图片无损放大 2~4 倍（支持 4K/8K 超清重构与透明通道保留），生成高清大图并输出物理保存路径。",
    parameters: {
      type: "object",
      properties: {
        input_path: { type: "string", description: "待放大图像的物理绝对路径 (PNG/JPG/WEBP)" },
        output_path: { type: "string", description: "可选输出图像物理路径 (若未指定则自动在同级生成)" },
        target_scale: { type: "number", description: "目标放大倍率 (2.0 或 4.0，默认 4.0)", default: 4.0 },
        max_output_side: { type: "number", description: "长边分辨率封顶 (4K极速填 3840, 8K旗舰填 8192)", default: 8192 }
      },
      required: ["input_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("native_upscale_image", params);
    }
  });
}
