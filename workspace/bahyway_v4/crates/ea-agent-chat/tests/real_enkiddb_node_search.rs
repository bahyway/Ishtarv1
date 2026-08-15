//! Real-network integration test for EaAgent's `!enkiddb` command.
//!
//! `EaChatEngine` has had a real `enkiddb_rag_client::search()` call wired
//! into it since the TamuzAI/EaAgent RAG-wiring work, and it defaults to
//! the actual PB-212 CQRS-split Read Node
//! (enkiddb_rag_client::DEFAULT_HOST:DEFAULT_PORT = 192.168.122.107:7102).
//! But the crate's own existing test suite only points `!enkiddb` at a
//! dead port (127.0.0.1:1) to exercise the *failure* path
//! (`ea_chat_engine.rs`'s `enkiddb_command_reports_real_network_error`-
//! style tests) — nothing in the workspace has ever run `!enkiddb` against
//! the actual live node this ecosystem deploys.
//!
//! This test is `#[ignore]`d by default because it requires real network
//! reachability to a specific internal VM IP that does not exist outside
//! the Architect's real infra (eriduous-vdi / the PB-212 VM pair) — it
//! will simply fail to connect anywhere else, including this repo's CI
//! sandbox. playbook_250 runs it explicitly with `--ignored` only after a
//! `wait_for` reachability check against the same host:port confirms the
//! node is actually up, so a down/unreachable node fails fast with a clear
//! message instead of this test's own 30s client-side timeout.
use ea_agent_chat::EaChatEngine;
use enkiddb_rag_client::{DEFAULT_HOST, DEFAULT_PORT};

#[test]
#[ignore = "requires real network reachability to the PB-212 EnkiDDB Read Node"]
fn enkiddb_command_returns_real_hits_from_the_live_node() {
    let mut engine = EaChatEngine::new();
    engine.set_enkiddb_target(DEFAULT_HOST, DEFAULT_PORT);

    let response = engine
        .chat("!enkiddb HeptaScript")
        .expect("chat() should not error even on empty results");

    assert!(
        !response.text.is_empty(),
        "expected a real, non-empty response from the live EnkiDDB Read Node"
    );
    // run_enkiddb_search()'s Err branch always formats failures with this
    // exact phrase (ea_chat_engine.rs) -- checking for it directly (rather
    // than guessing at OS-level error wording) is what actually proves the
    // call reached the real node instead of failing over the network.
    assert!(
        !response.text.contains("could not reach the Read Node"),
        "expected a real search result, not a network failure surfaced as text: {}",
        response.text
    );
}
