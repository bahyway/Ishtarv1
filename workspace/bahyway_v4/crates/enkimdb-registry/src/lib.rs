//! enkimdb-registry — EnkiMDB's Internal Release Profile registry.
//!
//! From the iPhone-session "All Engines Are Particles" discussion: every
//! sovereign engine, crate, app, and script is itself an artifact worth
//! tracking, queryable by release. That discussion's own playbook_96
//! design (ArtifactKind, ReleaseProfile, EnkiMdbRegistry) is landed here,
//! grounded concretely: `scan_workspace` reads the REAL `Cargo.toml` files
//! under this workspace's `crates/` tree, so the registry it builds always
//! reflects what's actually here, never a synthetic or aspirational list.
//!
//! HONEST SCOPE: this crate produces the registry data model and the
//! real-filesystem scanner. It does NOT (yet) mint KAKI particles or write
//! into a live EnkiMDB tribe -- that wiring (kaki_type 0x01 Identity KAKI
//! per artifact, tribe 0xFF00, EAV release.* attributes) is real,
//! well-specified, later work, deliberately not guessed at here.

pub mod registry;
pub mod scan;

pub use registry::{ArtifactKind, EnkiMdbRegistry, ReleaseProfile};
pub use scan::scan_workspace;

/// Reserved internal tribes, per the reviewed design. High tribe IDs,
/// deliberately outside any real data-tribe range (e.g. 0x0001
/// NAJAF_CEMETERY) so they can never collide with a real ingested tribe.
pub const TRIBE_BAHYWAY_INTERNAL: u16 = 0xFF00;
pub const TRIBE_BAHYWAY_TEMPLATES: u16 = 0xFFFF;
pub const TRIBE_BAHYWAY_SECURITY: u16 = 0xFF01;
pub const TRIBE_BAHYWAY_RELEASES: u16 = 0xFF02;
