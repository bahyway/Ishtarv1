//! enkiddb_bridge.rs — TamuzAI's real connection to EnkiDDB's RAG.
//!
//! The actual TCP client now lives in the shared `enkiddb-rag-client`
//! crate (extracted so EaAgent could reuse the exact same wire client
//! instead of a second hand-copied version -- see that crate's own doc
//! comment for why). This module just re-exports it under the name
//! `chat_engine.rs` and `lib.rs` already expect, so nothing outside this
//! crate had to change.
#![forbid(unsafe_code)]

pub use enkiddb_rag_client::*;
