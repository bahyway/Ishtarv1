# GL-NRG-001-A2 — The BabTurdiEngine
## Semantic Cost-Attribution: What a Client's Refused Data Costs to Defend Against

**Ecosystem:** BahyWay.Ecosystem v4.0 — Nergal AV outer-ledger analytics
**Consumes:** GL-BRT-001 (Birth Gate, Bāb Ṭurdi log), GL-NRG-001-A1 (Nāru forensics), GL-NSR-001 (Nasaru τ), GL-TPL-001 (minting), SLA/reward scoring
**Status:** DRAFT — awaiting Architect seal (CSR-08). **Name:** *Bāb Ṭurdi* — the Gate of the Turned-Away; its Engine reads their ledger.

---

## 1. The Boundary That Makes This Lawful (read first)

A tempting but **forbidden** inference: that a born Particle near a strained patch is complicit in, contaminated by, or guilty of the outside attack. It is not. A Particle passed the Birth Gate — KAKI, clean colophon, CONCORD standing. **Proximity to a siege spot is geometry, not guilt**: the attacker chose that arc; the Particle merely lives there.

**The No-Contamination-Inference Clause (sealed):** the BabTurdiEngine MUST NOT draw any corruption-, guilt-, or contamination-link from a Non-Particle to a Particle. It correlates client refusal-streams with defense cost — entirely on the outer (SUSA) ledger — and attributes that cost to the **client**, never to any inner Particle. Spatial coincidence between an inner Particle and an outer siege spot is reported as *the attack's chosen target*, never as the Particle's fault. Violating this clause is a breach that accrues τ and is inadmissible to any reward decision.

## 2. What the Engine Lawfully Establishes

Semantic vector search over the Bāb Ṭurdi log (refused Records, no KAKI) clusters a client's refused submissions by **meaning**, yielding a per-client **cost signature**:

- **Concentration:** does this client's refused data mass on the hardest-hit ingestion spot? (measured on the sheet, outside the wall)
- **Severity mix:** what fraction is malware-payload vs. mere schema/SLA formatting? (semantic cluster shares)
- **Defense cost imposed:** Gate adjudication load + Nergal siege-response cost attributable to this client's stream, with τ.

The lawful claim: *"Client C's refused stream concentrates on the strained spot, is p% malware-signature, and imposes cost K (±τ) — measurably higher than the cohort median."* This is a statement about the **client's outside behavior**, never about any inner Particle.

## 3. The SLA / Reward Consequence

Reward tiers (low-service-cost) are **earned by measured low cost**, not granted by default. If the BabTurdiEngine attributes above-median defense cost to a client's refusal-stream, that client's **cost basis rises** and the low-cost reward tier is **not earned** — not *denied by accusation*. The distinction is legal and moral: the client is not punished for wrongdoing; they are billed for the measured, journaled, falsifiable cost their submissions impose on the Gate and the wall. The client may lower it by sending cleaner data — the lever is named, as always.

**Never-Averaged corollary (from Civil Protection Calculus):** a client's cost signature is reported with its distribution and τ, never collapsed to a single blaming number. A quiet month and a siege month are not averaged into a false "medium."

## 4. Two Ledgers Stay Separate (enforced)

The engine reads ONLY the SUSA Bāb Ṭurdi ledger. It never joins to the NUZI provenance store — no query may return an inner KAKI alongside an outer refusal record. The cost signature is built from transport metadata + reason vectors alone. (Enforced by GL-BRT-001 §4 key-space disjunction.)

## 5. Playbook

- **PB-348** — BabTurdiEngine, landed at `workspace/bahyway_v4/crates/babturdi-engine` (flat crate convention, workspace member), `cargo test -p babturdi-engine` 4/4 passing (2026-08-21; host `uruk`, not the draft's `bahyway_host`). Semantic clustering of a client's refused stream, spot-concentration measure, defense-cost attribution with τ, reward-tier gate. Law tests **L61** (no Particle KAKI ever appears in a cost-attribution output — the No-Contamination boundary), **L62** (cost attributed to client with τ, never a bare number), **L63** (above-median cost → reward tier not earned; the client can lower it by cleaner data), **L64** (a client's quiet-vs-siege costs are never averaged). One real bug found and fixed during landing: `cohort_median` took the *upper* of the two middle values for an even-sized cohort, which for a 2-client cohort collapses the median to the costlier client's own cost — that client's cost was never `> its own cost`, so L63 always passed it as `LowCostEarned` regardless of how costly it was. Fixed to average the two middle values, the standard even-count median.

## 6. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
