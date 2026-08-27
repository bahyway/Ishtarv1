//! two-scribes — GL-LBR-001-A1 (The Migration Chapter): the Two Scribes
//! Rite, Kima Labirisu over warehouse migrations. PB-339. Pure Rust, zero
//! dependencies.
//!
//! The inversion: migration validation confronts OUTPUTS on sealed
//! inputs (Merkle-hashed, canonicalized), not the scribe's hand (unit
//! tests over thousands of procedures).
#![forbid(unsafe_code)]

use std::collections::HashMap;

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Rite I/II — canonicalize a row: key-ordered fields, float quantum ε.
/// Canonicalization must be deterministic and idempotent (L32).
pub fn canonicalize_row(fields: &mut Vec<(String, String)>, float_quantum: f64) {
    fields.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, v) in fields.iter_mut() {
        if let Ok(f) = v.parse::<f64>() {
            let q = (f / float_quantum).round() * float_quantum;
            *v = format!("{q:.6}");
        }
    }
}

pub fn hash_row(fields: &[(String, String)]) -> u64 {
    let mut buf = String::new();
    for (k, v) in fields {
        buf.push_str(k);
        buf.push('=');
        buf.push_str(v);
        buf.push(';');
    }
    fnv1a(buf.as_bytes())
}

fn merkle_of(hashes: &[u64]) -> u64 {
    let mut buf = Vec::with_capacity(hashes.len() * 8);
    for h in hashes {
        buf.extend_from_slice(&h.to_le_bytes());
    }
    fnv1a(&buf)
}

/// Rite II — Merkle table: row-group -> partition -> table -> warehouse
/// root. This struct holds one level (row-group hashes) plus its root;
/// higher levels compose the same way.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub row_group_hashes: Vec<u64>,
    pub root: u64,
}

impl MerkleTree {
    pub fn build(row_group_hashes: Vec<u64>) -> Self {
        let root = merkle_of(&row_group_hashes);
        Self { row_group_hashes, root }
    }
}

/// Drill to the diverging row-groups in O(log n) subtree comparisons: a
/// real binary divide-and-conquer over the leaf hash arrays, descending
/// only into halves whose subtree hash differs.
pub fn drill_diff(a: &MerkleTree, b: &MerkleTree) -> (bool, Vec<usize>) {
    if a.root == b.root {
        return (false, vec![]);
    }
    fn recurse(a: &[u64], b: &[u64], offset: usize, out: &mut Vec<usize>) {
        if a.len() == 1 {
            if a[0] != b[0] {
                out.push(offset);
            }
            return;
        }
        let mid = a.len() / 2;
        let (a1, a2) = a.split_at(mid);
        let (b1, b2) = b.split_at(mid);
        if merkle_of(a1) != merkle_of(b1) {
            recurse(a1, b1, offset, out);
        }
        if merkle_of(a2) != merkle_of(b2) {
            recurse(a2, b2, offset + mid, out);
        }
    }
    let mut out = Vec::new();
    recurse(&a.row_group_hashes, &b.row_group_hashes, 0, &mut out);
    (true, out)
}

/// Rite III — divergence kind, one of the four sealed classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DivergenceKind {
    MissingRows,
    ExtraRows,
    ValueDrift,
    TypeDrift,
}

pub fn classify_divergence(
    legacy_row_count: usize,
    new_row_count: usize,
    value_type_changed: bool,
) -> DivergenceKind {
    if new_row_count < legacy_row_count {
        DivergenceKind::MissingRows
    } else if new_row_count > legacy_row_count {
        DivergenceKind::ExtraRows
    } else if value_type_changed {
        DivergenceKind::TypeDrift
    } else {
        DivergenceKind::ValueDrift
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DivergenceSignature {
    pub stage: String,
    pub table: String,
    pub column: String,
    pub kind: DivergenceKind,
}

/// Rite III — FCA-style closure: procedures sharing an identical
/// divergence signature collapse into one concept (one root cause, one
/// inquest per concept).
pub fn cluster_by_signature(
    items: &[(String, DivergenceSignature)],
) -> HashMap<DivergenceSignature, Vec<String>> {
    let mut map: HashMap<DivergenceSignature, Vec<String>> = HashMap::new();
    for (procedure, sig) in items {
        map.entry(sig.clone()).or_default().push(procedure.clone());
    }
    map
}

/// Rite IV — a pre-registered, W5H2-signed decreed evolution particle.
#[derive(Debug, Clone)]
pub struct DecreedEvolution {
    pub signature: DivergenceSignature,
    pub w5h2_signed: bool,
}

pub fn covered_by_decree(sig: &DivergenceSignature, decrees: &[DecreedEvolution]) -> bool {
    decrees.iter().any(|d| &d.signature == sig && d.w5h2_signed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colophon {
    Concord,
    LawfulEvolution,
    SilentDrift,
}

/// §3 — the colophon per procedure: CONCORD, LAWFUL EVOLUTION, or
/// (uncovered) SILENT DRIFT -> migration RIGMU.
pub fn colophon(
    root_matches: bool,
    sig: Option<&DivergenceSignature>,
    decrees: &[DecreedEvolution],
) -> Colophon {
    if root_matches {
        return Colophon::Concord;
    }
    match sig {
        Some(s) if covered_by_decree(s, decrees) => Colophon::LawfulEvolution,
        _ => Colophon::SilentDrift,
    }
}

/// §4 — coverage is measured: the fraction of warehouse mass under
/// CONCORD/LAWFUL, replacing "tests written" as the true progress metric.
pub fn coverage_fraction(colophons: &[Colophon]) -> f64 {
    if colophons.is_empty() {
        return 1.0;
    }
    let ok = colophons
        .iter()
        .filter(|c| matches!(c, Colophon::Concord | Colophon::LawfulEvolution))
        .count();
    ok as f64 / colophons.len() as f64
}

/// A KAKIv4.0 / go-live gate passes only when the warehouse root is
/// CONCORD-or-Lawful across ALL stages (§3).
pub fn release_gate_passes(colophons: &[Colophon]) -> bool {
    !colophons.is_empty() && coverage_fraction(colophons) == 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // L32 — canonicalization is deterministic and idempotent.
    #[test]
    fn l32_canonicalization_deterministic_and_idempotent() {
        let mut fields = vec![
            ("b".to_string(), "1.00003".to_string()),
            ("a".to_string(), "2".to_string()),
        ];
        canonicalize_row(&mut fields, 0.001);
        let once = fields.clone();
        canonicalize_row(&mut fields, 0.001);
        assert_eq!(once, fields, "canonicalizing twice must be a no-op");
        assert_eq!(fields[0].0, "a", "key-ordered");
        assert_eq!(fields[1].0, "b");
    }

    // L33 — Merkle root comparison: identical row-groups -> equal root;
    // a single differing row drills to exactly that row-group.
    #[test]
    fn l33_merkle_drill_localizes_single_divergence() {
        let leaves_a: Vec<u64> = (0..16).map(|i| fnv1a(&[i as u8])).collect();
        let mut leaves_b = leaves_a.clone();

        let tree_a = MerkleTree::build(leaves_a.clone());
        let tree_b_same = MerkleTree::build(leaves_b.clone());
        assert_eq!(tree_a.root, tree_b_same.root);
        let (diverged, indices) = drill_diff(&tree_a, &tree_b_same);
        assert!(!diverged);
        assert!(indices.is_empty());

        leaves_b[9] = fnv1a(b"corrupted");
        let tree_b = MerkleTree::build(leaves_b);
        let (diverged, indices) = drill_diff(&tree_a, &tree_b);
        assert!(diverged);
        assert_eq!(indices, vec![9], "must localize exactly the diverging row-group");
    }

    // L34 — divergence signature extraction classifies each kind correctly.
    #[test]
    fn l34_divergence_classification() {
        assert_eq!(classify_divergence(100, 90, false), DivergenceKind::MissingRows);
        assert_eq!(classify_divergence(100, 110, false), DivergenceKind::ExtraRows);
        assert_eq!(classify_divergence(100, 100, true), DivergenceKind::TypeDrift);
        assert_eq!(classify_divergence(100, 100, false), DivergenceKind::ValueDrift);
    }

    // L35 — decree coverage: covered -> LAWFUL EVOLUTION; uncovered ->
    // SILENT DRIFT (undecreed divergence is a bug by definition).
    #[test]
    fn l35_decree_coverage_routes_colophon() {
        let sig = DivergenceSignature {
            stage: "transform".into(),
            table: "orders".into(),
            column: "total".into(),
            kind: DivergenceKind::ValueDrift,
        };
        let decrees = vec![DecreedEvolution { signature: sig.clone(), w5h2_signed: true }];
        assert_eq!(colophon(false, Some(&sig), &decrees), Colophon::LawfulEvolution);

        let other_sig = DivergenceSignature { column: "tax".into(), ..sig };
        assert_eq!(colophon(false, Some(&other_sig), &decrees), Colophon::SilentDrift);
        assert_eq!(colophon(true, None, &decrees), Colophon::Concord);
    }

    // L36 — coverage fraction and the all-or-nothing release gate.
    #[test]
    fn l36_coverage_and_release_gate() {
        let all_good = vec![Colophon::Concord, Colophon::LawfulEvolution, Colophon::Concord];
        assert_eq!(coverage_fraction(&all_good), 1.0);
        assert!(release_gate_passes(&all_good));

        let one_bad = vec![Colophon::Concord, Colophon::SilentDrift];
        assert_eq!(coverage_fraction(&one_bad), 0.5);
        assert!(!release_gate_passes(&one_bad), "gate refuses on ANY silent drift");
    }
}
