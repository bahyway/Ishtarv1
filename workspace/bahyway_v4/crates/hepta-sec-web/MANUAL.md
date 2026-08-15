# HeptaSecWeb — Developer Manual

> **DubSar Help** | `Manuals > HeptaSecWeb` | Crate Reference

## Overview

`hepta-sec-web` is the **HTTP boundary adapter** for the HeptaSec sovereignty
layer.  It extracts KAKI hashes from incoming HTTP request headers and runs
them through the full `HeptaSecSentinel` inspection pipeline before any
application logic executes.

This is the enforcement point that closes the "P0 gap" identified in the
security evaluation: without this crate, the sophisticated HeptaSec firewall
architecture had no enforcement point at the actual HTTP boundary.

**Dead Axiom at HTTP:** Any request that arrives without a valid `X-Kaki-Hash`
header is `Forbidden` before it touches the application.  No routing, no
handler dispatch, no database access.

---

## Architecture Position

```text
Internet → nginx/proxy → bahyway-server
                              │
                    WebSentinelGuard::inspect()   ← hepta-sec-web
                              │
                    ┌─────────┴───────────────────────┐
                    │  HttpVerdict                    │
                    │                                 │
                    ├─ Allow { kaki_hash }  → handler │
                    ├─ Forbidden            → 403     │
                    ├─ Quarantine { .. }    → 202     │
                    └─ Redirect { to }      → 302     │
                    └─────────────────────────────────┘
```

**Framework-agnostic:** This crate has no tokio, actix-web, hyper, or HTTP
framework dependency.  It accepts raw `(&[u8], &[u8])` header pairs and
returns an `HttpVerdict` enum.  Framework integration is left to the caller.

---

## HTTP Header Contract

Every request **must** carry:

```http
X-Kaki-Hash: <8-hex-char u32>
```

Example:
```http
GET /api/particles HTTP/1.1
X-Kaki-Hash: deadbeef
```

The value is an 8-character lowercase hex string representing the caller's
KAKI `uuid_hash()` — **not** the raw minted ID.  Whitespace around the value
is trimmed.

```rust
pub const KAKI_HASH_HEADER: &str = "X-Kaki-Hash";
```

---

## HttpVerdict

| Variant                                    | HTTP response | Meaning                                    |
|--------------------------------------------|---------------|--------------------------------------------|
| `Allow { kaki_hash: u32 }`                 | pass-through  | All checks passed; forward to handler      |
| `Forbidden`                                | 403           | Dead / revoked / blocked KAKI, or absent   |
| `Quarantine { kaki_hash, reason }`         | 202 / opaque  | Unknown or suspicious KAKI held for review |
| `Redirect { to: &'static str }`            | 302           | KAKI not registered; send to identity gate |

`Forbidden` discloses **no error detail** to the caller — an attacker learns
nothing about why the request was rejected.

---

## KakiExtractor — Parsing the Header

```rust
use hepta_sec_web::KakiExtractor;

// From a raw header value byte slice:
let hash: Option<u32> = KakiExtractor::from_header_value(b"deadbeef");
// → Some(0xDEAD_BEEF)

// From a flat slice of (name, value) pairs (case-insensitive name match):
let headers: &[(&[u8], &[u8])] = &[
    (b"content-type", b"application/json"),
    (b"x-kaki-hash",  b"deadbeef"),
];
let hash: Option<u32> = KakiExtractor::from_headers(headers);
// → Some(0xDEAD_BEEF)
```

Returns `None` when:
- The header is absent
- The value is not valid UTF-8
- The value is not a valid 8-hex-char string

---

## WebSentinelGuard — Full Inspection

```rust
use hepta_sec_web::{WebSentinelGuard, HttpVerdict};

// One guard instance per server process
let mut guard = WebSentinelGuard::new();

// Pre-register internal service KAKIs at known B11 scores
guard.register_trusted(internal_service_hash, 220, now_epoch);  // Golden

// On each incoming request:
let verdict = guard.inspect(
    &headers,       // &[(&[u8], &[u8])]
    body_len,       // u32 — request body size in bytes
    now_epoch,      // u32 — current seconds since sovereign epoch
);

match verdict {
    HttpVerdict::Allow { kaki_hash } => {
        // Pass kaki_hash to the handler so it can attach to the request context
        handle_request(kaki_hash, &request)
    }
    HttpVerdict::Forbidden => {
        response.status(403)
    }
    HttpVerdict::Quarantine { kaki_hash, reason } => {
        // Log reason; return 202 or opaque ack — do not reveal reason to caller
        log::warn!("quarantine: {kaki_hash:#010x} reason={reason}");
        response.status(202)
    }
    HttpVerdict::Redirect { to } => {
        response.status(302).header("Location", to)
    }
}
```

---

## Verdict Mapping — Trust State to HttpVerdict

| HeptaSec TrustState | uuid_hash present? | `inspect()` returns              |
|---------------------|--------------------|----------------------------------|
| Header absent       | —                  | `Forbidden`                      |
| `Unknown`           | yes                | `Quarantine { "kaki_held_..." }` |
| `Suspicious`        | yes                | `Quarantine { "kaki_held_..." }` |
| `Blocked` (Dead B11)| yes                | `Forbidden`                      |
| `Revoked`           | yes                | `Forbidden`                      |
| `Active`            | yes                | `Allow { kaki_hash }`            |
| `Golden`            | yes                | `Allow { kaki_hash }`            |

---

## WEB_GATEWAY_KAKI_HASH

The guard constructs a `KakiPacket` with `dst_kaki_hash = WEB_GATEWAY_KAKI_HASH`
for every HTTP request.  This constant is `FNV-1a(b"WebGateway")` and is
computed at compile time.

```rust
pub const WEB_GATEWAY_KAKI_HASH: u32 = fnv1a_u32(b"WebGateway");
```

---

## Integration with bahyway-server

In `bin/bahyway-server`, the recommended integration pattern:

```rust
use hepta_sec_web::{WebSentinelGuard, HttpVerdict};

struct AppState {
    guard: Mutex<WebSentinelGuard>,
    // ...
}

// In the outermost request dispatch:
fn dispatch(state: &AppState, raw_headers: &[(&[u8], &[u8])], body_len: u32) {
    let now_epoch = sovereign_epoch_now();
    let verdict = state.guard.lock().unwrap().inspect(raw_headers, body_len, now_epoch);
    match verdict {
        HttpVerdict::Allow { kaki_hash } => route(kaki_hash, ...),
        _ => reject(verdict),
    }
}
```

---

## Rate Limiting at the HTTP Layer

`WebSentinelGuard::inspect()` delegates to `HeptaSecSentinel::inspect_packet()`,
which in turn uses `PolicyEngine::evaluate()`.  To activate rate limiting
(`PolicyCondition::RateExceeds`) at the HTTP layer, call the sentinel's policy
engine via `evaluate_with_rate()` with a shared `RateTracker`:

```rust
// Advanced: attach a rate tracker to the web guard
let verdict = guard.sentinel()
    .policy_engine()  // Phase 2 accessor — not yet exposed in Phase 1
    .evaluate_with_rate(&packet, trust, &mut rate_tracker, now_epoch);
```

For Phase 1, rate limiting is enforced via nginx `limit_req_zone` at the proxy
layer while `evaluate_with_rate()` is wired up in the sentinel.

---

## Dependencies

- `hepta-sec-firewall`  — `KakiPacket`, `PacketProtocol`, `FirewallVerdict`
- `hepta-sec-sentinel`  — `HeptaSecSentinel`
- `hepta-sec-policy`    — `RateTracker` (available for advanced use)
- `bahyway-core`        — `BahywayError`

No tokio, no actix-web, no hyper.

---

## See Also

- `crates/hepta-sec-sentinel/MANUAL.md`  — HeptaSecSentinel, integrated pipeline
- `crates/hepta-sec-policy/MANUAL.md`    — RateTracker, evaluate_with_rate
- `crates/enkidb-kaki/MANUAL.md`         — uuid_hash() vs minted_id() contract
- `policies/heptasec_web_access.akk`     — WAYv4.0 policy template for this layer
