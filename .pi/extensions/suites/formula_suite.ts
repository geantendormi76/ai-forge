// 🛡️ 紫电 AI 桌面工坊 - 数学公式与 LaTeX 领域能力套件 (formula_suite.ts)
// 职责边界：调用 PP-FormulaNet-S 神经网络，直推 18000 端口，输出标准 LaTeX 公式字符串

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
        content: [{ type: "text", text: `❌ 数学公式 LaTeX 识别失败: ${json.error}` }],
        details: json
      };
    }

    // 核心分离：
    // 1. 给 8B 模型看纯净可直接渲染的 LaTeX 字符串
    const latexCode = json.data?.latex || "";
    const observationText = `【PP-FormulaNet 公式识别成功】:\n\n$$\n${latexCode}\n$$`;

    // 2. 给前端保留全量 details 与原始 token 信息
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

export function registerFormulaSuite(pi: any) {
  pi.registerTool({
    name: "formula_recognize_latex",
    label: "数学公式 LaTeX 视觉识别",
    description: "调用端侧 PP-FormulaNet-S 视觉大模型，识别图片中的复杂数学公式（含微积分、矩阵、分式、多行方程组等），输出可直接用于 Markdown 或学术排版的标准 LaTeX 字符串。如果图片中包含数学公式切片，请调用本工具提取 LaTeX。",
    parameters: {
      type: "object",
      properties: {
        image_path: {
          type: "string",
          description: "公式切片图像的物理绝对路径 (PNG/JPG/BMP)"
        }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("formula_recognize_latex", params);
    }
  });
}
