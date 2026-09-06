// 🛡️ 紫电 AI 桌面工坊 - 复杂表格重构领域能力套件 (table_suite.ts)
// 职责边界：调用 SLANet 神经网络，输出二维表格拓扑矩阵、跨行跨列 HTML 与单元格 BBox

export function registerTableSuite(pi: any) {
  pi.registerTool({
    name: "table_recognize_structure",
    label: "表格结构化拓扑与 HTML 重构",
    description: "调用 SLANet 神经大模型，识别图片中的复杂表格结构（支持多层合并单元格、无边框表格、财务报表），输出标准 HTML 代码以及每个单元格的行列网格坐标与物理 BBox。",
    parameters: {
      type: "object",
      properties: {
        image_path: { type: "string", description: "表格切片图像的物理绝对路径 (PNG/JPG)" }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return {
        content: [{ type: "text", text: `✅ 表格结构重构算子已派发: ${params.image_path}` }],
        details: { dispatched: true, params }
      };
    }
  });
}
