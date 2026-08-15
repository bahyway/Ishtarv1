//! storage-sector — the hardware-isolated terminal jail for confirmed-harmful
//! particles (BlackBox Station output, §12.3b).
//!
//! This is stricter than EnkiQDB: EnkiQDB holds fuzzy/unknown particles
//! awaiting Data Steward review; StorageSector holds particles that
//! BlackBox Station's post-quarantine scan has confirmed as harmful.
//! Nothing sealed here is ever removed, replayed, or requeued — there is
//! no method on this type that could do so.
//!
//! Sovereign name (2026-07-17): this jail is **IRKALLA** —
//! [`SOVEREIGN_NAME`]. Not a crate/type rename: `storage-sector` and
//! `StorageSector` keep their existing identifiers, same relationship
//! "NUZI" has to `bahyway-fabric::LineageLedger`. Before this, "IRKALLA"
//! appeared consistently across the Architect's own DubSar Nergal AV
//! panel work (playbooks 08/14/24/25) only as painted UI text over
//! generic `bee-mdm-bus` counters — a real, repeated name with no coded
//! identity behind it. This const gives it one, so the Nergal panel can
//! read a real sovereign name instead of a bare string literal.
//! Reserved tribe: [`IRKALLA_TRIBE`] = 0x0002, per that same panel work.

pub mod storage_sector;

pub use storage_sector::{StorageSector, SealedParticle, StorageSectorReport};

/// This jail's sovereign name — IRKALLA, per the Architect's naming
/// (2026-07-17, DubSar Nergal AV panel work). See the crate doc comment.
pub const SOVEREIGN_NAME: &str = "irkalla";

/// IRKALLA's reserved tribe ID, as used consistently across the Nergal
/// AV panel playbooks ("ThreatDB Tribe: IRKALLA (0x0002)").
pub const IRKALLA_TRIBE: u16 = 0x0002;

pub mod prelude {
    pub use super::storage_sector::{StorageSector, SealedParticle, StorageSectorReport};
}
