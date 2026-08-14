use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub fn decode_xml_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            while let Some(&next_c) = chars.peek() {
                if next_c == ';' {
                    chars.next();
                    break;
                }
                entity.push(chars.next().unwrap());
                if entity.len() > 10 {
                    break;
                }
            }
            match entity.as_str() {
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                "amp" => result.push('&'),
                hex if hex.starts_with("#x") || hex.starts_with("#X") => {
                    if let Ok(cp) = u32::from_str_radix(&hex[2..], 16) {
                        if let Some(ch) = char::from_u32(cp) {
                            result.push(ch);
                        } else {
                            result.push_str(&format!("&{};", hex));
                        }
                    } else {
                        result.push_str(&format!("&{};", hex));
                    }
                }
                dec if dec.starts_with('#') => {
                    if let Ok(cp) = dec[1..].parse::<u32>() {
                        if let Some(ch) = char::from_u32(cp) {
                            result.push(ch);
                        } else {
                            result.push_str(&format!("&{};", dec));
                        }
                    } else {
                        result.push_str(&format!("&{};", dec));
                    }
                }
                _ => {
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[derive(Debug)]
struct XmlElement {
    name: String,
    attributes: BTreeMap<String, String>,
    children: Vec<XmlElement>,
    text: String,
}

struct XmlParser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos..].chars().next().unwrap().is_whitespace() {
            self.pos += self.src[self.pos..].chars().next().unwrap().len_utf8();
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        if self.pos < self.src.len() && self.src[self.pos..].starts_with(ch) {
            self.pos += ch.len_utf8();
            Ok(())
        } else {
            Err(format!("XML 解析期望 '{}' 于偏移量 {}", ch, self.pos))
        }
    }

    fn read_name(&mut self) -> Result<String, String> {
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.src[self.pos..].chars().next().unwrap();
            if c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-') {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(format!("XML 期望标签名于偏移量 {}", self.pos));
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn read_attr_val(&mut self) -> Result<String, String> {
        if self.pos >= self.src.len() {
            return Err("XML 属性值未开始".into());
        }
        let quote = self.src[self.pos..].chars().next().unwrap();
        if quote != '"' && quote != '\'' {
            return Err(format!("XML 属性值期望引号于偏移量 {}", self.pos));
        }
        self.pos += quote.len_utf8();
        let start = self.pos;
        while self.pos < self.src.len() && !self.src[self.pos..].starts_with(quote) {
            let c = self.src[self.pos..].chars().next().unwrap();
            self.pos += c.len_utf8();
        }
        if self.pos >= self.src.len() {
            return Err("XML 属性值未闭合".into());
        }
        let val = decode_xml_entities(&self.src[start..self.pos]);
        self.pos += quote.len_utf8();
        Ok(val)
    }

    fn parse_element(&mut self) -> Result<XmlElement, String> {
        self.expect('<')?;
        let name = self.read_name()?;
        let mut attributes = BTreeMap::new();

        loop {
            self.skip_whitespace();
            if self.src[self.pos..].starts_with('>') {
                self.pos += 1;
                break;
            }
            if self.src[self.pos..].starts_with("/>") {
                self.pos += 2;
                return Ok(XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            let attr_name = self.read_name()?;
            self.skip_whitespace();
            self.expect('=')?;
            self.skip_whitespace();
            let attr_val = self.read_attr_val()?;
            attributes.insert(attr_name, attr_val);
        }

        let mut children = Vec::new();
        let mut text = String::new();

        loop {
            if self.pos >= self.src.len() {
                return Err(format!("标签 <{}> 未闭合", name));
            }
            if self.src[self.pos..].starts_with('<') {
                if self.src[self.pos..].starts_with("</") {
                    self.pos += 2;
                    let close_name = self.read_name()?;
                    self.skip_whitespace();
                    self.expect('>')?;
                    if close_name != name {
                        return Err(format!("XML 标签不匹配: 期望 </{}>, 实际 </{}>", name, close_name));
                    }
                    break;
                }
                if self.src[self.pos..].starts_with("<![CDATA[") {
                    self.pos += 9;
                    if let Some(end_idx) = self.src[self.pos..].find("]]>") {
                        text.push_str(&self.src[self.pos..self.pos + end_idx]);
                        self.pos += end_idx + 3;
                        continue;
                    } else {
                        return Err("CDATA 块未闭合".into());
                    }
                }
                children.push(self.parse_element()?);
            } else {
                let start = self.pos;
                while self.pos < self.src.len() && !self.src[self.pos..].starts_with('<') {
                    let c = self.src[self.pos..].chars().next().unwrap();
                    self.pos += c.len_utf8();
                }
                text.push_str(&decode_xml_entities(&self.src[start..self.pos]));
            }
        }

        Ok(XmlElement {
            name,
            attributes,
            children,
            text,
        })
    }
}

fn convert_node(node: &XmlElement) -> Value {
    let has_children = !node.children.is_empty();
    let has_attributes = !node.attributes.is_empty();
    let text = node.text.trim();
    let has_text = !text.is_empty();

    if !has_children && !has_attributes {
        return if has_text {
            Value::String(text.to_string())
        } else {
            Value::String(String::new())
        };
    }

    let mut map = Map::new();

    for (k, v) in &node.attributes {
        map.insert(format!("@{}", k), Value::String(v.clone()));
    }

    if has_children {
        for child in &node.children {
            let child_val = convert_node(child);
            if let Some(existing) = map.get_mut(&child.name) {
                if let Value::Array(arr) = existing {
                    arr.push(child_val);
                } else {
                    let prev = existing.clone();
                    *existing = Value::Array(vec![prev, child_val]);
                }
            } else {
                map.insert(child.name.clone(), child_val);
            }
        }
    }

    if has_text {
        map.insert("#text".to_string(), Value::String(text.to_string()));
    }

    Value::Object(map)
}

pub fn xml_to_json_value(xml: &str) -> Result<Value, String> {
    let mut cleaned = xml.trim_start_matches('\u{feff}').trim().to_string();
    if cleaned.is_empty() {
        return Err("XML 内容为空".into());
    }

    // 剔除 <?xml...?>
    if let Some(start) = cleaned.find("<?xml") {
        if let Some(end) = cleaned[start..].find("?>") {
            cleaned = format!("{}{}", &cleaned[..start], &cleaned[start + end + 2..]);
        }
    }
    // 剔除 <!DOCTYPE...>
    if let Some(start) = cleaned.find("<!DOCTYPE") {
        if let Some(end) = cleaned[start..].find('>') {
            cleaned = format!("{}{}", &cleaned[..start], &cleaned[start + end + 1..]);
        }
    }

    let mut parser = XmlParser::new(cleaned.trim());
    parser.skip_whitespace();
    let root = parser.parse_element()?;

    let mut root_map = Map::new();
    root_map.insert(root.name.clone(), convert_node(&root));
    Ok(Value::Object(root_map))
}
