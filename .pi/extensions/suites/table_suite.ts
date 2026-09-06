// 🛡️ 紫电 AI 桌面工坊 - 复杂表格结构化重构领域能力套件 (table_suite.ts)
// 职责边界：调用 SLANet 神经网络，直推 18000 端口，输出标准二维表格 HTML 与单元格拓扑

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
        content: [{ type: "text", text: `❌ 表格结构重构失败: ${json.error}` }],
        details: json
      };
    }

    // 核心分离：
    // 1. 给 8B 模型看干净的 HTML 表格或结构摘要，供其在下一轮直接写盘或排版
    const tableHtml = json.data?.html_structure || "";
    const observationText = `【SLANet 表格识别成功，共识别出 ${json.data?.total_cells || 0} 个单元格】:\n\n${tableHtml}`;

    // 2. 给前端 Artifact 看板保留完整的 cells、坐标与 html_structure
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

export function registerTableSuite(pi: any) {
  pi.registerTool({
    name: "table_recognize_structure",
    label: "表格结构化拓扑与 HTML 重构",
    description: "调用端侧 SLANet 神经大模型，识别图片中的二维复杂表格（支持送货单、财务报表、合并单元格等），输出可直接用于排版的标准 HTML 表格代码。如果用户需要对图片中的表格进行结构化整理或排版，请优先调用此工具。",
    parameters: {
      type: "object",
      properties: {
        image_path: {
          type: "string",
          description: "待重构表格切片或完整票据图片的物理绝对路径 (PNG/JPG/BMP)"
        }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("table_recognize_structure", params);
    }
  });
}
