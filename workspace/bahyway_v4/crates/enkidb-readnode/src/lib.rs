//! enkidb-readnode — ADR-012 Read Node: HeptaScript queries served from
//! `enkidb-datafile` Data Files instead of a replayed-into-memory Journal.
//!
//! [`materialize::materialize`] pays the O(n) "read every particle's
//! history once" cost that any index build already pays — the difference
//! is the result is *persisted* to disk (an entity-history store + an EAV
//! posting index), so [`readnode::ReadNode::open`] never re-pays it: it's
//! two file opens and two stat calls, not a journal replay. A query then
//! touches only the entities it actually matches, reconstructing just
//! their history on the fly and handing it to `heptascript::execute_over`
//! unchanged — so a Read Node answer can never disagree with a full scan.
//!
//! Scope: current-state, single-exact-equality WHERE queries only (see
//! `readnode` module docs). Historical or non-indexable queries return
//! `ReadNodeError::RequiresWriteNode` rather than a wrong answer.
#![forbid(unsafe_code)]

pub mod cached;
pub mod generation;
pub mod materialize;
pub mod readnode;

pub use cached::{CachedReadNode, CachedReadNodeError};
pub use generation::{list_generations, materialize_generation, Generation};
pub use materialize::{materialize, open_tribe_summaries, tribe_summary_paths, MaterializeStats, TRIBE_SUMMARY_KIND};
pub use readnode::{ReadNode, ReadNodeError};
