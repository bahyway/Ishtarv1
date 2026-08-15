//! graph-engine — Directed Graph, PageRank, Betweenness Centrality,
//! Strongly Connected Components. ALGEBRA_GLOSSARY.md Part V Layer 6,
//! items 25-28. Confirmed genuinely absent before this crate: no
//! graph-engine crate, no PageRank/Betweenness/SCC hit anywhere in the
//! workspace.
//!
//! Sovereign Rules (ALGEBRA_GLOSSARY.md): damping factor d=0.85 for
//! PageRank; SCC of size >=3 is a mandatory AlertEngine trigger (AML
//! ring detection) — wired here via `scc_alerts`, the same "consume,
//! don't duplicate" pattern Enbilulu established with alert-engine.

use alert_engine::{Alert, AlertSeverity, DriftCause};
use std::collections::{HashSet, VecDeque};

/// PageRank's Sovereign Rule damping factor.
pub const PAGERANK_DAMPING: f64 = 0.85;

/// A directed graph over nodes `0..n`, given as an adjacency list.
#[derive(Debug, Clone)]
pub struct DirectedGraph {
    pub n: usize,
    adj: Vec<Vec<usize>>,
}

impl DirectedGraph {
    pub fn new(n: usize) -> Self {
        DirectedGraph { n, adj: vec![Vec::new(); n] }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.adj[from].push(to);
    }

    pub fn neighbors(&self, node: usize) -> &[usize] { &self.adj[node] }

    pub fn out_degree(&self, node: usize) -> usize { self.adj[node].len() }

    // ------------------------------------------------------------------
    // PageRank — power iteration with damping.
    // ------------------------------------------------------------------

    /// PageRank scores, summing to 1.0. Dangling nodes (out-degree 0)
    /// redistribute their mass uniformly to all nodes, per the standard
    /// formulation.
    pub fn pagerank(&self, damping: f64, max_iter: usize, tol: f64) -> Vec<f64> {
        let n = self.n;
        if n == 0 { return Vec::new(); }
        let mut rank = vec![1.0 / n as f64; n];

        for _ in 0..max_iter {
            let dangling_mass: f64 = (0..n)
                .filter(|&i| self.out_degree(i) == 0)
                .map(|i| rank[i])
                .sum();

            let mut next = vec![(1.0 - damping) / n as f64; n];
            for i in 0..n {
                let deg = self.out_degree(i);
                if deg == 0 { continue; }
                let share = damping * rank[i] / deg as f64;
                for &j in self.neighbors(i) {
                    next[j] += share;
                }
            }
            // redistribute dangling mass uniformly
            let dangling_share = damping * dangling_mass / n as f64;
            for v in next.iter_mut() { *v += dangling_share; }

            let delta: f64 = next.iter().zip(rank.iter()).map(|(a, b)| (a - b).abs()).sum();
            rank = next;
            if delta < tol { break; }
        }
        rank
    }

    // ------------------------------------------------------------------
    // Betweenness Centrality — Brandes' algorithm (unweighted, BFS-based).
    // ------------------------------------------------------------------

    pub fn betweenness_centrality(&self) -> Vec<f64> {
        let n = self.n;
        let mut cb = vec![0.0f64; n];

        for s in 0..n {
            let mut stack: Vec<usize> = Vec::new();
            let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut sigma = vec![0.0f64; n];
            sigma[s] = 1.0;
            let mut dist = vec![-1i64; n];
            dist[s] = 0;
            let mut queue = VecDeque::new();
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                for &w in self.neighbors(v) {
                    if dist[w] < 0 {
                        dist[w] = dist[v] + 1;
                        queue.push_back(w);
                    }
                    if dist[w] == dist[v] + 1 {
                        sigma[w] += sigma[v];
                        pred[w].push(v);
                    }
                }
            }

            let mut delta = vec![0.0f64; n];
            while let Some(w) = stack.pop() {
                for &v in &pred[w] {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
                if w != s {
                    cb[w] += delta[w];
                }
            }
        }
        cb
    }

    // ------------------------------------------------------------------
    // Strongly Connected Components — Tarjan's algorithm.
    // ------------------------------------------------------------------

    pub fn strongly_connected_components(&self) -> Vec<Vec<usize>> {
        struct TarjanState {
            index_counter: usize,
            stack: Vec<usize>,
            on_stack: Vec<bool>,
            indices: Vec<i64>,
            lowlink: Vec<usize>,
            components: Vec<Vec<usize>>,
        }

        fn strongconnect(g: &DirectedGraph, v: usize, st: &mut TarjanState) {
            st.indices[v] = st.index_counter as i64;
            st.lowlink[v] = st.index_counter;
            st.index_counter += 1;
            st.stack.push(v);
            st.on_stack[v] = true;

            for &w in g.neighbors(v) {
                if st.indices[w] < 0 {
                    strongconnect(g, w, st);
                    st.lowlink[v] = st.lowlink[v].min(st.lowlink[w]);
                } else if st.on_stack[w] {
                    st.lowlink[v] = st.lowlink[v].min(st.indices[w] as usize);
                }
            }

            if st.lowlink[v] == st.indices[v] as usize {
                let mut component = Vec::new();
                loop {
                    let w = st.stack.pop().unwrap();
                    st.on_stack[w] = false;
                    component.push(w);
                    if w == v { break; }
                }
                st.components.push(component);
            }
        }

        let n = self.n;
        let mut st = TarjanState {
            index_counter: 0,
            stack: Vec::new(),
            on_stack: vec![false; n],
            indices: vec![-1; n],
            lowlink: vec![0; n],
            components: Vec::new(),
        };

        for v in 0..n {
            if st.indices[v] < 0 {
                strongconnect(self, v, &mut st);
            }
        }
        st.components
    }

    /// Sovereign Rule: SCC of size >= 3 is a mandatory AlertEngine
    /// trigger (AML ring detection). `node_uuid` maps node index ->
    /// the particle uuid_hash to attach to the emitted Alert.
    pub fn scc_alerts(&self, node_uuid: &[u32]) -> Vec<Alert> {
        self.strongly_connected_components()
            .into_iter()
            .filter(|c| c.len() >= 3)
            .map(|component| {
                // representative hash: XOR of member uuid_hashes, stable
                // regardless of Tarjan's internal component ordering.
                let rep_hash = component.iter().fold(0u32, |acc, &n| acc ^ node_uuid[n]);
                Alert::new(rep_hash, AlertSeverity::Critical, DriftCause::Fraud, component.len() as f32)
            })
            .collect()
    }

    /// Whether every node is reachable from `start` (used to sanity-check
    /// test fixtures / connectivity, not part of the glossary spec).
    pub fn reachable_from(&self, start: usize) -> HashSet<usize> {
        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        seen.insert(start);
        while let Some(v) = queue.pop_front() {
            for &w in self.neighbors(v) {
                if seen.insert(w) {
                    queue.push_back(w);
                }
            }
        }
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagerank_sums_to_one() {
        let mut g = DirectedGraph::new(4);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(2, 0);
        g.add_edge(2, 3);
        let pr = g.pagerank(PAGERANK_DAMPING, 200, 1e-12);
        let total: f64 = pr.iter().sum();
        assert!((total - 1.0).abs() < 1e-6, "pagerank should sum to 1, got {total}");
    }

    #[test]
    fn pagerank_ranks_more_linked_node_higher() {
        // Node 1 receives links from both 0 and 2; node 0 and 2 receive none.
        let mut g = DirectedGraph::new(3);
        g.add_edge(0, 1);
        g.add_edge(2, 1);
        let pr = g.pagerank(PAGERANK_DAMPING, 200, 1e-12);
        assert!(pr[1] > pr[0], "node 1 (2 inlinks) should outrank node 0 (0 inlinks)");
        assert!(pr[1] > pr[2], "node 1 (2 inlinks) should outrank node 2 (0 inlinks)");
    }

    #[test]
    fn betweenness_zero_for_endpoints_positive_for_bridge() {
        // Path graph 0 -> 1 -> 2 (directed). Node 1 lies on the only
        // shortest path from 0 to 2.
        let mut g = DirectedGraph::new(3);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        let cb = g.betweenness_centrality();
        assert!(cb[1] > 0.0, "bridge node should have positive betweenness, got {}", cb[1]);
        assert_eq!(cb[0], 0.0);
        assert_eq!(cb[2], 0.0);
    }

    #[test]
    fn scc_finds_a_three_cycle_and_isolates_others() {
        // 0 -> 1 -> 2 -> 0 is a 3-cycle (one SCC of size 3).
        // Node 3 is isolated (its own SCC of size 1).
        let mut g = DirectedGraph::new(4);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(2, 0);
        let components = g.strongly_connected_components();
        let sizes: Vec<usize> = {
            let mut s: Vec<usize> = components.iter().map(|c| c.len()).collect();
            s.sort_unstable();
            s
        };
        assert_eq!(sizes, vec![1, 3], "expected one singleton (node 3) and one 3-cycle");
    }

    #[test]
    fn scc_of_size_three_triggers_critical_fraud_alert() {
        let mut g = DirectedGraph::new(4);
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(2, 0);
        // node 3 has no edges -- singleton, must not trigger an alert.
        let uuids = [0xAAAA_0001, 0xAAAA_0002, 0xAAAA_0003, 0xAAAA_0004];
        let alerts = g.scc_alerts(&uuids);
        assert_eq!(alerts.len(), 1, "exactly one SCC of size >= 3 should alert");
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
        assert_eq!(alerts[0].cause, DriftCause::Fraud);
        assert_eq!(alerts[0].drift_distance, 3.0);
    }

    #[test]
    fn scc_below_threshold_does_not_alert() {
        // A 2-cycle: 0 <-> 1. Size 2 < 3, must NOT trigger an alert.
        let mut g = DirectedGraph::new(2);
        g.add_edge(0, 1);
        g.add_edge(1, 0);
        let uuids = [1, 2];
        let alerts = g.scc_alerts(&uuids);
        assert!(alerts.is_empty(), "SCC of size 2 must not trigger the >=3 AML rule");
    }
}
