# ConEngine — Connection Sovereignty Rules (CSR-01 through CSR-08)

**Standalone component reference. Follows `docs/08_pipeline_alaktu/TRANSPARENCY_STANDARD.md`.
Verified against real source (`crates/enkidb-con-engine`) and
`cargo test -p enkidb-con-engine` output on 2026-08-27 — 12 tests
passing (6 original + 6 new CSR-08 tests; CSR-08 landed this run,
closing the PB-170 gate, then corrected same-day to model its
Create/Supersede/Retire actions honestly under BahyWay's append-only
law rather than the sealed prose's literal "create, modify, delete" —
see status table below).**

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
| CSR-06 | KIBRATU Emission | connection error paths | ⚠️ the emission step itself is real, coded (stub emission) — but the specific 7-variant `KibratuCause` taxonomy some documents attach to it is not real; see `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` §K |
| CSR-07 | Tribe Isolation | `PooledConnection::send_frame()` | ✅ real, coded |
| CSR-08 | Architect Sovereignty | `csr::csr08_architect_sovereignty`, called last by `apply_all_rules` | ✅ real, coded (2026-08-27, corrected same day). Cross-cutting by design: it checks `ConContext.organ_mutation`/`architect_confirmed`, never `caller_role` — an ordinary data request (`organ_mutation: None`) is untouched ("diagnosis is autonomous"); a request proposing to append a Create/Supersede/Retire particle affecting a crate, engine, agent, template, KAKI, tribe, session, playbook, or configuration is rejected (`ConError::ArchitectConfirmationRequired`) unless `architect_confirmed` is set ("execution is the Architect's alone"). The law's own prose says "create, modify, or delete" — the code deliberately does not model a literal in-place modify or delete, since BahyWay is append-only (§0.3): `OrganAction::Supersede` is the "modify" case (a new particle citing the prior organ's KAKI, the prior organ untouched) and `OrganAction::Retire` is the "delete" case (an Event marking DEAD/retired, the organ's particles kept forever) |

`cargo test -p enkidb-con-engine`: 12 tests passing (CSR-01/02/07, NĀRU
journal verify, role ordering, NĀRU entry serialize, and 6 CSR-08 tests
covering an unconfirmed organ mutation, a confirmed one, a no-op on
ordinary data requests, the real 9 organ kinds, the real 3
append-only-honest actions, and an unconfirmed retirement).

## Known open gaps (PAZUZU threat-simulation cross-check)

Verified line-by-line against real source, not asserted:

- **PAZUZU-01** — `StubCredentialStore` accepts any zeroed blob. Confirmed, same gap as CSR-04 above.
- **PAZUZU-03** — NĀRU-SYNC is not running. Confirmed, stated in the code's own header.
- **PAZUZU-04** — No `max_connections` cap. Confirmed.
- **PAZUZU-05** — No opcode whitelist in `send_frame()`. Confirmed.

These are real, open security gaps to close before the connection layer's
posture matches what any document elsewhere claims about it — not
decorative test names. CSR-08 itself is now coded (PB-170 closed), but one
honest scope limit remains: this crate enforces the gate — it refuses an
organ mutation unless `architect_confirmed` is already `true` — it does
not itself provide the confirmation channel (a CLI prompt, a signed
approval token, a web form DUB.SAR actually clicks). Whatever sets that
boolean today is a separate, unaudited integration point; a caller that
can set `architect_confirmed: true` on its own `ConContext` has, in
effect, bypassed CSR-08's intent even though the code path is satisfied.
Closing that requires wiring a real confirmation source, not more logic
in this crate.

## Verify it yourself

```
cargo test -p enkidb-con-engine
```
