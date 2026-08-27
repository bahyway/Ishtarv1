
//! Pirištu-respecting export — the veil enforced at the ENGINE, not the UI.
//! Sealed buckets leave the vault as structure only; the gloss never travels.
use crate::piristu::{sealed, visible_label};
use crate::store::AnsharV4Store;

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn to_json(s: &AnsharV4Store, viewer: usize) -> String {
    let mut out = String::from("{\"nodes\":[");
    for (i, n) in s.nodes.values().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!(
            "{{\"id\":\"{}\",\"type\":\"{}\",\"label\":\"{}\",\"sealed\":{}}}",
            esc(&n.id), esc(&n.ntype), esc(&visible_label(n, viewer)),
            sealed(n, viewer)));
    }
    out.push_str("],\"edges\":[");
    for (i, e) in s.edges.iter().enumerate() {
        if i > 0 { out.push(','); }
        out.push_str(&format!(
            "{{\"from\":\"{}\",\"to\":\"{}\",\"line\":\"{}\"}}",
            esc(&e.from), esc(&e.to), esc(&e.line)));
    }
    out.push_str("]}");
    out
}
