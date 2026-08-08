import sys
import json
import pathlib

try:
    import pymupdf4llm
except ImportError:
    print("___JSON_START___")
    print(json.dumps({"success": False, "error": "pymupdf4llm 库未安装"}))
    print("___JSON_END___")
    sys.exit(1)

def process_pdf(pdf_path, output_dir, pages=None):
    try:
        pdf_path = pathlib.Path(pdf_path)
        output_dir = pathlib.Path(output_dir)
        images_dir = output_dir / "images"
        images_dir.mkdir(parents=True, exist_ok=True)

        md_text = pymupdf4llm.to_markdown(
            doc=str(pdf_path),
            pages=pages,
            write_images=True,
            image_path=str(images_dir),
            image_format="png"
        )

        extracted_images = [f.name for f in images_dir.glob("*.png")]

        payload = {
            "success": True,
            "markdown": md_text,
            "images": extracted_images
        }

        print("___JSON_START___")
        print(json.dumps(payload, ensure_ascii=False))
        print("___JSON_END___")

    except Exception as e:
        print("___JSON_START___")
        print(json.dumps({"success": False, "error": str(e)}, ensure_ascii=False))
        print("___JSON_END___")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("___JSON_START___")
        print(json.dumps({"success": False, "error": "参数不足: 需要 pdf_path 和 output_dir"}))
        print("___JSON_END___")
        sys.exit(1)

    pages = None
    if len(sys.argv) >= 4 and sys.argv[3].strip():
        pages = [int(p) - 1 for p in sys.argv[3].split(",") if p.strip().isdigit() and int(p) > 0]

    process_pdf(sys.argv[1], sys.argv[2], pages)
