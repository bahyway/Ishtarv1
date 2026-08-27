//! enkimdb — EnkiMDB: BahyWay.Ecosystem v4.0 Metadata Database.
//!
//! Catalogs BahyWay's *own* artifacts (workspace crates, playbooks) as
//! sovereign Identity-Kakis, real `enkidb_particles::Particle` EAV rows
//! namespaced under `artifact.*`. Every entry comes from an actual scan
//! of the filesystem at call time — no bootstrap/demo data.
//!
//! Callers choose the tribe id (recommend reserving one distinct from
//! any tribe used for ingested/external content — e.g. a dedicated
//! "BahyWay Internal" tribe — but this crate does not hardcode one; see
//! `enkidb_kaki::KakiMinter::new`).
//!
//! CQRS split (added 2026-07-12), identical shape to `enkiddb`'s: a real
//! `enkidb_journal::Journal` WAL in [`writenode::WriteNode`], and the
//! existing `enkidb-readnode`/`enkidb-datafile` machinery (ADR-012) in
//! [`readnode`] for the read path.
//!
//! Sovereign name (2026-07-12): EnkiMDB is **Euphrates** —
//! [`readnode::SOVEREIGN_NAME`]. [`readnode::materialize_version`] +
//! [`readnode::list_versions`] give every materialization a version tag
//! ("Euphrates v4.1", "Euphrates v4.2", ...) so DubSar Theater can list,
//! open, and compare multiple metadata-catalog generations side by side.
#![forbid(unsafe_code)]

pub mod artifact;
pub mod emitter;
pub mod passport_record;
pub mod pattern;
pub mod pattern_emitter;
pub mod pb;
pub mod pb_emitter;
pub mod readnode;
pub mod registry_emitter;
pub mod run_record;
pub mod scan;
pub mod tablet;
pub mod tablet_emitter;
pub mod writenode;

pub use artifact::{ArtifactKind, ArtifactProfile, ARTIFACT_TRIBE_ID};
pub use emitter::ArtifactEmitter;
pub use passport_record::PassportRecordSpec;
pub use pattern::PatternProfile;
pub use pattern_emitter::PatternEmitter;
pub use pb::{append_untriaged_rows, scan_pbs, update_triage_status, PbProfile};
pub use pb_emitter::PbEmitter;
pub use readnode::{
    list_versions, materialize_now, materialize_version, CachedReadNode, CachedReadNodeError,
    Generation, ReadNode, ReadNodeError, SOVEREIGN_NAME,
};
pub use registry_emitter::RegistryEmitter;
pub use run_record::AnuGovernorRunRecordSpec;
pub use scan::{scan_crates, scan_playbooks};
pub use tablet::{profile_tablet, TabletKind, TabletProfile};
pub use tablet_emitter::TabletEmitter;
pub use writenode::WriteNode;
