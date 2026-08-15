//! enkidb-journal — The append-only sovereign event log (§3, §6, §9).
//!
//! The Journal is the SOURCE OF TRUTH.  All current state is a PROJECTION
//! computed by the StoryEngine over the Journal.  Nothing is ever deleted
//! or modified.  Event-Kakis accumulate forever, partitioned by tribe + time.

pub mod entry;
pub mod error_registry;
pub mod event_cause;
pub mod journal;
pub mod partition;

pub use entry::{JournalEntry, EavTriple};
pub use error_registry::{ErrorOccurrenceSpec, ErrorSeverity, ErrorTypeSpec};
pub use event_cause::EventCause;
pub use journal::Journal;
pub use partition::PartitionKey;

pub mod prelude {
    pub use super::entry::{JournalEntry, EavTriple};
    pub use super::error_registry::{ErrorOccurrenceSpec, ErrorSeverity, ErrorTypeSpec};
    pub use super::event_cause::EventCause;
    pub use super::journal::Journal;
    pub use super::partition::PartitionKey;
}
