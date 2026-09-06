// 🛡️ 紫电 AI 桌面工坊 - 全能格式转换与解密领域能力套件 (converter_suite.ts)
// 职责边界：调用 FormatConvertService 纯血底座，直推 18000 端口，执行文档编译、音频解密与多媒体转码

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
        content: [{ type: "text", text: `❌ 格式转换执行失败: ${json.error}` }],
        details: json
      };
    }

    const outPath = json.data?.output_path || "源文件同级目录";
    const observationText = `【全能格式转换与解密完成】:\n源文件: ${json.data?.input_path}\n输出产物物理路径: ${outPath}\n转换格式: .${json.data?.detected_format || params.target_format}\n状态反馈: ${json.data?.message || "成功"}`;

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

export function registerConverterSuite(pi: any) {
  pi.registerTool({
    name: "native_format_convert",
    label: "全能格式转换与解密",
    description: "执行加密音频解密 (NCM/QMC/KGM/KWM/MFLAC ➔ FLAC/MP3)、Markdown 编译为 Word DOCX / 电子书 EPUB / 矢量 PDF、数据格式互转 (CSV ↔ JSON / Markdown) 以及多分辨率 ICO 图标生成。如果用户需要转换文件格式或编译文档，请调用本工具。",
    parameters: {
      type: "object",
      properties: {
        input_path: {
          type: "string",
          description: "待转换/待解密源文件的物理绝对路径"
        },
        target_format: {
          type: "string",
          description: "目标格式扩展名 (如 docx, epub, pdf, flac, mp3, csv, json, md, ico, mp4, wav)"
        },
        output_dir: {
          type: "string",
          description: "可选输出目录 (未指定则默认输出到源文件同级目录)"
        }
      },
      required: ["input_path", "target_format"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("native_format_convert", params);
    }
  });
}
