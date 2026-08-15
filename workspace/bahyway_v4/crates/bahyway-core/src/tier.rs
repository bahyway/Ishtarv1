//! EnkiDB Triple-Tier Architecture — Thermal tiering for the particle lifecycle.
//!
//! In Akkadian cosmology:
//!   APSU    = the primordial freshwater deep  → In-Memory  (Hot tier)
//!   ZAKARU  = "to speak / record"             → Streaming  (Warm tier)
//!   ABZU    = "the Deep Knowledge"            → Warehouse  (Cold tier)
//!
//! Every particle moves through these tiers as it ages:
//!   Birth → Memory (APSU) → Streaming journal (ZAKARU) → Sealed in Warehouse (ABZU)
//!
//! Reference: doc EnkiDB_For_Data_WareHouse.md — "The Three Faces of EnkiDB."

#![forbid(unsafe_code)]

/// Hours after last event before a particle migrates from Memory to Streaming.
pub const HOT_TO_WARM_HOURS: u32 = 1;

/// Hours after last event before a particle migrates from Streaming to Warehouse.
pub const WARM_TO_COLD_HOURS: u32 = 24;

/// The three storage tiers of EnkiDB.
///
/// Tiers are not just labels — they encode the expected access latency and
/// persistence guarantee for a particle at each stage of its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnkiTier {
    /// **APSU** — The Live Heap.
    ///
    /// Hot tier.  Contains particles actively under VGCA cleansing, Big-Ring
    /// rendering, and stakeholder observation.  Lock-free in-memory storage.
    /// Highest throughput; data is not yet durably persisted.
    Memory,

    /// **ZAKARU** — The Heartbeat.
    ///
    /// Warm tier.  Append-only Write-Ahead Log.  All Event-KAKI mutations are
    /// streamed here for HA/DR replication across cluster nodes.
    /// Immutable audit trail; semi-durable (survives node restarts).
    Streaming,

    /// **ABZU** — The Deep Knowledge.
    ///
    /// Cold tier.  Columnar geometric BigTable for 1B+ sealed particles.
    /// Indexed by the 7D Heptagon; partitioned by KAKI sovereign key.
    /// Highest durability; lower throughput; OLAP / forensics queries.
    Warehouse,
}

impl EnkiTier {
    /// Akkadian mythological name for this tier.
    pub fn akkadian_name(&self) -> &'static str {
        match self {
            EnkiTier::Memory    => "APSU",
            EnkiTier::Streaming => "ZAKARU",
            EnkiTier::Warehouse => "ABZU",
        }
    }

    /// Human-readable description of this tier's sovereign role.
    pub fn sovereign_role(&self) -> &'static str {
        match self {
            EnkiTier::Memory    => "Current State of the Universe",
            EnkiTier::Streaming => "Immutable Memory — Audit Trail",
            EnkiTier::Warehouse => "Deep Knowledge — OLAP and Forensics",
        }
    }

    /// Thermal designation matching industry data-lake terminology.
    pub fn thermal_label(&self) -> &'static str {
        match self {
            EnkiTier::Memory    => "Hot",
            EnkiTier::Streaming => "Warm",
            EnkiTier::Warehouse => "Cold",
        }
    }

    /// Returns `true` if this tier supports sub-millisecond read access.
    pub fn is_low_latency(&self) -> bool {
        matches!(self, EnkiTier::Memory)
    }

    /// Returns `true` if this tier guarantees durable persistence.
    pub fn is_durable(&self) -> bool {
        matches!(self, EnkiTier::Streaming | EnkiTier::Warehouse)
    }
}

/// Determine which EnkiDB tier a particle belongs to based on its lifecycle state.
///
/// # Arguments
/// * `is_sealed`              — `true` if the particle has become a Golden Record
///                              (sealed by a Data Steward; no further mutations).
/// * `hours_since_last_event` — hours elapsed since the most recent Event-KAKI
///                              was appended to this particle's orbit.
///
/// # Returns
/// * `Memory`    — particle is active and recent (not yet sealed, < `HOT_TO_WARM_HOURS` hours).
/// * `Streaming` — particle is in the warm journal window.
/// * `Warehouse` — particle is sealed or cold (> `WARM_TO_COLD_HOURS` hours old).
pub fn tier_for_particle(is_sealed: bool, hours_since_last_event: u32) -> EnkiTier {
    if is_sealed {
        return EnkiTier::Warehouse;
    }
    if hours_since_last_event < HOT_TO_WARM_HOURS {
        EnkiTier::Memory
    } else if hours_since_last_event < WARM_TO_COLD_HOURS {
        EnkiTier::Streaming
    } else {
        EnkiTier::Warehouse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_akkadian_names_unique() {
        let names = [
            EnkiTier::Memory.akkadian_name(),
            EnkiTier::Streaming.akkadian_name(),
            EnkiTier::Warehouse.akkadian_name(),
        ];
        // all distinct
        assert_ne!(names[0], names[1]);
        assert_ne!(names[1], names[2]);
        assert_ne!(names[0], names[2]);
    }

    #[test]
    fn test_thermal_labels() {
        assert_eq!(EnkiTier::Memory.thermal_label(),    "Hot");
        assert_eq!(EnkiTier::Streaming.thermal_label(), "Warm");
        assert_eq!(EnkiTier::Warehouse.thermal_label(), "Cold");
    }

    #[test]
    fn test_memory_is_low_latency_only() {
        assert!(EnkiTier::Memory.is_low_latency());
        assert!(!EnkiTier::Streaming.is_low_latency());
        assert!(!EnkiTier::Warehouse.is_low_latency());
    }

    #[test]
    fn test_durable_tiers() {
        assert!(!EnkiTier::Memory.is_durable());
        assert!(EnkiTier::Streaming.is_durable());
        assert!(EnkiTier::Warehouse.is_durable());
    }

    #[test]
    fn test_fresh_active_particle_is_memory() {
        assert_eq!(tier_for_particle(false, 0), EnkiTier::Memory);
    }

    #[test]
    fn test_warm_window_particle() {
        assert_eq!(tier_for_particle(false, HOT_TO_WARM_HOURS), EnkiTier::Streaming);
        assert_eq!(tier_for_particle(false, WARM_TO_COLD_HOURS - 1), EnkiTier::Streaming);
    }

    #[test]
    fn test_cold_particle_at_threshold() {
        assert_eq!(tier_for_particle(false, WARM_TO_COLD_HOURS), EnkiTier::Warehouse);
        assert_eq!(tier_for_particle(false, 720), EnkiTier::Warehouse); // 30 days
    }

    #[test]
    fn test_sealed_particle_always_warehouse() {
        assert_eq!(tier_for_particle(true, 0),   EnkiTier::Warehouse);
        assert_eq!(tier_for_particle(true, 1),   EnkiTier::Warehouse);
        assert_eq!(tier_for_particle(true, 999), EnkiTier::Warehouse);
    }

    #[test]
    fn test_sovereign_roles_non_empty() {
        for tier in [EnkiTier::Memory, EnkiTier::Streaming, EnkiTier::Warehouse] {
            assert!(!tier.sovereign_role().is_empty());
        }
    }
}
