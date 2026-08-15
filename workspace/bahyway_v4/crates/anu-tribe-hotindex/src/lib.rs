// ============================================================================
// Anu Tribe HotIndex — the hot resolution projection of the NUZI registry.
// BahyWay.Ecosystem v4.0
//
// From the iPhone-session PB-152 review ("ENLIL Tribe HotIndex v4.0") --
// renamed ENLIL -> Anu because ENLIL already names something else real and
// unrelated in this repo (the stability algebra in
// bahyway-algebra::enlil.rs), and because this repo's own index stack was
// already deliberately renamed away from ENLIL to the Anu Index Stack for
// exactly this reason (see docs/components/ANU_INDEX_STACK.md).
//
// NUZI stays the genealogical truth; this crate is its cache-resident
// shadow, rebuilt from each verified snapshot epoch. NUZI itself has no
// real backing crate anywhere in this workspace yet -- this crate does not
// need one to compile or test: it takes a plain resolution table
// (`&[(u16, u64)]`) as input, so it is self-contained today and can be
// wired to a real NUZI registry later without changing its own API.
//
// LAWS:
//   LAW-HOT-1  Single lookup is O(1) direct indexing -- table[tribe_id].
//              The full u16 domain is materialized: 64 KB validity plane
//              (L1/L2-resident) + 512 KB node plane (L2-resident). No
//              search exists here.
//   LAW-HOT-2  Batch validation is the 1-billion-under-1-second path:
//              branchless chunked loops shaped for LLVM autovectorization,
//              std::thread fan-out across cores. No unsafe, no deps, no
//              async runtime.
//   LAW-HOT-3  The index carries the registry epoch + seal of the snapshot
//              it projects; a judgment cites the epoch, never "the cache".
// ============================================================================
#![forbid(unsafe_code)]

pub mod batch;
pub mod hot_table;

pub use batch::{validate_batch, validate_batch_parallel, BatchReport};
pub use hot_table::HotIndex;
