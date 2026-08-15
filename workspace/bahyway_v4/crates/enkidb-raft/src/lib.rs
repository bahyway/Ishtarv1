//! enkidb-raft — Sovereign Raft consensus for EnkiDB distributed clusters.
//!
//! Pure state-machine implementation: no I/O, no async, no external dependencies.
//! The caller drives the machine via `tick(elapsed_ms)` and `handle(from, msg)`,
//! acting on the returned `Vec<RaftEffect>` (send RPCs, write WAL, apply commands).
//!
//! # Wire Protocol (TCP, text-based, sovereign)
//! ```text
//! VOTE <term> <candidate_id> <last_log_index> <last_log_term>
//! VOTE_OK <term> <granted:0|1>
//! APPEND <term> <leader_id> <prev_idx> <prev_term> <commit> <count> [entries...]
//! APPEND_OK <term> <match_index>
//! APPEND_FAIL <term>
//! HEARTBEAT <term> <leader_id> <commit>
//! ```
//!
//! # Election Timeouts
//! Randomised in [150, 300] ms using a deterministic LCG seeded from node_id + term.
//! No std::rand — completely sovereign.

#![forbid(unsafe_code)]

pub mod log;
pub mod message;
pub mod state;

pub use log::{LogEntry, RaftEav, LogView};
pub use message::RaftMessage;
pub use state::{
    RaftNode, RaftEffect, Role,
    ELECTION_TIMEOUT_MIN_MS, ELECTION_TIMEOUT_MAX_MS, HEARTBEAT_INTERVAL_MS,
};
