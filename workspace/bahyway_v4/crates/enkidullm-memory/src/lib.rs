//! enkidullm-memory — Sovereign conversation journal for EnkiduLLM.
//!
//! Every conversation turn is a KAKI-stamped particle.
//! Every session is a tribe. No SQLite. No external storage.
//! Cross-restart persistence via an append-only journal file is planned
//! for v4.1 (see `memory_store::MemoryStore` doc comment) — MemoryStore is
//! in-memory only today.
//!
//! Tribe ID: 0x1200 (conversations)
//! Roles: KISHIB=user message, ZIKRU=assistant response, PARZU=system event
#![forbid(unsafe_code)]

pub mod conversation;
pub mod memory_store;
pub mod search;
pub mod session;

pub use conversation::{ConversationParticle, Turn, TurnRole, CONV_TRIBE_ID};
pub use memory_store::{MemoryError, MemoryStore};
pub use search::{MemorySearch, SearchResult};
pub use session::{Session, SessionStats};
