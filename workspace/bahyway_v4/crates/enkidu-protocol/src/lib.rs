#![forbid(unsafe_code)]
//! ENKIDU-PROTOCOL — federated HeptaScript TCP wire protocol.
//!
//! Provides three pillars for >1B particle retrieval in <1 second:
//!
//! 1. `ring`   — SPSC lock-free ring buffer (cache-line padded, power-of-2)
//! 2. `pool`   — Buffer pool (ArrayQueue reuse, zero allocation on hot path)
//! 3. `frame`  — TCP wire frame codec (QUERY → RESULT_STREAM → RESULT_END)
//!
//! ## Architecture
//! ```text
//! HeptaScript ──► [QueryFrame] ──TCP──► EnkiDB Node
//!                                             │
//!                                    [SPSC Ring Buffer]
//!                                             │
//!                                    [Buffer Pool Bufs]
//!                                             │
//!                               ◄──[RESULT_STREAM frames]──
//! ```

pub mod frame;
pub mod pool;
pub mod ring;

pub use frame::{EnkiFrame, FrameCodec, FrameKind};
pub use pool::BufferPool;
pub use ring::SpscRing;
