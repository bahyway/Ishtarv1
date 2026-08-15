//! Sovereign CSV parser — no external crates.
//!
//! Handles:
//!   - Quoted fields ("value with, comma")
//!   - Escaped quotes inside quoted fields ("")
//!   - Empty fields → AkkValue::Null
//!   - Type inference: bool → Bool, integer → Int, float → Float, else → Text
//!
//! Does NOT handle multi-line quoted fields (MVP scope).

use akkvalue::AkkValue;

/// One parsed CSV row: column index → (attribute_name, inferred AkkValue).
#[derive(Debug, Clone)]
pub struct CsvRow {
    pub fields: Vec<(String, AkkValue)>,
}

/// Parse a CSV string into a list of rows with inferred AkkValue types.
///
/// The first line MUST be the header row (column names = attribute names).
/// Returns an error if the header row is missing or malformed.
pub fn parse_csv(input: &str) -> Result<Vec<CsvRow>, CsvError> {
    let mut lines = input.lines();

    // First line: headers
    let header_line = lines.next().ok_or(CsvError::EmptyInput)?;
    let headers = parse_line(header_line);
    if headers.is_empty() {
        return Err(CsvError::EmptyHeader);
    }
    let col_count = headers.len();

    // Remaining lines: data rows
    let mut rows = Vec::new();
    for (line_idx, line) in lines.enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // skip blank lines
        }
        let raw_fields = parse_line(trimmed);
        if raw_fields.len() != col_count {
            return Err(CsvError::ColumnCountMismatch {
                line:     line_idx + 2,
                expected: col_count,
                got:      raw_fields.len(),
            });
        }
        let fields = headers
            .iter()
            .zip(raw_fields.iter())
            .map(|(header, raw)| (header.clone(), infer_value(raw)))
            .collect();
        rows.push(CsvRow { fields });
    }
    Ok(rows)
}

/// Parse one CSV line into raw string fields (handles quoting).
fn parse_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur    = String::new();
    let mut chars  = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                // Escaped quote: "" inside a quoted field
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(cur.trim().to_string());
                cur = String::new();
            }
            ch => cur.push(ch),
        }
    }
    fields.push(cur.trim().to_string());
    fields
}

/// Infer the AkkValue type from a raw string field.
///
/// Rules (in order):
///  1. Empty → Null
///  2. "true" / "false" (case-insensitive) → Bool
///  3. Parseable as i64 → Int
///  4. Parseable as f64 → Float
///  5. Otherwise → Text
pub fn infer_value(raw: &str) -> AkkValue {
    let s = raw.trim();
    if s.is_empty() {
        return AkkValue::Null;
    }
    match s.to_lowercase().as_str() {
        "true"  => return AkkValue::Bool(true),
        "false" => return AkkValue::Bool(false),
        _ => {}
    }
    if let Ok(i) = s.parse::<i64>() {
        return AkkValue::Int(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return AkkValue::Float(f);
    }
    AkkValue::Text(s.to_string())
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CsvError {
    EmptyInput,
    EmptyHeader,
    ColumnCountMismatch { line: usize, expected: usize, got: usize },
}

impl core::fmt::Display for CsvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "CSV: input is empty"),
            Self::EmptyHeader => write!(f, "CSV: header row is empty"),
            Self::ColumnCountMismatch { line, expected, got } =>
                write!(f, "CSV: line {line}: expected {expected} columns, got {got}"),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_csv() {
        let csv = "name,age,score\nAlice,30,9.5\nBob,25,7.0";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fields[0], ("name".into(), AkkValue::Text("Alice".into())));
        assert_eq!(rows[0].fields[1], ("age".into(),  AkkValue::Int(30)));
        assert_eq!(rows[0].fields[2], ("score".into(), AkkValue::Float(9.5)));
        assert_eq!(rows[1].fields[0], ("name".into(), AkkValue::Text("Bob".into())));
    }

    #[test]
    fn null_empty_field() {
        let csv = "a,b\n1,";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows[0].fields[1], ("b".into(), AkkValue::Null));
    }

    #[test]
    fn bool_inference() {
        let csv = "flag\ntrue\nfalse\nTRUE\nFALSE";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows[0].fields[0].1, AkkValue::Bool(true));
        assert_eq!(rows[1].fields[0].1, AkkValue::Bool(false));
        assert_eq!(rows[2].fields[0].1, AkkValue::Bool(true));
        assert_eq!(rows[3].fields[0].1, AkkValue::Bool(false));
    }

    #[test]
    fn quoted_field_with_comma() {
        let csv = "name,address\nBahaa,\"Najaf, Iraq\"";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows[0].fields[1].1, AkkValue::Text("Najaf, Iraq".into()));
    }

    #[test]
    fn escaped_quote_in_quoted_field() {
        let csv = r#"name,note
Alice,"She said ""hello"""
"#;
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows[0].fields[1].1, AkkValue::Text("She said \"hello\"".into()));
    }

    #[test]
    fn negative_int() {
        let v = infer_value("-42");
        assert_eq!(v, AkkValue::Int(-42));
    }

    #[test]
    fn blank_lines_skipped() {
        let csv = "x\n1\n\n2\n";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn empty_input_error() {
        assert_eq!(parse_csv("").unwrap_err(), CsvError::EmptyInput);
    }

    #[test]
    fn column_mismatch_error() {
        let csv = "a,b\n1";
        assert!(matches!(
            parse_csv(csv),
            Err(CsvError::ColumnCountMismatch { line: 2, expected: 2, got: 1 })
        ));
    }

    #[test]
    fn coordinate_text_stays_text() {
        // coordinates aren't auto-detected; they need explicit construction
        let v = infer_value("32.0031,44.3282");
        // should be Text since it has a comma — actually the CSV parser splits on comma
        // so this would arrive as "32.0031" → Float
        let v2 = infer_value("32.0031");
        assert_eq!(v2, AkkValue::Float(32.0031));
        // verify the plain text case
        assert_eq!(v, AkkValue::Text("32.0031,44.3282".into()));
    }

    #[test]
    fn arabic_text_round_trip() {
        let csv = "name\nمحمد";
        let rows = parse_csv(csv).unwrap();
        assert_eq!(rows[0].fields[0].1, AkkValue::Text("محمد".into()));
    }
}
