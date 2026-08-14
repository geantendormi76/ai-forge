use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

pub struct ImagePageInput<'a> {
    pub width: u32,
    pub height: u32,
    pub rgb_data: &'a [u8],
}

struct PdfObject {
    number: usize,
    content: Vec<u8>,
}

fn pdf_number(val: f64) -> String {
    let s = format!("{:.2}", val);
    if s.ends_with(".00") {
        s[..s.len() - 3].to_string()
    } else {
        s
    }
}

pub fn build_images_pdf(pages: &[ImagePageInput]) -> Result<Vec<u8>, String> {
    if pages.is_empty() {
        return Err("至少需要提供一张图像用于生成 PDF".into());
    }

    let mut objects: Vec<PdfObject> = Vec::new();
    let mut page_refs = Vec::new();

    // 1. Catalog 对象
    objects.push(PdfObject {
        number: 1,
        content: b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec(),
    });

    // 2. 为每张图像构建 Page, Image XObject, Content Stream
    for (index, page) in pages.iter().enumerate() {
        let page_number = 3 + index * 3;
        let image_number = page_number + 1;
        let content_number = page_number + 2;

        let page_w = page.width.max(1) as f64;
        let page_h = page.height.max(1) as f64;
        page_refs.push(format!("{} 0 R", page_number));

        // Page Object
        let page_obj_str = format!(
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /Im{} {} 0 R >> >> /Contents {} 0 R >>\nendobj\n",
            page_number,
            pdf_number(page_w),
            pdf_number(page_h),
            index + 1,
            image_number,
            content_number
        );
        objects.push(PdfObject {
            number: page_number,
            content: page_obj_str.into_bytes(),
        });

        // 压缩 RGB 像素数据
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(page.rgb_data).map_err(|e| format!("Zlib 压缩失败: {}", e))?;
        let compressed_rgb = encoder.finish().map_err(|e| format!("Zlib 压缩流收尾失败: {}", e))?;

        // Image XObject
        let img_header = format!(
            "{} 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
            image_number,
            page.width,
            page.height,
            compressed_rgb.len()
        );
        let mut img_content = img_header.into_bytes();
        img_content.extend_from_slice(&compressed_rgb);
        img_content.extend_from_slice(b"\nendstream\nendobj\n");

        objects.push(PdfObject {
            number: image_number,
            content: img_content,
        });

        // Content Stream (PostScript 矩阵变换)
        let cm_content = format!(
            "q\n{} 0 0 {} 0 0 cm\n/Im{} Do\nQ\n",
            pdf_number(page_w),
            pdf_number(page_h),
            index + 1
        );
        let content_obj_str = format!(
            "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            content_number,
            cm_content.len(),
            cm_content
        );
        objects.push(PdfObject {
            number: content_number,
            content: content_obj_str.into_bytes(),
        });
    }

    // 3. Pages 汇总对象
    let pages_obj_str = format!(
        "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
        page_refs.join(" "),
        page_refs.len()
    );
    objects.push(PdfObject {
        number: 2,
        content: pages_obj_str.into_bytes(),
    });

    // 4. 按对象编号升序排列
    objects.sort_by_key(|o| o.number);

    // 5. 流式组装并计算物理字节偏移量
    let mut chunks = vec![b"%PDF-1.4\n".to_vec()];
    let mut offsets = vec![0usize; objects.len() + 1];

    for obj in &objects {
        let current_offset = chunks.iter().map(|c| c.len()).sum();
        offsets[obj.number] = current_offset;
        chunks.push(obj.content.clone());
    }

    let body_len: usize = chunks.iter().map(|c| c.len()).sum();

    // 6. Xref 索引表构建
    let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for number in 1..=objects.len() {
        xref.push_str(&format!("{:010} 00000 n \n", offsets[number]));
    }

    // 7. Trailer 根索引收尾
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        objects.len() + 1,
        body_len
    );

    let mut final_pdf = Vec::with_capacity(body_len + xref.len() + trailer.len());
    for chunk in chunks {
        final_pdf.extend_from_slice(&chunk);
    }
    final_pdf.extend_from_slice(xref.as_bytes());
    final_pdf.extend_from_slice(trailer.as_bytes());

    Ok(final_pdf)
}
