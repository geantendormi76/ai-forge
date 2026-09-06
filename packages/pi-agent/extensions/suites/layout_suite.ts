// 🛡️ 紫电 AI 桌面工坊 - 视觉版面分析领域能力套件 (layout_suite.ts)
// 职责边界：调用 PP-DocLayoutV3 神经网络，输出 25 类高精版面区块与自然阅读顺序

export function registerLayoutSuite(pi: any) {
  // 1. 全量版面结构化扫描算子
  pi.registerTool({
    name: "layout_detect_regions",
    label: "视觉版面分析与阅读序重构",
    description: "调用 PP-DocLayoutV3 视觉大模型，识别页面中的 25 种版面区块（大标题、段落文本、表格、公式、图片、印章等）并输出自然阅读顺序与 BBox 坐标。",
    parameters: {
      type: "object",
      properties: {
        image_path: { type: "string", description: "待分析页面的高清图片物理绝对路径 (PNG/JPG)" },
        score_threshold: { type: "number", description: "置信度门限 (默认 0.15，范围 0.0~1.0)", default: 0.15 },
        filter_categories: {
          type: "array",
          items: { type: "string" },
          description: "可选类别过滤列表 (如 [\"table\", \"display_formula\", \"image\", \"doc_title\"]，留空则返回全部)"
        }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return {
        content: [{ type: "text", text: `✅ 视觉版面分析算子已派发: ${params.image_path}` }],
        details: { dispatched: true, params }
      };
    }
  });
}
