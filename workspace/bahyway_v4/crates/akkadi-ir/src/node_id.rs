//! 𒁾 NodeId — Sovereign Identity for Every IR Node
//!
//! Derived deterministically via FNV-1a — same content → same NodeId.
//! Mirrors the KAKI principle: identity comes from content, not auto-increment.

/// Sovereign identity for an IR node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u64);

impl NodeId {
    pub const fn from_raw(v: u64) -> Self { Self(v) }

    /// FNV-1a hash of a content string.
    pub fn from_content(content: &str) -> Self {
        Self(fnv1a_64(content.as_bytes()))
    }

    pub fn from_parts(parts: &[&str]) -> Self {
        let combined: String = parts.join("·");
        Self::from_content(&combined)
    }

    pub fn raw(&self) -> u64 { self.0 }

    pub fn short(&self) -> String { format!("{:016X}", self.0)[..8].to_string() }
    pub fn hex(&self)   -> String { format!("{:016X}", self.0) }

    pub const NULL: NodeId = NodeId(0);
    pub fn is_null(&self) -> bool { self.0 == 0 }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "𒁾{}", self.short())
    }
}

const FNV_OFFSET: u64 = 14695981039346656037;
const FNV_PRIME:  u64 = 1099511628211;

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Builder for constructing a NodeId incrementally.
#[derive(Default)]
pub struct NodeIdBuilder {
    parts: Vec<String>,
}

impl NodeIdBuilder {
    pub fn new() -> Self { Self::default() }
    pub fn add(mut self, part: &str) -> Self { self.parts.push(part.to_string()); self }
    pub fn node_type(self, kind: &str)  -> Self { self.add(kind) }
    pub fn name(self, name: &str)       -> Self { self.add(name) }
    pub fn version(self, ver: &str)     -> Self { self.add(ver)  }

    pub fn build(self) -> NodeId {
        let refs: Vec<&str> = self.parts.iter().map(|s| s.as_str()).collect();
        NodeId::from_parts(&refs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_gives_same_id() {
        assert_eq!(NodeId::from_content("PARTICLE·citizen"), NodeId::from_content("PARTICLE·citizen"));
    }

    #[test]
    fn different_content_gives_different_id() {
        assert_ne!(NodeId::from_content("PARTICLE·citizen"), NodeId::from_content("TRIBE·IraqiMDM"));
    }

    #[test]
    fn null_is_zero() { assert!(NodeId::NULL.is_null()); }

    #[test]
    fn short_is_8_chars() { assert_eq!(NodeId::from_content("test").short().len(), 8); }

    #[test]
    fn hex_is_16_chars()  { assert_eq!(NodeId::from_content("test").hex().len(),  16); }

    #[test]
    fn builder_matches_from_parts() {
        let a = NodeId::from_parts(&["PARTICLE", "citizen"]);
        let b = NodeIdBuilder::new().node_type("PARTICLE").name("citizen").build();
        assert_eq!(a, b);
    }

    #[test]
    fn display_contains_cuneiform() { assert!(NodeId::from_content("x").to_string().contains('𒁾')); }

    #[test]
    fn empty_string_is_fnv_offset() { assert_eq!(NodeId::from_content("").raw(), FNV_OFFSET); }
}
