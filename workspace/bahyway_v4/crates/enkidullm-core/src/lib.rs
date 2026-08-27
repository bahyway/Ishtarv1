//! enkidullm-core — Sovereign book intelligence types.
//!
//! Books are KAKIs. Their metadata is in Orbits. Their history is in the Journal.
//! No assessment lives in the nucleus (§2.4). CrossTribe state is computed (§8.3).
//!
//! Tribe ID namespace for EnkiduLLM: 0x1001–0x10FF (distinct from particle tribes).
//! Book roles reuse the existing KakiRole taxonomy:
//!   KISHIB (0x01) — canonical reference (established, widely cited)
//!   ZIKRU  (0x02) — derived work (survey, commentary, synthesis)
//!   PARZU  (0x03) — practitioner guide (applied, operational)

#![forbid(unsafe_code)]

pub mod book_kaki;
pub mod eav_store;
pub mod orbit;

pub use book_kaki::{book_uuid_hash, BookDomainTribe, BookKaki};
pub use eav_store::{OrbitValue, VerticalEavStore};
pub use orbit::{
    BookArchetype, BookOrbit, ClassificationShell, ContentShell, CoreShell, IduState,
    KnowledgeTriple, ReputationShell,
};
