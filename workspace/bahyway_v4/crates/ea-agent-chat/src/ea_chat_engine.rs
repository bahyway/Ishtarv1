//! ea_chat_engine.rs — EaAgent chat orchestrator.
#![forbid(unsafe_code)]

use crate::ea_commands::{parse_ea_command, handle_ea_command, EaCommand};
use crate::ea_prompt::EaPromptBuilder;
use crate::ea_model_loader::{EaModelLoader, DEEPSEEK_MATH_MODEL};
use crate::ea_panel_state::EaPanelState;
use enkiddb_rag_client::{self, DEFAULT_HOST as ENKIDDB_DEFAULT_HOST, DEFAULT_PORT as ENKIDDB_DEFAULT_PORT};

#[derive(Debug)]
pub enum EaChatError {
    ModelNotLoaded,
    InferenceError(String),
}

impl std::fmt::Display for EaChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelNotLoaded    => write!(f, "DeepSeek-Math model not loaded. Use !help for quick commands."),
            Self::InferenceError(e) => write!(f, "Inference error: {e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EaChatResponse {
    pub text:       String,
    pub from_quick: bool,
    pub is_exact:   bool,  // true = sovereign exact math, no LLM
}

/// The EaAgent chat engine.
pub struct EaChatEngine {
    pub loader:  EaModelLoader,
    pub panel:   EaPanelState,
    prompt:      EaPromptBuilder,
    /// EnkiDDB Read Node this engine's `!enkiddb` command dials --
    /// defaults to the same host:port every other client in this
    /// ecosystem defaults to (TamuzAI's ChatEngine included). Override
    /// with `set_enkiddb_target` to point at a real deployed node (e.g.
    /// eriduous-vdi:7102).
    enkiddb_host: String,
    enkiddb_port: u16,
}

impl EaChatEngine {
    pub fn new() -> Self {
        let mut loader = EaModelLoader::new();
        loader.discover();
        Self {
            loader,
            panel:  EaPanelState::new(),
            prompt: EaPromptBuilder::new(),
            enkiddb_host: ENKIDDB_DEFAULT_HOST.to_string(),
            enkiddb_port: ENKIDDB_DEFAULT_PORT,
        }
    }

    /// Point `!enkiddb` at a specific EnkiDDB Read Node instead of the
    /// default `127.0.0.1:7102`.
    pub fn set_enkiddb_target(&mut self, host: impl Into<String>, port: u16) {
        self.enkiddb_host = host.into();
        self.enkiddb_port = port;
    }

    /// Process a user message — quick commands bypass the model.
    pub fn chat(&mut self, user_msg: &str) -> Result<EaChatResponse, EaChatError> {
        let msg = user_msg.trim();
        if msg.is_empty() {
            return Ok(EaChatResponse { text: String::new(), from_quick: true, is_exact: false });
        }

        self.panel.push_user(msg);

        // Quick commands — instant, no model needed
        if let Some(cmd) = parse_ea_command(msg) {
            if cmd == EaCommand::Clear {
                self.panel.clear();
                return Ok(EaChatResponse {
                    text: "Cleared.".into(), from_quick: true, is_exact: false
                });
            }

            // Real network I/O -- intercepted before handle_ea_command
            // (which stays pure/no-I/O for the other commands), the same
            // way TamuzAI's ChatEngine intercepts QuickCommand::Enkiddb.
            if let EaCommand::Enkiddb(query) = &cmd {
                let response = self.run_enkiddb_search(query);
                self.panel.push_agent(&response);
                return Ok(EaChatResponse { text: response, from_quick: true, is_exact: false });
            }

            let is_exact = matches!(cmd, EaCommand::Solve(_) | EaCommand::B11(_));
            let response = handle_ea_command(&cmd);

            if is_exact {
                self.panel.push_math(&response);
            } else {
                self.panel.push_agent(&response);
            }

            return Ok(EaChatResponse { text: response, from_quick: true, is_exact });
        }

        // Natural language — needs model
        if !self.loader.status.is_ready() {
            let resp = self.no_model_response(msg);
            self.panel.push_agent(&resp);
            return Ok(EaChatResponse { text: resp, from_quick: true, is_exact: false });
        }

        // Model ready — structured response (full inference wired in v4.2+)
        let resp = self.structured_response(msg);
        self.panel.push_agent(&resp);
        Ok(EaChatResponse { text: resp, from_quick: false, is_exact: false })
    }

    /// Runs a real `SEARCH:` call against `self.enkiddb_host:enkiddb_port`
    /// and formats the results for the chat panel -- EaAgent's first real
    /// answer from EnkiDDB's ingested documents rather than only its own
    /// sealed math knowledge base (Pauli/Jordan/Harmony/Laws) or the
    /// exact algebraic solver.
    fn run_enkiddb_search(&self, query: &str) -> String {
        if query.trim().is_empty() {
            return "Usage: !enkiddb <phrase> — e.g. !enkiddb spectral radius stability".to_string();
        }
        match enkiddb_rag_client::search(&self.enkiddb_host, self.enkiddb_port, 5, query) {
            Ok(hits) if hits.is_empty() => format!(
                "𒂗𒆠 EnkiDDB ({}:{}) — no results for \"{query}\".",
                self.enkiddb_host, self.enkiddb_port
            ),
            Ok(hits) => {
                let mut out = format!(
                    "𒂗𒆠 EnkiDDB ({}:{}) — {} result(s) for \"{query}\":\n",
                    self.enkiddb_host, self.enkiddb_port, hits.len()
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
                "𒂗𒆠 EnkiDDB ({}:{}) — could not reach the Read Node: {e}\n\
                 Confirm enkiddb-read-server is running there (see PB-208/PB-211).",
                self.enkiddb_host, self.enkiddb_port
            ),
        }
    }

    fn no_model_response(&self, msg: &str) -> String {
        let lower = msg.to_lowercase();
        if lower.contains("pauli")   { return handle_ea_command(&EaCommand::Pauli); }
        if lower.contains("jordan")  { return handle_ea_command(&EaCommand::Jordan); }
        if lower.contains("harmony") { return handle_ea_command(&EaCommand::Harmony); }
        if lower.contains("b11") || lower.contains("h(p)") || lower.contains("quality") {
            return handle_ea_command(&EaCommand::Hepta);
        }
        if lower.contains("law") { return handle_ea_command(&EaCommand::Laws); }

        format!(
            "𒂗𒆠 EaAgent understands: \"{msg}\"\n\n\
             DeepSeek-Math-7B not yet loaded.\n\
             Quick commands available:\n\
             • !pauli   — Pauli Exclusion Principle\n\
             • !jordan  — Jordan Normal Form stability\n\
             • !harmony — Tribe Harmony Coefficient\n\
             • !solve x^2-5x+6 — Exact algebraic solver\n\
             • !b11 0.9 0.8 0.7 0.85 0.75 0.6 0.95 — Compute B11\n\
             • !enkiddb <phrase> — Real SEARCH: against EnkiDDB's RAG\n\
             • !help    — All commands\n\n\
             To enable full AI:\n\
             {}", EaModelLoader::download_instructions()
        )
    }

    fn structured_response(&self, msg: &str) -> String {
        format!(
            "𒂗𒆠 Processing: \"{msg}\"\n\
             [DeepSeek-Math-7B inference — chain of thought will appear here]\n\
             Wire InferenceEngine in ea_chat_engine.rs for full response."
        )
    }

    pub fn status_line(&self) -> String {
        format!("EaAgent {} | {}", self.loader.status.status_str(), DEEPSEEK_MATH_MODEL)
    }

    pub fn is_ready(&self) -> bool { self.loader.status.is_ready() }
}

impl Default for EaChatEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_initializes() {
        let e = EaChatEngine::new();
        assert!(!e.panel.messages.is_empty());
    }
    #[test]
    fn pauli_command_works() {
        let mut e = EaChatEngine::new();
        let r = e.chat("!pauli").unwrap();
        assert!(r.from_quick);
        assert!(r.text.contains("Quantum"));
    }
    #[test]
    fn solve_is_exact() {
        let mut e = EaChatEngine::new();
        let r = e.chat("!solve x^2-5x+6").unwrap();
        assert!(r.is_exact);
    }
    #[test]
    fn b11_is_exact() {
        let mut e = EaChatEngine::new();
        let r = e.chat("!b11 1.0 1.0 1.0 1.0 1.0 1.0 1.0").unwrap();
        assert!(r.is_exact);
        assert!(r.text.contains("240"));
    }
    #[test]
    fn clear_resets_panel() {
        let mut e = EaChatEngine::new();
        e.chat("!pauli").unwrap();
        e.chat("!clear").unwrap();
        assert_eq!(e.panel.messages.len(), 1);
    }
    #[test]
    fn status_line_contains_model_name() {
        let e = EaChatEngine::new();
        assert!(e.status_line().contains("deepseek-math"));
    }
    #[test]
    fn empty_message_returns_empty() {
        let mut e = EaChatEngine::new();
        let r = e.chat("").unwrap();
        assert!(r.text.is_empty());
    }
    #[test]
    fn natural_language_no_model_response() {
        let mut e = EaChatEngine::new();
        let r = e.chat("What is Jordan Normal Form?").unwrap();
        assert!(!r.text.is_empty());
        assert!(r.from_quick);
    }
    #[test]
    fn enkiddb_defaults_match_the_ecosystem_convention() {
        let e = EaChatEngine::new();
        assert_eq!(e.enkiddb_host, "192.168.122.107");
        assert_eq!(e.enkiddb_port, 7102);
    }
    #[test]
    fn set_enkiddb_target_overrides_the_default() {
        let mut e = EaChatEngine::new();
        e.set_enkiddb_target("192.168.122.214", 7102);
        assert_eq!(e.enkiddb_host, "192.168.122.214");
        assert_eq!(e.enkiddb_port, 7102);
    }
    #[test]
    fn enkiddb_command_with_no_server_running_reports_a_real_connect_failure() {
        // Deliberately unbound port -- proves the failure path is a real
        // network error, not a silently-empty success.
        let mut e = EaChatEngine::new();
        e.set_enkiddb_target("127.0.0.1", 1); // port 1 is never a real EnkiDDB node
        let r = e.chat("!enkiddb anything").unwrap();
        assert!(r.from_quick);
        assert!(r.text.contains("could not reach"), "got: {}", r.text);
    }
    #[test]
    fn enkiddb_command_with_empty_query_shows_usage() {
        let mut e = EaChatEngine::new();
        let r = e.chat("!enkiddb").unwrap();
        assert!(r.text.contains("Usage: !enkiddb"));
    }
}
