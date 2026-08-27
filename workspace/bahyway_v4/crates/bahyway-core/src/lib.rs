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
    CRATE_EBBERU, CRATE_HEPTASCRIPT, CRATE_ISDU, CRATE_ISHTAR, CRATE_KAKKABU, CRATE_KUPRU,
    CRATE_LAMASSU, CRATE_NABU, CRATE_QUPPU, CRATE_SHEDU, CRATE_TEMU, CRATE_WAY, CRATE_ZAKARU,
    ECOSYSTEM_CRATES, EXT_AKK, EXT_AKKNB, EXT_HEPTA, EXT_WAY,
};
pub use death_legacy::{
    DeathDisposition, DeathState, InheritanceChannel, LegacyResidue, LegacySettlement, Resolution,
};
pub use error::{BahywayError, Result};
pub use event::{validate_a3, A3Violation, Event, EventKind};
pub use grid::{
    canonical_grid, cell_byte_constraints, cell_color, project, CellColor, CellHealthSnapshot,
    GridCell, GridOccupancySnapshot, HealthAlert, KakiByteConstraints, AMBER_HEALTH_THRESHOLD,
    COLLAPSE_HEALTH_THRESHOLD, CRITICAL_B11_THRESHOLD, GLASS_HEALTH_THRESHOLD, QUALITY_DIVISOR,
};
pub use hepta_gate::HeptaGate;
pub use journal_tx::{JournalError, JournalTier, JournalTransaction, NoopTransaction};
pub use lane::{compose_lanes, Lane};
pub use mandatory_attrs::{
    MandatoryAttr, ATTR_COLOR_RGB, ATTR_FRESHNESS, ATTR_STATE, ATTR_SYSTEMUSER, ATTR_USERGROUP,
    ATTR_USERNAME, ATTR_VELOCITY,
};
pub use particle_state::{compose_link_state, LinkState, ParticleState};
pub use tier::{tier_for_particle, EnkiTier, HOT_TO_WARM_HOURS, WARM_TO_COLD_HOURS};
pub use tribe::TribeId;

/// Convenience re-exports for crates that `use bahyway_core::prelude::*`.
pub mod prelude {
    pub use super::akkadian::{
        CRATE_LAMASSU, CRATE_NABU, CRATE_ZAKARU, EXT_AKK, EXT_AKKNB, EXT_HEPTA, EXT_WAY,
    };
    pub use super::error::{BahywayError, Result};
    pub use super::event::{validate_a3, Event, EventKind};
    pub use super::grid::{
        canonical_grid, cell_byte_constraints, cell_color, project, CellColor, CellHealthSnapshot,
        GridCell, GridOccupancySnapshot, HealthAlert, KakiByteConstraints, QUALITY_DIVISOR,
    };
    pub use super::hepta_gate::HeptaGate;
    pub use super::journal_tx::{JournalError, JournalTier, JournalTransaction, NoopTransaction};
    pub use super::lane::{compose_lanes, Lane};
    pub use super::particle_state::{compose_link_state, LinkState, ParticleState};
    pub use super::tier::{tier_for_particle, EnkiTier};
    pub use super::tribe::TribeId;
}
