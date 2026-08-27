//! enkidullm-chat — TamuzAI sovereign chat interface for EnkiduLLM.
//!
//! Provides two interfaces:
//!   1. CLI — terminal chat with TamuzAI (no egui required)
//!   2. DubSar panel — egui widget for the 𒀭 TamuzAI panel
//!
//! Target model: DeepSeek-Coder-6.7B-Instruct-Q4_K_M.gguf
//! Runs entirely offline — zero external API calls.
//!
//! Quick commands:
//!   !adr   <N>    — explain ADR-N
//!   !crate <name> — explain a crate
//!   !next         — what to build next
//!   !law  <N>     — explain sovereign law N
//!   !kaki         — explain KAKI byte layout
//!   !enkiddb <phrase> — real semantic search against a live EnkiDDB Read Node
//!   !help         — show all commands
#![forbid(unsafe_code)]

pub mod chat_engine;
pub mod commands;
pub mod enkiddb_bridge;
pub mod model_loader;
pub mod panel_state;
pub mod prompt;

pub use chat_engine::{ChatEngine, ChatError, ChatResponse};
pub use commands::{handle_quick_command, parse_quick_command, QuickCommand};
pub use enkiddb_bridge::{
    search as enkiddb_search, EnkiddbHit, DEFAULT_HOST as ENKIDDB_DEFAULT_HOST,
    DEFAULT_PORT as ENKIDDB_DEFAULT_PORT,
};
pub use model_loader::{ModelLoader, ModelStatus, DEFAULT_MODEL_NAME};
pub use panel_state::{MessageKind, PanelMessage, TamuzPanelState};
pub use prompt::{ChatTemplate, PromptBuilder, DEEPSEEK_SYSTEM};
