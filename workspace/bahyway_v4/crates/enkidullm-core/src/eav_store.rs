//! VerticalEavStore — columnar EAV store for book attributes.
//!
//! A "vertical line" = all values for one attribute across all book KAKIs.
//! O(log n) for single-attribute queries; O(n_attrs) for full-profile reconstruction.
//!
//! Key design: OrbitValue::KakiRef IS an inter-KAKI edge. The knowledge graph
//! emerges from typed values — not a separate data structure. The graph is implicit
//! in the data and explicit in the query planner (§8.3: CrossTribe is computed).

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// Typed Orbit value — no external serialization library.
/// Float is stored as bit-pattern u64 so OrbitValue can implement Ord for BTreeMap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrbitValue {
    Absent,
    Text(String),
    Integer(i64),
    /// f32 stored as bits — NaN is excluded at construction.
    Float(u32),
    Boolean(bool),
    /// Inter-KAKI edge: uuid_hash of another book.
    KakiRef(u32),
    /// External citation: DOI or ISBN string.
    DoiRef(String),
    /// Internal knowledge graph node reference.
    ConceptRef(String),
    /// Dense embedding vector — excluded from BTree index (too large).
    Embedding(Vec<u32>),
}

impl OrbitValue {
    /// Store an f32 (NaN-safe: NaN becomes Absent).
    pub fn from_f32(v: f32) -> Self {
        if v.is_nan() {
            OrbitValue::Absent
        } else {
            OrbitValue::Float(v.to_bits())
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        if let OrbitValue::Float(b) = self {
            Some(f32::from_bits(*b))
        } else {
            None
        }
    }

    /// Store a Vec<f32> as an embedding (bit-pattern encoded for no-serde storage).
    pub fn from_embedding(v: &[f32]) -> Self {
        OrbitValue::Embedding(v.iter().map(|f| f.to_bits()).collect())
    }

    pub fn as_embedding(&self) -> Option<Vec<f32>> {
        if let OrbitValue::Embedding(bits) = self {
            Some(bits.iter().map(|b| f32::from_bits(*b)).collect())
        } else {
            None
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            OrbitValue::Absent => 0,
            OrbitValue::Text(_) => 1,
            OrbitValue::Integer(_) => 2,
            OrbitValue::Float(_) => 3,
            OrbitValue::Boolean(_) => 4,
            OrbitValue::KakiRef(_) => 5,
            OrbitValue::DoiRef(_) => 6,
            OrbitValue::ConceptRef(_) => 7,
            OrbitValue::Embedding(_) => 8,
        }
    }
}

impl PartialOrd for OrbitValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrbitValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use OrbitValue::*;
        match (self, other) {
            (Text(a), Text(b)) => a.cmp(b),
            (Integer(a), Integer(b)) => a.cmp(b),
            (Float(a), Float(b)) => a.cmp(b),
            (Boolean(a), Boolean(b)) => a.cmp(b),
            (KakiRef(a), KakiRef(b)) => a.cmp(b),
            (DoiRef(a), DoiRef(b)) => a.cmp(b),
            (ConceptRef(a), ConceptRef(b)) => a.cmp(b),
            _ => self.discriminant().cmp(&other.discriminant()),
        }
    }
}

/// One vertical attribute column: all (kaki_uuid, value, version) entries + index.
#[derive(Debug, Default)]
pub struct VerticalLine {
    /// Append-only log — each update appends a new versioned entry.
    pub entries: Vec<(u32, OrbitValue, u64)>,
    /// Inverted index: value → kaki_uuids. Embeddings are NOT indexed.
    pub index: BTreeMap<OrbitValue, Vec<u32>>,
}

impl VerticalLine {
    pub fn insert(&mut self, kaki_uuid: u32, value: OrbitValue, version: u64) {
        if !matches!(value, OrbitValue::Embedding(_)) {
            self.index.entry(value.clone()).or_default().push(kaki_uuid);
        }
        self.entries.push((kaki_uuid, value, version));
    }

    /// O(log n) — index lookup.
    pub fn find_by_value(&self, value: &OrbitValue) -> Vec<u32> {
        self.index.get(value).cloned().unwrap_or_default()
    }

    /// Get highest-version value for a specific kaki_uuid. O(n entries).
    pub fn get_latest(&self, kaki_uuid: u32) -> Option<&OrbitValue> {
        self.entries
            .iter()
            .filter(|(uid, _, _)| *uid == kaki_uuid)
            .max_by_key(|(_, _, v)| *v)
            .map(|(_, val, _)| val)
    }

    /// Get all versioned values for a kaki — for temporal DIFF queries.
    pub fn history(&self, kaki_uuid: u32) -> Vec<(&OrbitValue, u64)> {
        self.entries
            .iter()
            .filter(|(uid, _, _)| *uid == kaki_uuid)
            .map(|(_, val, ver)| (val, *ver))
            .collect()
    }
}

/// The complete EAV store — collection of named vertical lines.
#[derive(Debug, Default)]
pub struct VerticalEavStore {
    pub lines: HashMap<String, VerticalLine>,
}

impl VerticalEavStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an attribute value. O(log n) for index.
    pub fn insert(&mut self, kaki_uuid: u32, attribute: &str, value: OrbitValue, version: u64) {
        self.lines
            .entry(attribute.to_string())
            .or_default()
            .insert(kaki_uuid, value, version);
    }

    /// Find all KAKIs where attribute = value. O(log n).
    pub fn find_by_attribute(&self, attribute: &str, value: &OrbitValue) -> Vec<u32> {
        self.lines
            .get(attribute)
            .map(|l| l.find_by_value(value))
            .unwrap_or_default()
    }

    /// Find all KAKIs where attribute value is >= min and <= max. O(k) where k = range hits.
    pub fn find_in_range(&self, attribute: &str, min: &OrbitValue, max: &OrbitValue) -> Vec<u32> {
        let Some(line) = self.lines.get(attribute) else {
            return Vec::new();
        };
        line.index
            .range(min.clone()..=max.clone())
            .flat_map(|(_, uuids)| uuids.iter().copied())
            .collect()
    }

    /// Scatter-gather: get all latest attribute values for one KAKI. O(n_attributes).
    pub fn get_profile(&self, kaki_uuid: u32) -> HashMap<&str, &OrbitValue> {
        let mut profile = HashMap::new();
        for (attr, line) in &self.lines {
            if let Some(val) = line.get_latest(kaki_uuid) {
                profile.insert(attr.as_str(), val);
            }
        }
        profile
    }

    /// BFS over KakiRef edges from a starting uuid_hash. Returns visited uuid_hashes.
    pub fn traverse_kaki_refs(&self, start: u32, max_depth: usize) -> Vec<u32> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        queue.push_back((start, 0usize));
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth || !visited.insert(current) {
                continue;
            }
            result.push(current);
            for line in self.lines.values() {
                if let Some(OrbitValue::KakiRef(target)) = line.get_latest(current) {
                    queue.push_back((*target, depth + 1));
                }
            }
        }
        result
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_find_text() {
        let mut store = VerticalEavStore::new();
        store.insert(
            0xA1,
            "domain",
            OrbitValue::Text("distributed_systems".into()),
            1,
        );
        store.insert(
            0xB2,
            "domain",
            OrbitValue::Text("machine_learning".into()),
            1,
        );
        let hits =
            store.find_by_attribute("domain", &OrbitValue::Text("distributed_systems".into()));
        assert_eq!(hits, vec![0xA1]);
    }

    #[test]
    fn find_in_float_range() {
        let mut store = VerticalEavStore::new();
        store.insert(0x01, "depth", OrbitValue::from_f32(0.9), 1);
        store.insert(0x02, "depth", OrbitValue::from_f32(0.5), 1);
        store.insert(0x03, "depth", OrbitValue::from_f32(0.3), 1);
        let hits = store.find_in_range(
            "depth",
            &OrbitValue::from_f32(0.7),
            &OrbitValue::from_f32(1.0),
        );
        assert!(hits.contains(&0x01));
        assert!(!hits.contains(&0x02));
    }

    #[test]
    fn get_profile_scatter_gather() {
        let mut store = VerticalEavStore::new();
        store.insert(
            0xA1,
            "domain",
            OrbitValue::Text("distributed_systems".into()),
            1,
        );
        store.insert(0xA1, "depth", OrbitValue::from_f32(0.85), 1);
        store.insert(0xA1, "citations", OrbitValue::Integer(9500), 1);
        let profile = store.get_profile(0xA1);
        assert_eq!(profile.len(), 3);
        assert!(profile.contains_key("domain"));
    }

    #[test]
    fn version_history() {
        let mut store = VerticalEavStore::new();
        store.insert(0xA1, "depth", OrbitValue::from_f32(0.5), 1);
        store.insert(0xA1, "depth", OrbitValue::from_f32(0.8), 2);
        let line = store.lines.get("depth").unwrap();
        let hist = line.history(0xA1);
        assert_eq!(hist.len(), 2);
        // Latest should be version 2
        let latest = line.get_latest(0xA1).unwrap().as_f32().unwrap();
        assert!((latest - 0.8).abs() < 1e-5);
    }

    #[test]
    fn kaki_ref_traversal() {
        let mut store = VerticalEavStore::new();
        // A cites B, B cites C
        store.insert(0xAA, "cites", OrbitValue::KakiRef(0xBB), 1);
        store.insert(0xBB, "cites", OrbitValue::KakiRef(0xCC), 1);
        let traversal = store.traverse_kaki_refs(0xAA, 3);
        assert!(traversal.contains(&0xAA));
        assert!(traversal.contains(&0xBB));
        assert!(traversal.contains(&0xCC));
    }

    #[test]
    fn no_cycle_in_traversal() {
        let mut store = VerticalEavStore::new();
        // A → B → A (cycle)
        store.insert(0xAA, "cites", OrbitValue::KakiRef(0xBB), 1);
        store.insert(0xBB, "cites", OrbitValue::KakiRef(0xAA), 1);
        let traversal = store.traverse_kaki_refs(0xAA, 10);
        // Should visit each node at most once
        let unique: HashSet<u32> = traversal.iter().copied().collect();
        assert_eq!(traversal.len(), unique.len());
    }

    #[test]
    fn embedding_not_indexed() {
        let mut store = VerticalEavStore::new();
        store.insert(
            0xA1,
            "embedding",
            OrbitValue::from_embedding(&[0.1, 0.2, 0.3]),
            1,
        );
        // Index should be empty for embeddings
        let line = store.lines.get("embedding").unwrap();
        assert!(line.index.is_empty());
        // But entries should have it
        assert_eq!(line.entries.len(), 1);
    }

    #[test]
    fn orbit_value_ordering() {
        assert!(OrbitValue::Integer(10) > OrbitValue::Integer(5));
        assert!(OrbitValue::Text("b".into()) > OrbitValue::Text("a".into()));
        assert_eq!(OrbitValue::Absent, OrbitValue::Absent);
    }

    #[test]
    fn float_roundtrip() {
        let v = OrbitValue::from_f32(0.72);
        let back = v.as_f32().unwrap();
        assert!((back - 0.72).abs() < 1e-6);
    }
}
