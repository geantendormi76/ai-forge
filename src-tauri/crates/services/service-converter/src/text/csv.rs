use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashSet};

pub fn parse_csv_records(raw: &str) -> Result<Vec<Vec<String>>, String> {
    let text = raw.trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut current_record = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current_field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(c);
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => {
                    current_record.push(current_field.clone());
                    current_field.clear();
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_record.push(current_field.clone());
                    current_field.clear();
                    if !current_record.iter().all(|f| f.trim().is_empty()) {
                        records.push(current_record.clone());
                    }
                    current_record.clear();
                }
                '\n' => {
                    current_record.push(current_field.clone());
                    current_field.clear();
                    if !current_record.iter().all(|f| f.trim().is_empty()) {
                        records.push(current_record.clone());
                    }
                    current_record.clear();
                }
                _ => current_field.push(c),
            }
        }
    }

    if in_quotes {
        return Err("CSV 引号未正确闭合".into());
    }

    if !current_field.is_empty() || !current_record.is_empty() {
        current_record.push(current_field);
        if !current_record.iter().all(|f| f.trim().is_empty()) {
            records.push(current_record);
        }
    }

    Ok(records)
}

pub fn csv_to_json_objects(csv: &str) -> Result<Vec<Value>, String> {
    let records = parse_csv_records(csv)?;
    if records.is_empty() {
        return Ok(Vec::new());
    }

    let headers = &records[0];
    let mut seen_headers = HashSet::new();
    let forbidden = ["__proto__", "prototype", "constructor"];

    for h in headers {
        let key = h.trim().to_lowercase();
        if forbidden.contains(&key.as_str()) {
            return Err(format!("CSV 包含不安全的表头: {}", h));
        }
        if seen_headers.contains(&key) {
            return Err(format!("CSV 存在重复表头: {}", h));
        }
        seen_headers.insert(key);
    }

    let mut result = Vec::new();
    for row in &records[1..] {
        let mut map = Map::new();
        for (i, header) in headers.iter().enumerate() {
            let field_val = row.get(i).cloned().unwrap_or_default();
            map.insert(header.clone(), Value::String(field_val));
        }
        result.push(Value::Object(map));
    }

    Ok(result)
}

pub fn csv_to_markdown(csv: &str) -> Result<String, String> {
    let records = parse_csv_records(csv)?;
    if records.is_empty() {
        return Ok(String::new());
    }

    let width = records.iter().map(|r| r.len()).max().unwrap_or(0);
    if width == 0 {
        return Ok(String::new());
    }

    let escape_cell = |val: &str| -> String {
        val.replace('\\', "\\\\")
            .replace('|', "\\|")
            .replace("\r\n", "\n")
            .replace('\n', "<br>")
    };

    let mut lines = Vec::new();

    // 表头
    let header_cells: Vec<String> = (0..width)
        .map(|i| escape_cell(records[0].get(i).map(|s| s.as_str()).unwrap_or("")))
        .collect();
    lines.push(format!("| {} |", header_cells.join(" | ")));

    // 分隔线
    let separators: Vec<&str> = vec!["---"; width];
    lines.push(format!("| {} |", separators.join(" | ")));

    // 数据行
    for row in &records[1..] {
        let cells: Vec<String> = (0..width)
            .map(|i| escape_cell(row.get(i).map(|s| s.as_str()).unwrap_or("")))
            .collect();
        lines.push(format!("| {} |", cells.join(" | ")));
    }

    Ok(lines.join("\n"))
}

pub fn flatten_json_value(
    val: &Value,
    prefix: &str,
    out: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    match val {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json_value(v, &path, out)?;
            }
        }
        Value::Array(arr) => {
            let json_str = serde_json::to_string(arr).map_err(|e| e.to_string())?;
            if out.contains_key(prefix) {
                return Err(format!("JSON 字段路径碰撞冲突: {}", prefix));
            }
            out.insert(prefix.to_string(), json_str);
        }
        Value::Null => {
            out.insert(prefix.to_string(), String::new());
        }
        Value::Bool(b) => {
            out.insert(prefix.to_string(), b.to_string());
        }
        Value::Number(n) => {
            out.insert(prefix.to_string(), n.to_string());
        }
        Value::String(s) => {
            if out.contains_key(prefix) {
                return Err(format!("JSON 字段路径碰撞冲突: {}", prefix));
            }
            out.insert(prefix.to_string(), s.clone());
        }
    }
    Ok(())
}

pub fn json_to_csv(json_str: &str) -> Result<String, String> {
    let parsed: Value = serde_json::from_str(json_str).map_err(|e| format!("无效的 JSON 格式: {}", e))?;
    let items = match parsed {
        Value::Array(arr) => arr,
        other => vec![other],
    };

    let mut flattened_rows = Vec::new();
    let mut all_headers = HashSet::new();

    for item in items {
        let mut map = BTreeMap::new();
        flatten_json_value(&item, "", &mut map)?;
        for k in map.keys() {
            all_headers.insert(k.clone());
        }
        flattened_rows.push(map);
    }

    let mut headers: Vec<String> = all_headers.into_iter().collect();
    headers.sort();

    let quote_cell = |s: &str| -> String {
        format!("\"{}\"", s.replace('"', "\"\""))
    };

    let mut csv_lines = Vec::new();
    let header_line = headers.iter().map(|h| quote_cell(h)).collect::<Vec<_>>().join(",");
    csv_lines.push(header_line);

    for row in flattened_rows {
        let line = headers
            .iter()
            .map(|h| quote_cell(row.get(h).map(|s| s.as_str()).unwrap_or("")))
            .collect::<Vec<_>>()
            .join(",");
        csv_lines.push(line);
    }

    Ok(csv_lines.join("\n"))
}

pub fn normalize_tsv_to_csv(tsv: &str) -> Result<String, String> {
    let text = tsv.trim_start_matches('\u{feff}');
    let mut lines = Vec::new();
    let quote_cell = |s: &str| -> String {
        format!("\"{}\"", s.replace('"', "\"\""))
    };

    for raw_line in text.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }
        let cells: Vec<String> = raw_line.split('\t').map(quote_cell).collect();
        lines.push(cells.join(","));
    }

    Ok(lines.join("\n"))
}
