//! HeptaDictionary — the runtime TĒMĒNU registry.
//!
//! Dual-indexed:
//!   catalog: HashMap<u32, DamaAttribute>  — primary lookup by attr_hash  (O(1))
//!   aliases: HashMap<&str, u32>           — secondary lookup by name     (O(1))
//!
//! Loaded at startup with BUILTIN_ATTRS (ILKUM core).
//! Extended at runtime by domain teams via `register_attr()` (SHU-GUR).

use crate::attribute::DamaAttribute;
use crate::builtin::BUILTIN_ATTRS;
use std::collections::HashMap;

/// The TĒMĒNU dictionary — sovereign attribute registry for the EAV Hepta manifold.
pub struct HeptaDictionary {
    /// Primary index: attr_hash → DamaAttribute.
    pub catalog: HashMap<u32, DamaAttribute>,
    /// Alias index: canonical name → attr_hash.
    pub aliases: HashMap<&'static str, u32>,
}

impl HeptaDictionary {
    /// Create a new dictionary pre-loaded with all ILKUM built-in attributes.
    pub fn new() -> Self {
        let mut d = Self {
            catalog: HashMap::new(),
            aliases: HashMap::new(),
        };
        for attr in BUILTIN_ATTRS {
            d.catalog.insert(attr.dama_id, attr.clone());
            d.aliases.insert(attr.name, attr.dama_id);
        }
        d
    }

    /// Resolve a human-readable attribute name to its hash.
    ///
    /// Returns `None` if the name is not registered.
    pub fn resolve_attr(&self, name: &str) -> Option<u32> {
        self.aliases.get(name).copied()
    }

    /// Look up the full `DamaAttribute` by its hash.
    ///
    /// Returns `None` if the hash is not registered.
    pub fn attr_metadata(&self, hash: u32) -> Option<&DamaAttribute> {
        self.catalog.get(&hash)
    }

    /// Register a new SHU-GUR (domain-extension) attribute.
    ///
    /// Returns `true` if newly registered, `false` if already present (Pauli
    /// Exclusion: same dama_id = same law state — reference existing entry).
    pub fn register_attr(&mut self, attr: DamaAttribute) -> bool {
        if self.catalog.contains_key(&attr.dama_id) {
            return false;
        }
        self.aliases.insert(attr.name, attr.dama_id);
        self.catalog.insert(attr.dama_id, attr);
        true
    }

    /// Returns an iterator over all registered attributes.
    pub fn all_attrs(&self) -> impl Iterator<Item = &DamaAttribute> {
        self.catalog.values()
    }

    /// Returns the count of registered attributes (ILKUM + SHU-GUR).
    pub fn len(&self) -> usize {
        self.catalog.len()
    }

    /// Returns `true` if the dictionary is empty (should never happen post-init).
    pub fn is_empty(&self) -> bool {
        self.catalog.is_empty()
    }
}

impl Default for HeptaDictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::AttrDataType;
    use crate::builtin::{ATTR_HASH_MOMENTUM, ATTR_HASH_QUALITY, ATTR_HASH_STATE};

    #[test]
    fn builtin_attrs_are_loaded() {
        let d = HeptaDictionary::new();
        assert!(d.len() >= 8); // 7 universal + PA-13 momentum
    }

    #[test]
    fn resolve_attr_by_name() {
        let d = HeptaDictionary::new();
        assert_eq!(d.resolve_attr("state"), Some(ATTR_HASH_STATE));
        assert_eq!(d.resolve_attr("quality"), Some(ATTR_HASH_QUALITY));
        assert_eq!(d.resolve_attr("momentum"), Some(ATTR_HASH_MOMENTUM));
        assert_eq!(d.resolve_attr("nonexistent"), None);
    }

    #[test]
    fn attr_metadata_by_hash() {
        let d = HeptaDictionary::new();
        let meta = d.attr_metadata(ATTR_HASH_STATE).unwrap();
        assert_eq!(meta.name, "state");
        assert!(meta.mandatory);
    }

    #[test]
    fn pauli_exclusion_prevents_double_registration() {
        let mut d = HeptaDictionary::new();
        use crate::attribute::DamaAttribute;
        let duplicate = DamaAttribute::new(
            ATTR_HASH_STATE, // same hash as "state"
            "state_alias",
            AttrDataType::Text,
            false,
            "This should be rejected",
        );
        let registered = d.register_attr(duplicate);
        assert!(
            !registered,
            "Pauli Exclusion: same dama_id must be rejected"
        );
    }

    #[test]
    fn shu_gur_extension_is_registered() {
        let mut d = HeptaDictionary::new();
        let before = d.len();
        let custom = DamaAttribute::new(
            0xBEEF,
            "domain_score",
            AttrDataType::Float64,
            false,
            "Domain-specific score",
        );
        assert!(d.register_attr(custom));
        assert_eq!(d.len(), before + 1);
        assert_eq!(d.resolve_attr("domain_score"), Some(0xBEEF));
    }

    #[test]
    fn momentum_is_non_mandatory_derived() {
        let d = HeptaDictionary::new();
        let meta = d.attr_metadata(ATTR_HASH_MOMENTUM).unwrap();
        assert!(
            !meta.mandatory,
            "PA-13 momentum is derived, not mandatory at birth"
        );
        assert_eq!(meta.data_type, AttrDataType::Float64);
    }
}
