//! enkidb-storage — Native storage primitives: append-only file writer,
//! memory-mapped readers, and fsync durability barriers (§11.2).

pub mod append_writer;
pub mod mmap_reader;
pub mod durability;

pub use append_writer::{AppendWriter, COMMIT_MARKER};
pub use mmap_reader::MmapReader;
pub use durability::FsyncPolicy;

pub mod prelude {
    pub use super::append_writer::AppendWriter;
    pub use super::mmap_reader::MmapReader;
    pub use super::durability::FsyncPolicy;
}
