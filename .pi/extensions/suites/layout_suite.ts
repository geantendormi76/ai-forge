// 🛡️ 紫电 AI 桌面工坊 - 视觉版面分析领域能力套件 (layout_suite.ts)
// 职责边界：调用 PP-DocLayoutV3 神经网络，直推 18000 端口，输出 25 类高精版面区块与自然阅读顺序

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
        content: [{ type: "text", text: `❌ 视觉版面分析失败: ${json.error}` }],
        details: json
      };
    }

    // 核心分离：
    // 1. 给 8B 模型看清晰简洁的版面区块大纲与自然阅读序
    const regions = json.data?.regions || [];
    const outline = regions.map((r: any, idx: number) => {
      const order = r.reading_order !== undefined ? `[阅读序 #${r.reading_order}]` : `[区块 #${idx + 1}]`;
      const label = r.label || "未知类型";
      const score = r.score ? `(置信度: ${(r.score * 100).toFixed(0)}%)` : "";
      return `${order} 类型: ${label} ${score}`;
    }).join("\n");

    const observationText = `【PP-DocLayoutV3 版面分析成功，共识别出 ${regions.length} 个结构区块】:\n分辨率: ${json.data?.image_width || 0}×${json.data?.image_height || 0}\n\n[自然阅读序大纲]:\n${outline || "未检测到明确独立区块"}`;

    // 2. 给前端保留全量 BBox 坐标与原始检测矩阵
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

export function registerLayoutSuite(pi: any) {
  pi.registerTool({
    name: "layout_detect_regions",
    label: "视觉版面分析与阅读序重构",
    description: "调用端侧 PP-DocLayoutV3 视觉大模型，深度剖析复杂图片、扫描件或文档页面的版面结构，检测大标题、段落正文、二维表格、数学公式与印章，并输出全局自然阅读顺序。在处理复杂排版或混合排版任务时，请优先使用本工具梳理结构。",
    parameters: {
      type: "object",
      properties: {
        image_path: {
          type: "string",
          description: "待分析页面的高清图片物理绝对路径 (PNG/JPG/BMP/WEBP)"
        },
        score_threshold: {
          type: "number",
          description: "置信度门限 (默认 0.15，范围 0.0~1.0)",
          default: 0.15
        },
        filter_categories: {
          type: "array",
          items: { type: "string" },
          description: "可选类别过滤列表 (如 [\"table\", \"display_formula\", \"image\", \"doc_title\"]，留空则返回全部)"
        }
      },
      required: ["image_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("layout_detect_regions", params);
    }
  });
}
