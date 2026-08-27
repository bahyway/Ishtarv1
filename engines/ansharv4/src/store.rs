//! ANŠARV4 store — the whole sky holds the whole graph (PB-646).
//! Ingest format: anšar lines (append-only, NĀRU-kin):
//!   NODE id|type|label|clearance|p1p2   (pillars as letters, e.g. AC)
//!   EDGE from|to|rel|line
//!   HYPER id|kind|label|m1,m2,...
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub ntype: String,
    pub label: String,
    pub clearance: String,
    pub pillars: Vec<char>,
}
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub rel: String,
    pub line: String,
}
#[derive(Debug, Clone)]
pub struct Hyper {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub members: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AnsharV4Store {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
    pub hyper: Vec<Hyper>,
}

#[derive(Debug, PartialEq)]
pub enum IngestVerdict {
    Accepted,
    RejectedDuplicate(String),
    RejectedMalformed(String),
}

impl AnsharV4Store {
    pub fn ingest_line(&mut self, raw: &str) -> IngestVerdict {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('#') {
            return IngestVerdict::Accepted;
        }
        let (kind, rest) = match raw.split_once(' ') {
            Some(p) => p,
            None => return IngestVerdict::RejectedMalformed(raw.into()),
        };
        let f: Vec<&str> = rest.split('|').collect();
        match kind {
            "NODE" if f.len() == 5 => {
                if self.nodes.contains_key(f[0]) {
                    return IngestVerdict::RejectedDuplicate(f[0].into());
                }
                self.nodes.insert(f[0].into(), Node {
                    id: f[0].into(), ntype: f[1].into(), label: f[2].into(),
                    clearance: f[3].into(), pillars: f[4].chars().collect(),
                });
                IngestVerdict::Accepted
            }
            "EDGE" if f.len() == 4 => {
                self.edges.push(Edge {
                    from: f[0].into(), to: f[1].into(),
                    rel: f[2].into(), line: f[3].into(),
                });
                IngestVerdict::Accepted
            }
            "HYPER" if f.len() == 4 => {
                self.hyper.push(Hyper {
                    id: f[0].into(), kind: f[1].into(), label: f[2].into(),
                    members: f[3].split(',').map(|s| s.into()).collect(),
                });
                IngestVerdict::Accepted
            }
            _ => IngestVerdict::RejectedMalformed(raw.into()),
        }
    }

    /// The integrity court — the five checks the Ziqqurratu swore.
    pub fn integrity(&self) -> Result<(), Vec<String>> {
        let mut faults = Vec::new();
        for e in &self.edges {
            if !self.nodes.contains_key(&e.from) || !self.nodes.contains_key(&e.to) {
                faults.push(format!("edge endpoint missing: {}->{}", e.from, e.to));
            }
        }
        for h in &self.hyper {
            if h.kind == "tribe" {
                let types: BTreeSet<_> = h.members.iter()
                    .filter_map(|m| self.nodes.get(m).map(|n| n.ntype.clone()))
                    .collect();
                if types.len() < 2 {
                    faults.push(format!("tribe {} spans <2 membranes", h.id));
                }
            }
        }
        for n in self.nodes.values() {
            if n.ntype == "TERM"
                && !self.edges.iter().any(|e| e.from == n.id && e.line == "suqu")
            {
                faults.push(format!("TERM {} rides no suqu", n.id));
            }
            if n.pillars.is_empty() {
                faults.push(format!("station {} has no pillar", n.id));
            }
        }
        if faults.is_empty() { Ok(()) } else { Err(faults) }
    }
}
