#![forbid(unsafe_code)]
//! enkidb-replication — Sovereign KAKI-signed append-only replication.
//!
//! Implements the Write→Broker→Read pipeline described in:
//! `docs/09_security/bahyway_security_evaluation.md`
//!
//! # Architecture
//!
//! ```text
//! WRITE POD
//!   ReplicationEmitter.emit(kind, delta, epoch)
//!     → KakiSealedEvent { KANĀKU Ed25519 seal + chained SHA3-256 }
//!     → appended to /replication/log.enkwal (:rw)
//!
//! BROKER (--network none)
//!   ReplicationBroker.sweep(now_epoch)
//!     → reads /replication/log.enkwal (:ro)
//!     → 7-layer verification
//!     → writes /read-input/delta.enkwal (:rw)
//!
//! READ POD
//!   ReplicationConsumer.consume(now_epoch)
//!     → reads /read-input/delta.enkwal (:ro)
//!     → second verification pass
//!     → calls apply_fn(kind, delta) → EnkiDB
//! ```
//!
//! # Sovereign constants
//!
//! ```rust
//! pub const REPL_MAX_SECS: u32 = 300;  // 5-minute freshness window
//! pub const GENESIS_DIGEST: [u8; 32] = [0u8; 32];  // first event anchor
//! ```

pub mod broker;
pub mod consumer;
pub mod emitter;
pub mod error;
pub mod event;
pub mod journal_bridge;

pub use broker::{BrokerConfig, PassportValidator, ReplicationBroker};
pub use consumer::{ApplyFn, ConsumerStats, ReplicationConsumer};
pub use emitter::ReplicationEmitter;
pub use error::{ReplicationError, ReplicationResult};
pub use event::{
    KakiSealedEvent, ReplEventKind, FRAME_MAGIC, GENESIS_DIGEST, REPL_DOMAIN, REPL_MAX_SECS,
};
pub use journal_bridge::{apply_delta_to_journal, emit_journal_entry, materialize_journal};
