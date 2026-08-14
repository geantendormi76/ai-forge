use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone)]
pub struct EpubChapter {
    pub title: String,
    pub body_xhtml: String,
}

pub fn escape_xml_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn split_chapters(raw: &str, is_markdown: bool) -> Vec<EpubChapter> {
    let text = raw.replace("\r\n", "\n");
    if is_markdown {
        let mut chapters = Vec::new();
        let mut current_title = String::new();
        let mut current_body = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let hashes = trimmed.chars().take_while(|&c| c == '#').count();
                if hashes <= 6 && trimmed[hashes..].starts_with(' ') {
                    if !current_title.is_empty() || !current_body.is_empty() {
                        chapters.push(EpubChapter {
                            title: if current_title.is_empty() { format!("第 {} 章", chapters.len() + 1) } else { current_title.clone() },
                            body_xhtml: current_body.join("\n"),
                        });
                        current_body.clear();
                    }
                    current_title = trimmed[hashes..].trim().to_string();
                    continue;
                }
            }
            if !trimmed.is_empty() {
                current_body.push(format!("<p>{}</p>", escape_xml_text(trimmed)));
            }
        }

        if !current_title.is_empty() || !current_body.is_empty() {
            chapters.push(EpubChapter {
                title: if current_title.is_empty() { format!("第 {} 章", chapters.len() + 1) } else { current_title },
                body_xhtml: current_body.join("\n"),
            });
        }

        if !chapters.is_empty() {
            return chapters;
        }
    }

    // 普通纯文本智能按段落分块
    let paragraphs: Vec<&str> = text.split("\n\n").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let mut chapters = Vec::new();
    let mut buffer = Vec::new();
    let mut char_count = 0;

    for p in paragraphs {
        buffer.push(format!("<p>{}</p>", escape_xml_text(p)));
        char_count += p.len();
        if char_count > 2000 {
            chapters.push(EpubChapter {
                title: format!("第 {} 节", chapters.len() + 1),
                body_xhtml: buffer.join("\n"),
            });
            buffer.clear();
            char_count = 0;
        }
    }

    if !buffer.is_empty() {
        chapters.push(EpubChapter {
            title: format!("第 {} 节", chapters.len() + 1),
            body_xhtml: buffer.join("\n"),
        });
    }

    if chapters.is_empty() {
        chapters.push(EpubChapter {
            title: "正文".to_string(),
            body_xhtml: format!("<p>{}</p>", escape_xml_text(&text)),
        });
    }

    chapters
}

pub fn build_epub_bytes(title: &str, chapters: &[EpubChapter]) -> Result<Vec<u8>, String> {
    if chapters.is_empty() {
        return Err("EPUB 至少需要包含一个章节".into());
    }

    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);

    // 1. 铁律：mimetype 必须是第一个条目且不压缩 (Stored)
    let stored_opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", stored_opts).map_err(|e| e.to_string())?;
    zip.write_all(b"application/epub+zip").map_err(|e| e.to_string())?;

    let deflated_opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflated_opts).map_err(|e| e.to_string())?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).map_err(|e| e.to_string())?;

    // 3. 章节 XHTML 文件
    let mut manifest_items = vec![
        r#"<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#.to_string()
    ];
    let mut spine_items = Vec::new();
    let mut nav_points = Vec::new();

    for (index, ch) in chapters.iter().enumerate() {
        let ch_id = format!("chapter-{}", index + 1);
        let ch_path = format!("OEBPS/{}.xhtml", ch_id);
        
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{}</title></head>
<body>
<h1>{}</h1>
{}
</body>
</html>"#,
            escape_xml_text(&ch.title),
            escape_xml_text(&ch.title),
            ch.body_xhtml
        );

        zip.start_file(&ch_path, deflated_opts).map_err(|e| e.to_string())?;
        zip.write_all(xhtml.as_bytes()).map_err(|e| e.to_string())?;

        manifest_items.push(format!(r#"<item id="{}" href="{}.xhtml" media-type="application/xhtml+xml"/>"#, ch_id, ch_id));
        spine_items.push(format!(r#"<itemref idref="{}"/>"#, ch_id));
        nav_points.push(format!(
            r#"    <navPoint id="nav-{}" playOrder="{}"><navLabel><text>{}</text></navLabel><content src="{}.xhtml"/></navPoint>"#,
            index + 1,
            index + 1,
            escape_xml_text(&ch.title),
            ch_id
        ));
    }

    // 4. OEBPS/content.opf
    let content_opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>{}</dc:title>
    <dc:language>zh-CN</dc:language>
    <dc:identifier id="bookid">urn:uuid:ai-forge-epub-2026</dc:identifier>
  </metadata>
  <manifest>
{}
  </manifest>
  <spine toc="ncx">
{}
  </spine>
</package>"#,
        escape_xml_text(title),
        manifest_items.join("\n"),
        spine_items.join("\n")
    );
    zip.start_file("OEBPS/content.opf", deflated_opts).map_err(|e| e.to_string())?;
    zip.write_all(content_opf.as_bytes()).map_err(|e| e.to_string())?;

    // 5. OEBPS/toc.ncx
    let toc_ncx = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head><meta name="dtb:uid" content="bookid"/></head>
  <docTitle><text>{}</text></docTitle>
  <navMap>
{}
  </navMap>
</ncx>"#,
        escape_xml_text(title),
        nav_points.join("\n")
    );
    zip.start_file("OEBPS/toc.ncx", deflated_opts).map_err(|e| e.to_string())?;
    zip.write_all(toc_ncx.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}
