//! AkkadianAOL v3.5 — LSP Server (tower-lsp)
//! Phase 3 stub — DubSar IDE integration pending.
//!
//! Planned features:
//!   - Completion from AkkToken::canonical_spelling()
//!   - Hover shows Arabic + English for each keyword
//!   - Diagnostics from Lexer::errors + SemanticAnalyser::diagnostics
//!   - Goto-definition resolves tablet::sign references

pub const LSP_PORT: u16 = 9100;

/// LSP server entry point — Phase 3 stub.
pub async fn run_lsp_server() {
    println!("[AkkadianAOL LSP] Phase 3 — DubSar IDE integration pending");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_lsp_stub_compiles() {
        assert!(true);
    }
}
