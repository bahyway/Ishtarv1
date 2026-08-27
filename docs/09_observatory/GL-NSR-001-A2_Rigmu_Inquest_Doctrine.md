# GL-NSR-001-A2 — The Rigmu Inquest Doctrine
## The Standard HeptaScript Investigation Every Great Cry Opens

**Ecosystem:** BahyWay.Ecosystem v4.0 — Nasaru Diagnosis Instrument · HeptaScript (Anti-SQL, W5H2)
**Amends:** GL-NSR-001 + A1 · consumes NUZI (archive/lineage), GL-STY-001 (NĀRU), GL-ONT-001 (OntoGraph harvest), GL-DST-003 (Madanu), TIAMAT τ doctrine
**Status:** SEALED — landed by PB-332 as `crates/nasaru-inquest`, 5/5 tests passing (L9-L12 plus one leading-axis sanity test).
**Author:** DUB.SAR 𒁾

---

## 1. Purpose

A RIGMU may not be answered with opinion. This doctrine seals the **Inquest**: the standard W5H2 investigation that opens automatically with every great cry, phrased in the five sovereign operations, and the only lawful path to closing the case. Where a Markov account is memoryless by construction, the Inquest is memoryful by construction: KAKI addresses, NĀRU witnesses, NUZI lineage — the null model may forget; the verdict may not.

## 2. The W5H2 Frame (mandatory fields of the explanation particle)

| Field | Question | Instrument |
|---|---|---|
| **WHEN** | Onset of departure from the Temennu | Bisection of the departure predicate against NUZI snapshots — O(log n) archive reads |
| **WHO** | Actors writing to the tribe in the bracket [onset−Δ, onset+Δ] | Lineage sweep, ranked by write mass inside the bracket |
| **WHAT** | What changed: spine vs world | Mandatory-EAV diff at onset (NUZI twin) vs world-witness check |
| **WHERE** | Which Hepta dimension slackened first | Per-dimension onset; the leading axis names the wound (Integrity? Temporal? Quality?) |
| **WHY** | Root cause — the conviction | Two-witness from the sealed list: upstream corruption · unauthorized write · world-change (golden stale) · pipeline fault |
| **HOW** | Mechanism narrative | Sen slope, decay signature, lineage path |
| **HOW MUCH** | Magnitude & horizon | θ̂/θ₀ ratio, T_θ countdown, τ accrued while open |

An explanation particle missing any field **cannot be minted** (kernel-enforced, L11).

## 3. The Five Sovereign Operations of the Inquest

```
ORBIT   tribe:<id> SPAN temennu..now            → the trajectory, no bending
PROVE   WHEN  onset BY bisection AGAINST nuzi   → sustained-departure predicate, log₂ reads
PROVE   WHO   BY lineage-sweep BRACKET onset±Δ  → ranked actors, write mass
PROVE   WHY   root-cause WITH two-witness       → evidence + NUZI lineage concord
WITNESS nāru  CASE <rigmu-id>                   → every act journaled as it happens
EMIT    explanation-particle KAKI <addr>        → W5H2-complete or refused
SYNC    case CLOSED → unfreeze GateG4           → only after signed conviction
```

No SQL exists here and none may be smuggled in: the Inquest is spoken only in the five operations.

## 4. Clauses

- **Onset before opinion.** WHEN is established first, by bisection, before any WHO is examined — anchoring bias is a named breach.
- **The bracket is symmetric.** WHO sweeps [onset−Δ, onset+Δ]: writes *after* onset can be concealment, not only cause.
- **The leading axis is evidence, not verdict.** WHERE (first-slackening Hepta dimension) informs WHY; it never substitutes for the two-witness conviction.
- **Wrong convictions are refused, not corrected.** A proposed WHY whose witnesses do not concord is rejected with the discord journaled; the case stays open and τ keeps accruing.
- **Signature is the last act.** Close requires: W5H2 complete + two-witness WHY + Architect/Madanu signature. Any lesser close attempt is refused and journaled (L12/L8).
- **The harvest.** Every closed explanation particle enters OntoGraph's Optional-EAV harvest (`inq.root_cause`, `inq.onset`, `inq.lead_axis`, `inq.actor_bracket`) — FCA closure over accumulated inquests is where hidden patterns in GOLDEN failures crystallize (Nebuchadnezzar over explanations). The Mandatory spine is untouched.

## 5. Playbook

- **PB-332** — `nasaru/inquest` kernel: sustained-departure predicate, onset bisection (= linear scan, proven), symmetric lineage sweep with actor ranking, per-dimension onset for the leading axis, W5H2-complete builder that refuses partial minting, close gate. Law tests **L9–L12**.

## 6. Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-27
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work.
```
