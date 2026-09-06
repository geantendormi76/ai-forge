// 🛡️ 紫电 AI 桌面工坊 - 动态 Python 办公沙箱与安全看门狗 (sandbox_suite.ts)
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import * as path from "node:path";
import * as fs from "node:fs/promises";
import * as os from "node:os";

const execFileAsync = promisify(execFile);

export function registerSandboxSuite(pi: any) {
  const projectRoot = process.cwd();
  const pyExePath = path.join(projectRoot, "src-tauri", "bin", "python-embed", "python.exe");

  pi.registerTool({
    name: "execute_python_sandbox",
    label: "Python 数据沙箱",
    description: "在免安装绿色沙箱中执行 Python 数据分析、图表绘制与 Excel 操作。超时 30 秒熔断。",
    parameters: {
      type: "object",
      properties: {
        code: { type: "string", description: "Python 3 代码。生成的文件路径或结果摘要必须通过 print() 打印。" },
        purpose: { type: "string", description: "执行目的说明" }
      },
      required: ["code"]
    },
    async execute(toolCallId: string, params: any) {
      const { code, purpose } = params;
      const tempScript = path.join(os.tmpdir(), `ai_forge_${Date.now()}_${Math.random().toString(36).slice(2, 6)}.py`);
      try {
        await fs.writeFile(tempScript, code, "utf-8");
        const { stdout, stderr } = await execFileAsync(pyExePath, [tempScript], {
          timeout: 30000,
          maxBuffer: 10 * 1024 * 1024,
          windowsHide: true,
          env: { ...process.env, PYTHONIOENCODING: "utf-8", PYTHONUNBUFFERED: "1" }
        });
        const outputText = [
          purpose ? `[任务目的] ${purpose}` : null,
          stdout ? `[标准输出]\n${stdout.trim()}` : "[无标准输出]",
          stderr ? `[日志/提示]\n${stderr.trim()}` : null
        ].filter(Boolean).join("\n\n");
        return { content: [{ type: "text", text: outputText }], details: { status: "success" } };
      } catch (err: any) {
        const errorMsg = err.killed ? "🚨 脚本执行超过 30 秒看门狗限制已熔断" : `🚨 Python 执行异常: ${err.message}\n${err.stderr || ""}`;
        return { content: [{ type: "text", text: errorMsg }], details: { status: "error" } };
      } finally {
        await fs.unlink(tempScript).catch(() => {});
      }
    }
  });

  // 安全防线：危险底层指令实时拦截
  pi.on("tool_call", async (event: any) => {
    if (event.toolName === "bash" || event.toolName === "sh") {
      const cmd = String(event.input?.command || "");
      const dangerousPatterns = [/rm\s+-rf/i, /del\s+\/s/i, /format\s+[a-z]:/i, /shutdown/i, /drop\s+database/i];
      for (const pattern of dangerousPatterns) {
        if (pattern.test(cmd)) {
          return {
            block: true,
            reason: `🛑 紫电 AI 安全中枢已拦截危险底层指令: "${cmd}"。请调用专属原子技能完成操作。`
          };
        }
      }
    }
  });
}
