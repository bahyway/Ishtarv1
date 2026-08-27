# GL-NSR-001-A1 — The Temennu Baseline & The Rigmu Escalation
## Amendment to the Markasu Alert Law: judged against the foundation, and the great cry on GOLDEN

**Ecosystem:** BahyWay.Ecosystem v4.0 — Nasaru Diagnosis Instrument
**Amends:** GL-NSR-001 (MARKASU-01) · consumes GL-ORG-001, GL-STY-001, GL-DST-003 (Madanu), GL-ALG-001-A2 (Kidinnu), Analysis-to-Solution Law
**Status:** SEALED — landed by PB-331 as `crates/markasu` (extends GL-NSR-001's crate), 10/10 tests passing (L5-L8 this amendment's own tests, plus L1-L4 from the base law in the same crate).
**Author:** DUB.SAR 𒁾

**Names.** *Temennu* — the foundation deposit, the inscribed document laid beneath a temple's foundations against which all later builders are judged. *Rigmu* — the cry, the clamor, the great noise. Both unspent on the ledger.

---

## 1. The Temennu Baseline (§B, replaces fixed θ_min)

A universal θ_min is unlawful: every tribe has its own natural stiffness. Each watched tribe receives a **Temennu**:

- **Laying the foundation.** During an enrollment window of E healthy windows (sanity-bounded, no MARKASU/FUZZY episodes), record the agreed reversion series θ̂ (accepted only when witnesses A and B agree within tolerance: |θ̂_A − θ̂_B| ≤ δ·θ̂_A). The Temennu is **θ₀ = median(θ̂)** with spread **s₀ = MAD(θ̂)** — robust, outlier-proof.
- **Sealing.** The Temennu is Kidinnu-sealed (Ed25519): tribe_id, θ₀, s₀, E, enrollment span, estimator versions. It is a *document*, not a variable.
- **Immutability.** The Temennu is re-laid **only by decree** (Madanu court or Shakkanakku survey with Architect seal). Automatic re-baselining is forbidden — silent baseline creep is how a slackening system teaches its own watchman to sleep.

## 2. The Two Alert Witnesses, re-founded (§C)

MARKASU keeps PROVE-form but the witnesses become *relative and directional*:

- **LEVEL witness:** θ̂ < κ·θ₀ beyond ε (default κ = 0.5) — the rope is at half its founded stiffness.
- **TREND witness:** Mann–Kendall statistic over the last M accepted windows significantly negative (Z ≤ −z_crit, default z_crit = 1.96), with **Sen's slope** reported as the decline rate and the horizon **T_θ = (θ̂ − κθ₀)/|slope|** journaled: *time until the level witness fires at current decay* — the trend analysis you asked for, as a countdown.

MARKASU-01 fires iff **LEVEL ∧ TREND** for W consecutive evaluations. Trend alone → journaled FUZZY watch (early-early warning); level alone without trend → suspect step-change, journaled FUZZY with a data-integrity flag (a cliff is more often a pipeline fault than a slackening spring).

## 3. The Rigmu Escalation (§D — the great cry on GOLDEN)

Severity is decided by the **state class of the tribe's particles**:

- **FUZZY tribes → MARKASU (bell).** Ordinary early warning; Nisaba detects, Kittu delivers, work proceeds.
- **GOLDEN tribes of EnkiDB·7004 or EnkiDW·7005 → RIGMU (the great cry).** GOLDEN is settled truth; its mooring does not slacken innocently. RIGMU therefore carries obligations, not just delivery:
  1. **Freeze:** Gate G4 promotions in the affected tribe halt; the feeding stream is flagged for EnkiQDB·7003 quarantine review.
  2. **Explanation obligation — no silent close.** A RIGMU cannot be acknowledged, only *answered*: it remains open until an **explanation particle** is attached — root cause from the sealed list {upstream corruption · unauthorized write · world-change (golden gone stale) · estimator/pipeline fault} — itself two-witness (evidence + NUZI lineage comparison locating *when* the drift began in the archive) and Architect- or Madanu-signed.
  3. **ŠĀRU:** dual reporting per TIAMAT doctrine; an unexplained RIGMU is itself a transparency deficit and accrues τ.
- **DEAD tribes:** no mooring watch — the dead are not moored.

## 4. Playbook

- **PB-331** — `nasaru/markasu` extension: Temennu enrollment (median/MAD, witness-agreement gate), Mann–Kendall + Sen slope, T_θ horizon, severity router (FUZZY→MARKASU, GOLDEN→RIGMU with freeze flag and open-until-explained state machine). Law tests: **L5** Temennu never auto-relays (creep attempt → unchanged seal); **L6** trend-only or level-only never rings; **L7** step-change fires the integrity flag, not the bell; **L8** RIGMU refuses to close without a signed explanation particle.

## 5. Seal

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
