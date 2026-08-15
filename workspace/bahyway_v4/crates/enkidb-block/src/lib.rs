//! enkidb-block — 64MB block-aligned cold storage format (KAKI_v4.0.pdf §9.3)
//!
//! Block layout:
//!   HEADER (256 bytes): magic b"ENKIv4.0", tribe_id, block_size, checksum_algorithm
//!   BLOCK 0: KAKI Nuclei (dense, 16 bytes each)
//!   BLOCK 1..N: EAV Orbit Chunks (zstd-compressed, with per-chunk header)
//!   BLOCK JOURNAL: Append-only mutation log

pub mod header;
pub mod kaki_block;
pub mod orbit_chunk;

pub use header::BlockHeader;
pub use kaki_block::KakiNucleusBlock;
pub use orbit_chunk::{OrbitChunkHeader, OrbitChunk};

/// 64 MB — the canonical block size from §9.3.
pub const BLOCK_SIZE_BYTES: u32 = 64 * 1024 * 1024;

/// Magic bytes identifying a BahyWay v4.0 cold storage block.
pub const BLOCK_MAGIC: &[u8; 8] = b"ENKIv4.0";

/// Checksum algorithm tag written into the block header.
pub const CHECKSUM_ALGO_CRC16_CCITT: u8 = 0x01;

pub mod prelude {
    pub use super::header::BlockHeader;
    pub use super::{BLOCK_MAGIC, BLOCK_SIZE_BYTES};
}
