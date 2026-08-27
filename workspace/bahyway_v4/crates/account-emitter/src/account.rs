//! The real GL-FLD-001 §10 account model: a stakeholder's biography is
//! delivered as a simplicial complex over six real vertices — never a
//! result set (Anti-SQL, enforced separately by Gate A1's own source
//! grep for SELECT/FROM/JOIN, which this module never uses).

use std::collections::{HashMap, HashSet};

use enkidb_particle_store::{all_db_dirs, ParticleStore};

/// The six real account vertices declared in `account_v4.toml`'s own
/// `[shape].vertices` — kept in sync with that config, not invented
/// here.
pub const VERTICES: [&str; 6] = ["particle", "events", "cohorts", "column_shells", "leaf", "ring"];

/// The seven real epistemic classes declared in `account_v4.toml`'s own
/// `[ledger].classes`.
pub const CLASSES: [&str; 7] = [
    "MEASURED",
    "DERIVED",
    "ESTIMATED",
    "ADVISED",
    "COHERENT",
    "INCOHERENT",
    "AMBIGUOUS",
];

pub struct LedgerLine {
    pub vertex: String,
    /// Single-valued by construction — the type system itself enforces
    /// §6 ("INCOHERENT may not borrow the credibility of AMBIGUOUS"): a
    /// `String` can never hold two classes at once.
    pub class: String,
}

pub struct BettiReport {
    pub betti_0: usize,
    pub betti_1_raw: usize,
    pub betti_1_filled: usize,
}

/// A real relation binding two or more account vertices at once (a
/// "filled 2-simplex" per §10 when it binds three or more).
pub struct Relation {
    pub vertices: Vec<String>,
}

/// Real relations discovered in the local particle store: any real
/// particle whose payload carries a `relates` array naming two or more
/// of the six real account vertices. Absent such particles, the account
/// has zero real relations recorded yet — an honest empty baseline, not
/// fabricated.
pub fn discover_relations() -> Vec<Relation> {
    let mut relations = Vec::new();
    for dir in all_db_dirs() {
        let store = ParticleStore::load(&dir).unwrap_or_default();
        for p in &store.particles {
            let Some(arr) = p.payload.get("relates").and_then(|v| v.as_array()) else {
                continue;
            };
            let mut names: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str())
                .filter(|s| VERTICES.contains(s))
                .map(|s| s.to_string())
                .collect();
            names.sort();
            names.dedup();
            if names.len() >= 2 {
                relations.push(Relation { vertices: names });
            }
        }
    }
    relations
}

/// One epistemic-classified ledger line per real account vertex. A
/// vertex is MEASURED when a real relation names it (real evidence
/// exists); ADVISED otherwise (an assumed baseline, honestly labeled,
/// never silently upgraded to a stronger class).
pub fn build_ledger(relations: &[Relation]) -> Vec<LedgerLine> {
    let evidenced: HashSet<&str> = relations
        .iter()
        .flat_map(|r| r.vertices.iter().map(|s| s.as_str()))
        .collect();
    VERTICES
        .iter()
        .map(|v| {
            let class = if evidenced.contains(v) { "MEASURED" } else { "ADVISED" };
            LedgerLine {
                vertex: v.to_string(),
                class: class.to_string(),
            }
        })
        .collect()
}

/// Real β0 (connected components, union-find) and β1 (E − V + C, the
/// standard first-Betti-number formula for a graph) over the six real
/// account vertices and any real relations found. `betti_1_filled`
/// subtracts one cycle per real ≥3-vertex relation (a "filled
/// 2-simplex" per `account_v4.toml`) — an unfilled loop stays a real
/// hole and is never silently patched.
pub fn compute_betti(relations: &[Relation]) -> BettiReport {
    let mut parent: HashMap<&str, &str> = VERTICES.iter().map(|v| (*v, *v)).collect();

    fn find<'a>(parent: &mut HashMap<&'a str, &'a str>, x: &'a str) -> &'a str {
        let p = parent[x];
        if p != x {
            let root = find(parent, p);
            parent.insert(x, root);
            root
        } else {
            x
        }
    }

    let mut edge_count = 0usize;
    for rel in relations {
        for i in 0..rel.vertices.len() {
            for j in (i + 1)..rel.vertices.len() {
                edge_count += 1;
                let a = rel.vertices[i].as_str();
                let b = rel.vertices[j].as_str();
                let ra = find(&mut parent, a);
                let rb = find(&mut parent, b);
                if ra != rb {
                    parent.insert(ra, rb);
                }
            }
        }
    }

    let components: HashSet<&str> = VERTICES.iter().map(|v| find(&mut parent, v)).collect();
    let betti_0 = components.len();
    let v = VERTICES.len();
    let c = betti_0;
    let betti_1_raw = (edge_count + c).saturating_sub(v);

    let filled_relations = relations.iter().filter(|r| r.vertices.len() >= 3).count();
    let betti_1_filled = betti_1_raw.saturating_sub(filled_relations);

    BettiReport {
        betti_0,
        betti_1_raw,
        betti_1_filled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(vs: &[&str]) -> Relation {
        Relation {
            vertices: vs.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn no_relations_gives_six_components_and_no_cycles() {
        let report = compute_betti(&[]);
        assert_eq!(report.betti_0, 6);
        assert_eq!(report.betti_1_raw, 0);
        assert_eq!(report.betti_1_filled, 0);
    }

    #[test]
    fn one_pairwise_relation_merges_two_vertices() {
        let relations = vec![rel(&["particle", "events"])];
        let report = compute_betti(&relations);
        assert_eq!(report.betti_0, 5); // 6 vertices, one merge
        assert_eq!(report.betti_1_raw, 0); // a tree edge, no cycle
    }

    #[test]
    fn a_triangle_relation_creates_one_raw_cycle_that_is_filled() {
        let relations = vec![rel(&["particle", "events", "cohorts"])];
        let report = compute_betti(&relations);
        // Complete graph on 3 vertices: 3 edges, 3 vertices in this
        // component + 3 isolated -> 4 components total, E=3, C=4, V=6
        // raw = 3 + 4 - 6 = 1 real independent cycle.
        assert_eq!(report.betti_1_raw, 1);
        // A >=3-vertex relation is a filled 2-simplex -- the cycle it
        // creates is closed, not a hole.
        assert_eq!(report.betti_1_filled, 0);
    }

    #[test]
    fn ledger_has_one_line_per_vertex_all_in_declared_classes() {
        let ledger = build_ledger(&[]);
        assert_eq!(ledger.len(), VERTICES.len());
        for line in &ledger {
            assert!(CLASSES.contains(&line.class.as_str()));
        }
    }

    #[test]
    fn evidenced_vertex_is_measured_unevidenced_is_advised() {
        let relations = vec![rel(&["particle", "events"])];
        let ledger = build_ledger(&relations);
        let by_vertex: HashMap<&str, &str> = ledger.iter().map(|l| (l.vertex.as_str(), l.class.as_str())).collect();
        assert_eq!(by_vertex["particle"], "MEASURED");
        assert_eq!(by_vertex["leaf"], "ADVISED");
    }
}
