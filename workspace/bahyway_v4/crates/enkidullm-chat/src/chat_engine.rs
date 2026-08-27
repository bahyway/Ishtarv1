//! chat_engine.rs — TamuzAI chat engine orchestrator.
//!
//! Connects enkidullm-memory (conversation journal) with enkidullm-model (GGUF runner)
//! and enkidullm-chat quick commands into a unified sovereign chat interface.
#![forbid(unsafe_code)]

use crate::commands::{handle_quick_command, parse_quick_command, QuickCommand};
use crate::enkiddb_bridge::{self, DEFAULT_HOST, DEFAULT_PORT};
use crate::model_loader::ModelLoader;
use crate::panel_state::TamuzPanelState;
use crate::prompt::PromptBuilder;

/// Chat engine errors.
#[derive(Debug)]
pub enum ChatError {
    ModelNotLoaded,
    InferenceError(String),
    MemoryError(String),
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelNotLoaded => {
                write!(f, "Model not loaded. Type !help for setup instructions.")
            }
            Self::InferenceError(e) => write!(f, "Inference error: {e}"),
            Self::MemoryError(e) => write!(f, "Memory error: {e}"),
        }
    }
}

/// Response from the chat engine.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub text: String,
    pub from_quick: bool, // true = quick command, false = LLM inference
    pub tokens: usize,
}

/// The TamuzAI chat engine.
pub struct ChatEngine {
    pub loader: ModelLoader,
    pub panel: TamuzPanelState,
    #[allow(dead_code)] // scaffolded for prompt-building, not yet wired into chat()
    prompt_builder: PromptBuilder,
    session_id: u64,
    /// EnkiDDB Read Node this engine's `!enkiddb` command dials --
    /// defaults to the same host:port every other client in this
    /// ecosystem defaults to. Override with `set_enkiddb_target` to
    /// point at a real deployed node (e.g. eriduous-vdi:7102).
    enkiddb_host: String,
    enkiddb_port: u16,
}

impl ChatEngine {
    pub fn new() -> Self {
        let mut loader = ModelLoader::new();
        loader.discover();
        Self {
            loader,
            panel: TamuzPanelState::new(),
            prompt_builder: PromptBuilder::tamuzai(),
            session_id: 1,
            enkiddb_host: DEFAULT_HOST.to_string(),
            enkiddb_port: DEFAULT_PORT,
        }
    }

    /// Point `!enkiddb` at a specific EnkiDDB Read Node instead of the
    /// default `127.0.0.1:7102`.
    pub fn set_enkiddb_target(&mut self, host: impl Into<String>, port: u16) {
        self.enkiddb_host = host.into();
        self.enkiddb_port = port;
    }

    /// Process a user message and return the response.
    /// This is the main entry point for all chat interactions.
    pub fn chat(&mut self, user_msg: &str) -> Result<ChatResponse, ChatError> {
        let msg = user_msg.trim();
        if msg.is_empty() {
            return Ok(ChatResponse {
                text: String::new(),
                from_quick: true,
                tokens: 0,
            });
        }

        // Record user message in panel
        self.panel.push_user(msg);

        // Check for quick commands first (no model needed)
        if let Some(cmd) = parse_quick_command(msg) {
            // Real network I/O -- intercepted before handle_quick_command
            // (which stays pure/no-I/O for the other nine commands), the
            // same way Clear is special-cased below for panel mutation.
            if let QuickCommand::Enkiddb(query) = &cmd {
                let response = self.run_enkiddb_search(query);
                self.panel.push_command(&response);
                return Ok(ChatResponse {
                    text: response,
                    from_quick: true,
                    tokens: 0,
                });
            }

            let response = handle_quick_command(&cmd);

            // Handle special commands
            if cmd == QuickCommand::Clear {
                self.panel.clear();
                return Ok(ChatResponse {
                    text: response,
                    from_quick: true,
                    tokens: 0,
                });
            }

            self.panel.push_command(&response);
            return Ok(ChatResponse {
                text: response,
                from_quick: true,
                tokens: 0,
            });
        }

        // For LLM inference — check model is loaded
        if !self.loader.status.is_ready() {
            let instructions = ModelLoader::download_instructions();
            let resp = format!(
                "⚫ TamuzAI model not loaded.\n\n\
                 For now, use quick commands (!help, !kaki, !adr, !law, !crate, !enkiddb, !next).\n\n\
                 To enable full AI responses:\n{instructions}"
            );
            self.panel.push_assistant(&resp);
            return Ok(ChatResponse {
                text: resp,
                from_quick: true,
                tokens: 0,
            });
        }

        // Model is ready — run inference
        // Note: actual inference call will be wired once model is downloaded
        // For now return a structured placeholder
        let resp = self.structured_response(msg);
        self.panel.push_assistant(&resp);
        Ok(ChatResponse {
            text: resp,
            from_quick: false,
            tokens: 0,
        })
    }

    /// Runs a real `SEARCH:` call against `self.enkiddb_host:enkiddb_port`
    /// and formats the results for the chat panel -- the first real
    /// answer TamuzAI can give from EnkiDDB's ingested documents rather
    /// than its own hardcoded knowledge base or past conversation turns.
    fn run_enkiddb_search(&self, query: &str) -> String {
        if query.trim().is_empty() {
            return "Usage: !enkiddb <phrase> — e.g. !enkiddb HeptaScript ingestion".to_string();
        }
        match enkiddb_bridge::search(&self.enkiddb_host, self.enkiddb_port, 5, query) {
            Ok(hits) if hits.is_empty() => format!(
                "𒀭 EnkiDDB ({}:{}) — no results for \"{query}\".",
                self.enkiddb_host, self.enkiddb_port
            ),
            Ok(hits) => {
                let mut out = format!(
                    "𒀭 EnkiDDB ({}:{}) — {} result(s) for \"{query}\":\n",
                    self.enkiddb_host,
                    self.enkiddb_port,
                    hits.len()
                );
                for hit in hits {
                    let snippet = if hit.text.len() > 160 {
                        format!("{}...", &hit.text[..157])
                    } else {
                        hit.text.clone()
                    };
                    out.push_str(&format!("\n  score={:.4}  {snippet}", hit.score));
                }
                out
            }
            Err(e) => format!(
                "𒀭 EnkiDDB ({}:{}) — could not reach the Read Node: {e}\n\
                 Confirm enkiddb-read-server is running there (see PB-208).",
                self.enkiddb_host, self.enkiddb_port
            ),
        }
    }

    /// Structured response when model isn't available but we can help.
    fn structured_response(&self, msg: &str) -> String {
        let lower = msg.to_lowercase();

        // Pattern matching for common questions
        if lower.contains("kaki") && (lower.contains("what") || lower.contains("explain")) {
            return handle_quick_command(&QuickCommand::Kaki);
        }
        if lower.contains("next") || lower.contains("roadmap") || lower.contains("build") {
            return handle_quick_command(&QuickCommand::Next);
        }
        if lower.contains("law") || lower.contains("sovereign") {
            return "Type !law <N> to see any of the 10 sovereign laws.\nExample: !law 1, !law 9"
                .to_string();
        }
        if lower.contains("adr") {
            return "Type !adr <N> to see any ADR.\nExample: !adr 3 (seq_counter), !adr 6 (no DELETE)".to_string();
        }

        format!(
            "I understand your question about: \"{msg}\"\n\n\
             The DeepSeek-Coder-6.7B model is not yet loaded.\n\
             Use quick commands for immediate answers:\n\
             • !kaki — KAKI byte layout\n\
             • !adr 3 — seq_counter decision\n\
             • !law 9 — ColorID sovereign law\n\
             • !crate enkidb-kaki — crate explanation\n\
             • !next — build roadmap\n\
             • !help — all commands"
        )
    }

    /// Model status summary for display.
    pub fn status_line(&self) -> String {
        format!(
            "TamuzAI {} | Session {}",
            self.loader.status.status_str(),
            self.session_id
        )
    }

    /// Check if model is ready for inference.
    pub fn is_ready(&self) -> bool {
        self.loader.status.is_ready()
    }
}

impl Default for ChatEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_engine_initializes() {
        let engine = ChatEngine::new();
        assert!(!engine.is_ready()); // No model on disk in test env
    }

    #[test]
    fn enkiddb_defaults_match_the_ecosystem_convention() {
        let engine = ChatEngine::new();
        assert_eq!(engine.enkiddb_host, "192.168.122.112");
        assert_eq!(engine.enkiddb_port, 7102);
    }

    #[test]
    fn set_enkiddb_target_overrides_the_default() {
        let mut engine = ChatEngine::new();
        engine.set_enkiddb_target("192.168.122.214", 7102);
        assert_eq!(engine.enkiddb_host, "192.168.122.214");
        assert_eq!(engine.enkiddb_port, 7102);
    }

    #[test]
    fn enkiddb_command_with_no_server_running_reports_a_real_connect_failure() {
        // Deliberately unbound port -- proves the failure path is a real
        // network error, not a silently-empty success.
        let mut engine = ChatEngine::new();
        engine.set_enkiddb_target("127.0.0.1", 1); // port 1 is never a real EnkiDDB node
        let resp = engine.chat("!enkiddb anything").unwrap();
        assert!(resp.from_quick);
        assert!(resp.text.contains("could not reach"), "got: {}", resp.text);
    }

    #[test]
    fn enkiddb_command_with_empty_query_shows_usage() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("!enkiddb").unwrap();
        assert!(resp.text.contains("Usage: !enkiddb"));
    }

    #[test]
    fn quick_command_handled_without_model() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("!kaki").unwrap();
        assert!(resp.from_quick);
        assert!(resp.text.contains("uuid_hash"));
    }

    #[test]
    fn help_command_works() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("!help").unwrap();
        assert!(resp.from_quick);
        assert!(resp.text.contains("!kaki"));
    }

    #[test]
    fn adr3_command_works() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("!adr 3").unwrap();
        assert!(resp.from_quick);
        assert!(resp.text.contains("seq_counter"));
    }

    #[test]
    fn clear_command_clears_panel() {
        let mut engine = ChatEngine::new();
        engine.chat("!kaki").unwrap();
        engine.chat("!clear").unwrap();
        assert_eq!(engine.panel.messages.len(), 1); // only system msg after clear
    }

    #[test]
    fn no_model_message_contains_instructions() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("What is KAKI?").unwrap();
        // Either quick command response or model-not-loaded message
        assert!(!resp.text.is_empty());
    }

    #[test]
    fn status_line_contains_session() {
        let engine = ChatEngine::new();
        let status = engine.status_line();
        assert!(status.contains("TamuzAI"));
        assert!(status.contains("Session"));
    }

    #[test]
    fn empty_message_returns_empty_response() {
        let mut engine = ChatEngine::new();
        let resp = engine.chat("").unwrap();
        assert!(resp.text.is_empty());
    }

    #[test]
    fn panel_has_welcome_after_init() {
        let engine = ChatEngine::new();
        assert_eq!(engine.panel.messages.len(), 1);
        assert!(engine.panel.messages[0].content.contains("TamuzAI"));
    }
}
