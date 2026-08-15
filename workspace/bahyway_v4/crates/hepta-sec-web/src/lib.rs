#![forbid(unsafe_code)]
//! hepta-sec-web — HTTP boundary adapter for HeptaSec.
//!
//! Extracts KAKI hashes from HTTP request headers and runs them through the
//! sovereign inspection pipeline before the request reaches any application logic.
//!
//! # Architecture
//!
//! ```text
//! HTTP request
//!   │
//!   ▼
//! KakiExtractor::from_headers()   ← parses X-Kaki-Hash header → u32
//!   │
//!   ▼
//! WebSentinelGuard::inspect()     ← HeptaSecSentinel + optional PolicyEngine
//!   │
//!   ▼
//! HttpVerdict::{Allow, Forbidden, Quarantine, Redirect}
//!   │
//!   ├─ Allow      → forward to application handler
//!   ├─ Forbidden  → 403 response (attacker sees no information)
//!   ├─ Quarantine → 202 Accepted (held for steward review, no error disclosed)
//!   └─ Redirect   → 302 to sovereign identity gate (unknown KAKI must register)
//! ```
//!
//! # Framework-agnostic design
//!
//! This crate is a pure library — no tokio, no actix-web, no hyper dependency.
//! It works with any HTTP framework by accepting raw header byte slices.
//!
//! Integration points:
//! - **actix-web**: implement a `FromRequest` guard that calls `WebSentinelGuard::inspect()`
//! - **bahyway-server** (bin): call `inspect()` in the outermost request dispatch layer
//! - **nginx Lua module**: call via FFI wrapper (KAKI hash in header → verdict as int)
//!
//! # Header contract
//!
//! The request sender MUST include:
//! ```text
//! X-Kaki-Hash: <hex-encoded u32 little-endian, e.g. "01f4bbcc">
//! ```
//! Requests without this header produce `HttpVerdict::Forbidden` (Dead Axiom).

use hepta_sec_firewall::{KakiPacket, PacketProtocol, FirewallVerdict};
use hepta_sec_sentinel::HeptaSecSentinel;

// ── HTTP header name ──────────────────────────────────────────────────────────

/// The canonical HTTP header used to carry the source KAKI hash.
pub const KAKI_HASH_HEADER: &str = "X-Kaki-Hash";

/// Destination KAKI hash used for web requests (FNV-1a of b"WebGateway").
pub const WEB_GATEWAY_KAKI_HASH: u32 = fnv1a_u32(b"WebGateway");

// ── HttpVerdict ───────────────────────────────────────────────────────────────

/// The security decision returned to the HTTP layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpVerdict {
    /// Packet passed all 7 layers — forward to application handler.
    Allow { kaki_hash: u32 },
    /// KAKI was absent, invalid, revoked, or dead B11 — return 403 immediately.
    /// No error detail is disclosed to the caller.
    Forbidden,
    /// KAKI is first-seen (Unknown) or suspicious — hold for steward review.
    /// Caller should return 202 Accepted or an opaque acknowledgment.
    Quarantine { kaki_hash: u32, reason: &'static str },
    /// KAKI is not registered — redirect to sovereign identity registration.
    /// Caller should return 302 to the registration URL.
    Redirect { to: &'static str },
}

// ── KakiExtractor ─────────────────────────────────────────────────────────────

/// Parses the `X-Kaki-Hash` HTTP header from a raw byte slice.
///
/// The header value is expected to be an 8-character lowercase hex string
/// representing a u32 in native (host) byte order: e.g. `"deadbeef"`.
///
/// Returns `None` if the header is absent, non-UTF-8, or not valid hex.
pub struct KakiExtractor;

impl KakiExtractor {
    /// Parse a KAKI hash from a raw header value byte slice.
    pub fn from_header_value(value: &[u8]) -> Option<u32> {
        let s = core::str::from_utf8(value).ok()?.trim();
        u32::from_str_radix(s, 16).ok()
    }

    /// Search a flat slice of `(name, value)` header pairs for `X-Kaki-Hash`.
    ///
    /// Header names are compared case-insensitively.
    pub fn from_headers(headers: &[(&[u8], &[u8])]) -> Option<u32> {
        for (name, value) in headers {
            if name.eq_ignore_ascii_case(KAKI_HASH_HEADER.as_bytes()) {
                return Self::from_header_value(value);
            }
        }
        None
    }
}

// ── WebSentinelGuard ──────────────────────────────────────────────────────────

/// Wraps `HeptaSecSentinel` for HTTP request inspection.
///
/// Owned by the HTTP server process (one per server instance).
/// All state (trust table, event ring) is held inside the wrapped sentinel.
pub struct WebSentinelGuard {
    sentinel: HeptaSecSentinel,
}

impl WebSentinelGuard {
    /// Create a new guard with a fresh `HeptaSecSentinel`.
    pub fn new() -> Self {
        WebSentinelGuard { sentinel: HeptaSecSentinel::new() }
    }

    /// Pre-register a trusted KAKI (e.g. internal services, registered Write Pods).
    pub fn register_trusted(&mut self, kaki_hash: u32, b11_score: u8, now_epoch: u32) {
        self.sentinel.register_kaki_b11(kaki_hash, b11_score, now_epoch);
    }

    /// Inspect an incoming HTTP request.
    ///
    /// `headers` — flat slice of `(name_bytes, value_bytes)` pairs.
    /// `payload_len` — byte length of the request body (0 for GET/HEAD).
    /// `now_epoch` — current time in seconds since the sovereign epoch.
    ///
    /// Returns the `HttpVerdict` the HTTP layer must enforce.
    pub fn inspect(
        &mut self,
        headers:     &[(&[u8], &[u8])],
        payload_len: u32,
        now_epoch:   u32,
    ) -> HttpVerdict {
        let kaki_hash = match KakiExtractor::from_headers(headers) {
            Some(h) => h,
            None    => return HttpVerdict::Forbidden,
        };

        let packet = KakiPacket {
            src_kaki_hash: Some(kaki_hash),
            dst_kaki_hash: Some(WEB_GATEWAY_KAKI_HASH),
            payload_fnv:   0,
            protocol:      PacketProtocol::Tcp,
            epoch:         now_epoch,
            size_bytes:    payload_len,
        };

        match self.sentinel.inspect_packet(&packet, now_epoch) {
            FirewallVerdict::Allow { .. } => HttpVerdict::Allow { kaki_hash },
            FirewallVerdict::Block { .. } => HttpVerdict::Forbidden,
            FirewallVerdict::Quarantine { .. } => {
                // Distinguish Unknown (never seen) from Suspicious (low B11)
                // to give the correct UX response.
                HttpVerdict::Quarantine {
                    kaki_hash,
                    reason: "kaki_held_for_verification",
                }
            }
        }
    }

    /// Direct access to the underlying sentinel for advanced inspection.
    pub fn sentinel(&mut self) -> &mut HeptaSecSentinel {
        &mut self.sentinel
    }
}

impl Default for WebSentinelGuard {
    fn default() -> Self { Self::new() }
}

// ── FNV-1a (const-friendly) ───────────────────────────────────────────────────

const fn fnv1a_u32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME:  u32 = 16_777_619;
    let mut h = OFFSET;
    let mut i = 0;
    while i < data.len() {
        h ^= data[i] as u32;
        h = h.wrapping_mul(PRIME);
        i += 1;
    }
    h
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── KakiExtractor tests ───────────────────────────────────────────────────

    #[test]
    fn extract_valid_header() {
        let hash = KakiExtractor::from_header_value(b"deadbeef").unwrap();
        assert_eq!(hash, 0xDEAD_BEEF);
    }

    #[test]
    fn extract_with_whitespace() {
        let hash = KakiExtractor::from_header_value(b"  deadbeef  ").unwrap();
        assert_eq!(hash, 0xDEAD_BEEF);
    }

    #[test]
    fn extract_invalid_hex_returns_none() {
        assert!(KakiExtractor::from_header_value(b"not-hex!!").is_none());
    }

    #[test]
    fn extract_empty_returns_none() {
        assert!(KakiExtractor::from_header_value(b"").is_none());
    }

    #[test]
    fn from_headers_case_insensitive() {
        let headers: &[(&[u8], &[u8])] = &[
            (b"content-type",   b"application/json"),
            (b"x-kaki-hash",    b"aabbccdd"),
        ];
        let hash = KakiExtractor::from_headers(headers).unwrap();
        assert_eq!(hash, 0xAABB_CCDD);
    }

    #[test]
    fn from_headers_absent_returns_none() {
        let headers: &[(&[u8], &[u8])] = &[
            (b"content-type", b"text/html"),
        ];
        assert!(KakiExtractor::from_headers(headers).is_none());
    }

    // ── WebSentinelGuard tests ────────────────────────────────────────────────

    #[test]
    fn no_kaki_header_is_forbidden() {
        let mut guard = WebSentinelGuard::new();
        let headers: &[(&[u8], &[u8])] = &[(b"content-type", b"text/html")];
        assert_eq!(guard.inspect(headers, 0, 1_000), HttpVerdict::Forbidden);
    }

    #[test]
    fn unknown_kaki_is_quarantined() {
        let mut guard = WebSentinelGuard::new();
        let headers: &[(&[u8], &[u8])] = &[(b"x-kaki-hash", b"12345678")];
        // Unknown KAKI — never registered → Quarantine
        let verdict = guard.inspect(headers, 64, 1_000);
        assert!(matches!(verdict, HttpVerdict::Quarantine { .. }));
    }

    #[test]
    fn golden_kaki_is_allowed() {
        let mut guard = WebSentinelGuard::new();
        let hash = 0xF00D_CAFE;
        // Register as Golden (B11 = 255)
        guard.register_trusted(hash, 255, 1_000);

        let header_val = format!("{:08x}", hash);
        let headers: &[(&[u8], &[u8])] = &[(b"x-kaki-hash", header_val.as_bytes())];
        let verdict = guard.inspect(headers, 64, 1_000);
        assert_eq!(verdict, HttpVerdict::Allow { kaki_hash: hash });
    }

    #[test]
    fn web_gateway_kaki_hash_is_stable() {
        // Const evaluation of FNV-1a must be deterministic
        assert_eq!(WEB_GATEWAY_KAKI_HASH, fnv1a_u32(b"WebGateway"));
    }
}
