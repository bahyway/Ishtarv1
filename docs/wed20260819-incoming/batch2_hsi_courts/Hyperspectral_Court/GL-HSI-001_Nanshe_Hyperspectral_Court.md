# GL-HSI-001 — The Nanshe Court
## The Hyperspectral Analyzing Engine: reading hidden light to foretell the leak

**Ecosystem:** BahyWay.Ecosystem v4.0
**Consumes:** GL-AGE-001 (two-witness doctrine) · GL-VSL-001-A1 (Lilu F2 HSI frontend, law L44) · GL-VIZ-001 (NASARU style, via Nasaru Visualization Instrument) · GL-ONT-001 (OntoGraph) · Igigi Watch doctrine (alarm on acceleration, not weather) · GL-LBR-001 (Labīru: truth has a timestamp)
**Gated behind:** PB-345 (WPDEngine Legacy Closure) — CONCORD colophon required
**Status:** DRAFT — awaiting Architect seal (CSR-08)

---

## 1. The Name — proposed, not sealed

**Nanshe** (𒀭𒀏) — goddess of divination through water, interpreter of dreams,
reader of signs hidden from ordinary sight. Register note, deposited honestly
per the Lilu precedent: Nanshe is god-class (lawful for an engine under NL-001),
her domain is *water* divination specifically — apt for the founding use case,
and read broadly (she reads what the surface conceals) she covers gas, oil, and
hydraulic domains without strain. Registry form: `Nanshe` — complete unbroken
Latin word, no diacritics. Ḫubullu gloss (GL-NAM-002): **"the hidden-sign
reader"** — plain language: *the engine that reads light humans cannot see and
foretells the leak months before it surfaces.*

## 2. The Particle Law

Every hyperspectral cube entering through SUSA is corrected (radiometric /
atmospheric), segmented into superpixel regions, and each region is **born as a
particle**:

- **Position** — the region's mean spectrum projected onto the top-7 principal
  components of the healthy subspace: the **Hepta shadow** (identical mapping to
  Lilu frontend F2, law L44). Real-valued, unique per particle — the Hepta Space
  Uniqueness Law holds.
- **Identity** — KAKIv4.0 minted at birth: κ[0..3] uuid_hash from region seal ·
  κ[4..5] tribe_id from network domain (Water / Gas / Oil / Hydraulic — four
  tribes) · κ[6] kaki_type = HSI-region · κ[7] kaki_role · κ[12..13] epoch
  timestamp · κ[14..15] CRC-16/CCITT.
- **Route** — SUSA → EnkiSDB (7001) → full gate chain → EnkiODB, sole write node
  `enkidb-ingest::bridge`, KISPU micro-batch commit boundary respected.

## 3. The Healthy Subspace Law (the Labīru clause)

The subspace **P** (top-7 eigenvectors of the healthy band covariance Σ) is
estimated **only** from epochs over corridors with no incident on record, and it
is itself a mortal truth: P carries a timestamp and a NUZI origin deposit.
Re-estimation is lawful only by decree — never silently — and every verdict
cites the subspace epoch it was judged against (LAW-HOT-3 lineage: *a judgment
cites the epoch, never "the cache"*). Covariance accumulates incrementally
(rank-one per KISPU batch); eigendecomposition is deterministic Jacobi on the
symmetric Σ — identical sealed input → identical eigenvectors to the bit
(reproducible-truth clause).

## 4. The Two-Witness Verdict Law

Two independent geometric witnesses, per GL-AGE-001:

- **T² (Hotelling)** — Mahalanobis distance *within* P: drift from the healthy
  centroid inside the known world.
- **Q (SPE)** — squared residual *orthogonal* to P: ‖x − PPᵀx‖² — spectral
  content the healthy world cannot reconstruct. Novel leak chemistry appears
  here first.

**No verdict fires on one witness.** A verdict requires T² AND Q above their
control limits, or one witness sustained across two consecutive epochs.
A single-epoch, single-witness excursion is weather.

## 5. The Horizon Law (T_leak)

Prophecy comes from trajectory, never snapshot. Per particle, across epochs:

- **δc** — Hepta-shadow displacement between epochs (Šību reuse, weight
  discipline inherited from GL-AGE-001).
- **dQ/dt and d²Q/dt²** — velocity and acceleration of the residual.

The Igigi bell rings on **acceleration**: a wet week moves Q; a developing leak
accelerates it epoch over epoch. A sustained two-witness ramp is fitted and the
threshold intercept yields **T_leak** — weeks until surface expression — the
Enbilulu horizon pattern (T_j) generalized from junctions to spectral regions.
Target lead time: ≥ 12 weeks, requiring revisit cadence of at most monthly,
ideally biweekly (deposited honestly: the three-month claim rests entirely on
temporal density of imagery).

## 6. Verdicts

| Verdict | Condition | Route |
|---|---|---|
| **NANSHE-OMEN** | two-witness sustained ramp, T_leak finite | signed Alert Event KAKI via Nisaba (detect/declare) → Kittu (deliver) |
| **EnkiQDB-QUARANTINE** | corrupt/ambiguous spectrum (cloud shadow, sensor artifact, gate breach) | EnkiQDB (7003) — evidence-grade, **never** a leak verdict |
| **NUZI-ARCHIVE** | ramp resolves / region healthy across window | NUZI deposit with full lineage; feeds lawful subspace re-estimation |

Age or drift alone never quarantines; quarantine requires breach co-occurrence
(GL-AGE-001 clause, restated).

## 7. Visualization

Nanshe emits scenes, never draws them: verdict particles compile through the
**Lilu F2 frontend** (bands → particles, seven-band Hepta shadow) into the
Nasaru Visualization Instrument; leak plumes classify as ḪARRĀNU roads and
contamination blooms as KIRṢU findings per the Ṣalmu Registry (GL-SHP-001).
Šala HTML rehearsals only; production is egui/WGPU on the Fedora bare-metal
host, Vulkan backend. Lilu Verb Law holds: the visualization may witness,
rehearse, and propose — it may never touch the verdict.

## 8. Law Tests (sealed with PB-346)

- **L-HSI-1** — single witness, single epoch → NO verdict fires (weather clause)
- **L-HSI-2** — planted two-witness ramp → NANSHE-OMEN with finite T_leak
- **L-HSI-3** — deterministic subspace: sealed input → bit-identical eigenvectors
- **L-HSI-4** — Hepta shadow: N regions → exactly N particles, positions unique
- **L-HSI-5** — corrupt spectrum → EnkiQDB-QUARANTINE, never NANSHE-OMEN
- **L-HSI-6** — every verdict cites subspace epoch (no-cache-judgment clause)

## 9. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
```
