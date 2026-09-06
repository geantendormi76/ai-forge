// 🛡️ 紫电 AI - PDF 物理几何与光栅化套件 (pdf_suite.ts)
async function callRustTool(tool_name: string, params: any) {
  try {
    const res = await fetch("http://127.0.0.1:18000/api/tool", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tool_name, params })
    });
    const json = await res.json();
    if (!json.success) {
      return { content: [{ type: "text", text: `❌ PDF 算子执行失败: ${json.error}` }], details: json };
    }
    return { content: [{ type: "text", text: JSON.stringify(json.data, null, 2) }], details: json.data };
  } catch (err: any) {
    return { content: [{ type: "text", text: `❌ 连接 Rust 底座异常: ${err.message}` }], details: { error: err.message } };
  }
}

export function registerPdfSuite(pi: any) {
  pi.registerTool({
    name: "pdf_get_meta",
    label: "PDF 元数据探测",
    description: "毫秒级获取 PDF 文档总页数与物理尺寸。",
    parameters: {
      type: "object",
      properties: { file_path: { type: "string", description: "PDF 文件的物理绝对路径" } },
      required: ["file_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("pdf_get_meta", params);
    }
  });

  pi.registerTool({
    name: "pdf_render_page_image",
    label: "PDF 页面位图渲染",
    description: "将 PDF 指定页码高保真光栅化渲染为指定 DPI（默认 300）的高清 PNG 物理图片。",
    parameters: {
      type: "object",
      properties: {
        file_path: { type: "string", description: "PDF 文件物理绝对路径" },
        page_index: { type: "number", description: "页码索引 (从 0 起始，第 1 页填 0)", default: 0 },
        target_dpi: { type: "number", description: "渲染 DPI 清晰度 (默认 300)", default: 300 }
      },
      required: ["file_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("pdf_render_page_image", params);
    }
  });
}
