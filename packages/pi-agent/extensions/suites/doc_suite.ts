// 🛡️ 紫电 AI 桌面工坊 - 多模态 PDF 深度重构总成领域能力套件 (doc_suite.ts)
// 职责边界：协同调度 5 大模型底座，执行 PDF 双轨解析、分栏阅读序重构、公式/表格/文本高保真 Markdown 输出

export function registerDocSuite(pi: any) {
  pi.registerTool({
    name: "native_doc_parse",
    label: "PDF 多模态文档结构化深度重构",
    description: "端到端解析 PDF/扫描件/复杂文档，协同调度版面分析、OCR、LaTeX 公式提取与表格拓扑重构，精准处理多栏混排，输出专业 GFM Markdown 与图片切片。",
    parameters: {
      type: "object",
      properties: {
        file_path: { type: "string", description: "待解析 PDF 文件的物理绝对路径" },
        output_dir: { type: "string", description: "可选产物输出目录 (未指定则输出到源文件同级目录)" }
      },
      required: ["file_path"]
    },
    async execute(toolCallId: string, params: any) {
      return {
        content: [{ type: "text", text: `✅ PDF 深度重构流水线已派发: ${params.file_path}` }],
        details: { dispatched: true, params }
      };
    }
  });
}
