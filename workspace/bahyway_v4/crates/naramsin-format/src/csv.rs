#![forbid(unsafe_code)]
//! Minimal sovereign CSV parser — no external dependencies.
//!
//! Parses RFC 4180-style CSV (comma delimiter, optional quoted fields) into
//! `Vec<Row>`. Schema-level errors (ragged rows) produce `FormatError::RaggedCsvRow`.
//! Semantic validation (field types, allowed values) is MASHSHARU's responsibility.

use crate::error::FormatError;
use crate::row::{Field, Row};

/// Parse a CSV byte slice into a row-oriented intermediate representation.
///
/// The first row is treated as the header. All subsequent rows must have
/// the same field count; a mismatch yields `FormatError::RaggedCsvRow`.
pub fn parse_csv(src: &str) -> Result<Vec<Row>, FormatError> {
    let mut lines = src.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(vec![]),
    };
    let headers: Vec<&str> = split_csv_line(header_line);
    let expected = headers.len() as u32;

    let mut rows = Vec::new();
    for (idx, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let values = split_csv_line(line);
        if values.len() as u32 != expected {
            return Err(FormatError::RaggedCsvRow {
                row: (idx + 2) as u32,
                expected,
                got: values.len() as u32,
            });
        }
        // SAFETY NOTE: &'static str for field names is deferred to Phase 2 schema binding.
        // For the skeleton, we use a leak-free approach: field names come from the header
        // slice. In production, MASHSHARU binds them against the tribe ontology.
        let fields: Vec<Field> = headers
            .iter()
            .zip(values.iter())
            .map(|(_, v)| {
                // Placeholder: name resolution happens at MASHSHARU gate
                Field {
                    name: "col",
                    value: v.to_string(),
                    is_null: v.is_empty(),
                }
            })
            .collect();
        rows.push(Row {
            index: (idx + 1) as u32,
            fields,
        });
    }
    Ok(rows)
}

/// Split a single CSV line on commas, respecting double-quoted fields.
fn split_csv_line(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                fields.push(line[start..i].trim_matches('"'));
                start = i + 1;
            }
            _ => {}
        }
    }
    fields.push(line[start..].trim_matches('"'));
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_csv() {
        let src = "a,b,c\n1,2,3\n4,5,6";
        let rows = parse_csv(src).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fields.len(), 3);
    }

    #[test]
    fn ragged_row_errors() {
        let src = "a,b,c\n1,2";
        assert!(parse_csv(src).is_err());
    }
}
