use crate::types::{LayoutType, ParsedDocument};

pub fn render_gfm_markdown(doc: &ParsedDocument) -> String {
    let mut lines = Vec::new();

    for elem in &doc.elements {
        let content = elem.content.trim();
        if content.is_empty() {
            continue;
        }

        match elem.layout_type {
            LayoutType::DocTitle => {
                lines.push(format!("# {}\n", content));
            }
            LayoutType::ParagraphTitle => {
                lines.push(format!("## {}\n", content));
            }
            LayoutType::Text | LayoutType::List => {
                lines.push(format!("{}\n", content));
            }
            LayoutType::Table => {
                lines.push(format!("{}\n", content));
            }
            LayoutType::Formula => {
                if content.starts_with('$') {
                    lines.push(format!("{}\n", content));
                } else {
                    lines.push(format!("$$\n{}\n$$\n", content));
                }
            }
            LayoutType::Image => {
                lines.push(format!("![Image]({})\n", if content.is_empty() { "image.png" } else { content }));
            }
            LayoutType::Header | LayoutType::Footer => {
                lines.push(format!("*<small>{}</small>*\n", content));
            }
            _ => {
                lines.push(format!("{}\n", content));
            }
        }
    }

    lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DocElement;

    #[test]
    fn test_render_gfm_markdown() {
        let doc = ParsedDocument {
            elements: vec![
                DocElement {
                    bbox: [0.0, 0.0, 100.0, 50.0],
                    layout_type: LayoutType::DocTitle,
                    order_index: 1,
                    content: "AI-Forge 文档工程".into(),
                    score: 0.99,
                },
                DocElement {
                    bbox: [0.0, 60.0, 100.0, 100.0],
                    layout_type: LayoutType::Text,
                    order_index: 2,
                    content: "这是一段高保真排版文本。".into(),
                    score: 0.98,
                },
            ],
            page_width: 800.0,
            page_height: 1000.0,
        };

        let md = render_gfm_markdown(&doc);
        assert!(md.contains("# AI-Forge 文档工程"));
        assert!(md.contains("这是一段高保真排版文本。"));
    }
}
