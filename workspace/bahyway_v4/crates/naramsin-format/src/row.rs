#![forbid(unsafe_code)]
//! Row-oriented intermediate representation — NARAMSIN Stage 1 output.
//!
//! All three format parsers (CSV, JSON, XML) produce the same `Vec<Row>` output
//! so that downstream MASHSHARU gate never needs to know the original format.

/// A single parsed row of data from any Stage 1 format.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Zero-based row index within the source file.
    pub index: u32,
    /// Fields in column order (CSV) or key-value order (JSON/XML).
    pub fields: Vec<Field>,
}

/// A single field within a parsed row.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Column/key name, as provided by the source.
    pub name: &'static str,
    /// Raw string value, unparsed — semantic conversion is MASHSHARU's job.
    pub value: String,
    /// True if this field was absent in the source (nullable position).
    pub is_null: bool,
}

impl Field {
    pub fn present(name: &'static str, value: impl Into<String>) -> Self {
        Self { name, value: value.into(), is_null: false }
    }

    pub fn null(name: &'static str) -> Self {
        Self { name, value: String::new(), is_null: true }
    }
}
