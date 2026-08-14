use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

struct RunToken {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

fn parse_md_inline_runs(line: &str) -> Vec<RunToken> {
    let mut runs = Vec::new();
    let mut chars = line.chars().peekable();
    let mut buf = String::new();

    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next();
            if !buf.is_empty() {
                runs.push(RunToken { text: buf.clone(), bold: false, italic: false, code: false });
                buf.clear();
            }
            let mut bold_buf = String::new();
            while let Some(b_c) = chars.next() {
                if b_c == '*' && chars.peek() == Some(&'*') {
                    chars.next();
                    break;
                }
                bold_buf.push(b_c);
            }
            runs.push(RunToken { text: bold_buf, bold: true, italic: false, code: false });
        } else if c == '`' {
            if !buf.is_empty() {
                runs.push(RunToken { text: buf.clone(), bold: false, italic: false, code: false });
                buf.clear();
            }
            let mut code_buf = String::new();
            for code_c in chars.by_ref() {
                if code_c == '`' {
                    break;
                }
                code_buf.push(code_c);
            }
            runs.push(RunToken { text: code_buf, bold: false, italic: false, code: true });
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        runs.push(RunToken { text: buf, bold: false, italic: false, code: false });
    }

    if runs.is_empty() {
        runs.push(RunToken { text: String::new(), bold: false, italic: false, code: false });
    }

    runs
}

fn docx_paragraph_xml(runs: &[RunToken], size_half_pt: Option<u32>, bold: bool, indent: Option<u32>) -> String {
    let mut p_pr: Vec<String> = Vec::new();
    if let Some(ind) = indent {
        p_pr.push(format!(r#"<w:ind w:left="{}"/>"#, ind));
    }
    let p_pr_str = if p_pr.is_empty() { String::new() } else { format!("<w:pPr>{}</w:pPr>", p_pr.join("")) };

    let mut r_xmls = Vec::new();
    for r in runs {
        let mut r_pr: Vec<String> = Vec::new();
        if r.bold || bold {
            r_pr.push("<w:b/>".to_string());
        }
        if r.italic {
            r_pr.push("<w:i/>".to_string());
        }
        if r.code {
            r_pr.push(r#"<w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/>"#.to_string());
        }
        if let Some(sz) = size_half_pt {
            r_pr.push(format!(r#"<w:sz w:val="{}"/><w:szCs w:val="{}"/>"#, sz, sz));
        }
        let r_pr_str = if r_pr.is_empty() { String::new() } else { format!("<w:rPr>{}</w:rPr>", r_pr.join("")) };
        r_xmls.push(format!(r#"<w:r>{}<w:t xml:space="preserve">{}</w:t></w:r>"#, r_pr_str, escape_xml(&r.text)));
    }

    format!("<w:p>{}{}</w:p>", p_pr_str, r_xmls.join(""))
}

pub fn build_docx_bytes(raw: &str, is_markdown: bool) -> Result<Vec<u8>, String> {
    let text = raw.replace("\r\n", "\n");
    let mut paragraphs = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            paragraphs.push("<w:p/>".to_string());
            continue;
        }

        if is_markdown {
            if trimmed.starts_with('#') {
                let hashes = trimmed.chars().take_while(|&c| c == '#').count();
                if hashes <= 6 && trimmed[hashes..].starts_with(' ') {
                    let heading_text = trimmed[hashes..].trim();
                    let size = [36, 32, 28, 26, 24, 24][hashes.min(6) - 1];
                    let runs = parse_md_inline_runs(heading_text);
                    paragraphs.push(docx_paragraph_xml(&runs, Some(size), true, None));
                    continue;
                }
            }
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let bullet_text = &trimmed[2..].trim();
                let mut runs = vec![RunToken { text: "• ".to_string(), bold: false, italic: false, code: false }];
                runs.extend(parse_md_inline_runs(bullet_text));
                paragraphs.push(docx_paragraph_xml(&runs, None, false, Some(360)));
                continue;
            }
        }

        let runs = parse_md_inline_runs(trimmed);
        paragraphs.push(docx_paragraph_xml(&runs, None, false, None));
    }

    let mut buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buf);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options).map_err(|e| e.to_string())?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).map_err(|e| e.to_string())?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options).map_err(|e| e.to_string())?;
    zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).map_err(|e| e.to_string())?;

    // 3. word/document.xml
    let doc_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{}<w:sectPr/></w:body></w:document>"#,
        paragraphs.join("")
    );
    zip.start_file("word/document.xml", options).map_err(|e| e.to_string())?;
    zip.write_all(doc_xml.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}
