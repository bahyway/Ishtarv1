#![forbid(unsafe_code)]
//! BatchSchema — infers schema from CSV records and writes a `.schema` descriptor (BeeMDM §12.4).
//!
//! For each ZIP batch:
//!   schema_name  = "{batch_name}_{timestamp}"
//!   table_name   = "tb_{batch_name}"
//!   mandatory    = columns always non-empty across all records
//!   optional     = columns that can be empty

use crate::kaki_generator::RawRecord;

/// Inferred schema for one CSV batch.
#[derive(Debug, Clone)]
pub struct BatchSchema {
    /// Base name of the batch (zip filename without extension).
    pub batch_name:      String,
    /// Unix timestamp (seconds) when the batch was staged.
    pub timestamp:       u64,
    /// Full schema identifier: `{batch_name}_{timestamp}`.
    pub schema_name:     String,
    /// EnkiDB table descriptor name: `tb_{batch_name}`.
    pub table_name:      String,
    /// FNV-1a hash of batch_name — used as the Entity KAKI seed.
    pub entity_seed:     u32,
    /// Columns always non-empty in every record.
    pub mandatory_attrs: Vec<String>,
    /// Columns that have at least one empty value.
    pub optional_attrs:  Vec<String>,
    /// Total column count from headers.
    pub total_columns:   usize,
    /// Total record count (data rows, not counting header).
    pub total_records:   usize,
}

impl BatchSchema {
    /// Infer the schema from a slice of `RawRecord`s produced by `kaki_generator`.
    pub fn infer(batch_name: &str, timestamp: u64, records: &[RawRecord]) -> Self {
        let headers: Vec<String> = if records.is_empty() {
            Vec::new()
        } else {
            records[0].headers.clone()
        };

        let total_columns = headers.len();
        let total_records = records.len();

        let mut mandatory = Vec::new();
        let mut optional  = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            let always_present = !records.is_empty() && records.iter().all(|r| {
                r.values.get(i).map(|v| !v.is_empty()).unwrap_or(false)
            });
            if always_present { mandatory.push(header.trim().to_string()); }
            else              { optional.push(header.trim().to_string()); }
        }

        let schema_name = format!("{}_{}", batch_name, timestamp);
        let table_name  = format!("tb_{}", batch_name);
        let entity_seed = fnv1a_32(batch_name.as_bytes());

        BatchSchema {
            batch_name: batch_name.to_string(),
            timestamp,
            schema_name,
            table_name,
            entity_seed,
            mandatory_attrs: mandatory,
            optional_attrs:  optional,
            total_columns,
            total_records,
        }
    }

    /// Render a human-readable `.schema` descriptor string suitable for writing to disk.
    pub fn to_descriptor(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("schema_name     = {}\n", self.schema_name));
        s.push_str(&format!("table_name      = {}\n", self.table_name));
        s.push_str(&format!("timestamp       = {}\n", self.timestamp));
        s.push_str(&format!("total_columns   = {}\n", self.total_columns));
        s.push_str(&format!("total_records   = {}\n", self.total_records));
        s.push_str(&format!("entity_seed     = {:#010x}\n", self.entity_seed));
        s.push_str("\n[mandatory_attributes]\n");
        for a in &self.mandatory_attrs {
            s.push_str(&format!("  {}\n", a));
        }
        s.push_str("\n[optional_attributes]\n");
        for a in &self.optional_attrs {
            s.push_str(&format!("  {}\n", a));
        }
        s
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_records() -> Vec<RawRecord> {
        vec![
            RawRecord {
                headers: vec!["id".into(), "name".into(), "nickname".into()],
                values:  vec![b"1".to_vec(), b"Ali".to_vec(), vec![]],
                seq:     1,
            },
            RawRecord {
                headers: vec!["id".into(), "name".into(), "nickname".into()],
                values:  vec![b"2".to_vec(), b"Fatima".to_vec(), b"Fatma".to_vec()],
                seq:     2,
            },
        ]
    }

    #[test]
    fn mandatory_vs_optional() {
        let records = make_records();
        let schema  = BatchSchema::infer("najaf_batch_001", 1_000_000, &records);
        assert!(schema.mandatory_attrs.contains(&"id".to_string()));
        assert!(schema.mandatory_attrs.contains(&"name".to_string()));
        // "nickname" is empty in row 1 → optional
        assert!(schema.optional_attrs.contains(&"nickname".to_string()));
    }

    #[test]
    fn schema_name_and_table_name() {
        let schema = BatchSchema::infer("najaf_batch_001", 1_700_000, &[]);
        assert_eq!(schema.schema_name, "najaf_batch_001_1700000");
        assert_eq!(schema.table_name,  "tb_najaf_batch_001");
    }

    #[test]
    fn descriptor_contains_key_fields() {
        let records = make_records();
        let schema  = BatchSchema::infer("test_batch", 999, &records);
        let desc    = schema.to_descriptor();
        assert!(desc.contains("schema_name"));
        assert!(desc.contains("tb_test_batch"));
        assert!(desc.contains("[mandatory_attributes]"));
        assert!(desc.contains("[optional_attributes]"));
        assert!(desc.contains("nickname"));
    }

    #[test]
    fn empty_records_produces_empty_schema() {
        let schema = BatchSchema::infer("empty_batch", 0, &[]);
        assert_eq!(schema.total_columns, 0);
        assert_eq!(schema.total_records, 0);
        assert!(schema.mandatory_attrs.is_empty());
        assert!(schema.optional_attrs.is_empty());
    }

    #[test]
    fn entity_seed_deterministic() {
        let s1 = BatchSchema::infer("batch_abc", 0, &[]);
        let s2 = BatchSchema::infer("batch_abc", 99, &[]);
        assert_eq!(s1.entity_seed, s2.entity_seed);
    }
}
