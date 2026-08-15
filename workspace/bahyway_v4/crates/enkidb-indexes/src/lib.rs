//! enkidb-indexes — the native EnkiDB indexes (§9.3).
//!
//! Sovereign name (2026-08-15, second rename): this index stack is
//! **Elu** (elû, Akkadian "high, upper, superior") — [`SOVEREIGN_NAME`].
//! Not a crate/type rename: `enkidb-indexes` and every type below are
//! unchanged identifiers -- same relationship "Tigris" has to `enkiddb`
//! and "Euphrates" has to `enkimdb`. Previously named "Anu" (2026-07-13);
//! the Architect reclaimed "Anu" for the AnuGovernor (formerly
//! Shakkanakku) rename and chose "Elu" here to free it without another
//! collision. Per NL-001's Orthography Clause (§6a), the plain-Latin
//! form "Elu" is used in every identifier; the diacritic form "elû" is
//! reserved for prose/documentation, same convention as Hala/naṣāru.
//! See `docs/05_storage/ELU_INDEX_STACK.md` for the full per-index W5H2
//! reference this rename accompanies. The original "ENLIL" rejection
//! reasoning (that name already spent on the Total Algebra Content,
//! GeoLaw-05's Jordan Normal Form / orbit stability) still applies and
//! is preserved there.
//!
//! All indexes are built natively in Rust from primitives.
//! NO TREE IN THE ORBITS: no LSM-tree, no B-tree, no R-tree, anywhere in this
//! crate -- confirmed by audit 2026-07-13 (see PB-182). Every index is either
//! direct-array-indexed, hash-indexed, or a flat sorted/bitmap structure.
//! This table replaces the two never-reconciled numbering schemes that used
//! to live here separately (an original "Index 1-7" concept list and a later
//! "W5H2 Performance Optimisations" addendum) with one list matching the
//! actual modules below.
//!
//! Module         Dimension              Native Structure              Source
//! ───────────────────────────────────────────────────────────────────────────
//! surrogate      KAKI ↔ u32             2-level MPHF (PTHash style)   all KAKIs
//! identity       D1 uuid_hash           hash map                     κ[0..3]
//! sovereignty    D2 tribe_id            sorted u32 per tribe          κ[4..5]
//! typerole       KAKI taxonomy          3×3 array of uuid lists       κ[6..7]
//! temporal       D6 timestamp           direct-indexed array          κ[12..13]
//!                (bounded u16 domain -- 65,536 buckets, never grows with
//!                particle count; was BTreeMap, replaced 2026-07-13, PB-182)
//! colorid        D3/D4/D5+radial        hash map (simplified          color_rgb EAV
//!                bounding-box scan for v4.0; a real spatial structure,
//!                if ever needed, is future work -- not a tree today)
//! eav            tribe attributes       inverted + bitmaps            optional EAV
//! eav_exact_index attr+value exact      HashMap + RoaringBitmap       optional EAV
//! radix_spline   epoch/quality range    learned index (12GB -> 2KB,   epoch/quality
//!                replaces the retired BTreeRange)
//! hepta_shell    7D spatial (E7)        zone table, 126 neighbours    orbital (r,θ,φ)
//! nairu_index    state+gate pruning     RoaringBitmap                 orbital state
//! snapshot_idx   latest-snapshot query  HashMap<uuid_hash,            snapshot EAV
//!                sorted Vec<(epoch,entry)>>, binary search
//!                (was BTreeMap<uuid_hash, BTreeMap<epoch,_>> -- an outer
//!                tree with one node PER PARTICLE, the same failure shape
//!                that made BTreeRange cost 12GB at 1B particles; replaced
//!                2026-07-13, PB-182)
//! bloom          probabilistic membership BloomFilter bitset          all KAKIs

/// This index stack's sovereign name — Elu, per the Architect's naming
/// (2026-08-15, second rename; previously "Anu," 2026-07-13). See the
/// crate doc comment for the full provenance of both renames.
pub const SOVEREIGN_NAME: &str = "elu";

pub mod identity;
pub mod sovereignty;
pub mod typerole;
pub mod temporal;
pub mod colorid;
pub mod eav;
pub mod snapshot_idx;
pub mod bloom;
pub mod surrogate;
pub mod radix_spline;
pub mod hepta_shell;
pub mod nairu_index;
pub mod eav_exact_index;

/// Vector ID kind for index registration in Default.Indexes.
pub use enkidb_vector_id::VectorId as IndexVectorId;

pub use bloom::BloomFilter;

pub mod prelude {
    pub use super::identity::IdentityIndex;
    pub use super::sovereignty::SovereigntyIndex;
    pub use super::typerole::TypeRoleIndex;
    pub use super::temporal::TemporalIndex;
    pub use super::colorid::ColorIdIndex;
    pub use super::eav::EavIndex;
    pub use super::snapshot_idx::SnapshotIndex;
    pub use super::bloom::BloomFilter;
    pub use super::surrogate::SurrogateMap;
    pub use super::radix_spline::RadixSplineIndex;
    pub use super::hepta_shell::HeptaShellIndex;
    pub use super::nairu_index::{NatiruIndex, BUCKET_ORBITALS};
    pub use super::eav_exact_index::EavExactIndex;
}
