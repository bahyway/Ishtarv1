//! Orbit-layer queries + the sūqu ride — summit to the plain.
use crate::store::AnsharV4Store;
use std::collections::{BTreeMap, BTreeSet};

/// Related stations grouped by membrane (StoryEngine's spine).
pub fn orbit_layers(s: &AnsharV4Store, id: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut add = |out: &mut BTreeMap<String, BTreeSet<String>>, other: &str| {
        if other == id { return; }
        if let Some(o) = s.nodes.get(other) {
            out.entry(o.ntype.clone()).or_default().insert(other.to_string());
        }
    };
    for h in s.hyper.iter().filter(|h| h.kind != "pillar") {
        if h.members.iter().any(|m| m == id) {
            for m in &h.members { add(&mut out, m); }
        }
    }
    for e in &s.edges {
        if e.from == id { add(&mut out, &e.to); }
        if e.to == id { add(&mut out, &e.from); }
    }
    out
}

/// Summit → … → APP through the given station. Every ride reaches the plain.
pub fn ride_path(s: &AnsharV4Store, id: &str) -> Vec<String> {
    let up = |x: &str| -> Option<String> {
        let n = s.nodes.get(x)?;
        match n.ntype.as_str() {
            "TERM" => s.edges.iter()
                .find(|e| e.to == x && e.rel == "names").map(|e| e.from.clone()),
            "LAW" => Some("ALG".into()),
            "CRATE" | "APP" => s.edges.iter()
                .find(|e| e.to == x && e.line == "suqu").map(|e| e.from.clone()),
            _ => None,
        }
    };
    let mut climb = Vec::new();
    let mut cur = id.to_string();
    let mut guard = 0;
    while cur != "ALG" && guard < 8 {
        climb.insert(0, cur.clone());
        match up(&cur) { Some(p) => cur = p, None => break }
        guard += 1;
    }
    let mut chain = vec!["ALG".to_string()];
    chain.extend(climb);
    let mut guard = 0;
    loop {
        let tail = chain.last().unwrap().clone();
        let t = s.nodes.get(&tail).map(|n| n.ntype.clone()).unwrap_or_default();
        if t == "APP" || guard > 8 { break; }
        let next = s.edges.iter()
            .find(|e| e.from == tail && e.line == "suqu"
                && s.nodes.get(&e.to).map(|n| n.ntype != t).unwrap_or(false))
            .map(|e| e.to.clone())
            .or_else(|| s.edges.iter()
                .find(|e| e.from == tail && e.line == "harranu"
                    && s.nodes.get(&e.to).map(|n| n.ntype == "TERM").unwrap_or(false))
                .map(|e| e.to.clone()));
        match next {
            Some(n) if !chain.contains(&n) => chain.push(n),
            _ => break,
        }
        guard += 1;
    }
    chain
}
