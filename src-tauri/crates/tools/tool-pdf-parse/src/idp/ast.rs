#![allow(non_snake_case, dead_code)]

use crate::idp::config::{BoundingBox, 排序标签};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub total_pages: usize,
    pub title: Option<String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            total_pages: 0,
            title: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentNode {
    pub metadata: DocumentMetadata,
    pub pages: Vec<PageNode>,
}

impl DocumentNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_page(&mut self, page: PageNode) {
        self.pages.push(page);
        self.metadata.total_pages = self.pages.len();
    }

    pub fn render_to_markdown(&self) -> String {
        let mut markdown_blocks = Vec::new();

        for page in &self.pages {
            for col_group in &page.column_groups {
                for block in &col_group.blocks {
                    let rendered = block.render_markdown();
                    if !rendered.trim().is_empty() {
                        markdown_blocks.push(rendered);
                    }
                }
            }
        }

        markdown_blocks.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageNode {
    pub page_index: usize,
    pub width: f32,
    pub height: f32,
    pub column_groups: Vec<ColumnGroupNode>,
}

impl PageNode {
    pub fn new(page_index: usize, width: f32, height: f32) -> Self {
        Self {
            page_index,
            width,
            height,
            column_groups: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnLayoutType {
    SingleColumn,
    MultiColumn { num_columns: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnGroupNode {
    pub layout_type: ColumnLayoutType,
    pub blocks: Vec<BlockNode>,
}

impl ColumnGroupNode {
    pub fn new(layout_type: ColumnLayoutType) -> Self {
        Self {
            layout_type,
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    DocTitle,
    SectionHeader,
    Paragraph,
    Table,
    Figure,
    Formula,
    Header,
    Footer,
}

impl BlockType {
    pub fn default_priority(&self) -> i32 {
        match self {
            BlockType::DocTitle => 100,
            BlockType::Header | BlockType::Footer => -10,
            BlockType::SectionHeader => 80,
            BlockType::Table | BlockType::Figure | BlockType::Formula => 50,
            BlockType::Paragraph => 10,
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label.to_lowercase().as_str() {
            "header" => BlockType::Header,
            "footer" | "number" => BlockType::Footer,
            "doc_title" | "document_title" | "title" => BlockType::DocTitle,
            "paragraph_title" | "section_header" | "figure_title" => BlockType::SectionHeader,
            "figure" | "image" | "illustration" | "chart" => BlockType::Figure,
            "table" => BlockType::Table,
            "formula" | "isolate_formula" | "display_formula" | "inline_formula" | "math" => {
                BlockType::Formula
            }
            _ => BlockType::Paragraph,
        }
    }

    pub fn to_sort_tag(&self) -> 排序标签 {
        match self {
            BlockType::Header => 排序标签::页眉,
            BlockType::Footer => 排序标签::页脚,
            BlockType::DocTitle => 排序标签::文档标题,
            BlockType::SectionHeader => 排序标签::段落标题,
            BlockType::Figure => 排序标签::视觉实体,
            BlockType::Table | BlockType::Formula => 排序标签::跨栏元素,
            BlockType::Paragraph => 排序标签::普通文本,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellNode {
    pub bbox: BoundingBox,
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockContent {
    Text(Vec<InlineNode>),
    Table {
        structure_tokens: Vec<String>,
        cells: Vec<CellNode>,
        is_cross_page: bool,
    },
    Figure {
        image_path: String,
        caption_text: Option<String>,
    },
    Formula {
        latex: String,
        is_display: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InlineNode {
    TextSpan {
        text: String,
        is_bold: bool,
        is_italic: bool,
    },
    InlineFormula {
        latex: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNode {
    pub id: String,
    pub block_type: BlockType,
    pub bbox: BoundingBox,
    pub priority: i32,
    pub content: BlockContent,
    pub caption: Option<Box<BlockNode>>,
    pub children: Vec<BlockNode>,
}

impl BlockNode {
    pub fn new(id: String, block_type: BlockType, bbox: BoundingBox, content: BlockContent) -> Self {
        let priority = block_type.default_priority();
        Self {
            id,
            block_type,
            bbox,
            priority,
            content,
            caption: None,
            children: Vec::new(),
        }
    }

    pub fn render_markdown(&self) -> String {
        let mut buf = String::new();

        match &self.content {
            BlockContent::Text(inlines) => {
                let text_spans: Vec<String> = inlines
                    .iter()
                    .map(|inline| match inline {
                        InlineNode::TextSpan { text, .. } => text.clone(),
                        InlineNode::InlineFormula { latex } => {
                            let clean = crate::idp::stitch::normalize_latex_formula(latex);
                            if clean.starts_with('$') {
                                clean
                            } else {
                                format!("${}$", clean)
                            }
                        }
                    })
                    .collect();
                let full_text = text_spans.join("");

                match self.block_type {
                    BlockType::DocTitle => buf.push_str(&format!("# {}\n", full_text.trim())),
                    BlockType::SectionHeader => buf.push_str(&format!("## {}\n", full_text.trim())),
                    _ => buf.push_str(&format!("{}\n", full_text)),
                }
            }
            BlockContent::Table {
                structure_tokens,
                cells,
                ..
            } => {
                if !structure_tokens.is_empty() && !cells.is_empty() {
                    let mut cell_idx = 0usize;
                    let mut table_buf = Vec::new();
                    table_buf.push("<table>\n<tbody>\n".to_string());
                    let mut inside_tr = false;

                    for token in structure_tokens {
                        if matches!(
                            token.as_str(),
                            "<html>"
                                | "<body>"
                                | "<table>"
                                | "</table>"
                                | "</body>"
                                | "</html>"
                                | "<tbody>"
                                | "</tbody>"
                        ) {
                            continue;
                        }
                        if token == "<tr>" {
                            if inside_tr {
                                table_buf.push("  </tr>\n".to_string());
                            }
                            table_buf.push("  <tr>\n".to_string());
                            inside_tr = true;
                            continue;
                        }
                        if token == "</tr>" {
                            if inside_tr {
                                table_buf.push("  </tr>\n".to_string());
                                inside_tr = false;
                            }
                            continue;
                        }
                        if token == "<td></td>" || token.starts_with("<td") {
                            // 🛡️ 手术 1：彻底清理单元格内部软折行 \n，消除 D\nt Titl 碎切病灶
                            let raw_text = cells
                                .get(cell_idx)
                                .map(|c| c.text.as_str())
                                .unwrap_or("")
                                .trim();
                            let clean_cell_text = raw_text.replace('\n', " ").replace("  ", " ");

                            table_buf.push(format!("    <td>{}</td>\n", clean_cell_text.trim()));
                            cell_idx += 1;
                            continue;
                        }
                    }
                    if inside_tr {
                        table_buf.push("  </tr>\n".to_string());
                    }
                    table_buf.push("</tbody>\n</table>".to_string());
                    buf.push_str(&format!("\n{}\n", table_buf.join("")));
                }
            }
            BlockContent::Figure {
                image_path,
                caption_text,
            } => {
                buf.push_str(&format!("\n![图片]({})\n", image_path));
                if let Some(cap) = caption_text {
                    buf.push_str(&format!("*{}*\n", cap.trim()));
                }
            }
            BlockContent::Formula { latex, is_display } => {
                let clean = crate::idp::stitch::normalize_latex_formula(latex);
                let trimmed = clean.trim_matches('$').trim();
                if *is_display {
                    buf.push_str(&format!("\n$$\n{}\n$$\n", trimmed));
                } else if clean.starts_with('$') {
                    buf.push_str(&format!("\n{}\n", clean));
                } else {
                    buf.push_str(&format!("\n$$\n{}\n$$\n", clean));
                }
            }
        }

        if let Some(cap_block) = &self.caption {
            buf.push_str(&format!("\n{}\n", cap_block.render_markdown().trim()));
        }

        buf
    }
}
