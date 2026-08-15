# 𒁾 CASE-IQ-001 — PIQITTU 𒉺𒁲: THE MUSTER
### Evidence-Grade Detection of Ghost Employees & Ghost Pensions ("الفضائيون — the spacemen") in National Payroll
### A BĀRÛTU deployment blueprint · Engine candidate: PiqittuEngine · Implementation playbook proposed: PB-303
### Status: DRAFT — unsealed until the Architect's ceremony (CSR-08)

*piqittu (Akkadian): the muster — the administrative inspection of personnel and herds.
Mesopotamian clerks ran musters because payrolls have grown ghosts for four thousand years.
This blueprint runs the same muster with sealed instruments. A ghost passes every payday
and fails every muster.*

---

## 1 · The Problem, Stated in the Ecosystem's Language

A ghost employee or ghost pensioner is an account that draws rations (B*) without a living
person (S) behind it. Three canonical species, each a coordinate on the Omen Table:

| Species | Signature | Omen coordinate |
|---|---|---|
| **The Perfect Ghost** | draws salary in flawless rhythm; generates **zero life-noise** — no leave, no sickness, no biometric events, no variance | RING × *inverse* LOPSIDED — see §3.1: the too-perfect account |
| **The Unreconciled Dead** | pension continues after the person entered the death registry | S/S′ CROSSING × KOMMA DISHONORED — the Living and Dead ledgers failing to meet |
| **The Double-Drawn** | one person (one biometric identity) drawing from two or more ministries | SECOND TRIBE across ministry orbits (ORBIT PAIR violation) |

The famous Iraqi audits (the 2014 discovery of ~50,000 ghost soldiers; subsequent
biometric campaigns) are this blueprint's **calibration cells** — documented history,
not hypothesis.

## 2 · Approved Measurement Units (the URUK Registry, applied)

Per GL-MET-001, every quantity is typed; nothing is a bare number:

- **S — the Living**: the verified personnel/pensioner roster. One particle per person,
  keyed to biometric identity where enrolled.
- **S′ — the Register of the Dead**: the civil death registry as first-class particles.
  *A dead person is not a deleted employee* — S′ entries are counted beings, never
  decrements of S.
- **B\* — the Rations**: salary and pension allocations. Payroll is the ration system of
  the state; the archaic scribes counted it in B\* and so do we.
- **U — Calendrics**: service time, paydays, muster intervals; all times carried as
  **adannu/kašādu** pairs (NIPPUR 5.2).
- **E — Weights → IQD**: currency is one system among many (GL-MET-001 M-6). Every
  B\*→IQD expression crosses a declared bridge and logs its KAKI. No report may present
  IQD as the root unit of the case; the root units are persons (S), deaths (S′), and
  rations (B\*).
- **η — huburu**: the liveness-noise unit (PB-300 lineage). The behavioral variance a
  living person cannot help generating. **The unit of being alive.**

## 3 · The Instruments (ZIBĀNĪTU, strapped)

### 3.1 The Liveness Gauge — "only ghosts are perfect"

For each account, over a sliding window of W paydays and the surrounding administrative
stream (attendance, leave, sickness, promotions, transfers, biometric touches):

- **Regularity R** ∈ [0,1]: order parameter of the payment rhythm (a payroll should be
  near-perfect for everyone — this axis alone accuses no one).
- **Liveness η** in huburu: normalized variance of life-signals around the account.
  Living humans: η well above floor. Ghosts: η ≈ 0.

**The Piqittu score**: `Π = R · (1 − η/η₀)` clipped to [0,1], with η₀ the sealed liveness
floor from calibration. High Π = paid perfectly, lives not at all. The inversion of every
previous case: here **absence of ε is the anomaly**. τ still governs the *instrument's*
honesty (Δ terms + declared ε as always); Π is what the instrument reads.

- Gate G4 obligations: η₀ sealed from a stakeholder-verified living cohort (Station rite);
  replay determinism; τ-stability bound so a verdict cannot flip on perturbation smaller
  than the declared ε — this is what makes readings **defensible as evidence**.

### 3.2 The Reconciliation Gauge — S × S′ across the bridge

A standing ORBIT PAIR between the pension ledger (B\* stream) and the death registry (S′):

```heptascript
ORBIT PAIR pension_ledger WITH death_registry
  RESONANCE REQUIRED ON PERSON IDENTITY
  TOLERANCE SEALED GRACE 60 DAYS            // lawful administrative lag = arkû, not fraud
PROVE PAIR YIELD UNRECONCILED SET
```

An unreconciled crossing older than the sealed grace window is a DETECT. The grace window
matters: a death reported late is **arkû — late, lawful** (NIPPUR 5.3), and the family's
final entitlements are not fraud. The gauge distinguishes lag from ghost by watermark
arithmetic, not suspicion.

### 3.3 The Double-Draw Gauge — one identity, two orbits

CUBE COUNT of biometric identities across ministry tribes; any identity with cardinality
> 1 in simultaneous active-pay status forms a SECOND TRIBE detection. Puhu discipline:
ministries are non-equivalent nuclei — a lawful transfer (sequential) is not a double
draw (simultaneous); adannu ordering decides.

### 3.4 What is *not* an instrument here

Benford's digit analysis applies to **derived and discretionary figures** (allowances,
overtime, arrears claims) — never to base salaries, which cluster at lawful grade steps
and would flag every honest ministry. Declaring where an instrument does *not* apply is
part of its Gate G4 contract. An instrument that cannot state its non-domain is not
sealed.

## 4 · The Protective Sin — §1 of the Deployment Constitution

**Silence mistaken for absence runs protectively.** Populations exist whose life-noise is
legitimately near zero in the state's sensors: the elderly, the disabled, the remote, the
displaced. For any account where η ≈ 0 **and** the person's profile matches a declared
low-sensor cohort, the verdict is not GHOST-CANDIDATE but **MUSTER-REQUIRED — PROTECTED**:
the state owes them a field visit, not a suspicion. The dead-sensor cat rule, applied to
grandmothers: *the PARZU is filed against the state's sensing, not against the citizen.*

## 5 · The Staircase as Due Process (A2S, load-bearing)

1. **DETECT** — an *account* (never a person) crosses a sealed threshold with sustain.
   Effect: the account joins the muster list. Nothing else. No payment is touched.
2. **PROVE** — the independent second witness is **physical**: biometric muster or field
   verification by a human officer. This is the Shakkanakku two-instrument rule with a
   human as the second instrument — and it is, historically, exactly how the real ghost
   battalions were caught.
3. **PREDICT** — cohort-level only: expected recovery ranges, never named individuals.
4. **PRESCRIBE** — a sealed **evidence dossier KAKI**: the full ledger walk (payments,
   life-signal absence, S′ crossing, muster outcome, every timestamp as adannu/kašādu,
   the divination trail, the τ-stability certificate), referred to the **Federal Board of
   Supreme Audit (ديوان الرقابة المالية الاتحادي)** and the **Commission of Integrity
   (هيئة النزاهة)**.

**Hard prohibition, sealed at grammar level**: no automated payment cutoff exists in this
system. `PRESCRIBE` in CASE-IQ-001 can *only* yield a referral dossier. Wrongly stopping
a real widow's pension is the single failure mode ranked worse than paying a ghost, and
the architecture makes it impossible rather than discouraged. Appeal is a first-class
particle: every dossier carries its own appeal channel and the account-holder's right to
demand the muster that clears them.

## 6 · Evidence Chain (why a court can trust it)

- **Immutability**: every reading is a KAKI; the case is answerable by ledger walk alone.
- **Dual time**: adannu/kašādu on every event — no clock dispute can unwind the sequence.
- **Declared Θ and ε**: every threshold sealed at a Station rite with named stakeholders;
  every uncertainty printed. The system testifies to what it cannot know.
- **τ-stability certificate**: bounded input perturbation ⇒ bounded verdict; proven at
  Gate G4 (Lean4), attached to every dossier.
- **Bridge log**: every IQD expression shows its conversion KAKI — the money numbers are
  downstream of the person-numbers, visibly.
- **The muster as ground truth**: no dossier ships on statistics alone; the physical
  witness closes it.

## 7 · Deployment Shape

- **Streams (PALGU)**: monthly payroll runs per ministry, pension runs, death-registry
  feed, biometric/attendance feeds — each with its own sealed contract, watermark, MĪLU
  ladder. Late registry entries are arkû, absorbed lawfully.
- **Engines**: PiqittuEngine (this case's instrument host) · ShamashEngine (density) ·
  ShalaEngine (dedupe of identity records, ML-001) · NuskuEngine (watch dashboards on the
  sovereign monitoring node) · EnkiDB CQRS pair (write 101 / read 107) · TIAMAT holds
  predicted recoveries InFlight until audited actuals confirm.
- **BĀRÛTU's role**: the three species above occupy their cells as calibration; the
  generator then mints the *next* species before they are invented — e.g., ghost
  **positions** (funded posts with rotating short-lived occupants), ghost **overtime**
  rhythms, muster-gaming patterns (η manufactured by scripted check-ins — detectable as
  *too-regular noise*, a second-order perfection). Each minted hypothesis watches from
  birth, padlocked.
- **Three Gates**: Ṭupšarru ṣeḫru sees cohort wheels and counts only; Engineer sees
  account-level gauges; Architect and the mandated audit institutions alone see
  identity-resolved dossiers. Data minimization is a gate, not a promise.

## 8 · Honest Limits (stated before anyone asks)

This system finds accounts whose *pattern of existence* fails; it cannot find a ghost
whose sponsor manufactures full synthetic life-noise across all sensed channels — it can
only raise that forgery's cost with every added independent channel. It will generate
false musters, and the design accepts them because a muster is a visit, not a punishment.
And it measures the state's own sensing as much as its citizens: every PROTECTED verdict
is a map of where the state cannot yet see, which is itself a deliverable.

---

*Blueprint drafted in service of DUB.SAR. Calibrated on documented history; deployable
only under the seals, the staircase, and the muster. The ration belongs to the living,
the count belongs to the dead, and the dossier belongs to the auditor — nothing here
belongs to the machine alone. 𒁾*
