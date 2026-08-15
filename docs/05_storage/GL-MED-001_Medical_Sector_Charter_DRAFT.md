# GL-MED-001 (candidate) — MEDICAL SECTOR CHARTER
## BahyWay.Ecosystem v4.0 · Sector Tablet · Status: DRAFT — all names & clauses pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Founding Testimony

This sector exists because its architect spent his life as a patient. The charter's
purpose is to end the **asymmetry of evidence** between the person who lives the
illness and the institutions that interpret it — so that consent is informed,
alternatives are visible, and no one faces a single specialist's verdict without
their own complete record in hand. The system restores agency; it never practices
medicine.

Inherited law: **Namtila principle** — every medical-sector engine is
advisory-only, never blocking, never holding cryptographic or procedural power
over a care decision.

---

## 1 · Triple-O Foundation

All are Particles, at the same level of importance. A 1994 blood value, a felt
symptom, a physician's note, a diagnosis code, an MRI header, a consent grant, a
gap between testimonies — each is a particle with KAKI v4.0 identity, EAV
Mandatory Attributes, OrbitalPosition, and a StoryEngine journal that keeps its
scars. Quality, state, and interpretation live exclusively in EAV space; KAKI
bytes are never abused (locked layout, no exceptions).

**Not a monolith.** Each department, clinic, and sub-clinic is a Tribe with its
own EAV databases and internal pipelines (EnkiSDB→…→EnkiDDB per node as needed).
The center is a **witness, never a master**: departments federate Ed25519-sealed,
append-only read-models upward (GulaFederation pattern generalized). Offline-first,
ABAC, append-only — non-negotiable (Najaf annex clauses apply verbatim).

**The patient is not a tribe. The patient is an orbit through tribes.**
A lifetime history is a CrossTribe (0x03) constellation with NUZI provenance,
threading laboratory, clinic, pharmacy, imaging, and surgery tribes into one
queryable life. Performance law: ≥1 billion particles retrieved in <1 second
(ENLIL stack; verified pattern PB-152) — "every time this marker moved after this
medicine class, across forty years" is a sub-second question.

---

## 2 · Tribes of the Sector (initial roster)

| Tribe | Particles (examples) |
|---|---|
| Laboratory | assays, reference ranges, instrument events |
| Clinic / Department (per specialty) | encounters, findings, orders |
| Pharmacy (GulaFederation) | medicines, batches, three price planes, τ spread |
| Imaging | study metadata, report particles (payloads by reference) |
| Surgery / Procedures | operations, outcomes, complications |
| Devices & Spare Parts | machines, maintenance rites, decay-vs-rite horizons |
| Consent | grants, revocations, scopes (see §4) |
| **Testimony** | libbu, asû, sakikkû planes and birītu gaps (see §3) |

Each department dashboard is a **court** in the Hepta world, navigable from the
ḪendursagaEngine flight deck; the Federation Central Dashboard is the sector's
NAV binding, one tribe-court per department.

---

## 3 · THE BIRĪTU LAW — Testimony & Gap (centerpiece of this charter)

*birītu (Akk.): the space between; the interval.*

### 3.1 · Three Testimony Planes

Every illness episode is witnessed on three co-equal planes, each a particle
class in the Testimony tribe:

- **LIBBU plane** (libbu: heart, inner self) — the patient's subjective
  testimony: symptoms as felt, severity as experienced, temporality as lived,
  meaning, fear, implication, goals. First-person, in the patient's words,
  structured into EAV axes but never paraphrased away.
- **ASÛ plane** (asû: physician) — the clinician's interpretation: observations,
  examination findings, reasoning, differential, severity as assessed.
- **SAKIKKÛ plane** — the formal diagnostic outcome: coded diagnosis, staging,
  evidence links (named for the Babylonian Diagnostic Series of Esagil-kin-apli;
  engine naming pending CSR-08).

All three are testimony, not truth-claims ranked by rank. Equal dignity is a law,
not a courtesy: a libbu particle is evidence with the same KAKI/EAV standing as
a laboratory value.

### 3.2 · The Gap as First-Class Particle

For each shared axis a ∈ {severity, location, temporality, causality,
implication, priority, …} the planes are compared under the Hepta metric with
the two stakeholder templates of the Transparency framework:

    δ(a) = d_g( Θ_patient(a) , Θ_physician(a) )        — per-axis divergence
    Δ    = ( δ(severity), δ(temporality), … )           — the Birītu vector
    ε_p , ε_a                                            — uncertainty of each testimony

Each computed gap is **minted as a CrossTribe 0x03 particle** with NUZI
provenance referencing the libbu and asû testimony KAKIs it separates. The gap
is not an error term. It is a finding.

### 3.3 · Birītu Clauses (B-1 … B-5)

- **B-1 · No Silent Reconciliation.** The system never averages, merges, or
  harmonizes the planes into one voice. Δ is rendered, never smoothed.
- **B-2 · Two-Voice Display.** Every consultation surface shows both testimonies
  side by side with the gap made visible between them (display-law sibling of
  UttuEngine's Two-Axis Law). Δ is presented as a vector; any scalar summary is
  forbidden on patient- and clinician-facing surfaces.
- **B-3 · Gap Persistence.** A gap closes only by *new appended testimony* —
  the patient revising their account, the physician revising theirs, or new
  evidence arriving. Gaps are never deleted; a closed gap keeps its scar in the
  StoryEngine journal. Chronic unresolved divergence is itself a diagnostic
  object: a persistent loop (β₁ ≥ 1) in testimony space is a topological alarm
  that the lived illness and the interpreted illness have not met for N cycles.
- **B-4 · Gap Into the Course of Action.** Care-plan particles must reference
  the open Birītu gaps that were on the table when the decision was made. A
  decision that ignores a wide δ(implication) does so *on the record*.
- **B-5 · Honest ε.** Both parties' uncertainty is displayed with their
  testimony. Admitting high ε is an act of transparency, never a demerit
  (ŠĀHU doctrine, inherited).

### 3.4 · Why This Is Novel

Existing systems collect patient-reported outcomes as secondary annotations and
treat divergence from clinical findings as noise to minimize. The Birītu Law
inverts this: divergence is a **first-class, persistent, provenance-bearing
measurement** that must be witnessed by all parties and cited by decisions.
This is the τ calculus applied to the oldest opacity of all — the distance
between being sick and being diagnosed.

---

## 4 · Patient Sovereignty Clause (consent law)

- The patient's Ed25519 key gates which tribes and which planes of their orbit
  any clinician, researcher, or dashboard may witness (ABAC-enforced).
- Grants and revocations are Consent-tribe particles: append-only, revocable
  forward, never retroactively falsifiable.
- Anonymous aggregate witnessing (research, provincial dashboards) requires an
  explicit aggregate-scope grant; no data plane may identify an individual
  patient without their key.
- GDPR/AVG-native: the sealed chain proves *who saw what, when, under which
  grant* — the audit is the architecture.

---

## 5 · Federation Dashboards (simulation / visualization seeds)

1. **Body-as-Cosmos** — the patient's orbit rendered as their own particle sky:
   organ systems as constellations, chronic conditions as persistent β₁
   structures (topology as biography), treatments as restoring rites in the
   decay-vs-rite calculus.
2. **Second Witness Court** — for a proposed decision: the evidence particles
   behind it, the patient's own historical response patterns, documented
   alternatives. Never a verdict; always a witnessed comparison.
3. **Birītu Court** — the two testimony skies face each other as mirrored
   planes; gap chords span the interval, width and heat proportional to δ,
   persistent loops glowing as chronic divergence. This court is *shown in the
   consultation room*, to both parties, by design.
4. **Federation Central (NAV)** — one court per department tribe, sealed
   read-models only, ḪendursagaEngine binding; the center witnesses, never
   commands.

Production body: Godot (PB-323 lineage). HTML renditions are Šala prototypes
only (Way-of-Work rule 5).

---

## 6 · Sequencing & Governance

- Governing law intact: the existing playbook program completes and tests
  first; the Medical Sector is the workload it is proven against.
- First dataset: the architect's own history, self-consented — patient zero as
  architect (a founding testimony and a legal clean-room in one).
- Pharmacy tribe enters via the GulaFederation suite (PB-321…326).
- Candidate seals for CSR-08: GL-MED-001 (this charter) · Birītu Law naming ·
  SakikkûEngine (diagnostic-evidence engine, for Esagil-kin-apli) ·
  Šamaš (public patient face) · GulaFederationEngine · whether the Birītu gap
  calculus enters GL-ALG-002 as a new Unified Algebra member (it composes
  Θ, ε, the Hepta metric g, and β₁ alarms — no new mathematics invented).

*Drafted in the reign of Gudea 1.0. Nothing herein is sealed until DUB.SAR
confirms under CSR-08.*
