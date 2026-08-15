//! ea-agent-chat — 𒂗𒆠 EaAgent chat interface for DubSar IDE.
//!
//! Provides the DubSar panel and CLI for EaAgent — the Math God.
//! Uses DeepSeek-Math-7B-Instruct-Q4_K_M.gguf for complex algebra.
//! Quick commands handle instant sovereign math without the model.
//!
//! Quick commands:
//!   !pauli   — explain Pauli Exclusion Principle
//!   !jordan  — explain Jordan Normal Form
//!   !solve   <equation> — solve algebraically (exact, no LLM)
//!   !b11     <7 values> — compute B11 from Hepta vector
//!   !tribe   <id>       — analyze tribe stability
//!   !help    — all commands
#![forbid(unsafe_code)]

pub mod ea_commands;
pub mod ea_prompt;
pub mod ea_panel_state;
pub mod ea_chat_engine;
pub mod ea_model_loader;

pub use ea_commands::{EaCommand, parse_ea_command, handle_ea_command};
pub use ea_prompt::{EaPromptBuilder, DEEPSEEK_MATH_SYSTEM};
pub use ea_panel_state::{EaPanelState, EaMessage, EaMessageKind};
pub use ea_chat_engine::{EaChatEngine, EaChatError, EaChatResponse};
pub use ea_model_loader::{EaModelLoader, EaModelStatus, DEEPSEEK_MATH_MODEL};
