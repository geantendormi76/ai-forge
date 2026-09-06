// 🛡️ 紫电 AI 桌面工坊 - 多模态文档深度重构总成领域能力套件 (doc_suite.ts)
// 职责边界：协同调度 5 大模型底座，执行 PDF 与复杂图片版面重构，输出标准 Markdown

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
        content: [{ type: "text", text: `❌ 文档排版重构失败: ${json.error}` }],
        details: json
      };
    }
    const cleanMd = json.data?.markdown || JSON.stringify(json.data);
    const observationText = `【文档/图片排版重构完成】:\n产物 Markdown 路径: ${json.data?.output_md_path || "未指定"}\n\n[排版内容预览]:\n${cleanMd.slice(0, 800)}...`;

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

export function registerDocSuite(pi: any) {
  pi.registerTool({
    name: "native_doc_parse",
    label: "多模态文档结构化排版与深度重构",
    description: "端到端重构 PDF、复杂票据、表格图片或扫描件的排版结构。协同调度版面分析、表格拓扑重构、OCR 文字提取与分栏阅读序重构，直接输出带排版结构与 Markdown 格式的完整文本内容。",
    parameters: {
      type: "object",
      properties: {
        file_path: {
          type: "string",
          description: "待排版解析的 PDF 物理绝对路径"
        },
        output_dir: {
          type: "string",
          description: "可选产物输出目录 (未指定则输出到源文件同级目录)"
        }
      },
      required: ["file_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("native_doc_parse", params);
    }
  });
}
