use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    DocTitle,
    ParagraphTitle,
    Text,
    Table,
    Formula,
    Image,
    Header,
    Footer,
    List,
    Reference,
    Seal,
    AsideText,
    Unknown,
}

impl LayoutType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LayoutType::DocTitle => "title",
            LayoutType::ParagraphTitle => "paragraph_title",
            LayoutType::Text => "text",
            LayoutType::Table => "table",
            LayoutType::Formula => "formula",
            LayoutType::Image => "image",
            LayoutType::Header => "header",
            LayoutType::Footer => "footer",
            LayoutType::List => "list",
            LayoutType::Reference => "reference",
            LayoutType::Seal => "seal",
            LayoutType::AsideText => "aside_text",
            LayoutType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocElement {
    pub bbox: [f32; 4],
    pub layout_type: LayoutType,
    pub order_index: u32,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub elements: Vec<DocElement>,
    pub page_width: f32,
    pub page_height: f32,
}
