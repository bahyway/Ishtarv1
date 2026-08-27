//! enkidb-storage — Native storage primitives: append-only file writer,
//! memory-mapped readers, and fsync durability barriers (§11.2).

pub mod append_writer;
pub mod durability;
pub mod mmap_reader;

pub use append_writer::{AppendWriter, COMMIT_MARKER};
pub use durability::FsyncPolicy;
pub use mmap_reader::MmapReader;

pub mod prelude {
    pub use super::append_writer::AppendWriter;
    pub use super::durability::FsyncPolicy;
    pub use super::mmap_reader::MmapReader;
}
