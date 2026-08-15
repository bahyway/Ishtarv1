//! Cross-crate integration test for HeptaSec Layer 8.
//!
//! Existing per-crate unit tests (in hepta-sec-firewall/policy/sentinel/web)
//! each exercise their own crate in isolation with synthetic KakiPacket/header
//! values. Nothing in the workspace previously chained all four crates
//! together through the *actual* entry point an HTTP server would use:
//! raw header bytes in, HttpVerdict out. This test does that, plus exercises
//! HeptaSecSentinel::purge_stale() end to end (see playbook_249 for context:
//! purge_stale used to always return 0 due to a dead-code bug, fixed
//! alongside this test).

use hepta_sec_web::{KakiExtractor, WebSentinelGuard, HttpVerdict, KAKI_HASH_HEADER};

const SATTATU_MAX_SECS: u32 = 54 * 3600;

fn header<'a>(value: &'a [u8]) -> [(&'a [u8], &'a [u8]); 1] {
    [(KAKI_HASH_HEADER.as_bytes(), value)]
}

#[test]
fn golden_kaki_end_to_end_allow() {
    let mut guard = WebSentinelGuard::new();
    let kaki_hash = 0xC0FF_EE01;
    guard.register_trusted(kaki_hash, 240, 0); // 240 == Golden per TrustState::from_b11

    let hdrs = header(b"c0ffee01");
    let verdict = guard.inspect(&hdrs, 128, 1);

    assert_eq!(verdict, HttpVerdict::Allow { kaki_hash });
}

#[test]
fn missing_header_is_forbidden() {
    let mut guard = WebSentinelGuard::new();
    let verdict = guard.inspect(&[], 0, 1);
    assert_eq!(verdict, HttpVerdict::Forbidden);
}

#[test]
fn malformed_header_is_forbidden() {
    let mut guard = WebSentinelGuard::new();
    let hdrs = header(b"not-hex!!");
    let verdict = guard.inspect(&hdrs, 0, 1);
    assert_eq!(verdict, HttpVerdict::Forbidden);
}

#[test]
fn unregistered_kaki_is_quarantined() {
    let mut guard = WebSentinelGuard::new();
    let hdrs = header(b"deadbeef");
    let verdict = guard.inspect(&hdrs, 0, 1);
    match verdict {
        HttpVerdict::Quarantine { kaki_hash, .. } => assert_eq!(kaki_hash, 0xDEAD_BEEF),
        other => panic!("expected Quarantine, got {other:?}"),
    }
}

#[test]
fn revoked_kaki_is_forbidden() {
    let mut guard = WebSentinelGuard::new();
    let kaki_hash = 0x1234_5678;
    guard.register_trusted(kaki_hash, 240, 0);
    guard.sentinel().revoke_kaki(kaki_hash, 1);

    let hdrs = header(b"12345678");
    let verdict = guard.inspect(&hdrs, 0, 2);
    assert_eq!(verdict, HttpVerdict::Forbidden);
}

#[test]
fn extractor_matches_guard_pipeline_directly() {
    let hdrs = header(b"aabbccdd");
    let extracted = KakiExtractor::from_headers(&hdrs);
    assert_eq!(extracted, Some(0xAABB_CCDD));

    let mut guard = WebSentinelGuard::new();
    guard.register_trusted(0xAABB_CCDD, 240, 0);
    let verdict = guard.inspect(&hdrs, 0, 1);
    assert_eq!(verdict, HttpVerdict::Allow { kaki_hash: 0xAABB_CCDD });
}

#[test]
fn purge_stale_reports_real_downgrade_count_through_the_full_stack() {
    let mut guard = WebSentinelGuard::new();
    // Three KAKIs registered at epoch 0: two go stale, one is refreshed.
    guard.register_trusted(0x0000_0001, 240, 0);
    guard.register_trusted(0x0000_0002, 240, 0);
    guard.register_trusted(0x0000_0003, 240, 0);

    // Touch KAKI #3 again just before the SATTATU_MAX window closes, so its
    // last_seen_epoch is refreshed and it should NOT be downgraded.
    let hdrs3 = header(b"00000003");
    let _ = guard.inspect(&hdrs3, 0, SATTATU_MAX_SECS - 1);

    let purged = guard.sentinel().purge_stale(SATTATU_MAX_SECS + 1);
    assert_eq!(purged, 2, "exactly KAKI #1 and #2 should have gone stale, not #3");

    // Re-running purge_stale immediately must not recount already-Unknown entries.
    let purged_again = guard.sentinel().purge_stale(SATTATU_MAX_SECS + 2);
    assert_eq!(purged_again, 0);
}
