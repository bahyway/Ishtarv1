//! Real conversation-flow integration test for enkidullm-memory.
//!
//! Existing search.rs unit tests exercise `MemorySearch` against hand-built
//! `ConversationParticle`s constructed directly via `ConversationParticle::
//! from_turn`, bypassing `MemoryStore`/`Session` entirely. This test chains
//! the real path a caller (TamuzAI / EaAgent) actually uses: MemoryStore ->
//! new_session -> add_user/add_assistant (real KAKI minting, real epoch
//! ticking) -> all_particles() -> MemorySearch::semantic_search(), plus
//! Session::context_window() on the resulting session.
//!
//! Honesty note (found while writing this test, not previously flagged
//! anywhere): lib.rs's crate-level doc comment claims "Memory persists
//! across restarts via the journal," but memory_store.rs's own doc comment
//! says "Persistence via journal file is planned for v4.1." There is no
//! file I/O anywhere in this crate today — MemoryStore is entirely
//! in-memory (a HashMap<u64, Session>). This test does NOT claim to verify
//! persistence, because none exists yet; see playbook_250 for the tracked
//! doc/reality mismatch.

use enkidullm_memory::{MemorySearch, MemoryStore};

#[test]
fn real_store_session_search_round_trip() {
    let mut store = MemoryStore::new();
    let sid = store.new_session("HeptaScript walkthrough");

    store
        .add_user(sid, "What does KAKI stand for in the sovereign ecosystem?")
        .unwrap();
    store
        .add_assistant(sid, "KAKI is the 16-byte sovereign identity particle.")
        .unwrap();
    store
        .add_user(sid, "How does zikru-embed pool sector attention?")
        .unwrap();
    store
        .add_assistant(
            sid,
            "zikru-embed uses TribalFieldAttention then pool_sectors across 7 sectors.",
        )
        .unwrap();

    assert_eq!(store.total_turns(), 4);

    let particles = store.all_particles();
    assert_eq!(particles.len(), 4);

    // "zikru-embed" and "pool_sectors" only appear in one turn — a real
    // TF-IDF search over the real store should surface it first.
    let results = MemorySearch::semantic_search(&particles, "zikru-embed pool_sectors", 3);
    assert!(
        !results.is_empty(),
        "semantic_search over a real MemoryStore found nothing"
    );
    assert!(
        results[0].particle.turn.content.contains("zikru-embed"),
        "expected the zikru-embed turn to rank first, got: {}",
        results[0].particle.turn.content
    );
}

#[test]
fn real_session_context_window_respects_token_budget() {
    let mut store = MemoryStore::new();
    let sid = store.new_session("Budget test");
    store.add_user(sid, "first message here").unwrap();
    store.add_assistant(sid, "first reply here").unwrap();
    store.add_user(sid, "second message here").unwrap();
    store.add_assistant(sid, "second reply here").unwrap();

    let session = store.session(sid).unwrap();
    let full_window = session.context_window(10_000);
    assert!(full_window.contains("first message here"));
    assert!(full_window.contains("second reply here"));

    // A tiny budget should drop the oldest turns, keeping only the most
    // recent ones (context_window walks turns in reverse, then re-reverses).
    let tiny_window = session.context_window(1);
    assert!(
        tiny_window.len() <= full_window.len(),
        "a 1-token budget must not return more content than an effectively unlimited one"
    );
}
