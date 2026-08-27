//! PIRIŠTU — conditional transparency, enforced at the engine.
//! mūdû mūdâ likallim, lā mūdû ay īmur.
use crate::store::{AnsharV4Store, Node};

pub const RINGS: [&str; 3] = ["PUBLIC", "PARTNER", "DUBSAR"];

pub fn ring(clearance: &str) -> usize {
    RINGS.iter().position(|r| *r == clearance).unwrap_or(0)
}
pub fn sealed(n: &Node, viewer: usize) -> bool {
    ring(&n.clearance) > viewer
}
/// Structure always; the label itself is inside the bucket.
pub fn visible_label(n: &Node, viewer: usize) -> String {
    if sealed(n, viewer) { "𒁾 pirištu".into() } else { n.label.clone() }
}
/// Monotone disclosure: every higher ring sees a superset. Law, proven.
pub fn certify_monotone(store: &AnsharV4Store) -> bool {
    store.nodes.values().all(|n| {
        let v: Vec<bool> = (0..3).map(|w| !sealed(n, w)).collect();
        !(v[0] && !v[1]) && !(v[1] && !v[2])
    })
}
