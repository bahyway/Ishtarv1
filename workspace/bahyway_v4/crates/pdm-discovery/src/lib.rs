//! pdm-discovery — the v1 relationship-based complex-builder that DubSar
//! PDM's schema-discovery paradigm needs and previously did not have.
//!
//! Context (sealed in this session's TDA+PDM evaluation): `LamassuEngine`
//! builds its simplicial complexes from KAKI-derived orbital position —
//! geometric proximity, not business meaning. Reading a resulting H2 void
//! as "missing data / a structural hole" only makes business sense if the
//! complex's edges encode actual relationships (foreign keys, shared
//! keys) between real entities. Nothing before this crate built that
//! kind of complex. This crate does, deliberately scoped as a v1:
//!
//!   1. Profile each column of each uploaded table (`profile_columns`).
//!   2. Detect candidate join-key relationships via **exact-value-overlap
//!      matching only** (`detect_join_keys`) — see its own doc comment for
//!      the heuristic's real, named limitations.
//!   3. Build one vertex per table and one edge per detected relationship,
//!      then hand that graph to `bahyway_algebra::persistence::
//!      clique_complex_persistence` — the SAME dimension-generic
//!      boundary-matrix reduction GeoEngine already uses for geometric
//!      point clouds, just triggered by a discovered relationship graph
//!      instead of Euclidean distance (`discover_schema`).
//!
//! This is explicitly a v1 heuristic, not a universal schema-discovery
//! solution: it catches exact-match shared keys (the common case for
//! well-formed foreign keys) and nothing else. It will miss: renamed or
//! reformatted keys (e.g. `"CUST-042"` vs `"42"`), keys related by a
//! transform (hashes, zero-padding), and semantic relationships with no
//! literal value overlap. It will also produce false positives on any
//! genuinely coincidental value overlap between low-cardinality columns —
//! `MIN_CANDIDATE_CARDINALITY` guards against the worst of that (boolean
//! and small-enum "status" columns), but does not eliminate it. Every
//! `CandidateRelationship` this crate emits is exactly that: a candidate
//! for a human stakeholder to confirm, edit, or reject in DubSar PDM's
//! Logical tab — never an auto-applied schema change.
#![forbid(unsafe_code)]

use std::collections::HashSet;

use bahyway_algebra::persistence::{clique_complex_persistence, PersistenceDiagram};

/// One uploaded table: a name plus its columns, each column already split
/// into per-row string values (same row order across all columns of one
/// table).
#[derive(Debug, Clone)]
pub struct NamedTable {
    pub name: String,
    pub columns: Vec<NamedColumn>,
}

#[derive(Debug, Clone)]
pub struct NamedColumn {
    pub name: String,
    pub values: Vec<String>,
}

/// Parse a minimal CSV blob (comma-separated, one header row, no quoting
/// or escaping) into a [`NamedTable`]. Deliberately minimal: this is the
/// v1 ingestion path for the discovery heuristic, not a general CSV
/// parser — a value containing a literal comma will split incorrectly.
pub fn parse_csv_table(table_name: &str, csv_text: &str) -> NamedTable {
    let mut lines = csv_text.lines().filter(|l| !l.trim().is_empty());
    let header: Vec<String> = match lines.next() {
        Some(h) => h.split(',').map(|s| s.trim().to_string()).collect(),
        None => {
            return NamedTable {
                name: table_name.to_string(),
                columns: Vec::new(),
            }
        }
    };
    let mut columns: Vec<NamedColumn> = header
        .iter()
        .map(|name| NamedColumn {
            name: name.clone(),
            values: Vec::new(),
        })
        .collect();
    for line in lines {
        for (i, field) in line.split(',').enumerate() {
            if let Some(col) = columns.get_mut(i) {
                col.values.push(field.trim().to_string());
            }
        }
    }
    NamedTable {
        name: table_name.to_string(),
        columns,
    }
}

/// One column's profile: what `detect_join_keys` reasons over.
#[derive(Debug, Clone)]
pub struct ColumnProfile {
    pub table: String,
    pub column: String,
    pub row_count: usize,
    pub distinct: HashSet<String>,
}

pub fn profile_columns(tables: &[NamedTable]) -> Vec<ColumnProfile> {
    let mut profiles = Vec::new();
    for table in tables {
        for col in &table.columns {
            let distinct: HashSet<String> = col
                .values
                .iter()
                .filter(|v| !v.is_empty())
                .cloned()
                .collect();
            profiles.push(ColumnProfile {
                table: table.name.clone(),
                column: col.name.clone(),
                row_count: col.values.len(),
                distinct,
            });
        }
    }
    profiles
}

/// Below this many distinct non-empty values, a column is excluded from
/// join-key candidacy entirely — a 2-valued boolean or small-enum
/// "status" column overlapping another table's equally small domain is
/// coincidence, not a relationship, and exact-value-overlap alone cannot
/// tell the two cases apart above this floor either. This is a heuristic
/// guard, not a proof: a genuine low-cardinality key (e.g. a 4-region
/// code) can still be missed.
pub const MIN_CANDIDATE_CARDINALITY: usize = 3;

/// A discovered candidate relationship between two columns in two
/// (normally distinct) tables — a proposal for stakeholder review, never
/// an applied schema change.
#[derive(Debug, Clone)]
pub struct CandidateRelationship {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    /// |intersection| / min(|distinct_a|, |distinct_b|) — 1.0 means the
    /// smaller column's entire value domain is contained in the other's.
    pub overlap_confidence: f64,
}

/// Detect candidate join-key relationships via exact-value-overlap
/// matching across every cross-table column pair. See this module's own
/// doc comment for the heuristic's named limitations.
pub fn detect_join_keys(
    profiles: &[ColumnProfile],
    min_confidence: f64,
) -> Vec<CandidateRelationship> {
    let mut out = Vec::new();
    for i in 0..profiles.len() {
        for j in (i + 1)..profiles.len() {
            let (a, b) = (&profiles[i], &profiles[j]);
            if a.table == b.table {
                continue;
            }
            if a.distinct.len() < MIN_CANDIDATE_CARDINALITY
                || b.distinct.len() < MIN_CANDIDATE_CARDINALITY
            {
                continue;
            }
            let smaller = a.distinct.len().min(b.distinct.len());
            if smaller == 0 {
                continue;
            }
            let overlap = a.distinct.intersection(&b.distinct).count();
            let confidence = overlap as f64 / smaller as f64;
            if confidence >= min_confidence {
                out.push(CandidateRelationship {
                    from_table: a.table.clone(),
                    from_column: a.column.clone(),
                    to_table: b.table.clone(),
                    to_column: b.column.clone(),
                    overlap_confidence: confidence,
                });
            }
        }
    }
    out
}

/// The full discovered-schema proposal: one vertex per table, edges from
/// detected relationships (best confidence between any two tables), and
/// the real persistence diagram GeoEngine computed over that graph. This
/// is the object DubSar PDM's Logical tab is meant to review/edit/approve
/// — nothing here is applied to any real schema automatically.
#[derive(Debug, Clone)]
pub struct SchemaProposal {
    pub tables: Vec<String>,
    pub relationships: Vec<CandidateRelationship>,
    pub diagram: PersistenceDiagram,
}

impl SchemaProposal {
    pub fn component_count(&self) -> usize {
        self.diagram.component_count()
    }

    /// Structural holes at the table-relationship level: a persistent H2
    /// class here reads as "these tables form an enclosed cluster with no
    /// direct anchor relationship tying it together" — a real candidate
    /// for "missing data / a structural hole" the way the evaluated TDA+
    /// PDM document described it, but computed over discovered business
    /// relationships, not KAKI orbital position.
    pub fn void_count(&self) -> usize {
        self.diagram.void_count()
    }

    /// Hand-rolled JSON (matching this workspace's existing
    /// `enkidb-query-server::rows_to_json` convention — no serde
    /// dependency), so this proposal can be handed directly to a client
    /// (e.g. DubSar PDM's Logical tab) over the same kind of wire this
    /// ecosystem already uses elsewhere.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\"tables\":[");
        for (i, t) in self.tables.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(&json_escape(t));
            out.push('"');
        }
        out.push_str("],\"relationships\":[");
        for (i, r) in self.relationships.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"from_table\":\"{}\",\"from_column\":\"{}\",\"to_table\":\"{}\",\"to_column\":\"{}\",\"overlap_confidence\":{}}}",
                json_escape(&r.from_table),
                json_escape(&r.from_column),
                json_escape(&r.to_table),
                json_escape(&r.to_column),
                r.overlap_confidence,
            ));
        }
        out.push_str(&format!(
            "],\"component_count\":{},\"void_count\":{}}}",
            self.component_count(),
            self.void_count()
        ));
        out
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Run the full v1 discovery pipeline: profile columns, detect join-key
/// candidates, build the table-relationship graph, and hand it to
/// GeoEngine. `min_confidence` gates relationship detection (see
/// `detect_join_keys`); `max_weight` gates which relationships enter the
/// filtration at all (pass `1.0` to admit every detected relationship
/// regardless of confidence, since edge weight here is `1.0 -
/// overlap_confidence`).
pub fn discover_schema(
    tables: &[NamedTable],
    min_confidence: f64,
    max_weight: f64,
) -> SchemaProposal {
    let table_names: Vec<String> = tables.iter().map(|t| t.name.clone()).collect();
    let profiles = profile_columns(tables);
    let relationships = detect_join_keys(&profiles, min_confidence);

    let index_of = |name: &str| table_names.iter().position(|n| n == name).unwrap();
    let mut best_edge: std::collections::HashMap<(usize, usize), f64> =
        std::collections::HashMap::new();
    for r in &relationships {
        let a = index_of(&r.from_table);
        let b = index_of(&r.to_table);
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let weight = 1.0 - r.overlap_confidence;
        best_edge
            .entry((lo, hi))
            .and_modify(|w| *w = w.min(weight))
            .or_insert(weight);
    }
    let edges: Vec<(usize, usize, f64)> =
        best_edge.into_iter().map(|((a, b), w)| (a, b, w)).collect();

    let diagram = clique_complex_persistence(table_names.len(), &edges, max_weight);

    SchemaProposal {
        tables: table_names,
        relationships,
        diagram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(name: &str, cols: &[(&str, &[&str])]) -> NamedTable {
        NamedTable {
            name: name.to_string(),
            columns: cols
                .iter()
                .map(|(cname, vals)| NamedColumn {
                    name: cname.to_string(),
                    values: vals.iter().map(|v| v.to_string()).collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn parse_csv_table_splits_header_and_rows() {
        let t = parse_csv_table("orders", "order_id,customer_id\n1,100\n2,101\n3,100\n");
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[0].name, "order_id");
        assert_eq!(t.columns[1].values, vec!["100", "101", "100"]);
    }

    #[test]
    fn detect_join_keys_finds_a_real_foreign_key_relationship() {
        let customers = table(
            "customers",
            &[
                ("customer_id", &["100", "101", "102", "103"]),
                ("name", &["a", "b", "c", "d"]),
            ],
        );
        let orders = table(
            "orders",
            &[
                ("order_id", &["1", "2", "3"]),
                ("customer_id", &["100", "101", "102"]),
            ],
        );
        let profiles = profile_columns(&[customers, orders]);
        let rels = detect_join_keys(&profiles, 0.99);
        let found = rels
            .iter()
            .find(|r| r.from_column == "customer_id" && r.to_column == "customer_id");
        assert!(
            found.is_some(),
            "customer_id should be detected as a shared key: {rels:?}"
        );
        assert_eq!(found.unwrap().overlap_confidence, 1.0);
    }

    #[test]
    fn detect_join_keys_ignores_low_cardinality_status_columns() {
        // Two 2-valued "status" columns that happen to fully overlap:
        // a real, named false-positive mode this heuristic must NOT
        // flag as a relationship, because MIN_CANDIDATE_CARDINALITY guards it.
        let a = table("invoices", &[("status", &["OPEN", "CLOSED", "OPEN"])]);
        let b = table("tickets", &[("status", &["OPEN", "CLOSED", "CLOSED"])]);
        let profiles = profile_columns(&[a, b]);
        let rels = detect_join_keys(&profiles, 0.5);
        assert!(
            rels.is_empty(),
            "low-cardinality status columns must not be flagged: {rels:?}"
        );
    }

    #[test]
    fn detect_join_keys_ignores_columns_within_the_same_table() {
        let t = table("t", &[("a", &["1", "2", "3"]), ("b", &["1", "2", "3"])]);
        let profiles = profile_columns(&[t]);
        let rels = detect_join_keys(&profiles, 0.5);
        assert!(
            rels.is_empty(),
            "same-table column pairs are not cross-table relationships"
        );
    }

    #[test]
    fn discover_schema_chordless_four_table_cycle_is_one_persistent_h1_loop() {
        // Four tables chained by shared keys into a ring, with no direct
        // relationship closing any shortcut across it: accounts-orders,
        // orders-shipments, shipments-returns, returns-accounts. No
        // accounts-shipments or orders-returns key exists. This is the
        // "missing anchor table" structural signature: four tables that
        // only relate to their immediate neighbours, forming a genuine
        // hole rather than a hub-and-spoke schema.
        // Each edge uses its own value domain so no two non-adjacent
        // tables accidentally share a column's values (which would add
        // an unintended chord and defeat the point of the test).
        let accounts = table(
            "accounts",
            &[
                ("account_id", &["A1", "A2", "A3", "A4"]),
                ("acct_id", &["Z1", "Z2", "Z3", "Z4"]),
            ],
        );
        let orders = table(
            "orders",
            &[
                ("account_id", &["A1", "A2", "A3"]),
                ("order_id", &["O1", "O2", "O3", "O4"]),
            ],
        );
        let shipments = table(
            "shipments",
            &[
                ("order_id", &["O1", "O2", "O3"]),
                ("ship_id", &["S1", "S2", "S3", "S4"]),
            ],
        );
        let returns = table(
            "returns",
            &[
                ("ship_id", &["S1", "S2", "S3"]),
                ("acct_id", &["Z1", "Z2", "Z3"]),
            ],
        );
        let proposal = discover_schema(&[accounts, orders, shipments, returns], 0.99, 1.0);
        assert_eq!(proposal.tables.len(), 4);
        assert_eq!(
            proposal.component_count(),
            1,
            "the ring is one connected schema"
        );
        assert!(
            proposal.diagram.h1_count() >= 1,
            "a chordless 4-table relationship ring must surface as a persistent H1 loop: {:?}",
            proposal.diagram
        );
    }

    #[test]
    fn discover_schema_hub_and_spoke_has_no_loop() {
        // Three tables all keyed off a single central table, no ring:
        // no H1 class should appear, since there is no chordless cycle.
        let customers = table("customers", &[("customer_id", &["C1", "C2", "C3", "C4"])]);
        let orders = table("orders", &[("customer_id", &["C1", "C2", "C3"])]);
        let addresses = table("addresses", &[("customer_id", &["C1", "C2", "C3"])]);
        let proposal = discover_schema(&[customers, orders, addresses], 0.99, 1.0);
        assert_eq!(proposal.component_count(), 1);
        assert_eq!(
            proposal.diagram.h1_count(),
            0,
            "a star/hub schema has no cycle to detect"
        );
    }

    #[test]
    fn schema_proposal_to_json_round_trips_expected_fields() {
        let customers = table("customers", &[("customer_id", &["C1", "C2", "C3"])]);
        let orders = table("orders", &[("customer_id", &["C1", "C2", "C3"])]);
        let proposal = discover_schema(&[customers, orders], 0.99, 1.0);
        let json = proposal.to_json();
        assert!(json.contains("\"tables\":[\"customers\",\"orders\"]"));
        assert!(json.contains("\"customer_id\""));
        assert!(json.contains("\"component_count\":1"));
    }
}
