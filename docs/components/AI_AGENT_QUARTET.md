# The Sovereign AI Agent Quartet

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
Verified against real source and `cargo test` output on 2026-07-21 — this
document names the fourth, still-absent member explicitly rather than
hedging around it, per the Architect's own standing instruction that
nothing gets documented as done when it is not real yet.**

---

## The 4-member roster

| Agent | Role | Crates | Status |
|---|---|---|---|
| **TamuzAI** | Code generation / book intelligence | `ea-agent-core`, `ea-agent-algebra`, `ea-agent-chat`, `enkidullm-core`, `enkidullm-chat`, `enkidullm-memory`, `enkidullm-model`, `enkidullm-ingest`, `enkidullm-audit` | ✅ LIVE — queries EnkiDDB's real RAG (`enkidullm-memory`), real TF-IDF+cosine memory search via `adapa-recall` |
| **EaAgent** (𒂗𒆠) | Mathematical truth — GeoLaws, algebraic proof (Ea/Enki, God of Wisdom, Mathematics, Crafts) | `ea-agent-algebra`, `ea-agent-oracle` | ✅ LIVE — Particles Algebra (PA-1–16), Jordan Normal Form, Pauli Exclusion collision detection, Spectral Radius stability, BIGRING TOP (Tribe Algebra + Orbits Calculus + Particles Algebra); also queries EnkiDDB's real RAG |
| **NINSUN** (𒀭𒊩𒇻) | Healer / Progressive Refiner, advisory-only | `ninsun-agent`, `ninsun-steward-bridge` | ✅ LIVE — real, tested (`ninsun-agent::analyze()`, `NinsunAdvisoryQueue`). ESARHADDON's real SMI urgency thresholds wired into its steward-queue priority; `adapa-recall` wired into its memory search. Never modifies a committed particle, never blocks the pipeline — advisory only |
| **NuskuAgent** | Governance lamp-bearer, WAY v2.0 policy | — | ❌ **Still absent.** No crate exists. v4.3+ future, consistent across every check this session and prior ones. Do not treat any mention of NuskuAgent elsewhere as evidence it has been built |

## What "Agent Quartet" is not

`crates/agent-council::AgentId` defines a **separate, narrower**
3-member body — `{TamuzAI, Ninsun, Pazuzu}` — as "the three sovereign AI
agents on the pattern governance council," scoped specifically to
Phase-1/Phase-2 Pattern-KAKI evaluation. It overlaps on two names
(TamuzAI, Ninsun) with the 4-member ecosystem-wide Quartet above by
coincidence of purpose, not identity. Two different bodies, two
different jobs — do not conflate them.

## NINSUN's architectural boundary (the one rule that matters most)

> "NINSUN proposes, the Sovereign decides."

All refinement suggestions are emitted as `NINSUN_REFINE` EAV particles
— advisory, never corrective. No particle is ever modified in place by
NINSUN. This is enforced structurally, not by convention: `ninsun-agent`
has no write path into `enkidb_journal::Journal` at all, only an emit
path into its own advisory queue.

## Verify it yourself

```
cargo test -p ea-agent-core -p ea-agent-algebra -p ea-agent-chat \
  -p ea-agent-oracle -p enkidullm-core -p enkidullm-chat \
  -p enkidullm-memory -p ninsun-agent -p ninsun-steward-bridge \
  -p adapa-recall
```
