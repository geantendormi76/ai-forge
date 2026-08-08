import sys
import json
import os
import io
import re
import struct
import hashlib
import traceback
import tempfile
import fitz
from PIL import Image, ImageEnhance, ImageOps
import numpy as np

# ==========================================
# 🛡️ 工业级防线：OS 级别文件描述符劫持 (FD Hijacking)
# ==========================================
def hijack_stdout():
    try:
        real_stdout_fd = os.dup(1)
        os.dup2(2, 1)
        ipc_out = os.fdopen(real_stdout_fd, 'wb')
        return ipc_out
    except Exception as e:
        print("FD 劫持失败: " + str(e), file=sys.stderr)
        sys.exit(1)

IPC_OUT = None
IPC_IN = None

def init_ipc():
    global IPC_OUT, IPC_IN
    IPC_OUT = hijack_stdout()
    IPC_IN = sys.stdin.buffer

def send_ipc_message(method: str, params: dict):
    msg = {"method": method, "params": params}
    data = json.dumps(msg, ensure_ascii=False).encode('utf-8')
    IPC_OUT.write(struct.pack('>I', len(data)))
    IPC_OUT.write(data)
    IPC_OUT.flush()

def recv_ipc_message():
    header = IPC_IN.read(4)
    if not header or len(header) < 4:
        return None
    msg_len = struct.unpack('>I', header)[0]
    data = IPC_IN.read(msg_len)
    if len(data) < msg_len:
        return None
    return json.loads(data.decode('utf-8'))

_ocr_engine = None
_rec_engine = None
_table_engine = None
_formula_engine = None

def resolve_model_dir(model_name: str) -> str:
    candidates = [
        os.path.join("/home/zhz/ai-forge/models/tool-pdf-parse", model_name),
        os.path.join("/home/zhz/ai-toolkit/models/tool-pdf-parse", model_name),
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return os.path.join("/home/zhz/ai-forge/models/tool-pdf-parse", model_name)

local_rec_dir = resolve_model_dir("PP-OCRv6_medium_rec")
local_det_dir = resolve_model_dir("PP-OCRv6_medium_det")
local_slanet_dir = resolve_model_dir("SLANet_plus")
local_formula_dir = resolve_model_dir("PP-FormulaNet-S")
local_model_dir = resolve_model_dir("PP-DocLayoutV3")

def get_rec_engine(primary_device="gpu:0"):
    global _rec_engine
    if _rec_engine is None:
        try:
            from paddlex import create_model
            if os.path.exists(local_rec_dir):
                _rec_engine = create_model(model_name="PP-OCRv6_medium_rec", model_dir=local_rec_dir, device=primary_device)
            else:
                _rec_engine = create_model(model_name="PP-OCRv6_medium_rec", device=primary_device)
        except Exception:
            _rec_engine = None
    return _rec_engine

def get_ocr_engine():
    global _ocr_engine
    if _ocr_engine is None:
        try:
            from paddleocr import PaddleOCR
            if os.path.exists(local_rec_dir) and os.path.exists(local_det_dir):
                _ocr_engine = PaddleOCR(
                    text_recognition_model_dir=local_rec_dir,
                    text_detection_model_dir=local_det_dir,
                    text_recognition_model_name="PP-OCRv6_medium_rec",
                    text_detection_model_name="PP-OCRv6_medium_det",
                    use_doc_orientation_classify=False,
                    use_doc_unwarping=False,
                    use_textline_orientation=True
                )
            else:
                _ocr_engine = PaddleOCR(
                    lang="ch",
                    use_doc_orientation_classify=False,
                    use_doc_unwarping=False,
                    use_textline_orientation=True
                )
        except Exception:
            _ocr_engine = None
    return _ocr_engine

def get_table_engine(primary_device="gpu:0"):
    global _table_engine
    if _table_engine is None:
        try:
            from paddlex import create_model
            if os.path.exists(local_slanet_dir):
                _table_engine = create_model(model_name="SLANet_plus", model_dir=local_slanet_dir, device=primary_device)
            else:
                _table_engine = create_model(model_name="SLANet_plus", device=primary_device)
        except Exception:
            _table_engine = None
    return _table_engine

def get_formula_engine(primary_device="gpu:0"):
    global _formula_engine
    if _formula_engine is None:
        try:
            from paddlex import create_model
            if os.path.exists(local_formula_dir):
                _formula_engine = create_model(model_name="PP-FormulaNet-S", model_dir=local_formula_dir, device=primary_device)
            else:
                _formula_engine = create_model(model_name="PP-FormulaNet-S", device=primary_device)
        except Exception:
            if primary_device != "cpu":
                try:
                    if os.path.exists(local_formula_dir):
                        _formula_engine = create_model(model_name="PP-FormulaNet-S", model_dir=local_formula_dir, device="cpu")
                    else:
                        _formula_engine = create_model(model_name="PP-FormulaNet-S", device="cpu")
                except Exception:
                    _formula_engine = None
            else:
                _formula_engine = None
    return _formula_engine

def enhance_image_for_ocr(pil_img):
    try:
        gray = pil_img.convert('L')
        auto = ImageOps.autocontrast(gray, cutoff=0.5)
        return auto.convert('RGB')
    except Exception:
        return pil_img

def parse_bbox_coords(raw_coords):
    if raw_coords is None:
        return [0.0, 0.0, 0.0, 0.0]
    if isinstance(raw_coords, (np.ndarray, list, tuple)):
        flat = np.array(raw_coords).flatten().tolist()
        if len(flat) == 4:
            return [float(c) for c in flat]
        if len(flat) == 8:
            xs = [float(flat[i]) for i in range(0, 8, 2)]
            ys = [float(flat[i+1]) for i in range(0, 8, 2)]
            return [min(xs), min(ys), max(xs), max(ys)]
        if len(flat) > 0:
            return [float(c) for c in flat[:4]]
    return [0.0, 0.0, 0.0, 0.0]

def extract_ocr_texts_pure_rec(rec_engine, crop_path):
    if rec_engine is None:
        return ""
    try:
        res = list(rec_engine.predict(crop_path, batch_size=1))
        text = ""
        if res:
            for item in res:
                if isinstance(item, dict):
                    text = item.get("rec_text", item.get("rec_texts", ""))
                    if isinstance(text, list):
                        text = " ".join(text)
                elif hasattr(item, "rec_text"):
                    text = item.rec_text
                elif hasattr(item, "rec_texts"):
                    text = " ".join(item.rec_texts)
        return str(text).strip()
    except Exception:
        return ""

def extract_ocr_texts(ocr_engine, crop_path):
    if ocr_engine is None:
        return ""
    try:
        res = ocr_engine.predict(crop_path)
        extracted_lines = []
        if res:
            for item in res:
                if isinstance(item, dict):
                    rec_texts = item.get("rec_texts", item.get("rec_text", []))
                    if rec_texts:
                        extracted_lines.extend(rec_texts)
                elif hasattr(item, "rec_texts"):
                    extracted_lines.extend(item.rec_texts)
                elif hasattr(item, "get"):
                    rec_texts = item.get("rec_texts", [])
                    if rec_texts:
                        extracted_lines.extend(rec_texts)
        return "\n".join(extracted_lines).strip()
    except Exception:
        return ""

def clean_and_normalize_formula(raw_latex):
    if not raw_latex:
        return ""
    clean = str(raw_latex).strip()
    clean = clean.replace("\\_", "_").replace("\\^", "^")
    clean = re.sub(r"\\+\s*cases\s*\\\\?$", r"\\end{cases}", clean)
    clean = re.sub(r"\\(parser|parser|text)\{([a-zA-Z\s]+)\}", lambda m: "\\text{" + m.group(2).replace(" ", "") + "}", clean)
    return clean

def run_deeptrack_pipeline(input_path: str, output_dir: str, pages_str: str = ""):
    if not os.path.exists(input_path):
        return False, 0.0, 0.0, [], [], f"物理文件不存在: {input_path}"

    images_dir = os.path.join(output_dir, "images")
    os.makedirs(images_dir, exist_ok=True)

    session_dir = tempfile.mkdtemp(prefix="deeptrack_sess_")

    ext = os.path.splitext(input_path)[1].lower()
    is_image_input = ext in [".png", ".jpg", ".jpeg", ".webp", ".bmp"]

    try:
        import paddle
        from paddlex import create_model

        use_gpu = paddle.is_compiled_with_cuda() and paddle.device.get_device().startswith("gpu")
        primary_device = "gpu:0" if use_gpu else "cpu"

        model = None
        try:
            if os.path.exists(local_model_dir):
                model = create_model(model_name="PP-DocLayoutV3", model_dir=local_model_dir, device=primary_device)
            else:
                model = create_model(model_name="PP-DocLayoutV3", device=primary_device)
        except Exception:
            if primary_device != "cpu":
                if os.path.exists(local_model_dir):
                    model = create_model(model_name="PP-DocLayoutV3", model_dir=local_model_dir, device="cpu")
                else:
                    model = create_model(model_name="PP-DocLayoutV3", device="cpu")
            else:
                raise

        all_elements = []
        extracted_images = []
        page_w, page_h = 0.0, 0.0

        if is_image_input:
            pil_img = Image.open(input_path).convert("RGB")
            target_max_dim = 2500.0
            max_dim = max(pil_img.width, pil_img.height)
            if max_dim > target_max_dim:
                scale = target_max_dim / max_dim
                new_w, new_h = int(pil_img.width * scale), int(pil_img.height * scale)
                pil_img = pil_img.resize((new_w, new_h), Image.Resampling.LANCZOS)

            page_w, page_h = float(pil_img.width), float(pil_img.height)

            temp_img_path = os.path.join(session_dir, "deeptrack_img_input.png")
            pil_img.save(temp_img_path)

            process_pages = [(0, temp_img_path, None, pil_img)]
        else:
            doc = fitz.open(input_path)
            total_pages = len(doc)

            target_pages = []
            if pages_str and pages_str.strip():
                for p in pages_str.split(","):
                    try:
                        p_num = int(p.strip()) - 1
                        if 0 <= p_num < total_pages:
                            target_pages.append(p_num)
                    except ValueError:
                        pass
            if not target_pages:
                target_pages = list(range(total_pages))

            process_pages = []
            for p_idx in target_pages:
                page = doc[p_idx]
                target_max_dim = 2500.0
                zoom = target_max_dim / max(page.rect.width, page.rect.height)
                mat = fitz.Matrix(zoom, zoom)
                pix = page.get_pixmap(matrix=mat)

                p_w, p_h = float(pix.width), float(pix.height)
                page_w, page_h = max(page_w, p_w), max(page_h, p_h)

                temp_img_path = os.path.join(session_dir, f"deeptrack_page_{p_idx}.png")
                pix.save(temp_img_path)
                page_pil = Image.open(temp_img_path)
                process_pages.append((p_idx, temp_img_path, page, page_pil))

        for p_idx, temp_img_path, fitz_page, page_pil in process_pages:
            output = model.predict(temp_img_path, batch_size=1)

            if output:
                for res_data in output:
                    boxes = []
                    if isinstance(res_data, dict):
                        boxes = res_data.get("boxes", res_data.get("dt_polys", []))
                    elif hasattr(res_data, "boxes"):
                        boxes = res_data.boxes
                    elif hasattr(res_data, "get"):
                        boxes = res_data.get("boxes", [])

                    if not boxes and hasattr(res_data, "json"):
                        json_res = res_data.json
                        if isinstance(json_res, dict) and "boxes" in json_res:
                            boxes = json_res["boxes"]

                    for elem_idx, b in enumerate(boxes):
                        label = "text"
                        coords = []
                        if isinstance(b, dict):
                            label = b.get("label", "text")
                            coords = b.get("coordinate", b.get("box", []))
                        elif hasattr(b, "label"):
                            label = getattr(b, "label")
                            coords = getattr(b, "coordinate", getattr(b, "box", []))

                        label_str = str(label).lower()

                        # 🛡️ 对症下药防线 1: 过滤侧边栏/页眉页脚水印，绝不让 arXiv 竖排文字污染主干
                        if label_str in ["aside_text", "sidebar_text", "page_number", "header", "footer"]:
                            continue

                        bbox_px = parse_bbox_coords(coords)
                        x0, y0, x1, y1 = bbox_px

                        vec_text = ""
                        if fitz_page is not None:
                            dpi_scale = page_w / fitz_page.rect.width if fitz_page.rect.width > 0 else 1.0
                            rect_pt = fitz.Rect(x0 / dpi_scale, y0 / dpi_scale, x1 / dpi_scale, y1 / dpi_scale)
                            vec_text = fitz_page.get_text("text", clip=rect_pt).strip()

                        final_text = None
                        cells_data = None
                        structure_tokens_data = None

                        if label_str in ["table"]:
                            crop_box = (max(0, int(x0) - 12), max(0, int(y0) - 10), min(page_pil.width, int(x1) + 12), min(page_pil.height, int(y1) + 10))
                            if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                crop_img = page_pil.crop(crop_box)
                                crop_temp_path = os.path.join(session_dir, f"deeptrack_table_{p_idx}_{elem_idx}.png")
                                crop_img.save(crop_temp_path)

                                table_eng = get_table_engine(primary_device)
                                if table_eng is not None:
                                    try:
                                        t_out = list(table_eng.predict(crop_temp_path, batch_size=1))
                                        if t_out:
                                            cells_list = []
                                            for t_item in t_out:
                                                c_boxes = []
                                                struct_toks = []
                                                if isinstance(t_item, dict):
                                                    c_boxes = t_item.get("bbox", t_item.get("boxes", []))
                                                    struct_toks = t_item.get("structure", [])
                                                elif hasattr(t_item, "get"):
                                                    c_boxes = t_item.get("bbox", t_item.get("boxes", []))
                                                    struct_toks = t_item.get("structure", [])
                                                else:
                                                    c_boxes = getattr(t_item, "bbox", getattr(t_item, "boxes", []))
                                                    struct_toks = getattr(t_item, "structure", [])

                                                if struct_toks:
                                                    structure_tokens_data = [str(tok) for tok in struct_toks]

                                                for cb in c_boxes:
                                                    cb_coords = parse_bbox_coords(cb)
                                                    abs_cb = [x0 + cb_coords[0], y0 + cb_coords[1], x0 + cb_coords[2], y0 + cb_coords[3]]

                                                    cell_text = ""
                                                    if fitz_page is not None:
                                                        cell_rect_pt = fitz.Rect(abs_cb[0] / dpi_scale, abs_cb[1] / dpi_scale, abs_cb[2] / dpi_scale, abs_cb[3] / dpi_scale)
                                                        cell_text = fitz_page.get_text("text", clip=cell_rect_pt).strip()

                                                    if not cell_text:
                                                        crop_cell_box = (
                                                            max(0, int(cb_coords[0]) - 10),
                                                            max(0, int(cb_coords[1]) - 8),
                                                            min(crop_img.width, int(cb_coords[2]) + 10),
                                                            min(crop_img.height, int(cb_coords[3]) + 8)
                                                        )
                                                        if crop_cell_box[2] > crop_cell_box[0] and crop_cell_box[3] > crop_cell_box[1]:
                                                            cell_crop_img = crop_img.crop(crop_cell_box)
                                                            cell_crop_img = enhance_image_for_ocr(cell_crop_img)
                                                            cell_crop_path = os.path.join(session_dir, f"deeptrack_cell_{p_idx}_{elem_idx}.png")
                                                            cell_crop_img.save(cell_crop_path)
                                                            rec_eng = get_rec_engine(primary_device)
                                                            cell_text = extract_ocr_texts_pure_rec(rec_eng, cell_crop_path)

                                                    clean_c_text = cell_text.replace('\n', ' ').strip()
                                                    cells_list.append({
                                                        "bbox": abs_cb,
                                                        "text": clean_c_text if clean_c_text else None
                                                    })
                                            if cells_list:
                                                cells_data = cells_list
                                    except Exception:
                                        cells_data = None

                            if not cells_data and vec_text:
                                final_text = vec_text

                        elif label_str in ["formula", "isolate_formula", "display_formula", "inline_formula", "math"]:
                            crop_box = (max(0, int(x0) - 25), max(0, int(y0) - 10), min(page_pil.width, int(x1) + 25), min(page_pil.height, int(y1) + 10))
                            if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                crop_img = page_pil.crop(crop_box)
                                crop_formula_path = os.path.join(session_dir, f"deeptrack_formula_{p_idx}_{elem_idx}.png")
                                crop_img.save(crop_formula_path)

                                formula_eng = get_formula_engine(primary_device)
                                if formula_eng is not None:
                                    try:
                                        f_out = list(formula_eng.predict(crop_formula_path, batch_size=1))
                                        latex_text = ""
                                        if f_out:
                                            for res in f_out:
                                                if isinstance(res, dict):
                                                    latex_text = res.get("rec_formula", res.get("rec_text", str(res)))
                                                elif hasattr(res, "rec_formula"):
                                                    latex_text = res.rec_formula
                                                elif hasattr(res, "get"):
                                                    latex_text = res.get("rec_formula", str(res))
                                                else:
                                                    try:
                                                        latex_text = res["rec_formula"]
                                                    except Exception:
                                                        latex_text = str(res)
                                                if latex_text:
                                                    break
                                        if latex_text:
                                            final_text = clean_and_normalize_formula(latex_text)
                                    except Exception:
                                        final_text = None

                                if not final_text:
                                    enhanced_formula = enhance_image_for_ocr(crop_img)
                                    enhanced_formula.save(crop_formula_path)
                                    rec_eng = get_rec_engine(primary_device)
                                    final_text = extract_ocr_texts_pure_rec(rec_eng, crop_formula_path)
                                if not final_text and vec_text:
                                    final_text = vec_text
                            else:
                                final_text = vec_text if vec_text else None

                        elif label_str in ["figure", "image", "illustration"]:
                            crop_box = (max(0, int(x0)), max(0, int(y0)), min(page_pil.width, int(x1)), min(page_pil.height, int(y1)))
                            if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                crop_img = page_pil.crop(crop_box)

                                img_byte_arr = io.BytesIO()
                                crop_img.save(img_byte_arr, format='PNG')
                                img_bytes = img_byte_arr.getvalue()

                                md5_hash = hashlib.md5(img_bytes).hexdigest()
                                img_filename = f"{md5_hash}.png"
                                img_full_path = os.path.join(images_dir, img_filename)

                                if not os.path.exists(img_full_path):
                                    with open(img_full_path, "wb") as f:
                                        f.write(img_bytes)

                                rel_path = f"images/{img_filename}"
                                final_text = rel_path
                                if rel_path not in extracted_images:
                                    extracted_images.append(rel_path)
                        else:
                            # 🛡️ 物理防线 2: 优先使用精准向量文本（若向量文本字数充足）
                            if vec_text and len(vec_text.strip()) > 3:
                                final_text = vec_text
                            else:
                                # 🛡️ 物理防线 3: 精准复原 ai-toolkit 原始 Crop 框比例
                                left_margin_px = 30 if x0 > 400 else 150

                                if label_str in ["paragraph_title", "table_title", "header", "footer", "number", "formula_number"]:
                                    crop_box = (max(0, int(x0) - left_margin_px), max(0, int(y0) - 8), min(page_pil.width, int(x1) + 150), min(page_pil.height, int(y1) + 8))
                                    if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                        crop_img = page_pil.crop(crop_box)
                                        crop_block_path = os.path.join(session_dir, f"deeptrack_block_{p_idx}_{elem_idx}.png")
                                        crop_img.save(crop_block_path)
                                        rec_eng = get_rec_engine(primary_device)
                                        final_text = extract_ocr_texts_pure_rec(rec_eng, crop_block_path)
                                elif label_str in ["figure_title", "chart_title"]:
                                    crop_box = (max(0, int(x0) - left_margin_px), max(0, int(y0) - 20), min(page_pil.width, int(x1) + 250), min(page_pil.height, int(y1) + 20))
                                    if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                        crop_img = page_pil.crop(crop_box)
                                        crop_block_path = os.path.join(session_dir, f"deeptrack_block_{p_idx}_{elem_idx}.png")
                                        crop_img.save(crop_block_path)
                                        ocr = get_ocr_engine()
                                        final_text = extract_ocr_texts(ocr, crop_block_path)
                                elif label_str in ["doc_title", "document_title", "title"]:
                                    crop_box = (max(0, int(x0) - left_margin_px), max(0, int(y0) - 20), min(page_pil.width, int(x1) + 300), min(page_pil.height, int(y1) + 20))
                                    if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                        crop_img = page_pil.crop(crop_box)
                                        crop_block_path = os.path.join(session_dir, f"deeptrack_block_{p_idx}_{elem_idx}.png")
                                        crop_img.save(crop_block_path)
                                        ocr = get_ocr_engine()
                                        final_text = extract_ocr_texts(ocr, crop_block_path)
                                else:
                                    crop_box = (max(0, int(x0) - 100), max(0, int(y0) - 12), min(page_pil.width, int(x1) + 100), min(page_pil.height, int(y1) + 12))
                                    if crop_box[2] > crop_box[0] and crop_box[3] > crop_box[1]:
                                        crop_img = page_pil.crop(crop_box)
                                        crop_block_path = os.path.join(session_dir, f"deeptrack_block_{p_idx}_{elem_idx}.png")
                                        crop_img.save(crop_block_path)
                                        ocr = get_ocr_engine()
                                        final_text = extract_ocr_texts(ocr, crop_block_path)

                        all_elements.append({
                            "page_index": p_idx,
                            "bbox": bbox_px,
                            "label": str(label),
                            "text": final_text,
                            "cells": cells_data,
                            "structure_tokens": structure_tokens_data
                        })

        return True, page_w, page_h, all_elements, extracted_images, None

    except Exception as e:
        err_msg = f"DeepTrack Worker 运行异常: {str(e)}\n{traceback.format_exc()}"
        return False, 0.0, 0.0, [], [], err_msg

def main():
    if len(sys.argv) > 1:
        input_path = sys.argv[1]
        output_dir = sys.argv[2] if len(sys.argv) > 2 else "/tmp"
        pages_str = sys.argv[3] if len(sys.argv) > 3 else ""
        ok, pw, ph, elems, imgs, err = run_deeptrack_pipeline(input_path, output_dir, pages_str)
        res_obj = {
            "success": ok,
            "page_width": pw,
            "page_height": ph,
            "elements": elems,
            "images": imgs,
            "error": err
        }
        print("___JSON_START___")
        print(json.dumps(res_obj, ensure_ascii=False))
        print("___JSON_END___")
    else:
        init_ipc()
        send_ipc_message("system.ready", {"status": "DeepTrack Worker 已点火就绪"})
        while True:
            try:
                msg = recv_ipc_message()
                if msg is None:
                    break
                method = msg.get("method")
                params = msg.get("params", {})
                if method == "parse_pdf":
                    inp = params.get("input_path", "")
                    out = params.get("output_dir", "")
                    pgs = params.get("pages_str", "")
                    ok, pw, ph, elems, imgs, err = run_deeptrack_pipeline(inp, out, pgs)
                    send_ipc_message("parse_result", {
                        "success": ok,
                        "page_width": pw,
                        "page_height": ph,
                        "elements": elems,
                        "images": imgs,
                        "error": err
                    })
                elif method == "exit":
                    break
            except Exception as e:
                send_ipc_message("error", {"message": str(e)})

if __name__ == "__main__":
    main()
