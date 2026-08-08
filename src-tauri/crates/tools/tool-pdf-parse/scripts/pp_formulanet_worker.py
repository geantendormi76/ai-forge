import sys
import json
import os
import traceback

def main():
    if len(sys.argv) < 2:
        output_res(False, latex="", error="缺少参数: 请提供图片路径 image_path")
        return

    image_path = sys.argv[1]
    if not os.path.exists(image_path):
        output_res(False, latex="", error=f"输入图片文件不存在: {image_path}")
        return

    local_model_dir = "/home/zhz/ai-toolkit/models/tool-pdf-parse/PP-FormulaNet-S"

    try:
        import paddle
        from paddlex import create_model

        # 1. 动态探测 GPU CUDA 显卡算子支持状态
        use_gpu = paddle.is_compiled_with_cuda() and paddle.device.get_device().startswith("gpu")
        primary_device = "gpu:0" if use_gpu else "cpu"

        model = None
        try:
            # 优先点火 GPU 显卡加速引擎
            if os.path.exists(local_model_dir):
                model = create_model(model_name="PP-FormulaNet-S", model_dir=local_model_dir, device=primary_device)
            else:
                model = create_model(model_name="PP-FormulaNet-S", device=primary_device)
        except Exception:
            # 2. 异常降级自愈：GPU 初始化失败时自动无缝切入 CPU 保底
            if primary_device != "cpu":
                if os.path.exists(local_model_dir):
                    model = create_model(model_name="PP-FormulaNet-S", model_dir=local_model_dir, device="cpu")
                else:
                    model = create_model(model_name="PP-FormulaNet-S", device="cpu")
            else:
                raise

        output = model.predict(image_path, batch_size=1)
        
        latex_text = ""
        if output:
            for res in output:
                if isinstance(res, dict):
                    latex_text = res.get("rec_formula", res.get("rec_text", str(res)))
                elif hasattr(res, "get"):
                    latex_text = res.get("rec_formula", str(res))
                elif hasattr(res, "rec_formula"):
                    latex_text = res.rec_formula
                else:
                    try:
                        latex_text = res["rec_formula"]
                    except Exception:
                        latex_text = str(res)
                if latex_text:
                    break

        output_res(True, latex=str(latex_text).strip(), error=None)

    except Exception as e:
        err_msg = f"PP-FormulaNet-S 执行失败: {str(e)}\n{traceback.format_exc()}"
        output_res(False, latex="", error=err_msg)

def output_res(success: bool, latex: str, error: str = None):
    res_obj = {
        "success": success,
        "latex": latex,
        "error": error
    }
    print("___JSON_START___")
    print(json.dumps(res_obj, ensure_ascii=False))
    print("___JSON_END___")

if __name__ == "__main__":
    main()
