//! bahyway-core — Common types, errors, and traits for BahyWay.Ecosystem v4.0

pub mod akkadian;
pub mod death_legacy;
pub mod error;
pub mod event;
pub mod grid;
pub mod hepta_gate;
pub mod journal_tx;
pub mod lane;
pub mod mandatory_attrs;
pub mod particle_state;
pub mod tier;
pub mod tribe;

pub use akkadian::{
    CRATE_NABU, CRATE_ZAKARU, CRATE_LAMASSU, CRATE_ISHTAR, CRATE_ISDU,
    CRATE_EBBERU, CRATE_KAKKABU, CRATE_TEMU, CRATE_SHEDU, CRATE_QUPPU,
    CRATE_KUPRU, CRATE_HEPTASCRIPT, CRATE_WAY,
    EXT_AKK, EXT_HEPTA, EXT_WAY, EXT_AKKNB, ECOSYSTEM_CRATES,
};
pub use death_legacy::{
    DeathDisposition, DeathState, InheritanceChannel, LegacyResidue, LegacySettlement, Resolution,
};
pub use error::{BahywayError, Result};
pub use event::{Event, EventKind, A3Violation, validate_a3};
pub use grid::{
    GridCell, project, canonical_grid, QUALITY_DIVISOR,
    CellColor, cell_color, CRITICAL_B11_THRESHOLD,
    GLASS_HEALTH_THRESHOLD, AMBER_HEALTH_THRESHOLD, COLLAPSE_HEALTH_THRESHOLD,
    KakiByteConstraints, cell_byte_constraints,
    GridOccupancySnapshot, CellHealthSnapshot, HealthAlert,
};
pub use hepta_gate::HeptaGate;
pub use journal_tx::{JournalTransaction, JournalError, JournalTier, NoopTransaction};
pub use lane::{Lane, compose_lanes};
pub use mandatory_attrs::{
    MandatoryAttr, ATTR_STATE, ATTR_COLOR_RGB, ATTR_USERNAME, ATTR_FRESHNESS,
    ATTR_VELOCITY, ATTR_SYSTEMUSER, ATTR_USERGROUP,
};
pub use particle_state::{ParticleState, LinkState, compose_link_state};
pub use tier::{EnkiTier, tier_for_particle, HOT_TO_WARM_HOURS, WARM_TO_COLD_HOURS};
pub use tribe::TribeId;

/// Convenience re-exports for crates that `use bahyway_core::prelude::*`.
pub mod prelude {
    pub use super::akkadian::{
        CRATE_NABU, CRATE_ZAKARU, CRATE_LAMASSU, EXT_AKK, EXT_HEPTA, EXT_WAY, EXT_AKKNB,
    };
    pub use super::error::{BahywayError, Result};
    pub use super::event::{Event, EventKind, validate_a3};
    pub use super::grid::{
        GridCell, project, canonical_grid, QUALITY_DIVISOR,
        CellColor, cell_color, KakiByteConstraints, cell_byte_constraints,
        GridOccupancySnapshot, CellHealthSnapshot, HealthAlert,
    };
    pub use super::hepta_gate::HeptaGate;
    pub use super::journal_tx::{JournalTransaction, JournalError, JournalTier, NoopTransaction};
    pub use super::lane::{Lane, compose_lanes};
    pub use super::particle_state::{ParticleState, LinkState, compose_link_state};
    pub use super::tier::{EnkiTier, tier_for_particle};
    pub use super::tribe::TribeId;
}
