# ConEngine — Connection Sovereignty Rules (CSR-01 through CSR-08)

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
Verified against real source (`crates/enkidb-con-engine`) and
`cargo test -p enkidb-con-engine` output on 2026-07-21 — 10 tests
passing.**

---

## What ConEngine is

`enkidb-con-engine` (`crates/enkidb-con-engine`) is the Sovereign
Connection Engine: named for the Sumerian concept of CSR (Connection
Security Rules), it enforces passport validation, role checks, audit
journaling, credential expiry, cross-tribe gating, KIBRATU event
emission, and tribe isolation on every connection request. Real
modules: `error`, `roles` (`SovereignRole`), `audit` (`NaruEntry`,
`NaruJournal`), `csr` (`ConContext`, `Operation`, `apply_all_rules`),
`pool` (`ConnectionPool`, `PooledConnection`).

## The 8 CSR rules — real status, not aspirational

| Rule | Name | Enforcement point | Status |
|---|---|---|---|
| CSR-01 | Sargon Gate | `ConEngine::boot()` + `query()` | ✅ real, coded |
| CSR-02 | Role Gate | `SessionRegistry::resolve()` | ✅ real, coded |
| CSR-03 | NĀRU Frame Journal | `PooledConnection::send_frame()` | ✅ real, coded |
| CSR-04 | Credential Check | `CredentialStore` trait | ⚠️ real trait, coded — but the only implementation is `StubCredentialStore`, which accepts any zeroed blob unconditionally. Real `AkkadiSafeEngine` wiring is still pending |
| CSR-05 | Gilgamesh Gate | `ConEngine::query()` | ✅ real, coded |
| CSR-06 | KIBRATU Emission | connection error paths | ⚠️ the emission step itself is real, coded (stub emission) — but the specific 7-variant `KibratuCause` taxonomy some documents attach to it is not real; see `docs/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` §K |
| CSR-07 | Tribe Isolation | `PooledConnection::send_frame()` | ✅ real, coded |
| CSR-08 | Architect Sovereignty | **All agents** — cross-cutting, not a single ConEngine code path | ❌ sealed as governance law, confirmed word-for-word by two independent sources — **not yet coded.** `enkidb-con-engine/src/csr.rs` implements CSR-01 through CSR-07 only |

`cargo test -p enkidb-con-engine`: 10 tests passing (CSR-01/02/07, NĀRU
journal verify, role ordering, NĀRU entry serialize).

## Known open gaps (PAZUZU threat-simulation cross-check)

Verified line-by-line against real source, not asserted:

- **PAZUZU-01** — `StubCredentialStore` accepts any zeroed blob. Confirmed, same gap as CSR-04 above.
- **PAZUZU-03** — NĀRU-SYNC is not running. Confirmed, stated in the code's own header.
- **PAZUZU-04** — No `max_connections` cap. Confirmed.
- **PAZUZU-05** — No opcode whitelist in `send_frame()`. Confirmed.

These are real, open security gaps to close before the connection layer's
posture matches what any document elsewhere claims about it — not
decorative test names. CSR-08 (governance sovereignty) being uncoded is
the standing, still-open PB-170 gate.

## Verify it yourself

```
cargo test -p enkidb-con-engine
```
