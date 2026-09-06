// 🛡️ 紫电 AI 桌面工坊 - 纯血原子模型底座微内核注册器 (ai_forge_suite.ts)
// 基于 2026 SOTA 模块化插件体系，各底座独立自治，主入口实现 0 耦合装配
import { registerSandboxSuite } from "./suites/sandbox_suite";
import { registerPdfSuite } from "./suites/pdf_suite";
import { registerLayoutSuite } from "./suites/layout_suite";
import { registerOcrSuite } from "./suites/ocr_suite";
import { registerFormulaSuite } from "./suites/formula_suite";
import { registerTableSuite } from "./suites/table_suite";
import { registerVisionSuite } from "./suites/vision_suite";
import { registerAudioSuite } from "./suites/audio_suite";
import { registerTranslateSuite } from "./suites/translate_suite";
import { registerConverterSuite } from "./suites/converter_suite";
import { registerDocSuite } from "./suites/doc_suite";

export default function (pi: any) {
  // 1. 挂载数据沙箱与安全中枢
  registerSandboxSuite(pi);

  // 2. 挂载 service-pdfium 物理几何与位图渲染底座 (4 大原子算子)
  registerPdfSuite(pi);

  // 3. 挂载 service-layout 视觉版面分析底座 (PP-DocLayoutV3 25 类阅读序)
  registerLayoutSuite(pi);

  // 4. 挂载 service-ocr 视觉文字提取底座 (PP-OCRv6 双阶段识别)
  registerOcrSuite(pi);

  // 5. 挂载 service-formula 数学公式 LaTeX 识别底座 (PP-FormulaNet-S)
  registerFormulaSuite(pi);

  // 6. 挂载 service-table 复杂表格结构化重构底座 (SLANet)
  registerTableSuite(pi);

  // 7. 挂载视觉超分底座
  registerVisionSuite(pi);

  // 8. 挂载 MOSS 语音大模型底座
  registerAudioSuite(pi);

  // 9. 挂载 Hy-MT2 神经机器翻译底座
  registerTranslateSuite(pi);

  // 10. 挂载全能格式转换与解密底座
  registerConverterSuite(pi);

  // 11. 挂载多模态文档深度重构复合流水线
  registerDocSuite(pi);
}
