// 🛡️ 紫电 AI 桌面工坊 - 全能格式转换与解密领域能力套件 (converter_suite.ts)
// 职责边界：加密音频解密 (NCM/QMC/KGM/KWM)、文档生成 (DOCX/EPUB/PDF)、数据清洗 (CSV/JSON/XML) 与多媒体转码

export function registerConverterSuite(pi: any) {
  pi.registerTool({
    name: "native_format_convert",
    label: "全能格式转换与解密",
    description: "执行加密音频解密 (NCM/QMC/KGM/KWM/MFLAC ➔ FLAC/MP3)、Markdown 编译为 Word DOCX / 电子书 EPUB / 矢量 PDF、数据格式互转 (CSV ↔ JSON / Markdown) 以及多分辨率 ICO 图标生成。",
    parameters: {
      type: "object",
      properties: {
        input_path: { type: "string", description: "待转换/待解密源文件的物理绝对路径" },
        target_format: {
          type: "string",
          description: "目标格式扩展名 (如 docx, epub, pdf, flac, mp3, csv, json, md, ico, mp4, wav)"
        },
        output_dir: { type: "string", description: "可选输出目录 (未指定则输出到源文件同级目录)" }
      },
      required: ["input_path", "target_format"]
    },
    async execute(toolCallId: string, params: any) {
      return {
        content: [{ type: "text", text: `✅ 格式转换算子已派发: ${params.input_path} ➔ .${params.target_format}` }],
        details: { dispatched: true, params }
      };
    }
  });
}
