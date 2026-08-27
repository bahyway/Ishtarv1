//! PatternProfile — a discovered Pattern, ready to be minted as a real
//! EnkiMDB particle.
//!
//! Replaces `enki-pattern::PatternRegistry`'s own separate Hot/Warm/
//! Cold/Crystallized store (2026-07-31): under the Architect's law ("no
//! particle stays or evaluates outside the 7 Types EnkiDB"), a
//! discovered pattern's home is EnkiMDB, with tier as a real EAV
//! attribute (`pattern.tier`) computed from age via
//! `enki_pattern::tier::StorageTier::from_age` — that pure computation
//! is reused as-is; only the SEPARATE STORE it used to live in is
//! replaced.

use enki_pattern::tier::StorageTier;
use nisaba::discovery::DiscoveredPattern;

/// A discovered pattern plus the age (in BASE orbitals) needed to
/// compute its storage tier at mint time.
#[derive(Debug, Clone)]
pub struct PatternProfile {
    pub pattern: DiscoveredPattern,
    /// Age in BASE orbitals at the moment this profile is minted —
    /// determines the initial `pattern.tier` EAV value. Callers
    /// re-minting an amendment (see ADR-016's Model-as-Particle law)
    /// supply the pattern's current age, not its age at first discovery.
    pub age_orbitals: u64,
}

impl PatternProfile {
    pub fn new(pattern: DiscoveredPattern, age_orbitals: u64) -> Self {
        PatternProfile {
            pattern,
            age_orbitals,
        }
    }

    pub fn tier(&self) -> StorageTier {
        StorageTier::from_age(self.age_orbitals)
    }
}
