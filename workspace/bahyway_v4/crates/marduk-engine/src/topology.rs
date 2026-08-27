//! Topology verb (BC-MRD-001 §0). Scoped exactly as BC-MRD-001 and
//! GL-MRD-002 (Nēberu Slicer) define it: "Betti β0/β1 of relation
//! graphs" — not a full simplicial-complex homology engine. For a
//! plain graph (1-dimensional CW complex) both numbers are elementary:
//!
//!   β0 = number of connected components
//!   β1 = |E| - |V| + β0   (the graph's circuit/cycle rank)
//!
//! β0 answers "how many separate clusters"; β1 answers "how many
//! independent loops" — a mule ring (Šazu, BC-MRD-001 §4.1) or an
//! organized illegal-connection ring is literally β1 > 0 on the
//! relevant relation graph. Zero implementation of this existed
//! anywhere in the workspace before this file (checked directly: no
//! hit for "Betti" in any crate).

/// An undirected relation graph: `node_count` nodes, indexed 0..node_count,
/// connected by `edges` (unordered pairs; self-loops and parallel edges
/// are accepted and counted — they only ever raise β1, never β0).
pub struct RelationGraph {
    pub node_count: usize,
    pub edges: Vec<(usize, usize)>,
}

/// Union-find (disjoint set) with path compression + union by rank —
/// the standard O(E α(V)) way to count connected components.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
    }
}

/// β0: the number of connected components.
pub fn betti_0(g: &RelationGraph) -> usize {
    if g.node_count == 0 {
        return 0;
    }
    let mut uf = UnionFind::new(g.node_count);
    for &(a, b) in &g.edges {
        uf.union(a, b);
    }
    let mut roots: Vec<usize> = (0..g.node_count).map(|i| uf.find(i)).collect();
    roots.sort_unstable();
    roots.dedup();
    roots.len()
}

/// β1: the circuit rank, |E| - |V| + β0. Always >= 0 for a graph with
/// edges only between valid node indices (guaranteed here since
/// `betti_0`'s union-find only ever unions indices < node_count).
pub fn betti_1(g: &RelationGraph) -> usize {
    let b0 = betti_0(g) as isize;
    let e = g.edges.len() as isize;
    let v = g.node_count as isize;
    let b1 = e - v + b0;
    debug_assert!(
        b1 >= 0,
        "circuit rank computed negative -- invalid graph (edge index out of range?)"
    );
    b1.max(0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_has_no_components_and_no_cycles() {
        let g = RelationGraph {
            node_count: 0,
            edges: vec![],
        };
        assert_eq!(betti_0(&g), 0);
        assert_eq!(betti_1(&g), 0);
    }

    #[test]
    fn isolated_nodes_are_each_their_own_component() {
        let g = RelationGraph {
            node_count: 5,
            edges: vec![],
        };
        assert_eq!(betti_0(&g), 5);
        assert_eq!(betti_1(&g), 0);
    }

    #[test]
    fn a_tree_has_one_component_and_no_cycles() {
        // 0-1-2-3, a path (a tree): 4 nodes, 3 edges.
        let g = RelationGraph {
            node_count: 4,
            edges: vec![(0, 1), (1, 2), (2, 3)],
        };
        assert_eq!(betti_0(&g), 1);
        assert_eq!(betti_1(&g), 0);
    }

    #[test]
    fn a_ring_has_one_component_and_one_independent_loop() {
        // 0-1-2-3-0, a 4-cycle: 4 nodes, 4 edges.
        let g = RelationGraph {
            node_count: 4,
            edges: vec![(0, 1), (1, 2), (2, 3), (3, 0)],
        };
        assert_eq!(betti_0(&g), 1);
        assert_eq!(betti_1(&g), 1);
    }

    #[test]
    fn two_components_one_tree_one_ring() {
        // Component A: 0-1 (tree, 2 nodes 1 edge). Component B: 2-3-4-2 (a
        // triangle, 3 nodes 3 edges, one independent loop).
        let g = RelationGraph {
            node_count: 5,
            edges: vec![(0, 1), (2, 3), (3, 4), (4, 2)],
        };
        assert_eq!(betti_0(&g), 2);
        assert_eq!(betti_1(&g), 1);
    }

    #[test]
    fn mule_ring_signature_two_independent_loops() {
        // A "mule ring" analog: two overlapping cycles sharing one node
        // (a figure-eight) -- 5 nodes, 6 edges, one component, two
        // independent loops.
        let g = RelationGraph {
            node_count: 5,
            edges: vec![(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)],
        };
        assert_eq!(betti_0(&g), 1);
        assert_eq!(betti_1(&g), 2);
    }
}
