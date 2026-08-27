#![forbid(unsafe_code)]
//! Storage tier classification for patterns in ENKI-PATTERN.

use enkidb_kaki::{Kaki, PatternLifecycle, PatternType};

/// The four storage tiers of the ENKI-PATTERN registry.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum StorageTier {
    /// Hot: 0–3 orbitals old. Full 7D data, sub-millisecond access.
    Hot = 0,
    /// Warm: 3–30 orbitals. Reduced semantic dimensions cached.
    Warm = 1,
    /// Cold: 30–365 orbitals. Aggregate statistics only in memory.
    Cold = 2,
    /// Crystallized: 365+ orbitals. Blob-only, archived to long-term storage.
    Crystallized = 3,
}

impl StorageTier {
    /// Compute the tier from the pattern's age in BASE orbitals.
    pub fn from_age(age_orbitals: u64) -> Self {
        match age_orbitals {
            0..=2 => StorageTier::Hot,
            3..=29 => StorageTier::Warm,
            30..=364 => StorageTier::Cold,
            _ => StorageTier::Crystallized,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            StorageTier::Hot => "Hot",
            StorageTier::Warm => "Warm",
            StorageTier::Cold => "Cold",
            StorageTier::Crystallized => "Crystallized",
        }
    }

    /// Expected access latency description for this tier.
    pub fn latency_hint(self) -> &'static str {
        match self {
            StorageTier::Hot => "<1ms",
            StorageTier::Warm => "<10ms",
            StorageTier::Cold => "<1s",
            StorageTier::Crystallized => "minutes",
        }
    }
}

impl core::fmt::Display for StorageTier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A full pattern record stored in the registry.
#[derive(Clone, Debug)]
pub struct PatternRecord {
    pub pattern_kaki: Kaki,
    pub pattern_type: PatternType,
    pub lifecycle: PatternLifecycle,
    pub confidence_millipercent: u16,
    pub constituent_count: u32,
    pub constituent_merkle_root: [u8; 32],
    /// BASE orbital when this pattern was first published.
    pub orbital_published: u64,
    /// BASE orbital of the current registry read (used to compute tier).
    pub orbital_now: u64,
    /// Nash equilibrium score at last evaluation (0.0=stable, 1.0=breaking).
    pub nash_equilibrium_score: f64,
}

impl PatternRecord {
    pub fn age_orbitals(&self) -> u64 {
        self.orbital_now.saturating_sub(self.orbital_published)
    }

    pub fn storage_tier(&self) -> StorageTier {
        StorageTier::from_age(self.age_orbitals())
    }

    pub fn is_canonical(&self) -> bool {
        self.lifecycle == PatternLifecycle::Canonical
    }

    pub fn is_deprecated(&self) -> bool {
        self.lifecycle == PatternLifecycle::Deprecated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_from_age() {
        assert_eq!(StorageTier::from_age(0), StorageTier::Hot);
        assert_eq!(StorageTier::from_age(2), StorageTier::Hot);
        assert_eq!(StorageTier::from_age(3), StorageTier::Warm);
        assert_eq!(StorageTier::from_age(29), StorageTier::Warm);
        assert_eq!(StorageTier::from_age(30), StorageTier::Cold);
        assert_eq!(StorageTier::from_age(364), StorageTier::Cold);
        assert_eq!(StorageTier::from_age(365), StorageTier::Crystallized);
    }

    #[test]
    fn tier_ordering() {
        assert!(StorageTier::Hot < StorageTier::Warm);
        assert!(StorageTier::Warm < StorageTier::Cold);
        assert!(StorageTier::Cold < StorageTier::Crystallized);
    }
}
