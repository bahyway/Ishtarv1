# GL-NSR-001 — The Nasaru Alert Law
## Founding Alert MARKASU-01: The Slackening Mooring (Ornstein–Uhlenbeck Reversion Watch)

**Ecosystem:** BahyWay.Ecosystem v4.0 — Nasaru Diagnosis Instrument
**Domain:** GL-NSR (Nasaru alerts) — consumes GL-ORG-001 (Homeostasis), GL-TPL-002 (Living Shape & Drift), GL-ALG-003 (Abūbu), GL-STY-001 (NĀRU Journal); detection per Nisaba, delivery per Kittu
**Status:** SEALED — landed by PB-330 as `crates/markasu`, 10/10 tests passing (L1-L4 the base alert law + L5-L8 the A1 Temennu/Rigmu amendment, same crate). Bell placement (§4: fourth bell vs. sub-bell of Balance) remains an open Igigi Watch decree, not resolved by this seal.
**Author:** DUB.SAR 𒁾

**Name.** *Markasu* — the mooring rope, the bond; the cosmic *markas šamê u erṣeti*, "bond of heaven and earth." A healthy tribe is moored to its center; the alert fires when the mooring slackens — before the tribe has visibly moved. The name is unspent on the ledger.

---

## 1. Purpose — the first alert

Nasaru has courts that judge *positions* (Poisson nulls, K(r), Betti). This law seals its first alert on *motion*: the early warning that homeostasis is failing while position still looks healthy. It is the instrument's first bell rung on a **prediction**, not an observation — DETECT→PROVE→**PREDICT** made operational.

## 2. The Mooring Model

A healthy tribe's centroid X(t) around its Apsu center μ is Ornstein–Uhlenbeck:

  **dX = θ (μ − X) dt + σ dW**

- **θ** — the reversion rate: the stiffness of the mooring. Relaxation time τ_r = 1/θ.
- **σ** — the huburu amplitude (ḫb-calibrated; pure Brownian noise is the σ-only limit).
- Stationary spread: **Var(X) = σ²/(2θ)** — the slack rope shows first as swelling variance.

Regime map (aligned with GL-ALG-003): θ ≥ θ_min → elastic / homeostasis · θ falling below θ_min → plastic / drift-in-formation · θ → 0 → **unmoored**: free Brownian motion, MSD linear forever, Abūbu-adjacent.

## 3. The Two Witnesses (PROVE-form, mandatory)

No alert on a single estimator. Over a rolling window Δ-sampled:

- **Witness A — the rope's memory:** lag-1 autocorrelation φ̂ of the centered series; **θ̂_A = −ln φ̂ / Δ**.
- **Witness B — the rope's slack:** increment variance σ̂² = ⟨ΔX²⟩/Δ against sample variance; **θ̂_B = σ̂² / (2 · Var̂(X))**.

MARKASU-01 fires iff **both** θ̂_A < θ_min and θ̂_B < θ_min, each beyond its estimator ε (delta-method), for W consecutive windows (default W=3). One witness alone → FUZZY observation, journaled, no bell. The early-warning claim is sealed as a law test: the alert must precede any excursion of |X−μ| beyond the healthy band (L2 below).

## 4. Verdict, custody, delivery

On firing: verdict particle minted (KAKI-addressed) carrying θ̂_A, θ̂_B, ε_A, ε_B, τ_r, window count, regime tag; NĀRU-witnessed (GL-STY-001); detection stands in **Nisaba**, delivery rides **Kittu** to the decreed recipients; escalation to Madanu if the tribe must be held at r\*. Optional EAV harvest only (`nsr.theta_a`, `nsr.theta_b`, `nsr.tau_r`, `nsr.regime`); the Mandatory spine is untouched.

**Bell placement — CSR-08 decision required:** Igigi Watch holds the sealed Three Bells (Shape, Balance, Breath). MARKASU rings either as a **fourth bell — the Mooring** — or as a sub-bell of Balance. This tablet does not amend GL-ORG-001/Igigi doctrine unilaterally; the Architect decrees.

## 5. Playbook

- **PB-330** — `nasaru/markasu` kernel (pure Rust, zero deps): OU simulator (exact discretization X⁺ = μ + e^{−θΔ}(X−μ) + σ√((1−e^{−2θΔ})/2θ)·ξ), rolling two-witness estimators, alert state machine (HEALTHY → FUZZY → MARKASU), law tests:
  - **L1** — healthy tribe (θ = 2·θ_min), long run: zero alerts (false-bell rate bound).
  - **L2** — scripted slackening (θ decays): the bell rings **while |X−μ| is still inside the healthy band** — early warning proven, not asserted.
  - **L3** — estimator consistency: θ̂_A, θ̂_B → θ on long stationary series within ε.
  - **L4** — two-witness enforced: corrupt one witness → FUZZY only, never a bell.

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
