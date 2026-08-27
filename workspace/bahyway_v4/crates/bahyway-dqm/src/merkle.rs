//! Sovereign Merkle Tree — hash-based lineage integrity verification.
//!
//! Uses FNV-1a (same as bahyway-fabric's lineage hashing) to build a binary
//! Merkle tree over a sequence of data blocks.  The root hash changes if
//! ANY block in the pipeline changes — making tampering detectable instantly.
//!
//! Design: no SHA-256 (external dep) — FNV-1a is sovereign and sufficient for
//! integrity checking within a trusted pipeline.  For cross-system proofs,
//! the root can be signed by kupru's Ed25519 seal.
//!
//! Used by: DqmDimension::Accuracy (lineage integrity), pipeline hop verification

/// FNV-1a 64-bit hash — same as bahyway-fabric's `fnv1a_hash`.
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Combine two child hashes into a parent node hash.
fn combine(left: u64, right: u64) -> u64 {
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&left.to_le_bytes());
    buf[8..].copy_from_slice(&right.to_le_bytes());
    fnv1a(&buf)
}

/// A sovereign Merkle tree built over a sequence of leaf hashes.
///
/// The tree is stored as a flat Vec in level-order (BFS):
///   index 0 = root, indices 1..2 = root's children, etc.
/// Leaves are the last level.
#[derive(Debug, Clone)]
pub struct MerkleTree {
    nodes: Vec<u64>,
    leaf_count: usize,
}

impl MerkleTree {
    /// Build a Merkle tree from a slice of leaf data blocks.
    /// Each block is hashed with FNV-1a before insertion.
    pub fn from_blocks(blocks: &[&[u8]]) -> Self {
        if blocks.is_empty() {
            return MerkleTree {
                nodes: vec![0],
                leaf_count: 0,
            };
        }
        let leaves: Vec<u64> = blocks.iter().map(|b| fnv1a(b)).collect();
        Self::from_leaf_hashes(&leaves)
    }

    /// Build a Merkle tree from pre-computed leaf hashes (e.g. pipeline hop hashes).
    pub fn from_leaf_hashes(hashes: &[u64]) -> Self {
        if hashes.is_empty() {
            return MerkleTree {
                nodes: vec![0],
                leaf_count: 0,
            };
        }

        // Pad to next power of two for a complete binary tree
        let leaf_count = hashes.len();
        let padded = next_power_of_two(leaf_count);
        let mut level: Vec<u64> = hashes.to_vec();
        // Pad with duplicates of the last leaf (standard Merkle padding)
        while level.len() < padded {
            let last = *level.last().unwrap();
            level.push(last);
        }

        // Build bottom-up
        let total_nodes = 2 * padded - 1;
        let mut nodes = vec![0u64; total_nodes];
        let leaf_start = padded - 1;

        // Fill leaves
        for (i, &h) in level.iter().enumerate() {
            nodes[leaf_start + i] = h;
        }

        // Fill internal nodes right-to-left
        let mut i = leaf_start;
        while i > 0 {
            i -= 1;
            nodes[i] = combine(nodes[2 * i + 1], nodes[2 * i + 2]);
        }

        MerkleTree { nodes, leaf_count }
    }

    /// The root hash — a single fingerprint of the entire data sequence.
    /// If any leaf changes, the root changes.
    pub fn root(&self) -> u64 {
        self.nodes[0]
    }

    /// Number of original (un-padded) leaves.
    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Merkle proof path for leaf at index `i`.
    /// Returns sibling hashes from leaf level to root.
    /// The verifier can recompute the root from the leaf hash + this proof.
    pub fn proof(&self, leaf_index: usize) -> Option<Vec<u64>> {
        if leaf_index >= self.leaf_count {
            return None;
        }
        let padded = next_power_of_two(self.leaf_count.max(1));
        let leaf_start = padded - 1;
        let mut pos = leaf_start + leaf_index;
        let mut proof = Vec::new();

        while pos > 0 {
            let sibling = if pos % 2 == 1 { pos + 1 } else { pos - 1 };
            proof.push(self.nodes[sibling]);
            pos = (pos - 1) / 2;
        }
        Some(proof)
    }

    /// Verify that `leaf_hash` at `leaf_index` is consistent with `root`.
    ///
    /// Parity note: leaves are stored at `leaf_start + leaf_index` where
    /// `leaf_start = padded - 1` (always odd).  So a leaf at an even
    /// `leaf_index` sits at an odd tree position (left child) and a leaf
    /// at an odd `leaf_index` sits at an even tree position (right child).
    pub fn verify(leaf_hash: u64, leaf_index: usize, proof: &[u64], root: u64) -> bool {
        let mut current = leaf_hash;
        let mut index = leaf_index;
        for &sibling in proof {
            current = if index.is_multiple_of(2) {
                // even leaf_index → odd tree pos → LEFT child
                combine(current, sibling)
            } else {
                // odd leaf_index → even tree pos → RIGHT child
                combine(sibling, current)
            };
            index /= 2;
        }
        current == root
    }
}

fn next_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut p = 1;
    while p < n {
        p <<= 1;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_block_root_is_its_hash() {
        let data = b"sovereign";
        let tree = MerkleTree::from_blocks(&[data]);
        assert_eq!(tree.root(), fnv1a(data));
    }

    #[test]
    fn empty_tree_has_zero_root() {
        let tree = MerkleTree::from_blocks(&[]);
        assert_eq!(tree.root(), 0);
    }

    #[test]
    fn same_blocks_produce_same_root() {
        let blocks: Vec<&[u8]> = vec![b"A", b"B", b"C", b"D"];
        let t1 = MerkleTree::from_blocks(&blocks);
        let t2 = MerkleTree::from_blocks(&blocks);
        assert_eq!(t1.root(), t2.root());
    }

    #[test]
    fn changed_block_changes_root() {
        let blocks1: Vec<&[u8]> = vec![b"A", b"B", b"C", b"D"];
        let blocks2: Vec<&[u8]> = vec![b"A", b"B", b"X", b"D"];
        let t1 = MerkleTree::from_blocks(&blocks1);
        let t2 = MerkleTree::from_blocks(&blocks2);
        assert_ne!(t1.root(), t2.root());
    }

    #[test]
    fn proof_verifies_correctly() {
        let hashes = vec![10u64, 20, 30, 40];
        let tree = MerkleTree::from_leaf_hashes(&hashes);
        for i in 0..4 {
            let proof = tree.proof(i).unwrap();
            assert!(
                MerkleTree::verify(hashes[i], i, &proof, tree.root()),
                "proof for leaf {i} failed"
            );
        }
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let hashes = vec![10u64, 20, 30, 40];
        let tree = MerkleTree::from_leaf_hashes(&hashes);
        let proof = tree.proof(0).unwrap();
        // Use a wrong hash for leaf 0
        assert!(!MerkleTree::verify(999, 0, &proof, tree.root()));
    }

    #[test]
    fn odd_leaf_count_works() {
        let blocks: Vec<&[u8]> = vec![b"hop1", b"hop2", b"hop3"];
        let tree = MerkleTree::from_blocks(&blocks);
        assert_eq!(tree.leaf_count(), 3);
        // All proofs verify
        for i in 0..3 {
            let h = fnv1a(blocks[i]);
            let proof = tree.proof(i).unwrap();
            assert!(MerkleTree::verify(h, i, &proof, tree.root()));
        }
    }
}
