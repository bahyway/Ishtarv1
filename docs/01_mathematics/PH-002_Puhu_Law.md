# PH-002 — Puhu Law: TOP Algebra & Nucleus-Exchange Invariance

**Status:** SEALED (concept), Corollary 1 empirically grounded as of
this build (playbook_242/243).
**Domain:** cross-domain — Massartu pattern (`massartu-core`), any
Tribe that plugs a nucleus into it.
**Naming:** "Puhu Law," after *šar pūḫi* — the Mesopotamian
substitute-king ritual, in which a stand-in occupied the throne during
an omen of danger to the real king, absorbing the threat while the
office itself continued unbroken. The law this document states is the
same shape: **the Tribe is the throne, not the king** — a domain
nucleus (electricity, water, a manual log, whatever comes next) sits
in the seat temporarily and can be substituted without breaking the
office it occupies.

## 1. Problem Statement

A cross-domain pattern like Massartu (`docs/07_file_formats/SPEC-MAS-001.md`) is only
as trustworthy as its claim to be domain-neutral. It is easy to *say*
a pattern is domain-neutral while its actual test coverage only ever
exercises one domain's numbers under different labels — at which
point "domain-neutral" is an assertion, not a property. Puhu Law
names the property Massartu is required to actually have, and states
what would count as evidence for it.

## 2. Law Statement — TOP Algebra

Three primitives, unchanged from their existing sealed meanings
elsewhere in this ecosystem:

- **Tribe (T)** — an organizational/domain boundary (a KAKI Tribe).
- **Orbit (O)** — the trajectory a monitored unit traces through
  Hepta Space over time (PH-001's Orbit-Oriented Ontology primitive —
  see §4).
- **Particle (P)** — one 7D EAV observation.

A **nucleus** is the pair of domain-specific implementations
(`ResidualLaw`, `Prover`) a Tribe plugs into Massartu's `watch_cycle`
for one Orbit. Puhu Law states: **the four-step law (DETECT → PROVE →
PREDICT → PRESCRIBE) and the shape of its guarantees belong to the
throne (`watch_cycle`, `Horizon`, `RiskFis`, `Prescriber`), not to
whichever nucleus currently occupies it.** Exchanging one nucleus for
another must not require, or silently depend on, changing
`watch_cycle` itself.

### Corollary 1 — Nucleus-Exchange Invariance

**Claim:** the identical, unmodified `watch_cycle` function correctly
triages structurally different nuclei — a real-physics nucleus and a
nucleus with no dedicated physics engine at all — producing tiers and
prescriptions each nucleus's own evidence actually supports.

**Evidence (this build):**
`crates/massartu-core/src/lib.rs`,
`tests::nucleus_exchange::same_watch_cycle_drives_a_real_physics_nucleus_and_a_manual_one`
drives `watch_cycle` against:

1. An **electricity** nucleus whose `kappa_history` is computed from
   a real `fdd_core::residuals` call over an actual
   `egd_engine::Network<Phasor>`, and whose `isolation_proven` is
   wired to the real `egd_engine::exclusion::simulate_exclusion`.
2. A **generic/manual** nucleus with no dedicated engine, using only
   `Prover`'s honest trait defaults (residual trusted,
   isolation never proven).

Both reach a non-Sound, non-Unproven tier under the same call; the
electricity nucleus (whose exclusion is genuinely provable, via a
redundant spare feed) is eligible for `Prescription::ExcludeProven`/
`Repair`, while the manual nucleus is correctly capped at
`Prescription::MitigateOnly`/`Inspect` — the *same* prescriber logic
producing different, evidence-appropriate answers because the
nuclei's PROVE-gate answers genuinely differ, not because
`watch_cycle` special-cased either domain.

**What this evidence does and does not establish:** this is empirical
evidence from two nuclei, not a formal proof of invariance over all
possible future nuclei — a nucleus that violates the `ResidualLaw`/
`Prover` trait contracts (e.g. a `kappa_history` that isn't actually
time-ordered, or a `Prover` that lies about `isolation_proven`) is not
and cannot be caught by `watch_cycle` itself; Puhu Law covers
substitution *within* the contract, not enforcement of the contract
by construction. A stronger claim would need either a proof over the
trait signatures or a substantially larger and more adversarial suite
of nuclei; neither exists yet.

## 3. Corollary 2 — The Throne Outlives the King

A direct consequence of Corollary 1: retiring or replacing a nucleus
(a domain's engine gets rewritten, or a manual log is finally replaced
by a real simulator) requires no change to `watch_cycle`,
`TrendHorizon`, `SimpleRiskFis`, or `SimplePrescriber` — only a new
`ResidualLaw`/`Prover` pair conforming to the existing traits. This is
what the Massartu spec (`docs/07_file_formats/SPEC-MAS-001.md`) calls "correct by
construction": a new domain costs a new nucleus, never a new
`watch_cycle`. (Direct structural echo of PH-001's own composition
law, "new domains cost new nouns, never new verbs" — see §4.)

## 4. Relationship to PH-001 (Triple-O)

Orbit is PH-001's own primitive (Orbit-Oriented Ontology), not a term
minted here. **Honest status note:** PH-001's real, sealed content —
three axioms (position / state-is-position / change-is-motion), the
KAKI/EAV/Hepta-Space primitives, and its composition law — exists and
was verified in full against this repository on 2026-07-07
(`docs/13_changelog/BATCH5_RM001_RM002_PBCOLLECTIONS_PH001_VERIFIED_2026-07-07.md`,
§4, sourced from `PH001_TripleO_Definition.md.docx`), but as of this
build **no standalone `docs/PH-001.md` has been landed in this repo**
— it is cited here by that verification record, not by a first-party
tablet this document can point to directly. Puhu Law does not restate
or amend PH-001; it borrows exactly one primitive (Orbit) and adds two
new ones (Tribe, Particle, both already sealed elsewhere in this
ecosystem) to name what a "nucleus" is substituted *within*.

## 5. Honest Limits

- No King-plot-style residual classifier exists anywhere in this
  ecosystem (`docs/07_file_formats/GL-EGD-001.md`); `Prover::residual_trustworthy`
  stays at its trusting default for every nucleus that doesn't
  override it, electricity included.
- Only one domain (`egd-engine`) currently wires a real
  `isolation_proven`. Every other nucleus, present or future, gets
  `false` until it builds a real exclusion simulator — this is by
  design (Puhu Law does not relax PROVE), not an oversight.
- Corollary 1's evidence is two nuclei, not an exhaustive or
  adversarial suite (see §2's evidence-scope note). Treat the
  invariance claim as demonstrated for well-behaved nuclei, not
  proven in general.
- No standalone PH-001 tablet exists in this repo yet (§4) — a
  follow-on item if the Architect wants the philosophical foundation
  as a first-party file rather than a verification-record citation.

## 6. Authority Boundary

Consistent with the Analysis-to-Solution Law (GL-MRD-002 Rev.2):
every `Prescription` Massartu emits is advisory-only. Puhu Law governs
which nucleus may occupy the throne and what evidence that requires;
it grants no nucleus, and no substitute, any blocking or
cryptographic authority (Namtila/NINSUN law).

## See also

- `docs/07_file_formats/SPEC-MAS-001.md` — the Massartu pattern's own boundary spec,
  naming decisions (including the Puhu-adjacent Ishum reservation),
  and honest-scope notes this law depends on.
- `crates/massartu-core/src/lib.rs` — the real implementation and the
  cited test.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
