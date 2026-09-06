// 🛡️ 紫电 AI 桌面工坊 - 语音大模型转写与说话人分段领域能力套件 (audio_suite.ts)
// 职责边界：调用 MOSS-Transcribe GGUF 神经大模型，直推 18000 端口，输出带精确时间戳与说话人分段的文本流

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
        content: [{ type: "text", text: `❌ MOSS 语音转写执行失败: ${json.error}` }],
        details: json
      };
    }

    const duration = json.data?.duration_sec ? `${json.data.duration_sec.toFixed(1)} 秒` : "未知";
    const transcript = json.data?.full_transcript || "未提取到有效语音内容";
    const observationText = `【MOSS 语音转写与说话人分段完成】:\n音频时长: ${duration}, 共 ${json.data?.total_segments || 0} 个台词片段\n\n[完整台词流]:\n${transcript}`;

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

export function registerAudioSuite(pi: any) {
  pi.registerTool({
    name: "native_asr_transcribe",
    label: "音频/视频语音识别与说话人分离",
    description: "调用端侧 MOSS 语音大模型，将录音、会议音频或视频文件转写为带精确毫秒时间戳和说话人标识 (如 S01, S02) 的结构化台词流。如果用户需要转写录音、识别视频声音或生成对话纪要，请调用本工具。",
    parameters: {
      type: "object",
      properties: {
        audio_path: {
          type: "string",
          description: "音视频文件的物理绝对路径 (WAV/MP3/MP4/MKV/FLAC)"
        },
        language: {
          type: "string",
          description: "可选语言代码 (如 auto, zh, en, ja, yue)",
          default: "auto"
        },
        prompt: {
          type: "string",
          description: "可选前置提示词或专有名词引导"
        },
        hotwords: {
          type: "string",
          description: "可选领域热词列表 (逗号分隔)"
        }
      },
      required: ["audio_path"]
    },
    async execute(toolCallId: string, params: any) {
      return await callRustTool("native_asr_transcribe", params);
    }
  });
}
