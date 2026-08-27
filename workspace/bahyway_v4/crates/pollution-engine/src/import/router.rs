//! # import::router — Format Detection & Payload Parsing
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Sovereign replacement: `serde_json` → hand-rolled recursive-descent parser;
//! `csv` crate → hand-rolled DSV parser. Zero external dependencies.

use super::ImportError;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────
//  PAYLOAD FORMAT
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadFormat {
    JsonArray,
    NdJson,
    Csv,
    Tsv,
}

impl std::fmt::Display for PayloadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonArray => write!(f, "JSON-Array"),
            Self::NdJson => write!(f, "NDJSON"),
            Self::Csv => write!(f, "CSV"),
            Self::Tsv => write!(f, "TSV"),
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  SOURCE HINT
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceHint {
    OpenAq,
    EeaWaterbase,
    NoaaIncident,
    NoaaAdios,
    OpenMeteo,
    Unknown(String),
}

impl SourceHint {
    pub fn from_url(url: &str) -> Self {
        let u = url.to_lowercase();
        if u.contains("openaq.org") || u.contains("openaq") {
            Self::OpenAq
        } else if u.contains("eea.europa") || u.contains("waterbase") || u.contains("wise") {
            Self::EeaWaterbase
        } else if u.contains("incidentnews") || (u.contains("noaa") && u.contains("incident")) {
            Self::NoaaIncident
        } else if u.contains("adios") || (u.contains("noaa") && u.contains("oil")) {
            Self::NoaaAdios
        } else if u.contains("open-meteo") || u.contains("openmeteo") {
            Self::OpenMeteo
        } else {
            Self::Unknown(url.to_string())
        }
    }

    pub fn suggested_domain_hint(&self) -> Option<&'static str> {
        match self {
            Self::OpenAq => Some("AIR"),
            Self::EeaWaterbase => Some("WATER"),
            Self::NoaaIncident => Some("OIL"),
            Self::NoaaAdios => Some("OIL"),
            Self::OpenMeteo => Some("AIR"),
            Self::Unknown(_) => None,
        }
    }

    pub fn adapter_name(&self) -> &'static str {
        match self {
            Self::OpenAq => "openaq",
            Self::EeaWaterbase => "eea_waterbase",
            Self::NoaaIncident => "noaa_incident",
            Self::NoaaAdios => "noaa_adios",
            Self::OpenMeteo => "open_meteo",
            Self::Unknown(_) => "generic",
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  IMPORT PAYLOAD
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ImportPayload {
    pub format: PayloadFormat,
    pub source: SourceHint,
    pub field_names: Vec<String>,
    pub rows: Vec<HashMap<String, String>>,
    pub source_url: String,
}

impl ImportPayload {
    pub fn has_field(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.field_names.iter().any(|f| f == &lower)
    }

    pub fn sample_values(&self, field: &str, n: usize) -> Vec<String> {
        let lower = field.to_lowercase();
        self.rows
            .iter()
            .filter_map(|row| row.get(&lower).cloned())
            .filter(|v| !v.is_empty())
            .take(n)
            .collect()
    }

    pub fn parse_f64(row: &HashMap<String, String>, field: &str) -> Option<f64> {
        row.get(&field.to_lowercase())
            .and_then(|v| v.trim().parse::<f64>().ok())
    }

    pub fn parse_bool(row: &HashMap<String, String>, field: &str) -> Option<bool> {
        row.get(&field.to_lowercase())
            .map(|v| matches!(v.to_lowercase().trim(), "true" | "1" | "yes" | "t"))
    }

    pub fn get_str<'a>(row: &'a HashMap<String, String>, field: &str) -> &'a str {
        row.get(&field.to_lowercase())
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

// ─────────────────────────────────────────────────────────────
//  IMPORT ROUTER
// ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct ImportRouter;

impl ImportRouter {
    pub fn parse(&self, raw: &str, source_url: &str) -> Result<ImportPayload, ImportError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ImportError::EmptyPayload);
        }

        let source = SourceHint::from_url(source_url);
        let format = self.detect_format(trimmed);

        let rows = match &format {
            PayloadFormat::JsonArray => self.parse_json_array(trimmed)?,
            PayloadFormat::NdJson => self.parse_ndjson(trimmed)?,
            PayloadFormat::Csv => self.parse_dsv(trimmed, b',')?,
            PayloadFormat::Tsv => self.parse_dsv(trimmed, b'\t')?,
        };

        if rows.is_empty() {
            return Err(ImportError::EmptyPayload);
        }

        let mut field_set = std::collections::HashSet::new();
        for row in &rows {
            for key in row.keys() {
                field_set.insert(key.clone());
            }
        }
        let mut field_names: Vec<String> = field_set.into_iter().collect();
        field_names.sort();

        Ok(ImportPayload {
            format,
            source,
            field_names,
            rows,
            source_url: source_url.to_string(),
        })
    }

    fn detect_format(&self, raw: &str) -> PayloadFormat {
        match raw.chars().next().unwrap_or(' ') {
            '[' => PayloadFormat::JsonArray,
            '{' => PayloadFormat::NdJson,
            _ => {
                let first_line = raw.lines().next().unwrap_or("");
                let tabs = first_line.chars().filter(|&c| c == '\t').count();
                let commas = first_line.chars().filter(|&c| c == ',').count();
                if tabs > commas {
                    PayloadFormat::Tsv
                } else {
                    PayloadFormat::Csv
                }
            }
        }
    }

    // ── JSON parsers ─────────────────────────────────────────

    fn parse_json_array(&self, raw: &str) -> Result<Vec<HashMap<String, String>>, ImportError> {
        let value = parse_json(raw)?;

        let array = match value {
            JsonValue::Array(arr) => arr,
            JsonValue::Object(ref obj) => {
                for key in &["results", "data", "measurements", "locations"] {
                    if let Some(JsonValue::Array(arr)) =
                        obj.iter().find(|(k, _)| k == key).map(|(_, v)| v)
                    {
                        return Ok(arr.iter().map(|v| flatten_json(v, "")).collect());
                    }
                }
                vec![value]
            }
            other => vec![other],
        };

        Ok(array.iter().map(|v| flatten_json(v, "")).collect())
    }

    fn parse_ndjson(&self, raw: &str) -> Result<Vec<HashMap<String, String>>, ImportError> {
        raw.lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, line)| {
                let v = parse_json(line).map_err(|e| ImportError::CsvParseError {
                    row: i,
                    reason: e.to_string(),
                })?;
                Ok(flatten_json(&v, ""))
            })
            .collect()
    }

    // ── DSV (CSV / TSV) parser ────────────────────────────────

    fn parse_dsv(
        &self,
        raw: &str,
        delimiter: u8,
    ) -> Result<Vec<HashMap<String, String>>, ImportError> {
        let delim = delimiter as char;
        let mut lines = raw.lines().filter(|l| !l.trim().is_empty());

        let header_line = lines.next().ok_or(ImportError::EmptyPayload)?;
        let headers: Vec<String> = split_dsv_line(header_line, delim)
            .into_iter()
            .map(|h| h.to_lowercase().trim().to_string())
            .collect();

        let mut rows = Vec::new();
        for line in lines {
            let fields = split_dsv_line(line, delim);
            let mut map = HashMap::new();
            for (j, value) in fields.into_iter().enumerate() {
                if let Some(header) = headers.get(j) {
                    if !header.is_empty() {
                        map.insert(header.clone(), value.trim().to_string());
                    }
                }
            }
            rows.push(map);
        }

        Ok(rows)
    }
}

// ─────────────────────────────────────────────────────────────
//  SOVEREIGN JSON PARSER  (replaces serde_json)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

fn parse_json(input: &str) -> Result<JsonValue, ImportError> {
    let bytes = input.as_bytes();
    let (val, _) = parse_value(bytes, 0).map_err(ImportError::JsonParseError)?;
    Ok(val)
}

fn skip_ws(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn parse_value(bytes: &[u8], pos: usize) -> Result<(JsonValue, usize), String> {
    let pos = skip_ws(bytes, pos);
    match bytes.get(pos) {
        Some(b'"') => {
            let (s, p) = parse_string(bytes, pos)?;
            Ok((JsonValue::Str(s), p))
        }
        Some(b'[') => parse_array(bytes, pos),
        Some(b'{') => parse_object(bytes, pos),
        Some(b't') => {
            if bytes.get(pos..pos + 4) == Some(b"true") {
                Ok((JsonValue::Bool(true), pos + 4))
            } else {
                Err(format!("expected 'true' at pos {pos}"))
            }
        }
        Some(b'f') => {
            if bytes.get(pos..pos + 5) == Some(b"false") {
                Ok((JsonValue::Bool(false), pos + 5))
            } else {
                Err(format!("expected 'false' at pos {pos}"))
            }
        }
        Some(b'n') => {
            if bytes.get(pos..pos + 4) == Some(b"null") {
                Ok((JsonValue::Null, pos + 4))
            } else {
                Err(format!("expected 'null' at pos {pos}"))
            }
        }
        Some(b'-') | Some(b'0'..=b'9') => parse_number(bytes, pos),
        Some(c) => Err(format!("unexpected char '{}' at pos {pos}", *c as char)),
        None => Err("unexpected end of input".into()),
    }
}

fn parse_string(bytes: &[u8], pos: usize) -> Result<(String, usize), String> {
    debug_assert_eq!(bytes[pos], b'"');
    let mut i = pos + 1;
    let mut s = String::new();

    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                return Ok((s, i));
            }
            b'\\' => {
                i += 1;
                match bytes.get(i) {
                    Some(b'"') => {
                        s.push('"');
                        i += 1;
                    }
                    Some(b'\\') => {
                        s.push('\\');
                        i += 1;
                    }
                    Some(b'/') => {
                        s.push('/');
                        i += 1;
                    }
                    Some(b'n') => {
                        s.push('\n');
                        i += 1;
                    }
                    Some(b'r') => {
                        s.push('\r');
                        i += 1;
                    }
                    Some(b't') => {
                        s.push('\t');
                        i += 1;
                    }
                    Some(b'b') => {
                        s.push('\x08');
                        i += 1;
                    }
                    Some(b'f') => {
                        s.push('\x0C');
                        i += 1;
                    }
                    Some(b'u') => {
                        i += 1;
                        if i + 4 > bytes.len() {
                            return Err("truncated unicode escape".into());
                        }
                        let hex = std::str::from_utf8(&bytes[i..i + 4])
                            .map_err(|_| "invalid unicode escape")?;
                        let code = u32::from_str_radix(hex, 16)
                            .map_err(|_| format!("invalid hex in \\u{}", hex))?;
                        s.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        i += 4;
                    }
                    Some(c) => {
                        s.push(*c as char);
                        i += 1;
                    }
                    None => return Err("truncated escape".into()),
                }
            }
            b if b < 0x80 => {
                s.push(b as char);
                i += 1;
            }
            _ => {
                // Multi-byte UTF-8
                let b0 = bytes[i];
                let extra = if b0 >= 0xF0 {
                    3
                } else if b0 >= 0xE0 {
                    2
                } else {
                    1
                };
                if i + extra >= bytes.len() {
                    return Err("truncated UTF-8".into());
                }
                let chunk = &bytes[i..=i + extra];
                let ch = std::str::from_utf8(chunk).map_err(|_| "invalid UTF-8 in string")?;
                s.push_str(ch);
                i += extra + 1;
            }
        }
    }
    Err("unterminated string".into())
}

fn parse_array(bytes: &[u8], pos: usize) -> Result<(JsonValue, usize), String> {
    debug_assert_eq!(bytes[pos], b'[');
    let mut i = skip_ws(bytes, pos + 1);
    let mut arr = Vec::new();

    if bytes.get(i) == Some(&b']') {
        return Ok((JsonValue::Array(arr), i + 1));
    }

    loop {
        let (val, next) = parse_value(bytes, i)?;
        arr.push(val);
        i = skip_ws(bytes, next);
        match bytes.get(i) {
            Some(b']') => {
                i += 1;
                break;
            }
            Some(b',') => {
                i = skip_ws(bytes, i + 1);
            }
            Some(c) => return Err(format!("expected ',' or ']', got '{}' at {i}", *c as char)),
            None => return Err("unterminated array".into()),
        }
    }
    Ok((JsonValue::Array(arr), i))
}

fn parse_object(bytes: &[u8], pos: usize) -> Result<(JsonValue, usize), String> {
    debug_assert_eq!(bytes[pos], b'{');
    let mut i = skip_ws(bytes, pos + 1);
    let mut obj: Vec<(String, JsonValue)> = Vec::new();

    if bytes.get(i) == Some(&b'}') {
        return Ok((JsonValue::Object(obj), i + 1));
    }

    loop {
        i = skip_ws(bytes, i);
        if bytes.get(i) != Some(&b'"') {
            return Err(format!("expected '\"' for key at {i}"));
        }
        let (key, next) = parse_string(bytes, i)?;
        i = skip_ws(bytes, next);
        if bytes.get(i) != Some(&b':') {
            return Err(format!("expected ':' at {i}"));
        }
        i = skip_ws(bytes, i + 1);
        let (val, next) = parse_value(bytes, i)?;
        obj.push((key, val));
        i = skip_ws(bytes, next);
        match bytes.get(i) {
            Some(b'}') => {
                i += 1;
                break;
            }
            Some(b',') => {
                i = skip_ws(bytes, i + 1);
            }
            Some(c) => return Err(format!("expected ',' or '}}', got '{}' at {i}", *c as char)),
            None => return Err("unterminated object".into()),
        }
    }
    Ok((JsonValue::Object(obj), i))
}

fn parse_number(bytes: &[u8], pos: usize) -> Result<(JsonValue, usize), String> {
    let start = pos;
    let mut i = pos;
    if bytes.get(i) == Some(&b'-') {
        i += 1;
    }
    while matches!(bytes.get(i), Some(b'0'..=b'9')) {
        i += 1;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while matches!(bytes.get(i), Some(b'0'..=b'9')) {
            i += 1;
        }
    }
    if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        while matches!(bytes.get(i), Some(b'0'..=b'9')) {
            i += 1;
        }
    }
    let num_str = std::str::from_utf8(&bytes[start..i]).map_err(|_| "invalid number bytes")?;
    let n = num_str
        .parse::<f64>()
        .map_err(|e| format!("invalid number '{}': {}", num_str, e))?;
    Ok((JsonValue::Number(n), i))
}

// ─────────────────────────────────────────────────────────────
//  JSON FLATTENER  (nested → flat HashMap<String,String>)
// ─────────────────────────────────────────────────────────────

fn flatten_json(value: &JsonValue, prefix: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    match value {
        JsonValue::Object(obj) => {
            for (k, v) in obj {
                let key = if prefix.is_empty() {
                    k.to_lowercase()
                } else {
                    format!("{}_{}", prefix, k.to_lowercase())
                };
                map.extend(flatten_json(v, &key));
            }
        }
        JsonValue::Array(arr) => {
            let joined: Vec<String> = arr.iter().map(json_to_str).collect();
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), joined.join(","));
            }
        }
        JsonValue::Null => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), String::new());
            }
        }
        other => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), json_to_str(other));
            }
        }
    }
    map
}

fn json_to_str(v: &JsonValue) -> String {
    match v {
        JsonValue::Null => String::new(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        JsonValue::Str(s) => s.clone(),
        JsonValue::Array(a) => a.iter().map(json_to_str).collect::<Vec<_>>().join(","),
        JsonValue::Object(_) => "[object]".to_string(),
    }
}

// ─────────────────────────────────────────────────────────────
//  SOVEREIGN CSV / TSV PARSER  (replaces the csv crate)
// ─────────────────────────────────────────────────────────────

fn split_dsv_line(line: &str, delim: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' {
            if in_quotes {
                if chars.peek() == Some(&'"') {
                    // Escaped quote ""
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                in_quotes = true;
            }
        } else if c == delim && !in_quotes {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}
